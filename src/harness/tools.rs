use crate::harness::agents::{
    AgentNodeKind, AgentNodeStatus, AgentNodeTemplate,
};
use crate::harness::context_window::{
    BuiltContextWindow, LLMContext, build_context_window_report,
};
use crate::harness::llm_api::{ChatMessage, LlmApiClient};
use crate::harness::prompts::{AgentKind, tool_prompt};
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
                        description: "Optional target actor DID. If provided and `collection_ids` is omitted, search all cached collections related to that actor.".to_string(),
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
                    "The search prompt is fully dynamic per call, but the tool contract, examples, and result format are hardcoded in this binary.".to_string(),
                    "You must provide either `collection_ids` or `actor_did`; do not call this tool with neither.".to_string(),
                    "If `collection_ids` is provided, the tool searches exactly those collections.".to_string(),
                    "If `collection_ids` is omitted and `actor_did` is provided, the tool searches all collections related to that actor.".to_string(),
                    "Collection IDs must be exact cached IDs such as `recent_posts_unaddressed:did:plc:...` or `clearsky_lists:did:plc:...`; a bare collection kind like `clearsky_lists` is not enough by itself.".to_string(),
                    "For interaction or frequency questions like who this actor replies to, mentions, or interacts with most, prefer explicit conversational `collection_ids` such as `recent_replies_sent`, `recent_posts_unaddressed`, `pinned_posts`, or `replies_to_actor` instead of searching all actor collections.".to_string(),
                    "When a collection contains structured fields such as `list_name` or `list_description`, use those exact fields as evidence instead of inventing new labels or categories.".to_string(),
                    "Authored likes are not currently exposed as a searchable collection, so do not assume a likes collection exists yet.".to_string(),
                    "Returns one synthesized block with a chosen URI plus grounded evidence snippets or repeated themes from the matching items.".to_string(),
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
pub struct LlmSearchResultItem {
    pub uri: String,
    pub source_collection_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmSearchResult {
    pub title: String,
    pub summary: String,
    pub search_results: Vec<LlmSearchResultItem>,
}

pub struct LlmSearchExecution {
    pub result: Option<LlmSearchResult>,
    pub context_window: BuiltContextWindow,
}

pub struct ToolExecutionOutput {
    pub rendered: String,
    pub context_windows: Vec<ToolContextWindow>,
    pub agent_node: Option<AgentNodeTemplate>,
}

pub struct ToolContextWindow {
    pub title: String,
    pub window: BuiltContextWindow,
}

struct CollectionSearchOutcome {
    collection_id: String,
    collection_label: String,
    execution: Result<LlmSearchExecution, String>,
}

const LLM_SEARCH_PARENT_SYSTEM_PROMPT: &str =
    "Synthesize grounded per-collection search results. Keep collection boundaries explicit, compare what each collection supports, and retain failures as diagnostics. Return a compact combined result block with a cross-collection `summary:` plus the strongest real `selected_result_*` anchor when available. Do not invent evidence beyond the provided child results.";

pub struct LlmSearchComparator<'a> {
    pub prompt: &'a str,
    pub llm_client: &'a LlmApiClient,
    pub max_output_tokens: usize,
}

