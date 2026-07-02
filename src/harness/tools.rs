use crate::harness::context_window::{LLMContext, build_context_window};
use crate::harness::llm_api::{ChatMessage, LlmApiClient};
use crate::model::{LabeledPostCollection, PostRecord};
use crate::net_backend::{
    NotificationStore, PostDetails, extract_post_details, extract_reply_node,
};
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use bsky_sdk::api::types::string::Did;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolArgumentSpec {
    pub name: String,
    pub value_type: String,
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub arguments: Vec<ToolArgumentSpec>,
    pub when_to_use: String,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolRegistry {
    tools: Vec<ToolSpec>,
}

impl ToolRegistry {
    pub fn new(tools: Vec<ToolSpec>) -> Self {
        Self { tools }
    }

    pub fn harness_defaults() -> Self {
        Self::new(vec![
            ToolSpec {
                name: "read_selected_post".to_string(),
                description: "Read the full text and facets for the post or reply currently selected in the UI.".to_string(),
                arguments: vec![],
                when_to_use: "Use when the current UI context suggests the selected notification matters, but you need the actual post or reply text before answering.".to_string(),
                notes: vec![
                    "Reads the currently selected notification from the detail view.".to_string(),
                    "Returns reply text, reply parent/root URIs, facets, and quoted text when present.".to_string(),
                ],
            },
            ToolSpec {
                name: "llm_search".to_string(),
                description: "Run a narrower LLM pass over cached sources for a target actor and choose a single semantically relevant record.".to_string(),
                arguments: vec![
                    ToolArgumentSpec {
                        name: "actor_did".to_string(),
                        value_type: "string".to_string(),
                        description: "Optional target actor DID. If omitted, the actor from the selected notification is used.".to_string(),
                        required: false,
                    },
                    ToolArgumentSpec {
                        name: "source_kinds".to_string(),
                        value_type: "array<string>".to_string(),
                        description: "Optional sources to search for the target actor: `recent_posts`, `replies_to_actor`, `clearsky_lists`. If omitted, all available sources are merged.".to_string(),
                        required: false,
                    },
                    ToolArgumentSpec {
                        name: "label".to_string(),
                        value_type: "string".to_string(),
                        description: "Optional label for the synthesized search space.".to_string(),
                        required: false,
                    },
                    ToolArgumentSpec {
                        name: "prompt".to_string(),
                        value_type: "string".to_string(),
                        description: "A compact search instruction describing what makes a post relevant.".to_string(),
                        required: true,
                    },
                ],
                when_to_use: "Use when semantic relevance matters more than the compact UI preview and you need the model to inspect a cached collection.".to_string(),
                notes: vec![
                    "The calling agent must write the search prompt.".to_string(),
                    "If `actor_did` is omitted, the tool searches the actor from the currently selected notification.".to_string(),
                    "It synthesizes a temporary merged collection from the target actor's cached sources.".to_string(),
                    "Use this to follow mentions of other actors while keeping the original actor context in the main conversation.".to_string(),
                    "`replies_to_actor` is currently best-effort and comes from cached replies on that actor's pinned posts, not a general network-wide reply search.".to_string(),
                    "Returns one synthesized block with a chosen URI and relevance analysis.".to_string(),
                ],
            },
        ])
    }

    pub fn specs(&self) -> &[ToolSpec] {
        &self.tools
    }

    pub fn render_for_prompt(&self) -> String {
        self.tools
            .iter()
            .map(|tool| {
                let args = tool
                    .arguments
                    .iter()
                    .map(|arg| {
                        format!(
                            "- {} ({}, {}): {}",
                            arg.name,
                            arg.value_type,
                            if arg.required { "required" } else { "optional" },
                            arg.description
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let notes = tool
                    .notes
                    .iter()
                    .map(|note| format!("- {note}"))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "Tool: {}\nDescription: {}\nArguments:\n{}\nWhen To Use: {}\nNotes:\n{}",
                    tool.name, tool.description, args, tool.when_to_use, notes
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptToolCall {
    pub name: String,
    pub args: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmSearchResult {
    pub title: String,
    pub uri: String,
    pub synthesized_block: String,
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

    pub async fn llm_search(
        &self,
        collection: &LabeledPostCollection,
        prompt: &str,
        llm_client: &LlmApiClient,
    ) -> Result<Option<LlmSearchResult>, Box<dyn std::error::Error>> {
        let primary = LlmSearchComparator {
            prompt,
            llm_client,
            max_output_tokens: 512,
        };
        match primary.compare(collection).await {
            Ok(result) => Ok(result),
            Err(primary_err) => {
                let reduced = reduced_search_collection(collection, 8, 280);
                let retry = LlmSearchComparator {
                    prompt,
                    llm_client,
                    max_output_tokens: 256,
                };
                retry.compare(&reduced).await.map_err(|retry_err| {
                    format!(
                        "llm_search failed on full collection ({primary_err}) and reduced retry ({retry_err})"
                    )
                    .into()
                })
            }
        }
    }

    pub fn recent_posts_collection_id(&self, did: &Did) -> String {
        format!("recent_posts:{}", did.as_str())
    }

    pub fn pinned_posts_collection_id(&self, did: &Did) -> String {
        format!("pinned_posts:{}", did.as_str())
    }

    pub fn clearsky_lists_collection_id(&self, did: &Did) -> String {
        format!("clearsky_lists:{}", did.as_str())
    }

    pub fn replies_to_actor_collection_id(&self, did: &Did) -> String {
        format!("replies_to_actor:{}", did.as_str())
    }

    pub fn tool_registry(&self) -> ToolRegistry {
        ToolRegistry::harness_defaults()
    }

    pub fn render_tool_inventory(&self) -> String {
        self.tool_registry().render_for_prompt()
    }

    pub fn read_selected_post(&self, notification: Option<&Notification>) -> String {
        match notification {
            Some(notification) => render_selected_post(notification),
            None => "No notification is currently selected in the UI.".to_string(),
        }
    }

    pub async fn execute_prompt_tool_call(
        &self,
        tool_call: &PromptToolCall,
        selected_notification: Option<&Notification>,
        llm_client: &LlmApiClient,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match tool_call.name.as_str() {
            "read_selected_post" => Ok(self.read_selected_post(selected_notification)),
            "llm_search" => {
                let collection =
                    self.resolve_search_collection(&tool_call.args, selected_notification)?;
                let prompt = require_string_arg(&tool_call.args, "prompt")?;
                let result = self.llm_search(&collection, &prompt, llm_client).await?;
                Ok(render_llm_result(result.as_ref()))
            }
            other => Err(format!("unknown tool `{other}`").into()),
        }
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

    fn resolve_search_collection(
        &self,
        args: &Value,
        selected_notification: Option<&Notification>,
    ) -> Result<LabeledPostCollection, Box<dyn std::error::Error>> {
        let actor_did = match optional_string_arg(args, "actor_did") {
            Some(raw_did) => raw_did
                .parse()
                .map_err(|err| format!("tool arg `actor_did` must be a DID: {err}"))?,
            None => selected_notification
                .map(|notification| notification.author.data.did.clone())
                .ok_or_else(|| {
                    "llm_search requires either `actor_did` or a selected notification".to_string()
                })?,
        };
        let source_kinds = optional_string_array_arg(args, "source_kinds")?;
        let label = optional_string_arg(args, "label");
        let collections = self.actor_search_collections(&actor_did, &source_kinds)?;
        merged_collection_from_refs(
            collections.iter().collect(),
            format!("merged_actor:{}", actor_did.as_str()),
            label.unwrap_or_else(|| {
                format!(
                    "Merged search sources for selected actor {}",
                    actor_did.as_str()
                )
            }),
        )
    }

    fn actor_search_collections(
        &self,
        actor_did: &Did,
        source_kinds: &[String],
    ) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
        let requested = if source_kinds.is_empty() {
            vec![
                "recent_posts".to_string(),
                "replies_to_actor".to_string(),
                "clearsky_lists".to_string(),
            ]
        } else {
            source_kinds.to_vec()
        };

        let mut collections = Vec::new();
        for source_kind in requested {
            match source_kind.as_str() {
                "recent_posts" => {
                    let collection_id = self.recent_posts_collection_id(actor_did);
                    if let Some(collection) = self.store.get_post_collection(&collection_id) {
                        collections.push(collection.clone());
                    }
                }
                "clearsky_lists" => {
                    let collection_id = self.clearsky_lists_collection_id(actor_did);
                    if let Some(collection) = self.store.get_post_collection(&collection_id) {
                        collections.push(collection.clone());
                    }
                }
                "replies_to_actor" => {
                    if let Some(collection) = self.build_replies_to_actor_collection(actor_did) {
                        collections.push(collection);
                    }
                }
                other => {
                    return Err(format!(
                        "unknown source_kind `{other}`; expected recent_posts, replies_to_actor, or clearsky_lists"
                    )
                    .into());
                }
            }
        }

        if collections.is_empty() {
            return Err("no cached sources were available for the selected actor".into());
        }

        Ok(collections)
    }

    fn build_replies_to_actor_collection(&self, actor_did: &Did) -> Option<LabeledPostCollection> {
        let posts = self
            .store
            .get_pinned_posts(actor_did)?
            .iter()
            .flat_map(|post| {
                self.store
                    .get_pinned_post_replies(&post.uri)
                    .unwrap_or(&[])
                    .iter()
                    .map(move |reply| PostRecord {
                        uri: reply.uri.clone(),
                        author_handle: reply.author_handle.clone(),
                        body: format!("source_post_uri: {}\nreply_text: {}", post.uri, reply.text),
                    })
            })
            .collect::<Vec<_>>();

        if posts.is_empty() {
            return None;
        }

        let mut metadata = HashMap::new();
        metadata.insert(
            "collection_kind".to_string(),
            "replies_to_actor".to_string(),
        );
        metadata.insert("actor_did".to_string(), actor_did.as_str().to_string());

        Some(
            LabeledPostCollection::new(
                self.replies_to_actor_collection_id(actor_did),
                format!("Replies to pinned posts by {}", actor_did.as_str()),
                posts,
            )
            .with_metadata(metadata),
        )
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
    let uri = find_matching_uri_from_response(collection, response)?;

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

fn find_matching_uri_from_response(
    collection: &LabeledPostCollection,
    response: &str,
) -> Option<String> {
    if let Some(uri) = response
        .lines()
        .find_map(|line| line.strip_prefix("uri:").map(str::trim))
        .filter(|uri| collection.posts.iter().any(|post| post.uri == *uri))
    {
        return Some(uri.to_string());
    }

    if let Some(post) = collection
        .posts
        .iter()
        .find(|post| response.contains(&post.uri))
    {
        return Some(post.uri.clone());
    }

    let response_lower = response.to_ascii_lowercase();
    let mut best_score = 0usize;
    let mut best_uri = None;

    for post in &collection.posts {
        let score = candidate_match_score(post, &response_lower);
        if score > best_score {
            best_score = score;
            best_uri = Some(post.uri.clone());
        }
    }

    if best_score > 0 { best_uri } else { None }
}

fn candidate_match_score(post: &PostRecord, response_lower: &str) -> usize {
    let mut score = 0usize;

    for hint in candidate_hints(post) {
        let hint = hint.trim();
        if hint.len() < 4 {
            continue;
        }

        let hint_lower = hint.to_ascii_lowercase();
        if response_lower.contains(&hint_lower) {
            score += hint_lower.len();
        }
    }

    score
}

fn candidate_hints(post: &PostRecord) -> Vec<&str> {
    let mut hints = Vec::new();
    for line in post.body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("list_name:") {
            hints.push(value.trim());
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("list_description:") {
            hints.push(value.trim());
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("source_collection_label:") {
            hints.push(value.trim());
            continue;
        }
    }

    if hints.is_empty() {
        if let Some(first_line) = post.body.lines().find(|line| !line.trim().is_empty()) {
            hints.push(first_line.trim());
        }
    }

    hints
}

pub fn prompt_tool_protocol_instructions() -> &'static str {
    "Answer the user's query directly when no tool is needed. If one tool would help, emit exactly one tool request block in this format and nothing else:
TOOL_CALL
name: llm_search
args: {\"actor_did\":\"did:plc:...\",\"source_kinds\":[\"recent_posts\",\"replies_to_actor\",\"clearsky_lists\"],\"prompt\":\"...\"}

Valid llm_search examples:
1. Search only Clearsky lists for another actor: {\"actor_did\":\"did:plc:...\",\"source_kinds\":[\"clearsky_lists\"],\"prompt\":\"what labels or accusations appear in the lists this actor is on?\"}
2. Search recent posts and replies to the actor: {\"actor_did\":\"did:plc:...\",\"source_kinds\":[\"recent_posts\",\"replies_to_actor\"],\"prompt\":\"what disputes or accusations involve this actor?\"}
3. Search all available sources for the selected actor: {\"prompt\":\"what is this actor accused of, based on their posts, replies to them, and the lists they are on?\"}

Available tools are listed below. The Current UI Context section is intentionally compact and does not include full post text. Use read_selected_post when you need the selected post or reply body and facets. Use llm_search for cached source search about either the selected actor or another actor by DID. After a tool result is provided, either answer directly or request one more tool."
}

pub fn parse_prompt_tool_call(response: &str) -> Option<PromptToolCall> {
    let mut lines = response.lines();
    while let Some(line) = lines.next() {
        if line.trim() != "TOOL_CALL" {
            continue;
        }

        let mut name = None;
        let mut args_lines = Vec::new();

        for line in lines.by_ref() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("name:") {
                name = Some(value.trim().to_string());
                continue;
            }

            if let Some(value) = trimmed.strip_prefix("args:") {
                args_lines.push(value.trim().to_string());
                args_lines.extend(lines.map(str::to_owned));
                break;
            }
        }

        let name = name?;
        let raw_args = args_lines.join("\n").trim().to_string();
        let args = serde_json::from_str(&raw_args).ok()?;
        return Some(PromptToolCall { name, args });
    }

    None
}

fn render_llm_result(result: Option<&LlmSearchResult>) -> String {
    match result {
        Some(result) => format!(
            "post: {}\nuri: {}\n{}",
            result.title, result.uri, result.synthesized_block
        ),
        None => "No matching cached posts.".to_string(),
    }
}

fn merged_collection_from_refs(
    collections: Vec<&LabeledPostCollection>,
    id: String,
    label: String,
) -> Result<LabeledPostCollection, Box<dyn std::error::Error>> {
    if collections.is_empty() {
        return Err("no cached collections matched the requested search scope".into());
    }

    let mut metadata = HashMap::new();
    metadata.insert(
        "collection_kind".to_string(),
        "merged_search_space".to_string(),
    );
    metadata.insert(
        "source_collections".to_string(),
        collections
            .iter()
            .map(|collection| collection.id.clone())
            .collect::<Vec<_>>()
            .join(", "),
    );

    let posts = collections
        .iter()
        .flat_map(|collection| {
            collection.posts.iter().map(|post| PostRecord {
                uri: post.uri.clone(),
                author_handle: post.author_handle.clone(),
                body: format!(
                    "source_collection_id: {}\nsource_collection_label: {}\n{}",
                    collection.id, collection.label, post.body
                ),
            })
        })
        .collect::<Vec<_>>();

    Ok(LabeledPostCollection::new(id, label, posts).with_metadata(metadata))
}

fn reduced_search_collection(
    collection: &LabeledPostCollection,
    max_posts: usize,
    max_body_chars: usize,
) -> LabeledPostCollection {
    let posts = collection
        .posts
        .iter()
        .take(max_posts)
        .map(|post| PostRecord {
            uri: post.uri.clone(),
            author_handle: post.author_handle.clone(),
            body: truncate_chars(&post.body, max_body_chars),
        })
        .collect::<Vec<_>>();

    LabeledPostCollection::new(
        collection.id.clone(),
        format!("{} (reduced retry view)", collection.label),
        posts,
    )
    .with_related_collections(collection.related_collection_ids.clone())
    .with_metadata(collection.metadata.clone())
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn render_selected_post(notification: &Notification) -> String {
    let reply = extract_reply_node(&notification.data.record);
    let details = extract_post_details(&notification.data.record);
    let mut lines = vec![
        format!("reason: {}", notification.data.reason),
        format!(
            "author_handle: {}",
            notification.author.data.handle.as_str()
        ),
        format!("author_did: {}", notification.author.data.did.as_str()),
        format!("uri: {}", notification.data.uri),
    ];

    if let Some(reason_subject) = notification.data.reason_subject.as_deref() {
        lines.push(format!("reason_subject: {reason_subject}"));
    }
    if let Some(parent_uri) = reply.parent_uri.as_deref() {
        lines.push(format!("reply_parent_uri: {parent_uri}"));
    }
    if let Some(root_uri) = reply.root_uri.as_deref() {
        lines.push(format!("reply_root_uri: {root_uri}"));
    }

    lines.push(String::new());
    lines.extend(render_post_details(details).lines().map(str::to_owned));
    lines.join("\n")
}

fn render_post_details(details: Option<PostDetails>) -> String {
    let Some(details) = details else {
        return "No readable post or reply payload was found on the selected notification."
            .to_string();
    };

    let mut lines = vec!["text:".to_string()];
    match details.text {
        Some(text) if !text.is_empty() => lines.extend(text.lines().map(str::to_owned)),
        _ => lines.push("<no text>".to_string()),
    }

    lines.push(String::new());
    lines.push("facets:".to_string());
    if details.facets.is_empty() {
        lines.push("<none>".to_string());
    } else {
        for facet in details.facets {
            let mut parts = vec![format!("type: {}", facet.feature_type)];
            if let Some(uri) = facet.uri {
                parts.push(format!("uri: {uri}"));
            }
            if let Some(did) = facet.did {
                parts.push(format!("did: {did}"));
            }
            if let Some(tag) = facet.tag {
                parts.push(format!("tag: {tag}"));
            }
            lines.push(parts.join(" | "));
        }
    }

    lines.push(String::new());
    lines.push("quoted_text:".to_string());
    match details.quoted_text {
        Some(text) if !text.is_empty() => lines.extend(text.lines().map(str::to_owned)),
        _ => lines.push("<none>".to_string()),
    }

    lines.join("\n")
}

fn require_string_arg(args: &Value, key: &str) -> Result<String, Box<dyn std::error::Error>> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("tool arg `{key}` must be a string").into())
}

fn optional_string_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn parse_did_arg(args: &Value, key: &str) -> Result<Did, Box<dyn std::error::Error>> {
    let raw = require_string_arg(args, key)?;
    raw.parse()
        .map_err(|err| format!("tool arg `{key}` must be a DID: {err}").into())
}

fn optional_string_array_arg(
    args: &Value,
    key: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    match args.get(key) {
        None => Ok(Vec::new()),
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("tool arg `{key}` must contain only strings").into())
            })
            .collect(),
        Some(_) => Err(format!("tool arg `{key}` must be an array of strings").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ToolRegistry, merged_collection_from_refs, parse_llm_search_result, parse_prompt_tool_call,
        render_post_details,
    };
    use crate::model::{LabeledPostCollection, PostRecord};
    use crate::net_backend::{PostDetails, PostFacet};

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

    #[test]
    fn llm_search_result_can_fall_back_to_list_name_match() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky test lists",
            vec![
                PostRecord {
                    uri: "https://example.com/list-a".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: accused troll\nlist_description: actors accused of trolling"
                        .to_string(),
                },
                PostRecord {
                    uri: "https://example.com/list-b".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: cat fans\nlist_description: unrelated".to_string(),
                },
            ],
        );

        let result = parse_llm_search_result(
            &collection,
            "title: strongest hit\nanalysis: The actor appears on the \"accused troll\" list, which implies a trolling accusation.",
        )
        .expect("expected parsed result");

        assert_eq!(result.uri, "https://example.com/list-a");
    }

    #[test]
    fn renders_tool_inventory_from_registry() {
        let rendered = ToolRegistry::harness_defaults().render_for_prompt();
        assert!(rendered.contains("Tool: llm_search"));
    }

    #[test]
    fn parses_prompt_tool_call_block() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: llm_search\nargs: {\"actor_did\":\"did:plc:testactor\",\"source_kinds\":[\"recent_posts\",\"clearsky_lists\"],\"prompt\":\"find mentions of cats\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(tool_call.args["actor_did"], "did:plc:testactor");
        assert_eq!(tool_call.args["source_kinds"][0], "recent_posts");
        assert_eq!(tool_call.args["prompt"], "find mentions of cats");
    }

    #[test]
    fn merges_collections_into_a_search_space() {
        let left = LabeledPostCollection::new(
            "recent:test",
            "Recent",
            vec![PostRecord {
                uri: "at://one".to_string(),
                author_handle: "alpha.test".to_string(),
                body: "first body".to_string(),
            }],
        );
        let right = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky",
            vec![PostRecord {
                uri: "https://example.com/list".to_string(),
                author_handle: "clearsky".to_string(),
                body: "list body".to_string(),
            }],
        );

        let merged = merged_collection_from_refs(
            vec![&left, &right],
            "merged:test".to_string(),
            "Merged".to_string(),
        )
        .expect("expected merged collection");

        assert_eq!(merged.posts.len(), 2);
        assert!(
            merged.posts[0]
                .body
                .contains("source_collection_id: recent:test")
        );
        assert!(
            merged.posts[1]
                .body
                .contains("source_collection_id: clearsky:test")
        );
    }

    #[test]
    fn renders_post_details_with_facets() {
        let rendered = render_post_details(Some(PostDetails {
            text: Some("hello".to_string()),
            facets: vec![PostFacet {
                feature_type: "app.bsky.richtext.facet#link".to_string(),
                uri: Some("https://example.com".to_string()),
                did: None,
                tag: None,
            }],
            quoted_text: Some("quoted text".to_string()),
        }));

        assert!(rendered.contains("text:"));
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("https://example.com"));
        assert!(rendered.contains("quoted text"));
    }
}
