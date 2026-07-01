use crate::harness::context_window::{LLMContext, build_context_window};
use crate::harness::llm_api::{ChatMessage, LlmApiClient};
use crate::model::LabeledPostCollection;
use crate::net_backend::NotificationStore;
use bsky_sdk::api::types::string::Did;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeywordSearchResult {
    pub title: String,
    pub uri: String,
    pub keyword_matches: usize,
    pub excerpt: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmSearchResult {
    pub title: String,
    pub uri: String,
    pub synthesized_block: String,
}

#[derive(Clone, Debug)]
pub struct KeywordSearchComparator {
    pub keywords: Vec<String>,
    pub max_results: usize,
}

impl KeywordSearchComparator {
    pub fn compare(&self, collection: &LabeledPostCollection) -> Vec<KeywordSearchResult> {
        let keywords = normalize_keywords(&self.keywords);
        if keywords.is_empty() {
            return Vec::new();
        }

        let mut results = collection
            .posts
            .iter()
            .filter_map(|post| {
                let haystack = post.body.to_ascii_lowercase();
                let keyword_matches = keywords
                    .iter()
                    .filter(|keyword| haystack.contains(keyword.as_str()))
                    .count();
                if keyword_matches == 0 {
                    return None;
                }

                Some(KeywordSearchResult {
                    title: format!("@{} post in {}", post.author_handle, collection.label),
                    uri: post.uri.clone(),
                    keyword_matches,
                    excerpt: excerpt_for_keywords(&post.body, &keywords),
                })
            })
            .collect::<Vec<_>>();

        results.sort_by(|left, right| {
            right
                .keyword_matches
                .cmp(&left.keyword_matches)
                .then_with(|| left.uri.cmp(&right.uri))
        });
        results.truncate(self.max_results);
        results
    }
}

pub struct LlmSearchComparator<'a> {
    pub prompt: &'a str,
    pub llm_client: &'a LlmApiClient,
    pub max_output_tokens: usize,
}

impl<'a> LlmSearchComparator<'a> {
    pub async fn compare(
        &self,
        collection: &LabeledPostCollection,
    ) -> Result<Option<LlmSearchResult>, Box<dyn std::error::Error>> {
        if collection.posts.is_empty() {
            return Ok(None);
        }

        let mut context = LLMContext::new(
            "Choose one post from the provided collection. Return a compact result block with `uri:`, `title:`, and `analysis:` fields. The analysis must quote the chosen post and explain why it matches the prompt.",
        );
        context.push_section("Collection", serialize_collection(collection));
        context.push_section("Search Prompt", self.prompt);

        let rendered_context = build_context_window(&context, &self.llm_client.context_limits());
        let response = self
            .llm_client
            .complete_chat(
                vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: context.header().to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: rendered_context,
                    },
                ],
                self.max_output_tokens,
            )
            .await?;

        Ok(parse_llm_search_result(collection, &response))
    }
}

pub struct BlueskyTools<'a> {
    store: &'a NotificationStore,
}

impl<'a> BlueskyTools<'a> {
    pub fn new(store: &'a NotificationStore) -> Self {
        Self { store }
    }

    pub fn keyword_search(
        &self,
        collection_id: &str,
        keywords: Vec<String>,
    ) -> Vec<KeywordSearchResult> {
        let Some(collection) = self.store.get_post_collection(collection_id) else {
            return Vec::new();
        };

        KeywordSearchComparator {
            keywords,
            max_results: 2,
        }
        .compare(collection)
    }

    pub async fn llm_search(
        &self,
        collection_id: &str,
        prompt: &str,
        llm_client: &LlmApiClient,
    ) -> Result<Option<LlmSearchResult>, Box<dyn std::error::Error>> {
        let Some(collection) = self.store.get_post_collection(collection_id) else {
            return Ok(None);
        };

        LlmSearchComparator {
            prompt,
            llm_client,
            max_output_tokens: 512,
        }
        .compare(collection)
        .await
    }

    pub fn recent_posts_collection_id(&self, did: &Did) -> String {
        format!("recent_posts:{}", did.as_str())
    }

    pub fn pinned_posts_collection_id(&self, did: &Did) -> String {
        format!("pinned_posts:{}", did.as_str())
    }