impl<'a> LlmSearchComparator<'a> {
    pub async fn compare(
        &self,
        collection: &LabeledPostCollection,
    ) -> Result<LlmSearchExecution, Box<dyn std::error::Error>> {
        if collection.posts.is_empty() {
            let context = LLMContext::new(
                "Inspect the provided collection carefully. Return a compact result block with `uri:`, `title:`, and `analysis:` fields. Always choose one anchor item with a real `uri:` from the collection. The `analysis:` field is evidence-only: quote exact short snippets, list names, list descriptions, or other text taken from the collection, and note repeated themes across multiple items when relevant. For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context; do not invent a separate label field unless it appears explicitly in the collection text. Do not add higher-level interpretation beyond brief grouping of repeated evidence. Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.",
            );
            return Ok(LlmSearchExecution {
                result: None,
                context_window: build_context_window_report(
                    &context,
                    &self.llm_client.context_limits(),
                ),
            });
        }

        let mut context = LLMContext::new(AgentKind::LlmSearch.system_prompt());
        context.push_section("Collection", serialize_collection(collection));
        context.push_section("Search Prompt", self.prompt);

        let context_window = build_context_window_report(&context, &self.llm_client.context_limits());
        let rendered_context = context_window.rendered.clone();
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

        Ok(LlmSearchExecution {
            result: parse_llm_search_result(collection, &response),
            context_window,
        })
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
    ) -> Result<LlmSearchExecution, Box<dyn std::error::Error>> {
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
    ) -> Result<ToolExecutionOutput, Box<dyn std::error::Error>> {
        match tool_call.name.as_str() {
            "read_selected_post" => Ok(ToolExecutionOutput {
                rendered: self.read_selected_post(selected_notification),
                context_windows: Vec::new(),
                agent_node: None,
            }),
            "list_collections" => {
                let actor_did = optional_did_arg(&tool_call.args, "actor_did")?;
                Ok(ToolExecutionOutput {
                    rendered: self.list_collections(actor_did.as_ref()),
                    context_windows: Vec::new(),
                    agent_node: None,
                })
            }
            "read_collection_item" => {
                let collection_id = require_string_arg(&tool_call.args, "collection_id")?;
                let item_uri = require_string_arg(&tool_call.args, "item_uri")?;
                Ok(ToolExecutionOutput {
                    rendered: self.read_collection_item(&collection_id, &item_uri)?,
                    context_windows: Vec::new(),
                    agent_node: None,
                })
            }
            "llm_search" => {
                let prompt = require_string_arg(&tool_call.args, "prompt")?;
                let collections =
                    self.resolve_search_collections(&tool_call.args, selected_notification)?;
                let outcomes = self
                    .run_collection_searches(&collections, &prompt, llm_client)
                    .await;
                let rendered = synthesize_llm_search_results(&prompt, &outcomes, llm_client).await;
                Ok(ToolExecutionOutput {
                    rendered,
                    context_windows: outcomes
                        .iter()
                        .filter_map(|outcome| match outcome.execution.as_ref() {
                            Ok(execution) => Some(ToolContextWindow {
                                title: format!(
                                    "llm_search: {}",
                                    outcome.collection_label
                                ),
                                window: execution.context_window.clone(),
                            }),
                            Err(_) => None,
                        })
                        .collect(),
                    agent_node: Some(build_llm_search_agent_node(
                        &prompt,
                        &outcomes,
                        llm_client,
                    )),
                })
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
                let mut lines = vec![
                    format!("post: {}", result.title),
                    format!("summary: {}", result.summary),
                ];
                for (index, search_result) in result.search_results.iter().enumerate() {
                    lines.push(format!("search_result_{}_uri: {}", index + 1, search_result.uri));
                    if let Some(source_collection_id) = search_result.source_collection_id.as_deref() {
                        lines.push(format!(
                            "search_result_{}_source_collection_id: {}",
                            index + 1,
                            source_collection_id
                        ));
                    }
                }
                context.push_section(
                    title,
                    lines.join("\n"),
                );
            }
            None => context.push_section(title, "No matching cached posts."),
        }
    }

    async fn run_collection_searches(
        &self,
        collections: &[LabeledPostCollection],
        prompt: &str,
        llm_client: &LlmApiClient,
    ) -> Vec<CollectionSearchOutcome> {
        let mut outcomes = Vec::with_capacity(collections.len());

        for collection in collections {
            let execution = self
                .llm_search(collection, prompt, llm_client)
                .await
                .map_err(|err| err.to_string());
            outcomes.push(CollectionSearchOutcome {
                collection_id: collection.id.clone(),
                collection_label: collection.label.clone(),
                execution,
            });
        }

        outcomes
    }

    fn resolve_search_collections(
        &self,
        args: &Value,
        _selected_notification: Option<&Notification>,
    ) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
        let collection_ids = optional_string_array_arg(args, "collection_ids")?;
        let mut collections = if !collection_ids.is_empty() {
            self.collections_from_ids(&collection_ids)?
        } else if let Some(actor_did) = optional_did_arg(args, "actor_did")? {
            self.collections_for_actor(&actor_did)
        } else {
            return Err(
                "llm_search requires either `collection_ids` or `actor_did`"
                    .to_string()
                    .into(),
            );
        };

