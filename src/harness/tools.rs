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
                name: "list_collections".to_string(),
                description: "List cached collections, either globally or for one actor.".to_string(),
                arguments: vec![ToolArgumentSpec {
                    name: "actor_did".to_string(),
                    value_type: "string".to_string(),
                    description: "Optional actor DID. If provided, only collections related to that actor are listed.".to_string(),
                    required: false,
                }],
                when_to_use: "Use when you need to know what cached collections exist before searching or reading an item.".to_string(),
                notes: vec![
                    "Returns compact collection summaries.".to_string(),
                    "If `actor_did` is omitted, all cached collections are listed.".to_string(),
                    "A synthesized `replies_to_actor:<did>` collection is listed when it can be built from cached pinned-post replies.".to_string(),
                ],
            },
            ToolSpec {
                name: "read_collection_item".to_string(),
                description: "Read one specific item from a collection in a richer form suitable for loading into context.".to_string(),
                arguments: vec![
                    ToolArgumentSpec {
                        name: "collection_id".to_string(),
                        value_type: "string".to_string(),
                        description: "The collection containing the item.".to_string(),
                        required: true,
                    },
                    ToolArgumentSpec {
                        name: "item_uri".to_string(),
                        value_type: "string".to_string(),
                        description: "The URI of the item to read.".to_string(),
                        required: true,
                    },
                ],
                when_to_use: "Use after search when you want one item and its richer details in the active context.".to_string(),
                notes: vec![
                    "Reads the exact item selected from a search result.".to_string(),
                    "For reply-oriented collections, returns the synthesized reply record body.".to_string(),
                ],
            },
            ToolSpec {
                name: "llm_search".to_string(),
                description: "Run a narrower LLM pass over cached collections and return grounded evidence anchored to one real record.".to_string(),
                arguments: vec![
                    ToolArgumentSpec {
                        name: "actor_did".to_string(),
                        value_type: "string".to_string(),
                        description: "Optional target actor DID. If provided and `collection_ids` is omitted, search all collections related to that actor.".to_string(),
                        required: false,
                    },
                    ToolArgumentSpec {
                        name: "collection_ids".to_string(),
                        value_type: "array<string>".to_string(),
                        description: "Optional explicit collection IDs to search. If provided, these take precedence over `actor_did`.".to_string(),
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
                    "If `collection_ids` is provided, the tool searches exactly those collections.".to_string(),
                    "If `collection_ids` is omitted and `actor_did` is provided, the tool searches all collections related to that actor.".to_string(),
                    "If both are omitted, the tool searches all cached collections plus any synthesized `replies_to_actor` collections it can build for cached actors.".to_string(),
                    "Returns one synthesized block with a chosen URI plus grounded evidence snippets or repeated labels from the matching items.".to_string(),
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
    pub source_collection_id: Option<String>,
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
            "Inspect the provided collection carefully. Return a compact result block with `uri:`, `title:`, and `analysis:` fields. Always choose one anchor item with a real `uri:` from the collection. The `analysis:` field is evidence-only: quote exact short snippets, list names, list descriptions, or other text taken from the collection, and note repeated labels or overlapping themes across multiple items when relevant. Do not add higher-level interpretation beyond brief grouping of repeated evidence. Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.",
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
            max_output_tokens: 768,
        };
        match primary.compare(collection).await {
            Ok(result) => Ok(result),
            Err(primary_err) => {
                let (max_posts, max_body_chars) = reduced_search_budget(collection);
                let reduced = reduced_search_collection(collection, max_posts, max_body_chars);
                let retry = LlmSearchComparator {
                    prompt,
                    llm_client,
                    max_output_tokens: 512,
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
            "list_collections" => {
                let actor_did = optional_did_arg(&tool_call.args, "actor_did")?;
                Ok(self.list_collections(actor_did.as_ref()))
            }
            "read_collection_item" => {
                let collection_id = require_string_arg(&tool_call.args, "collection_id")?;
                let item_uri = require_string_arg(&tool_call.args, "item_uri")?;
                self.read_collection_item(&collection_id, &item_uri)
            }
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

    pub fn list_collections(&self, actor_did: Option<&Did>) -> String {
        let collections = match actor_did {
            Some(actor_did) => self.collections_for_actor(actor_did),
            None => self.all_collections(),
        };

        if collections.is_empty() {
            return match actor_did {
                Some(actor_did) => {
                    format!("No cached collections are available for {}.", actor_did.as_str())
                }
                None => "No cached collections are available.".to_string(),
            };
        }

        collections
            .into_iter()
            .map(render_collection_summary)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn read_collection_item(
        &self,
        collection_id: &str,
        item_uri: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let collection = self
            .resolve_collection_by_id(collection_id)
            .or_else(|| self.find_collection_for_item_uri(item_uri))
            .ok_or_else(|| format!("unknown collection `{collection_id}`"))?;
        let post = collection
            .posts
            .iter()
            .find(|post| post.uri == item_uri)
            .ok_or_else(|| format!("item `{item_uri}` was not found in `{collection_id}`"))?;

        let mut lines = vec![
            format!("collection_id: {}", collection.id),
            format!("label: {}", collection.label),
            format!("collection_kind: {}", collection.collection_kind),
        ];

        if let Some(actor_did) = collection.actor_did.as_deref() {
            lines.push(format!("actor_did: {actor_did}"));
        }
        lines.push(format!("last_refreshed_at: {}", collection.last_refreshed_at));
        if let Some(ttl) = collection.refresh_ttl_seconds {
            lines.push(format!("refresh_ttl_seconds: {ttl}"));
        }
        if !collection.related_collection_ids.is_empty() {
            lines.push(format!(
                "related_collection_ids: {}",
                collection.related_collection_ids.join(", ")
            ));
        }
        lines.push(String::new());
        lines.push(format!("item_uri: {}", post.uri));
        lines.push(format!("author_handle: {}", post.author_handle));
        lines.push("body:".to_string());
        if post.body.is_empty() {
            lines.push("<empty>".to_string());
        } else {
            lines.extend(post.body.lines().map(str::to_owned));
        }

        Ok(lines.join("\n"))
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
                        "post: {}\nuri: {}\n{}\n{}",
                        result.title,
                        result.uri,
                        result
                            .source_collection_id
                            .as_ref()
                            .map(|id| format!("source_collection_id: {id}"))
                            .unwrap_or_default(),
                        result.synthesized_block
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
        let collection_ids = optional_string_array_arg(args, "collection_ids")?;
        let label = optional_string_arg(args, "label");
        let collections = if !collection_ids.is_empty() {
            self.collections_from_ids(&collection_ids)?
        } else if let Some(actor_did) = optional_did_arg(args, "actor_did")? {
            self.collections_for_actor(&actor_did)
        } else if let Some(notification) = selected_notification {
            self.collections_for_actor(&notification.author.data.did)
        } else {
            self.all_collections()
        };

        if collections.is_empty() {
            return Err(
                "no cached collections matched the requested search scope".to_string().into(),
            );
        }

        merged_collection_from_refs(
            collections,
            label
                .clone()
                .unwrap_or_else(|| "merged_search_scope".to_string()),
            label.unwrap_or_else(|| {
                "Merged search scope".to_string()
            }),
        )
    }

    fn all_collections(&self) -> Vec<LabeledPostCollection> {
        let mut collections = self
            .store
            .post_collections()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        for actor_did in self.store.cached_actor_dids() {
            if let Some(collection) = self.build_replies_to_actor_collection(&actor_did) {
                collections.push(collection);
            }
        }
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        collections
    }

    fn collections_for_actor(&self, actor_did: &Did) -> Vec<LabeledPostCollection> {
        let mut collections = self
            .store
            .actor_post_collections(actor_did)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(collection) = self.build_replies_to_actor_collection(actor_did) {
            collections.push(collection);
        }
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        collections
    }

    fn collections_from_ids(
        &self,
        collection_ids: &[String],
    ) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
        collection_ids
            .iter()
            .map(|collection_id| {
                self.resolve_collection_by_id(collection_id)
                    .ok_or_else(|| format!("unknown collection `{collection_id}`").into())
            })
            .collect()
    }

    fn resolve_collection_by_id(&self, collection_id: &str) -> Option<LabeledPostCollection> {
        if let Some(collection) = self.store.get_post_collection(collection_id) {
            return Some(collection.clone());
        }

        let did = collection_id
            .strip_prefix("replies_to_actor:")
            .and_then(|value| value.parse::<Did>().ok())?;
        self.build_replies_to_actor_collection(&did)
    }

    fn find_collection_for_item_uri(&self, item_uri: &str) -> Option<LabeledPostCollection> {
        let mut matches = self
            .all_collections()
            .into_iter()
            .filter(|collection| collection.posts.iter().any(|post| post.uri == item_uri));
        let first = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(first)
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
        metadata.insert("source".to_string(), "pinned_post_replies".to_string());

        Some(
            LabeledPostCollection::new(
                self.replies_to_actor_collection_id(actor_did),
                format!("Replies to pinned posts by {}", actor_did.as_str()),
                posts,
            )
            .with_collection_kind("replies_to_actor")
            .with_actor_did(actor_did.as_str())
            .with_refresh_state(0, Some(15 * 60))
            .with_metadata(metadata),
        )
    }
}

fn serialize_collection(collection: &LabeledPostCollection) -> String {
    let mut lines = vec![
        format!("collection_id: {}", collection.id),
        format!("label: {}", collection.label),
        format!("collection_kind: {}", collection.collection_kind),
        format!("item_count: {}", collection.posts.len()),
        format!("last_refreshed_at: {}", collection.last_refreshed_at),
    ];

    if let Some(actor_did) = collection.actor_did.as_deref() {
        lines.push(format!("actor_did: {actor_did}"));
    }
    if let Some(ttl) = collection.refresh_ttl_seconds {
        lines.push(format!("refresh_ttl_seconds: {ttl}"));
    }

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
        lines.push(format!("item[{index}]"));
        lines.push(format!("uri: {}", post.uri));
        lines.push(format!("author: {}", post.author_handle));
        lines.extend(render_search_post_fields(post));
        lines.push(String::new());
    }

    lines.join("\n")
}

fn render_search_post_fields(post: &PostRecord) -> Vec<String> {
    let body_fields = body_field_map(&post.body);

    if body_fields.contains_key("list_name") || body_fields.contains_key("list_description") {
        let mut lines = vec!["type: moderation_list".to_string()];
        if let Some(source_collection_id) = body_fields.get("source_collection_id") {
            lines.push(format!("source_collection_id: {}", source_collection_id));
        }
        if let Some(list_name) = body_fields.get("list_name") {
            lines.push(format!("list_name: {}", list_name));
        }
        if let Some(list_description) = body_fields.get("list_description") {
            lines.push(format!("list_description: {}", list_description));
        }
        return lines;
    }

    vec![format!("body: {}", post.body)]
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
        uri: uri.clone(),
        source_collection_id: collection
            .posts
            .iter()
            .find(|post| post.uri == uri)
            .and_then(source_collection_id_from_post),
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
args: {\"actor_did\":\"did:plc:...\",\"prompt\":\"...\"}

Valid llm_search examples:
1. Search all cached collections for another actor: {\"actor_did\":\"did:plc:...\",\"prompt\":\"what labels or accusations appear across this actor's cached collections?\"}
2. Search two known collections directly: {\"collection_ids\":[\"recent_posts_unaddressed:did:plc:...\",\"replies_to_actor:did:plc:...\"],\"prompt\":\"what disputes or accusations involve this actor?\"}
3. Search all available collections: {\"prompt\":\"what is this actor accused of, based on the cached collections available here?\"}

Available tools are listed below. The Current UI Context section is intentionally compact and does not include full post text. Use read_selected_post when you need the selected post or reply body and facets. Use list_collections before search when you need to inspect cache boundaries. Use llm_search for cached collection search. If an llm_search result includes `source_collection_id`, reuse that exact value for `read_collection_item`; do not infer collection IDs from an item URI. Use read_collection_item when a chosen item should be loaded into context with more detail. After a tool result is provided, either answer directly or request one more tool."
}

pub fn parse_prompt_tool_call(response: &str) -> Option<PromptToolCall> {
    let mut lines = response.lines();
    while let Some(line) = lines.next() {
        let trimmed_line = line.trim();
        if trimmed_line != "TOOL_CALL"
            && !trimmed_line.ends_with("TOOL_CALL")
            && !trimmed_line.contains("TOOL_CALL")
        {
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
        Some(result) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("uri: {}", result.uri),
            ];
            if let Some(source_collection_id) = result.source_collection_id.as_deref() {
                lines.push(format!("source_collection_id: {source_collection_id}"));
            }
            lines.push(result.synthesized_block.clone());
            lines.join("\n")
        }
        None => "No matching cached posts.".to_string(),
    }
}

fn source_collection_id_from_post(post: &PostRecord) -> Option<String> {
    body_field_map(&post.body)
        .get("source_collection_id")
        .cloned()
}

fn body_field_map(body: &str) -> HashMap<String, String> {
    body.lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_string(), value.trim().to_string()))
        })
        .collect()
}

fn render_collection_summary(collection: LabeledPostCollection) -> String {
    let mut lines = vec![
        format!("id: {}", collection.id),
        format!("label: {}", collection.label),
        format!("collection_kind: {}", collection.collection_kind),
        format!("item_count: {}", collection.posts.len()),
        format!("last_refreshed_at: {}", collection.last_refreshed_at),
    ];

    if let Some(actor_did) = collection.actor_did.as_deref() {
        lines.push(format!("actor_did: {actor_did}"));
    }
    if !collection.related_collection_ids.is_empty() {
        lines.push(format!(
            "related_collection_ids: {}",
            collection.related_collection_ids.join(", ")
        ));
    }

    lines.join("\n")
}

fn merged_collection_from_refs(
    collections: Vec<LabeledPostCollection>,
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

    Ok(LabeledPostCollection::new(id, label, posts)
        .with_collection_kind("merged_search_space")
        .with_refresh_state(0, None)
        .with_metadata(metadata))
}

fn reduced_search_collection(
    collection: &LabeledPostCollection,
    max_posts: usize,
    max_body_chars: usize,
) -> LabeledPostCollection {
    let posts = sample_posts_evenly(&collection.posts, max_posts)
        .into_iter()
        .map(|post| PostRecord {
            uri: post.uri.clone(),
            author_handle: post.author_handle.clone(),
            body: reduced_search_body(&post.body, max_body_chars),
        })
        .collect::<Vec<_>>();

    let reduced = LabeledPostCollection::new(
        collection.id.clone(),
        format!("{} (reduced retry view)", collection.label),
        posts,
    )
    .with_collection_kind(collection.collection_kind.clone())
    .with_refresh_state(
        collection.last_refreshed_at,
        collection.refresh_ttl_seconds,
    )
    .with_related_collections(collection.related_collection_ids.clone())
    .with_metadata(collection.metadata.clone());

    if let Some(actor_did) = collection.actor_did.as_ref() {
        reduced.with_actor_did(actor_did.clone())
    } else {
        reduced
    }
}

fn reduced_search_body(body: &str, max_body_chars: usize) -> String {
    let fields = body_field_map(body);
    if fields.contains_key("list_name") || fields.contains_key("list_description") {
        let mut lines = Vec::new();
        for key in [
            "source_collection_id",
            "source_collection_label",
            "list_name",
            "list_description",
            "list_did",
            "created_date",
            "date_added",
            "url",
        ] {
            if let Some(value) = fields.get(key) {
                lines.push(format!("{key}: {value}"));
            }
        }
        return lines.join("\n");
    }

    truncate_chars(body, max_body_chars)
}

fn reduced_search_budget(collection: &LabeledPostCollection) -> (usize, usize) {
    let list_like_posts = collection
        .posts
        .iter()
        .filter(|post| {
            let body = &post.body;
            body.contains("list_name:") || body.contains("post[0].list_name:")
        })
        .count();

    if collection.collection_kind == "clearsky_lists"
        || (collection.collection_kind == "merged_search_space"
            && list_like_posts.saturating_mul(2) >= collection.posts.len())
    {
        return (24, 220);
    }

    (12, 280)
}

fn sample_posts_evenly(posts: &[PostRecord], max_posts: usize) -> Vec<&PostRecord> {
    if posts.len() <= max_posts {
        return posts.iter().collect();
    }

    let last_index = posts.len() - 1;
    (0..max_posts)
        .map(|slot| {
            let index = if max_posts <= 1 {
                0
            } else {
                slot * last_index / (max_posts - 1)
            };
            &posts[index]
        })
        .collect()
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

fn optional_did_arg(args: &Value, key: &str) -> Result<Option<Did>, Box<dyn std::error::Error>> {
    optional_string_arg(args, key)
        .map(|raw| {
            raw.parse()
                .map_err(|err| format!("tool arg `{key}` must be a DID: {err}").into())
        })
        .transpose()
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
        ToolRegistry, merged_collection_from_refs, parse_llm_search_result,
        parse_prompt_tool_call, reduced_search_collection, render_post_details,
        serialize_collection, source_collection_id_from_post,
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
            "TOOL_CALL\nname: llm_search\nargs: {\"actor_did\":\"did:plc:testactor\",\"collection_ids\":[\"recent_posts_unaddressed:did:plc:testactor\",\"clearsky_lists:did:plc:testactor\"],\"prompt\":\"find mentions of cats\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(tool_call.args["actor_did"], "did:plc:testactor");
        assert_eq!(
            tool_call.args["collection_ids"][0],
            "recent_posts_unaddressed:did:plc:testactor"
        );
        assert_eq!(tool_call.args["prompt"], "find mentions of cats");
    }

    #[test]
    fn parses_prompt_tool_call_block_with_channel_prefix() {
        let tool_call = parse_prompt_tool_call(
            "<|channel>thought\nplan text\n<channel|>TOOL_CALL\nname: list_collections\nargs: {\"actor_did\":\"did:plc:testactor\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "list_collections");
        assert_eq!(tool_call.args["actor_did"], "did:plc:testactor");
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
            vec![left, right],
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
    fn extracts_source_collection_id_from_merged_post_body() {
        let post = PostRecord {
            uri: "at://one".to_string(),
            author_handle: "alpha.test".to_string(),
            body: "source_collection_id: clearsky:test\nsource_collection_label: Clearsky\nlist_name: accused troll".to_string(),
        };

        assert_eq!(
            source_collection_id_from_post(&post).as_deref(),
            Some("clearsky:test")
        );
    }

    #[test]
    fn serializes_clearsky_list_posts_compactly_for_search() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky test lists",
            vec![PostRecord {
                uri: "https://example.com/list-a".to_string(),
                author_handle: "clearsky".to_string(),
                body: "source_collection_id: clearsky:test\nsource_collection_label: Clearsky test lists\nlist_name: accused troll\nlist_description: actors accused of trolling\nlist_did: did:plc:abc\ncreated_date: 2026-05-07T14:12:26.507000+00:00\nurl: https://example.com/list-a".to_string(),
            }],
        )
        .with_collection_kind("clearsky_lists");

        let rendered = serialize_collection(&collection);

        assert!(rendered.contains("item[0]"));
        assert!(rendered.contains("type: moderation_list"));
        assert!(rendered.contains("list_name: accused troll"));
        assert!(rendered.contains("list_description: actors accused of trolling"));
        assert!(!rendered.contains("created_date:"));
        assert!(!rendered.contains("url: https://example.com/list-a"));
    }

    #[test]
    fn reduced_search_keeps_full_list_description_for_moderation_lists() {
        let description = "Collection of low-quality to outright terrible accounts, by my own standards. Trolls, blue resisters, follower farmers, mod list abusers, maga, bots, scammers, spammers, impersonators, groypers, jerks, racists. Smut, engagement bait, stolen content, useless political memes, crypto.";
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky test lists",
            vec![PostRecord {
                uri: "https://example.com/list-a".to_string(),
                author_handle: "clearsky".to_string(),
                body: format!(
                    "source_collection_id: clearsky:test\nsource_collection_label: Clearsky test lists\nlist_name: Block These Fools\nlist_description: {description}\nlist_did: did:plc:abc\ncreated_date: 2026-05-07T14:12:26.507000+00:00\nurl: https://example.com/list-a"
                ),
            }],
        )
        .with_collection_kind("clearsky_lists");

        let reduced = reduced_search_collection(&collection, 1, 40);
        let rendered = serialize_collection(&reduced);

        assert!(rendered.contains(description));
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