    pub fn load_keyword_results_into_context(
        &self,
        context: &mut LLMContext,
        title: &str,
        results: &[KeywordSearchResult],
    ) {
        if results.is_empty() {
            context.push_section(title, "No matching cached posts.");
            return;
        }

        let body = results
            .iter()
            .map(|result| {
                format!(
                    "matches: {}\npost: {}\nuri: {}\n{}",
                    result.keyword_matches, result.title, result.uri, result.excerpt
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        context.push_section(title, body);
    }

    pub fn load_llm_result_into_context(
        &self,
        context: &mut LLMContext,
        title: &str,
        result: Option<&LlmSearchResult>,
    ) {
        match result {
            Some(result) => {
                context.push_section(
                    title,
                    format!(
                        "post: {}\nuri: {}\n{}",
                        result.title, result.uri, result.synthesized_block
                    ),
                );
            }
            None => context.push_section(title, "No matching cached posts."),
        }
    }
}

fn serialize_collection(collection: &LabeledPostCollection) -> String {
    let mut lines = vec![
        format!("collection_id: {}", collection.id),
        format!("label: {}", collection.label),
    ];

    if !collection.related_collection_ids.is_empty() {
        lines.push(format!(
            "related_collections: {}",
            collection.related_collection_ids.join(", ")
        ));
    }

    if !collection.metadata.is_empty() {
        for (key, value) in &collection.metadata {
            lines.push(format!("metadata.{key}: {value}"));
        }
    }

    lines.push(String::new());
    for (index, post) in collection.posts.iter().enumerate() {
        lines.push(format!("post[{index}].uri: {}", post.uri));
        lines.push(format!("post[{index}].author: {}", post.author_handle));
        lines.push(format!("post[{index}].body: {}", post.body));
        lines.push(String::new());
    }

    lines.join("\n")
}

fn parse_llm_search_result(
    collection: &LabeledPostCollection,
    response: &str,
) -> Option<LlmSearchResult> {
    let uri = response
        .lines()
        .find_map(|line| line.strip_prefix("uri:").map(str::trim))
        .filter(|uri| collection.posts.iter().any(|post| post.uri == *uri))?
        .to_string();

    let title = response
        .lines()
        .find_map(|line| line.strip_prefix("title:").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("LLM-selected post in {}", collection.label));

    let synthesized_block = response
        .lines()
        .skip_while(|line| !line.starts_with("analysis:"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Some(LlmSearchResult {
        title,
        uri,
        synthesized_block: if synthesized_block.is_empty() {
            response.trim().to_string()
        } else {
            synthesized_block
        },
    })
}

fn excerpt_for_keywords(text: &str, keywords: &[String]) -> String {
    let normalized_keywords = keywords.iter().collect::<HashSet<_>>();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if normalized_keywords
            .iter()
            .any(|keyword| lower.contains(keyword.as_str()))
        {
            return line.to_owned();
        }
    }

    text.lines().next().unwrap_or_default().to_owned()
}

fn normalize_keywords(keywords: &[String]) -> Vec<String> {
    keywords
        .iter()
        .flat_map(|keyword| keyword.split_whitespace())
        .map(|keyword| keyword.trim_matches(|ch: char| !ch.is_alphanumeric()))
        .filter(|keyword| !keyword.is_empty())
        .map(|keyword| keyword.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{KeywordSearchComparator, excerpt_for_keywords, normalize_keywords, parse_llm_search_result};
    use crate::model::{LabeledPostCollection, PostRecord};

    #[test]
    fn normalizes_keyword_arrays() {
        assert_eq!(
            normalize_keywords(&["cats, dogs".to_string(), "birds".to_string()]),
            vec!["cats", "dogs", "birds"]
        );
    }

    #[test]
    fn prefers_matching_excerpt_lines() {
        let excerpt = excerpt_for_keywords("first line\nmatching cats line", &["cats".to_string()]);
        assert_eq!(excerpt, "matching cats line");
    }

    #[test]
    fn keyword_search_returns_top_two_matches() {
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent test posts",
            vec![
                PostRecord {
                    uri: "at://one".to_string(),
                    author_handle: "alpha.test".to_string(),
                    body: "cats dogs birds".to_string(),
                },
                PostRecord {
                    uri: "at://two".to_string(),
                    author_handle: "beta.test".to_string(),
                    body: "cats dogs".to_string(),
                },
                PostRecord {
                    uri: "at://three".to_string(),
                    author_handle: "gamma.test".to_string(),
                    body: "cats".to_string(),
                },
            ],
        );

        let results = KeywordSearchComparator {
            keywords: vec!["cats".to_string(), "dogs".to_string(), "birds".to_string()],
            max_results: 2,
        }
        .compare(&collection);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].uri, "at://one");
        assert_eq!(results[1].uri, "at://two");
    }

    #[test]
    fn llm_search_result_requires_a_known_uri() {
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent test posts",
            vec![PostRecord {
                uri: "at://one".to_string(),
                author_handle: "alpha.test".to_string(),
                body: "cats dogs birds".to_string(),
            }],
        );

        let result = parse_llm_search_result(
            &collection,
            "title: picked post\nuri: at://one\nanalysis: quote and context",
        )
        .expect("expected parsed result");

        assert_eq!(result.uri, "at://one");
        assert_eq!(result.title, "picked post");
    }
}