        if collections.is_empty() {
            return Err(
                "no cached collections matched the requested search scope".to_string().into(),
            );
        }

        collections.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(collections)
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
        let mut collections = Vec::new();

        for collection_id in collection_ids {
            match self.resolve_collection_by_id(collection_id) {
                Some(collection) => collections.push(collection),
                None if collection_id.starts_with("replies_to_actor:") => {}
                None => return Err(format!("unknown collection `{collection_id}`").into()),
            }
        }

        if collections.is_empty() {
            return Err("no cached collections matched the requested search scope".into());
        }

        Ok(collections)
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
    let search_results = find_matching_uris_from_response(collection, response)
        .into_iter()
        .map(|uri| LlmSearchResultItem {
            source_collection_id: collection
                .posts
                .iter()
                .find(|post| post.uri == uri)
                .and_then(source_collection_id_from_post)
                .or_else(|| Some(collection.id.clone())),
            uri,
        })
        .collect::<Vec<_>>();
    if search_results.is_empty() {
        return None;
    }

    let title = response
        .lines()
        .find_map(|line| line.strip_prefix("title:").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("LLM-selected post in {}", collection.label));

    let summary = extract_llm_search_summary(response)
        .filter(|summary| !summary.is_empty())
        .unwrap_or_else(|| fallback_llm_search_summary(collection, &search_results));

    Some(LlmSearchResult {
        title,
        summary,
        search_results,
    })
}

fn find_matching_uris_from_response(
    collection: &LabeledPostCollection,
    response: &str,
) -> Vec<String> {
    let mut uris = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();
        if let Some(uri) = trimmed.strip_prefix("uri:") {
            push_search_uri(collection, &mut uris, uri.trim(), 4);
        }
    }

    for post in &collection.posts {
        if uris.len() >= 4 {
            break;
        }
        if response.contains(&post.uri) {
            push_search_uri(collection, &mut uris, &post.uri, 4);
        }
    }

    if uris.len() >= 4 {
        return uris;
    }

    let response_lower = response.to_ascii_lowercase();
    let mut scored = collection
        .posts
        .iter()
        .map(|post| (candidate_match_score(post, &response_lower), post.uri.clone()))
        .filter(|(score, _)| *score > 0)
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

    for (_, uri) in scored {
        if uris.len() >= 4 {
            break;
        }
        push_search_uri(collection, &mut uris, &uri, 4);
    }

    uris
}

fn extract_llm_search_summary(response: &str) -> Option<String> {
    response
        .lines()
        .find_map(summary_line_value)
        .filter(|summary| is_valid_llm_search_summary(summary))
        .map(str::to_owned)
        .or_else(|| {
            let analysis = response
                .lines()
                .skip_while(|line| !line.starts_with("analysis:"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            is_valid_llm_search_summary(&analysis).then_some(analysis)
        })
}

fn summary_line_value(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("summary:")
        .or_else(|| trimmed.strip_prefix("\"summary\":"))
        .map(str::trim)
        .map(|value| value.trim_matches(',').trim_matches('"'))
}

fn is_valid_llm_search_summary(summary: &str) -> bool {
    let trimmed = summary.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with("## ")
        && !trimmed.starts_with("collection_id:")
        && !trimmed.starts_with("item[")
        && !trimmed.starts_with("{")
}

fn fallback_llm_search_summary(
    collection: &LabeledPostCollection,
    search_results: &[LlmSearchResultItem],
) -> String {
    let mut hints = Vec::new();
    for search_result in search_results {
        let Some(post) = collection.posts.iter().find(|post| post.uri == search_result.uri) else {
            continue;
        };
        for hint in candidate_hints(post) {
            let hint = hint.trim();
            if hint.len() < 3 || hints.iter().any(|seen| seen == hint) {
                continue;
            }
            hints.push(hint.to_string());
            if hints.len() >= 3 {
                break;
            }
        }
        if hints.len() >= 3 {
            break;
        }
    }

    if hints.is_empty() {
        format!(
            "Grounded evidence was found in {} selected search result(s).",
            search_results.len()
        )
    } else {
        format!("Grounded evidence centers on: {}.", hints.join("; "))
    }
}

fn push_search_uri(
    collection: &LabeledPostCollection,
    uris: &mut Vec<String>,
    uri: &str,
    limit: usize,
) {
    if uris.len() >= limit {
        return;
    }
    if collection.posts.iter().any(|post| post.uri == uri) && !uris.iter().any(|seen| seen == uri)
    {
        uris.push(uri.to_string());
    }
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
    tool_prompt()
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
                args_lines.extend(lines.by_ref().map(str::to_owned));
                break;
            }
        }

        let name = name?;
        let raw_args = extract_first_args_object(&args_lines.join("\n"))?;
        let args = parse_tool_args_json(&raw_args)?;
        return Some(PromptToolCall { name, args });
    }

    None
}

fn extract_first_args_object(raw_args: &str) -> Option<String> {
    let chars = raw_args.char_indices().collect::<Vec<_>>();
    let start = chars.iter().find(|(_, ch)| *ch == '{').map(|(idx, _)| *idx)?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in raw_args[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            continue;
        }

        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let end = start + idx + ch.len_utf8();
                return Some(raw_args[start..end].to_string());
            }
        }
    }

    None
}

fn parse_tool_args_json(raw_args: &str) -> Option<Value> {
    serde_json::from_str(raw_args)
        .ok()
        .or_else(|| serde_json::from_str(&repair_tool_args_json(raw_args)).ok())
}

fn repair_tool_args_json(raw_args: &str) -> String {
    let normalized_quotes = raw_args.replace("<|\"|>", "\"");
    quote_bare_object_keys(&normalized_quotes)
}

fn quote_bare_object_keys(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }

        if ch == '{' || ch == ',' {
            out.push(ch);
            i += 1;

            while i < chars.len() && chars[i].is_whitespace() {
                out.push(chars[i]);
                i += 1;
            }

            let key_start = i;
            if i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '_') {
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let key_end = i;
                let mut probe = i;
                while probe < chars.len() && chars[probe].is_whitespace() {
                    probe += 1;
                }

                if probe < chars.len() && chars[probe] == ':' {
                    out.push('"');
                    for key_char in &chars[key_start..key_end] {
                        out.push(*key_char);
                    }
                    out.push('"');
                    while i < probe {
                        out.push(chars[i]);
                        i += 1;
                    }
                    continue;
                }

                for key_char in &chars[key_start..key_end] {
                    out.push(*key_char);
                }
                continue;
            }

            continue;
        }

        out.push(ch);
        i += 1;
    }

    out
}

fn render_llm_result(result: Option<&LlmSearchResult>) -> String {
    match result {
        Some(result) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
            ];
            for (index, search_result) in result.search_results.iter().enumerate() {
                lines.push(format!("search_result_{}_uri: {}", index + 1, search_result.uri));
                if let Some(source_collection_id) = search_result.source_collection_id.as_deref() {
                    lines.push(format!(
                        "search_result_{}_source_collection_id: {}",
                        index + 1,
                        source_collection_id
                    ));
                }
            }
            lines.join("\n")
        }
        None => "No matching cached posts.".to_string(),
    }
}

fn render_llm_result_compact(result: Option<&LlmSearchResult>) -> String {
    match result {
        Some(result) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
            ];
            for (index, search_result) in result.search_results.iter().enumerate() {
                lines.push(format!("search_result_{}_uri: {}", index + 1, search_result.uri));
                if let Some(source_collection_id) = search_result.source_collection_id.as_deref() {
                    lines.push(format!(
                        "search_result_{}_source_collection_id: {}",
                        index + 1,
                        source_collection_id
                    ));
                }
            }
            lines.join("\n")
        }
        None => "No matching cached posts.".to_string(),
    }
}

async fn synthesize_llm_search_results(
    prompt: &str,
    outcomes: &[CollectionSearchOutcome],
    llm_client: &LlmApiClient,
) -> String {
    if outcomes.len() <= 1 {
        return render_combined_llm_search_results(outcomes);
    }

    let context = build_llm_search_parent_context(prompt, outcomes);
    let context_window = build_context_window_report(&context, &llm_client.context_limits());
    let rendered_context = context_window.rendered.clone();
    match llm_client
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
            768,
        )
        .await
    {
        Ok(summary) => render_combined_llm_search_results_with_summary(outcomes, Some(summary.trim())),
        Err(_) => render_combined_llm_search_results_with_summary(outcomes, None),
    }
}

fn build_llm_search_agent_node(
    prompt: &str,
    outcomes: &[CollectionSearchOutcome],
    llm_client: &LlmApiClient,
) -> AgentNodeTemplate {
    AgentNodeTemplate {
        agent_type: AgentNodeKind::ToolAgent,
        label: "llm_search tool agent".to_string(),
        status: if outcomes
            .iter()
            .any(|outcome| outcome.execution.is_err())
        {
            AgentNodeStatus::Failed
        } else {
            AgentNodeStatus::Completed
        },
        tool_name: Some("llm_search".to_string()),
        collection_id: None,
        context_window_report: Some(build_llm_search_tool_context_window(
            prompt,
            outcomes,
            llm_client,
        )),
        result_summary: Some(render_combined_llm_search_results(outcomes)),
        children: outcomes
            .iter()
            .map(build_collection_search_agent_node)
            .collect(),
    }
}

fn build_collection_search_agent_node(
    outcome: &CollectionSearchOutcome,
) -> AgentNodeTemplate {
    let (status, context_window_report, result_summary) = match outcome.execution.as_ref() {
        Ok(execution) => (
            AgentNodeStatus::Completed,
            Some(execution.context_window.clone()),
            Some(render_llm_result(execution.result.as_ref())),
        ),
        Err(err) => (
            AgentNodeStatus::Failed,
            None,
            Some(format!("Tool execution failed: {err}")),
        ),
    };

    AgentNodeTemplate {
        agent_type: AgentNodeKind::CollectionSearchAgent,
        label: format!("collection search: {}", outcome.collection_label),
        status,
        tool_name: None,
        collection_id: Some(outcome.collection_id.clone()),
        context_window_report,
        result_summary,
        children: Vec::new(),
    }
}

fn build_llm_search_tool_context_window(
    prompt: &str,
    outcomes: &[CollectionSearchOutcome],
    llm_client: &LlmApiClient,
) -> BuiltContextWindow {
    let context = build_llm_search_parent_context(prompt, outcomes);
    build_context_window_report(&context, &llm_client.context_limits())
}

fn build_llm_search_parent_context(
    prompt: &str,
    outcomes: &[CollectionSearchOutcome],
) -> LLMContext {
    let mut context = LLMContext::new(LLM_SEARCH_PARENT_SYSTEM_PROMPT);
    context.push_section("Original Search Prompt", prompt);
    context.push_section(
        "Per-Collection Results",
        outcomes
            .iter()
            .map(|outcome| {
                let mut lines = vec![
                    format!("collection_id: {}", outcome.collection_id),
                    format!("collection_label: {}", outcome.collection_label),
                ];
                match outcome.execution.as_ref() {
                    Ok(execution) => {
                        lines.push("status: ok".to_string());
                        lines.push(render_llm_result_compact(execution.result.as_ref()));
                    }
                    Err(err) => {
                        lines.push("status: failed".to_string());
                        lines.push(format!("error: {err}"));
                    }
                }
                lines.join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
    );
    context
}

fn render_combined_llm_search_results(outcomes: &[CollectionSearchOutcome]) -> String {
    render_combined_llm_search_results_with_summary(outcomes, None)
}

fn render_combined_llm_search_results_with_summary(
    outcomes: &[CollectionSearchOutcome],
    parent_summary: Option<&str>,
) -> String {
    if outcomes.len() == 1 {
        return match outcomes.first() {
            Some(outcome) => match outcome.execution.as_ref() {
                Ok(execution) => render_llm_result(execution.result.as_ref()),
                Err(err) => format!(
                    "collection_id: {}\nlabel: {}\nstatus: failed\nerror: {}",
                    outcome.collection_id, outcome.collection_label, err
                ),
            },
            None => "No matching cached posts.".to_string(),
        };
    }

    let mut lines = vec![
        "llm_search searched collections independently and combined the grounded results below."
            .to_string(),
    ];
    if let Some(summary) = normalize_parent_summary(parent_summary) {
        lines.push(format!("summary: {summary}"));
    } else if let Some(summary) = fallback_parent_summary(outcomes) {
        lines.push(format!("summary: {summary}"));
    }

    if let Some((outcome, result)) = outcomes.iter().find_map(|outcome| match outcome.execution.as_ref() {
        Ok(execution) => execution.result.as_ref().map(|result| (outcome, result)),
        Err(_) => None,
    }) {
        if let Some(search_result) = result.search_results.first() {
            lines.push(format!("selected_result_uri: {}", search_result.uri));
            if let Some(source_collection_id) = search_result.source_collection_id.as_deref() {
                lines.push(format!("selected_result_source_collection_id: {source_collection_id}"));
            }
        }
        lines.push(format!("selected_result_collection_id: {}", outcome.collection_id));
        lines.push(format!(
            "selected_result_collection_label: {}",
            outcome.collection_label
        ));
    }

    for outcome in outcomes {
        lines.push(String::new());
        lines.push(format!("collection_id: {}", outcome.collection_id));
        lines.push(format!("collection_label: {}", outcome.collection_label));
        match outcome.execution.as_ref() {
            Ok(execution) => {
                lines.push("status: ok".to_string());
                lines.push(render_llm_result_compact(execution.result.as_ref()));
            }
            Err(err) => {
                lines.push("status: failed".to_string());
                lines.push(format!("error: {err}"));
            }
        }
    }

    lines.join("\n")
}

fn normalize_parent_summary(summary: Option<&str>) -> Option<String> {
    let summary = summary?.trim();
    if summary.is_empty() {
        return None;
    }

    let mut kept = Vec::new();
    for line in summary.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("summary:") {
            kept.push(value.trim().to_string());
            continue;
        }
        if trimmed.starts_with("selected_result_")
            || trimmed.starts_with("collection_id:")
            || trimmed.starts_with("collection_label:")
            || trimmed.starts_with("status:")
            || trimmed.starts_with("error:")
        {
            continue;
        }
        kept.push(trimmed.to_string());
    }

    if kept.is_empty() {
        None
    } else {
        Some(kept.join(" "))
    }
}

fn fallback_parent_summary(outcomes: &[CollectionSearchOutcome]) -> Option<String> {
    let summaries = outcomes
        .iter()
        .filter_map(|outcome| {
            let execution = outcome.execution.as_ref().ok()?;
            let result = execution.result.as_ref()?;
            Some(format!("{}: {}", outcome.collection_label, result.summary))
        })
        .collect::<Vec<_>>();

    if summaries.is_empty() {
        None
    } else {
        Some(summaries.join(" | "))
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
    let _ = max_body_chars;
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

    body.to_string()
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
        BlueskyTools, ToolRegistry, merged_collection_from_refs, parse_llm_search_result,
        parse_prompt_tool_call, reduced_search_collection, render_llm_result, render_post_details,
        serialize_collection, source_collection_id_from_post,
    };
    use crate::model::{LabeledPostCollection, PostRecord};
    use crate::net_backend::{NotificationStore, PostDetails, PostFacet};

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

        assert_eq!(result.search_results[0].uri, "at://one");
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

        assert_eq!(result.search_results[0].uri, "https://example.com/list-a");
    }

    #[test]
    fn llm_search_result_collects_up_to_four_search_results() {
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent test posts",
            vec![
                PostRecord {
                    uri: "at://one".to_string(),
                    author_handle: "alpha.test".to_string(),
                    body: "cats dogs".to_string(),
                },
                PostRecord {
                    uri: "at://two".to_string(),
                    author_handle: "beta.test".to_string(),
                    body: "cats birds".to_string(),
                },
                PostRecord {
                    uri: "at://three".to_string(),
                    author_handle: "gamma.test".to_string(),
                    body: "cats fish".to_string(),
                },
                PostRecord {
                    uri: "at://four".to_string(),
                    author_handle: "delta.test".to_string(),
                    body: "cats lizards".to_string(),
                },
            ],
        );

        let result = parse_llm_search_result(
            &collection,
            "title: cited posts\nsummary: repeated cat references\nuri: at://one\nuri: at://two\nuri: at://three\nuri: at://four",
        )
        .expect("expected parsed result");

        assert_eq!(result.search_results.len(), 4);
        assert_eq!(result.search_results[0].uri, "at://one");
        assert_eq!(result.search_results[3].uri, "at://four");
    }

    #[test]
    fn render_llm_result_includes_summary_and_search_results() {
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent test posts",
            vec![PostRecord {
                uri: "at://one".to_string(),
                author_handle: "alpha.test".to_string(),
                body: "source_collection_id: recent:test\ncats dogs birds".to_string(),
            }],
        );
        let result = parse_llm_search_result(
            &collection,
            "title: picked post\nsummary: quote and context\nuri: at://one",
        )
        .expect("expected parsed result");

        let rendered = render_llm_result(Some(&result));
        assert!(rendered.contains("summary: quote and context"));
        assert!(rendered.contains("search_result_1_uri: at://one"));
        assert!(rendered.contains("search_result_1_source_collection_id: recent:test"));
    }

    #[test]
    fn llm_search_result_falls_back_to_grounded_summary_instead_of_raw_dump() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky test lists",
            vec![
                PostRecord {
                    uri: "https://example.com/list-a".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: AI Fanatics\nlist_description: magical thinking".to_string(),
                },
                PostRecord {
                    uri: "https://example.com/list-b".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: Please stop\nlist_description: people who should stop"
                        .to_string(),
                },
            ],
        );

        let result = parse_llm_search_result(
            &collection,
            "## Collection\ncollection_id: clearsky:test\nitem[0]\nuri: https://example.com/list-a\nuri: https://example.com/list-b",
        )
        .expect("expected parsed result");

        assert!(!result.summary.contains("## Collection"));
        assert!(result.summary.contains("AI Fanatics") || result.summary.contains("Please stop"));
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
    fn parses_prompt_tool_call_block_with_repaired_args_json() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: llm_search\nargs:\n{collection_ids:[<|\"|>clearsky_lists:did:plc:testactor<|\"|>,<|\"|>recent_posts_unaddressed:did:plc:testactor<|\"|>],\nprompt:<|\"|>cluster the sentiment<|\"|>}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(
            tool_call.args["collection_ids"][0],
            "clearsky_lists:did:plc:testactor"
        );
        assert_eq!(
            tool_call.args["collection_ids"][1],
            "recent_posts_unaddressed:did:plc:testactor"
        );
        assert_eq!(tool_call.args["prompt"], "cluster the sentiment");
    }

    #[test]
    fn parses_first_tool_call_when_trailing_thought_continues() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: list_collections\nargs: {actor_did: \"did:plc:testactor\"}\n\n<|channel>thought\nextra commentary\n<channel|>TOOL_CALL\nname: llm_search\nargs: {collection_ids:[\"recent_replies_sent:did:plc:testactor\"], prompt:\"who do they talk to most\"}",
        )
        .expect("expected first tool call");

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
    fn collections_from_ids_skips_missing_replies_to_actor_collection() {
        let mut store = NotificationStore::new();
        store.cache_post_collection(
            LabeledPostCollection::new("recent:test", "Recent", vec![])
                .with_collection_kind("recent_posts_unaddressed"),
        );
        let tools = BlueskyTools::new(&store);

        let collections = tools
            .collections_from_ids(&[
                "recent:test".to_string(),
                "replies_to_actor:did:plc:testactor".to_string(),
            ])
            .expect("expected recent collection to survive optional replies_to_actor miss");

        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, "recent:test");
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
    fn reduced_search_keeps_full_body_for_regular_posts() {
        let body = "source_collection_id: recent_posts_unaddressed:did:plc:test\nsource_collection_label: Recent top-level posts by did:plc:test\nSome companies who told their staff to use AI now are discovering the actual costs were hidden in the hype cycle and the rollback is messy.";
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent posts",
            vec![PostRecord {
                uri: "at://did:plc:test/app.bsky.feed.post/abc".to_string(),
                author_handle: "author.test".to_string(),
                body: body.to_string(),
            }],
        )
        .with_collection_kind("recent_posts_unaddressed");

        let reduced = reduced_search_collection(&collection, 1, 40);
        let rendered = serialize_collection(&reduced);

        assert!(rendered.contains(body));
        assert!(!rendered.contains("..."));
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
