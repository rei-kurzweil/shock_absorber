use crate::harness::agents::{AgentNodeKind, AgentNodeStatus, AgentNodeTemplate};
use crate::harness::context_window::{BuiltContextWindow, LLMContext, build_context_window_report};
use crate::harness::llm_api::{ChatMessage, LlmApiClient};
use crate::harness::prompts::{AgentKind, tool_prompt};
use crate::model::{LabeledPostCollection, PostRecord};
use crate::net_backend::{
    NotificationStore, PostDetails, cache_global_search_posts, ensure_actor_profile_cached,
    ensure_clearsky_lists_cached, ensure_pinned_posts_cached, ensure_post_replies_cached,
    ensure_recent_posts_cached, ensure_recent_replies_received_cached, extract_post_details,
    extract_reply_node,
};
use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use bsky_sdk::api::types::string::Did;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedSender;

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
                    "Actor-scoped reply collections may include recent inbound replies from other actors when they have been cached.".to_string(),
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
                description: "Search Bluesky at a high level, including looking up handles/users or searching posts by topic, then return grounded evidence anchored to real records.".to_string(),
                arguments: vec![
                    ToolArgumentSpec {
                        name: "query".to_string(),
                        value_type: "string".to_string(),
                        description: "The user's search request in natural language, such as who a handle is or what Bluesky posts say about a topic.".to_string(),
                        required: true,
                    },
                ],
                when_to_use: "Use when you need Bluesky-grounded evidence about one or more handles/users, or about a broader topic that requires searching posts.".to_string(),
                notes: vec![
                    "The root agent only supplies the high-level query; the harness decides whether to do handle lookup, actor-centric collection search, or broader Bluesky post search.".to_string(),
                    "If the query names a handle or user, the search should anchor on that actor's profile and may inspect posts for grounding.".to_string(),
                    "If the query is topical rather than person-centric, the search may use Bluesky-wide post search and normalize the results into a collection before running narrower LLM search.".to_string(),
                    "When a collection contains structured fields such as `list_name` or `list_description`, use those exact fields as evidence instead of inventing new labels or categories.".to_string(),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionReviewStatus {
    Pass,
    Fail,
}

impl CollectionReviewStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionReviewVerdict {
    pub status: CollectionReviewStatus,
    pub reason: String,
    pub repair_needed: bool,
    pub repair_instructions: Option<String>,
}

pub struct LlmSearchExecution {
    pub result: Option<LlmSearchResult>,
    pub original_result: Option<LlmSearchResult>,
    pub context_window: BuiltContextWindow,
    pub diagnostic: Option<String>,
    pub review_verdict: Option<CollectionReviewVerdict>,
    pub review_context_window: Option<BuiltContextWindow>,
    pub repair_diagnostic: Option<String>,
}

impl LlmSearchExecution {
    fn is_usable(&self) -> bool {
        self.result.is_some()
            && !matches!(
                self.review_verdict.as_ref().map(|verdict| &verdict.status),
                Some(CollectionReviewStatus::Fail)
            )
    }
}

pub struct ToolExecutionOutput {
    pub rendered: String,
    pub context_windows: Vec<ToolContextWindow>,
    pub agent_node: Option<AgentNodeTemplate>,
}

#[derive(Clone, Debug)]
pub enum ToolProgressEvent {
    AgentUpdate {
        label: String,
        depth: usize,
        content: String,
    },
}

pub struct ToolContextWindow {
    pub title: String,
    pub window: BuiltContextWindow,
}

#[derive(Clone, Debug)]
struct InternalLlmSearchToolSpec {
    name: &'static str,
    description: &'static str,
    arguments: &'static [&'static str],
}

#[derive(Clone, Debug)]
struct ResolvedActorRef {
    actor_ref: String,
    handle: String,
    did: Did,
}

struct CollectionSearchOutcome {
    collection_id: String,
    collection_label: String,
    execution: Result<LlmSearchExecution, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchIntent {
    ReputationLists,
    General,
}

impl SearchIntent {
    fn as_str(self) -> &'static str {
        match self {
            Self::ReputationLists => "reputation_lists",
            Self::General => "general",
        }
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
    ) -> Result<LlmSearchExecution, Box<dyn std::error::Error>> {
        if collection.posts.is_empty() {
            let context = LLMContext::new(
                "Inspect the provided collection carefully. Return a compact result block with `uri:`, `title:`, and `analysis:` fields. Always choose one anchor item with a real `uri:` from the collection. The `analysis:` field is evidence-only: quote exact short snippets, list names, list descriptions, or other text taken from the collection, and note repeated themes across multiple items when relevant. For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context; do not invent a separate label field unless it appears explicitly in the collection text. Do not add higher-level interpretation beyond brief grouping of repeated evidence. Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.",
            );
            return Ok(LlmSearchExecution {
                result: None,
                original_result: None,
                context_window: build_context_window_report(
                    &context,
                    &self.llm_client.context_limits(),
                ),
                diagnostic: None,
                review_verdict: None,
                review_context_window: None,
                repair_diagnostic: None,
            });
        }

        let mut context = LLMContext::new(AgentKind::CollectionSearch.system_prompt());
        context.push_section("Collection", serialize_collection(collection));
        context.push_section("Search Prompt", self.prompt);

        let context_window =
            build_context_window_report(&context, &self.llm_client.context_limits());
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

        let result = parse_llm_search_result(collection, &response);
        Ok(LlmSearchExecution {
            result: result.clone(),
            original_result: result,
            context_window,
            diagnostic: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        })
    }
}

pub struct BlueskyTools;

impl BlueskyTools {
    pub fn new() -> Self {
        Self
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
                let mut retried = retry.compare(&reduced).await.map_err(|retry_err| -> Box<dyn std::error::Error> {
                    format!(
                        "llm_search failed on full collection ({primary_err}) and reduced retry ({retry_err})"
                    )
                    .into()
                })?;
                retried.diagnostic = Some(format!(
                    "Primary full-collection search failed and a reduced retry view was used instead. Primary failure: {primary_err}"
                ));
                Ok(retried)
            }
        }
    }

    pub fn recent_posts_collection_id(&self, did: &Did) -> String {
        format!("recent_posts_unaddressed:{}", did.as_str())
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
        agent: &BskyAgent,
        store: &mut NotificationStore,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
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
                    rendered: self.list_collections(store, actor_did.as_ref()),
                    context_windows: Vec::new(),
                    agent_node: None,
                })
            }
            "read_collection_item" => {
                let collection_id = require_string_arg(&tool_call.args, "collection_id")?;
                let item_uri = require_string_arg(&tool_call.args, "item_uri")?;
                Ok(ToolExecutionOutput {
                    rendered: self.read_collection_item(store, &collection_id, &item_uri)?,
                    context_windows: Vec::new(),
                    agent_node: None,
                })
            }
            "llm_search" => {
                let query = require_string_arg(&tool_call.args, "query")?;
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "llm_search tool agent".to_string(),
                        depth: 1,
                        content: format!("query:\n{query}\n\nstatus: running"),
                    });
                }
                let (rendered, outcomes) = self
                    .execute_internal_llm_search(
                        &query,
                        selected_notification,
                        agent,
                        store,
                        llm_client,
                        observer.clone(),
                    )
                    .await?;
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "llm_search tool agent".to_string(),
                        depth: 1,
                        content: format!(
                            "status: completed\n\nsummary:\n{}",
                            summarize_progress_text(&rendered)
                        ),
                    });
                }
                Ok(ToolExecutionOutput {
                    rendered,
                    context_windows: outcomes
                        .iter()
                        .filter_map(|outcome| match outcome.execution.as_ref() {
                            Ok(execution) => Some(ToolContextWindow {
                                title: format!("llm_search: {}", outcome.collection_label),
                                window: execution.context_window.clone(),
                            }),
                            Err(_) => None,
                        })
                        .collect(),
                    agent_node: Some(build_llm_search_agent_node(&query, &outcomes, llm_client)),
                })
            }
            other => Err(format!("unknown tool `{other}`").into()),
        }
    }

    async fn execute_internal_llm_search(
        &self,
        query: &str,
        selected_notification: Option<&Notification>,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<(String, Vec<CollectionSearchOutcome>), Box<dyn std::error::Error>> {
        let search_intent = classify_search_intent(query);
        let initial_scope_hints = if let Some(actor_refs) = detect_actor_refs(query) {
            format!(
                "{}\npreferred_search_intent: {}\npreferred_search_order: {}",
                render_resolved_actor_refs(
                    &self.resolve_actor_refs(agent, store, &actor_refs).await?
                ),
                search_intent.as_str(),
                preferred_collection_order_hint(search_intent)
            )
        } else if let Some(post_uri) = detect_post_uri(query) {
            format!("detected_post_uri: {post_uri}")
        } else {
            "No actor refs detected from the query. Use `search_global_posts` if you need a Bluesky-wide topical search.".to_string()
        };

        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "{}\n\n{}",
                    AgentKind::LlmSearch.system_prompt(),
                    render_internal_llm_search_tool_protocol()
                ),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "User query:\n{query}\n\nInitial scope hints:\n{initial_scope_hints}\n\nUse tools when needed, then finish with a direct grounded synthesis."
                ),
            },
        ];

        let mut outcomes = Vec::new();
        let mut searched_collection_ids = HashSet::new();
        let mut final_summary = None;
        let mut diagnostics = Vec::new();

        for round in 0..6usize {
            let response = llm_client.complete_chat(messages.clone(), 768).await?;
            let tool_call = match validate_internal_tool_response(&response) {
                InternalToolResponse::FinalSummary(summary) => {
                    final_summary = Some(summary);
                    break;
                }
                InternalToolResponse::Invalid(reason) => {
                    let diagnostic = format!(
                        "status: invalid_tool_call\nreason: {reason}\ninstruction: re-emit exactly one valid internal TOOL_CALL block with no extra prose"
                    );
                    diagnostics.push(format!("internal planner validation failed: {reason}"));
                    if let Some(observer) = observer.as_ref() {
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: "llm_search planner".to_string(),
                            depth: 2,
                            content: diagnostic.clone(),
                        });
                    }
                    messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response,
                    });
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!(
                            "Tool Result\nname: protocol_validation\nargs: {{}}\n\n{}\n\nRe-emit exactly one valid TOOL_CALL block and nothing else.",
                            diagnostic
                        ),
                    });
                    continue;
                }
                InternalToolResponse::ToolCall(tool_call) => tool_call,
            };

            let tool_call = match validate_internal_llm_search_tool_call(&tool_call) {
                Ok(tool_call) => tool_call,
                Err(reason) => {
                    let diagnostic = format!(
                        "status: invalid_tool_call\nname: {}\nreason: {}\ninstruction: re-emit exactly one valid tool call with corrected arguments",
                        tool_call.name, reason
                    );
                    diagnostics.push(format!("internal planner validation failed: {reason}"));
                    if let Some(observer) = observer.as_ref() {
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: format!("internal tool validation: {}", tool_call.name),
                            depth: 2,
                            content: diagnostic.clone(),
                        });
                    }
                    let tool_args = serde_json::to_string(&tool_call.args)?;
                    messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: response,
                    });
                    messages.push(ChatMessage {
                        role: "user".to_string(),
                        content: format!(
                            "Tool Result\nname: {}\nargs: {}\n\n{}\n\nRe-emit exactly one valid tool call.",
                            tool_call.name, tool_args, diagnostic
                        ),
                    });
                    continue;
                }
            };

            let rendered_tool_result = match tool_call.name.as_str() {
                "resolve_actor_refs" => {
                    let query_arg = require_string_arg(&tool_call.args, "query")?;
                    self.execute_internal_resolve_actor_refs(
                        &query_arg,
                        agent,
                        store,
                        observer.clone(),
                    )
                    .await?
                }
                "hydrate_actor_scope" => {
                    let actor_did = parse_did_arg(&tool_call.args, "actor_did")?;
                    self.execute_internal_hydrate_actor_scope(
                        &actor_did,
                        agent,
                        store,
                        &tool_call.args,
                        observer.clone(),
                    )
                    .await?
                }
                "collection_search" => {
                    let collection_id = require_string_arg(&tool_call.args, "collection_id")?;
                    let prompt = require_string_arg(&tool_call.args, "prompt")?;
                    if !searched_collection_ids.insert(collection_id.clone()) {
                        format!(
                            "collection_id: {collection_id}\nstatus: skipped\nreason: this collection was already searched in this llm_search run"
                        )
                    } else {
                        let collection = self
                            .resolve_collection_by_id(store, &collection_id)
                            .ok_or_else(|| format!("unknown collection `{collection_id}`"))?;
                        let mut collection_outcomes = self
                            .run_collection_searches(
                                &[collection],
                                &prompt,
                                llm_client,
                                observer.clone(),
                            )
                            .await;
                        let outcome = collection_outcomes
                            .pop()
                            .ok_or("missing collection search outcome")?;
                        let rendered = match outcome.execution.as_ref() {
                            Ok(execution) => render_collection_outcome_result(
                                &outcome.collection_id,
                                &outcome.collection_label,
                                execution,
                            ),
                            Err(err) => {
                                format!(
                                    "collection_id: {collection_id}\nstatus: failed\nerror: {err}"
                                )
                            }
                        };
                        outcomes.push(outcome);
                        rendered
                    }
                }
                "search_global_posts" => {
                    let query_arg = require_string_arg(&tool_call.args, "query")?;
                    self.execute_internal_search_global_posts(
                        &query_arg,
                        agent,
                        store,
                        observer.clone(),
                    )
                    .await?
                }
                other => {
                    format!("Tool execution failed: unknown internal llm_search tool `{other}`")
                }
            };

            let tool_args = serde_json::to_string(&tool_call.args)?;
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response,
            });
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Tool Result\nname: {}\nargs: {}\n\n{}\n\n{}",
                    tool_call.name,
                    tool_args,
                    rendered_tool_result,
                    if round == 5 {
                        "This was the final allowed internal tool round. Finish with a direct grounded synthesis now. Do not emit TOOL_CALL."
                    } else {
                        "Use this result to continue the search or finish with a direct grounded synthesis."
                    }
                ),
            });
        }

        if outcomes.is_empty() {
            let collections = self
                .resolve_search_collections(
                    agent,
                    store,
                    query,
                    search_intent,
                    selected_notification,
                )
                .await?;
            outcomes = self
                .run_collection_searches(&collections, query, llm_client, observer.clone())
                .await;
        }

        let rendered = render_combined_llm_search_results_with_summary(
            &outcomes,
            final_summary.as_deref(),
            &diagnostics,
        );
        Ok((rendered, outcomes))
    }

    pub fn list_collections(&self, store: &NotificationStore, actor_did: Option<&Did>) -> String {
        let collections = match actor_did {
            Some(actor_did) => self.collections_for_actor(store, actor_did),
            None => self.all_collections(store),
        };

        if collections.is_empty() {
            return match actor_did {
                Some(actor_did) => {
                    format!(
                        "No cached collections are available for {}.",
                        actor_did.as_str()
                    )
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
        store: &NotificationStore,
        collection_id: &str,
        item_uri: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let collection = self
            .resolve_collection_by_id(store, collection_id)
            .or_else(|| self.find_collection_for_item_uri(store, item_uri))
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
        lines.push(format!(
            "last_refreshed_at: {}",
            collection.last_refreshed_at
        ));
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
                    lines.push(format!(
                        "search_result_{}_uri: {}",
                        index + 1,
                        search_result.uri
                    ));
                    if let Some(source_collection_id) =
                        search_result.source_collection_id.as_deref()
                    {
                        lines.push(format!(
                            "search_result_{}_source_collection_id: {}",
                            index + 1,
                            source_collection_id
                        ));
                    }
                }
                context.push_section(title, lines.join("\n"));
            }
            None => context.push_section(title, "No matching cached posts."),
        }
    }

    async fn run_collection_searches(
        &self,
        collections: &[LabeledPostCollection],
        prompt: &str,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Vec<CollectionSearchOutcome> {
        let mut outcomes = Vec::with_capacity(collections.len());

        for collection in collections {
            if let Some(observer) = observer.as_ref() {
                let _ = observer.send(ToolProgressEvent::AgentUpdate {
                    label: format!("collection search: {}", collection.label),
                    depth: 2,
                    content: format!("collection_id: {}\nstatus: running", collection.id),
                });
            }
            let execution = match self.llm_search(collection, prompt, llm_client).await {
                Ok(execution) => self
                    .review_collection_search_execution(
                        collection,
                        prompt,
                        execution,
                        llm_client,
                        observer.clone(),
                    )
                    .await
                    .map_err(|err| err.to_string()),
                Err(err) => Err(err.to_string()),
            };
            let progress_content = match execution.as_ref() {
                Ok(execution) => format!(
                    "collection_id: {}\nstatus: {}{}\n\nsummary:\n{}",
                    collection.id,
                    if execution.is_usable() {
                        "completed"
                    } else {
                        "failed"
                    },
                    execution
                        .diagnostic
                        .as_deref()
                        .map(|diagnostic| format!("\nretry_diagnostic: {diagnostic}"))
                        .unwrap_or_default(),
                    summarize_progress_text(&render_collection_outcome_result(
                        &collection.id,
                        &collection.label,
                        execution,
                    ))
                ),
                Err(err) => format!(
                    "collection_id: {}\nstatus: failed\nerror: {}",
                    collection.id, err
                ),
            };
            if let Some(observer) = observer.as_ref() {
                let _ = observer.send(ToolProgressEvent::AgentUpdate {
                    label: format!("collection search: {}", collection.label),
                    depth: 2,
                    content: progress_content,
                });
            }
            outcomes.push(CollectionSearchOutcome {
                collection_id: collection.id.clone(),
                collection_label: collection.label.clone(),
                execution,
            });
        }

        outcomes
    }

    async fn resolve_search_collections(
        &self,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        query: &str,
        search_intent: SearchIntent,
        _selected_notification: Option<&Notification>,
    ) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
        let mut collections = if let Some(post_uri) = detect_post_uri(query) {
            ensure_post_replies_cached(agent, store, &post_uri).await?;
            vec![
                self.resolve_collection_by_id(store, &post_replies_collection_id(&post_uri))
                    .ok_or_else(|| format!("post replies collection missing for `{post_uri}`"))?,
            ]
        } else if let Some(actor_refs) = detect_actor_refs(query) {
            self.collections_for_actor_refs(agent, store, &actor_refs, search_intent)
                .await?
        } else {
            vec![cache_global_search_posts(agent, store, query, 25).await?]
        };
        if collections.is_empty() {
            return Err("no search collections were available for this query".into());
        }

        dedupe_collections_by_id(&mut collections);
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(collections)
    }

    fn all_collections(&self, store: &NotificationStore) -> Vec<LabeledPostCollection> {
        let mut collections = store
            .post_collections()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        for actor_did in store.cached_actor_dids() {
            if store
                .get_recent_replies_received_collection(&actor_did)
                .is_none()
            {
                if let Some(collection) = self.build_replies_to_actor_collection(store, &actor_did)
                {
                    collections.push(collection);
                }
            }
        }
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        collections
    }

    fn collections_for_actor(
        &self,
        store: &NotificationStore,
        actor_did: &Did,
    ) -> Vec<LabeledPostCollection> {
        let mut collections = store
            .actor_post_collections(actor_did)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        if store
            .get_recent_replies_received_collection(actor_did)
            .is_none()
        {
            if let Some(collection) = self.build_replies_to_actor_collection(store, actor_did) {
                collections.push(collection);
            }
        }
        collections.sort_by(|left, right| left.id.cmp(&right.id));
        collections
    }

    async fn collections_for_actor_refs(
        &self,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        actor_refs: &[String],
        search_intent: SearchIntent,
    ) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
        let mut collections = Vec::new();
        let mut seen_actor_dids = HashSet::new();

        for actor_ref in actor_refs {
            let profile = ensure_actor_profile_cached(agent, store, actor_ref).await?;
            if !seen_actor_dids.insert(profile.did.as_str().to_string()) {
                continue;
            }
            match search_intent {
                SearchIntent::ReputationLists => {
                    let _ = ensure_clearsky_lists_cached(store, &profile.did).await;
                    if let Some(collection) = self.resolve_collection_by_id(
                        store,
                        &self.clearsky_lists_collection_id(&profile.did),
                    ) {
                        collections.push(collection);
                    }

                    ensure_recent_replies_received_cached(agent, store, &profile.did, 20, 50)
                        .await?;
                    if let Some(collection) = self.resolve_collection_by_id(
                        store,
                        &format!("recent_replies_received:{}", profile.did.as_str()),
                    ) {
                        collections.push(collection);
                    }

                    if let Some(collection) = self.resolve_collection_by_id(
                        store,
                        &format!("actor_profile:{}", profile.did.as_str()),
                    ) {
                        collections.push(collection);
                    }

                    ensure_recent_posts_cached(agent, store, &profile.did, 20).await?;
                    if let Some(collection) = self.resolve_collection_by_id(
                        store,
                        &self.recent_posts_collection_id(&profile.did),
                    ) {
                        collections.push(collection);
                    }
                }
                SearchIntent::General => {
                    ensure_recent_posts_cached(agent, store, &profile.did, 20).await?;
                    ensure_pinned_posts_cached(agent, store, &profile.did).await?;
                    ensure_recent_replies_received_cached(agent, store, &profile.did, 20, 50)
                        .await?;
                    let _ = ensure_clearsky_lists_cached(store, &profile.did).await;
                    collections.extend(self.collections_for_actor(store, &profile.did));
                }
            }
        }

        if collections.is_empty() {
            return Err("no actor-backed collections were available for this query".into());
        }

        Ok(collections)
    }

    async fn resolve_actor_refs(
        &self,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        actor_refs: &[String],
    ) -> Result<Vec<ResolvedActorRef>, Box<dyn std::error::Error>> {
        let mut resolved = Vec::new();
        let mut seen_actor_dids = HashSet::new();

        for actor_ref in actor_refs {
            let profile = ensure_actor_profile_cached(agent, store, actor_ref).await?;
            if !seen_actor_dids.insert(profile.did.as_str().to_string()) {
                continue;
            }
            resolved.push(ResolvedActorRef {
                actor_ref: actor_ref.clone(),
                handle: profile.handle.clone(),
                did: profile.did.clone(),
            });
        }

        Ok(resolved)
    }

    async fn execute_internal_resolve_actor_refs(
        &self,
        query: &str,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "resolve_actor_refs".to_string(),
                depth: 2,
                content: format!("status: running\nquery: {query}"),
            });
        }
        let actor_refs = detect_actor_refs(query).unwrap_or_default();
        let resolved = self.resolve_actor_refs(agent, store, &actor_refs).await?;
        let rendered = render_resolved_actor_refs(&resolved);
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "resolve_actor_refs".to_string(),
                depth: 2,
                content: format!(
                    "status: completed\n\nsummary:\n{}",
                    summarize_progress_text(&rendered)
                ),
            });
        }
        Ok(rendered)
    }

    async fn execute_internal_hydrate_actor_scope(
        &self,
        actor_did: &Did,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        args: &Value,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: format!("hydrate_actor_scope: {}", actor_did.as_str()),
                depth: 2,
                content: "status: running".to_string(),
            });
        }
        let rendered = self
            .hydrate_actor_scope(agent, store, actor_did, args)
            .await?;
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: format!("hydrate_actor_scope: {}", actor_did.as_str()),
                depth: 2,
                content: format!(
                    "status: completed\n\nsummary:\n{}",
                    summarize_progress_text(&rendered)
                ),
            });
        }
        Ok(rendered)
    }

    async fn execute_internal_search_global_posts(
        &self,
        query: &str,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "search_global_posts".to_string(),
                depth: 2,
                content: format!("status: running\nquery: {query}"),
            });
        }
        let collection = cache_global_search_posts(agent, store, query, 25).await?;
        let rendered = format!(
            "status: completed\ncollection_id: {}\ncollection_label: {}\nitem_count: {}",
            collection.id,
            collection.label,
            collection.posts.len()
        );
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "search_global_posts".to_string(),
                depth: 2,
                content: rendered.clone(),
            });
        }
        Ok(rendered)
    }

    async fn review_collection_search_execution(
        &self,
        collection: &LabeledPostCollection,
        prompt: &str,
        mut execution: LlmSearchExecution,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<LlmSearchExecution, Box<dyn std::error::Error>> {
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: format!("collection review: {}", collection.label),
                depth: 3,
                content: format!("collection_id: {}\nstatus: running", collection.id),
            });
        }

        let review_context = build_collection_review_context(collection, prompt, &execution);
        let review_window =
            build_context_window_report(&review_context, &llm_client.context_limits());
        let review_response = llm_client
            .complete_chat(
                vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: review_context.header().to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: review_window.rendered.clone(),
                    },
                ],
                384,
            )
            .await
            .ok();

        let heuristic = heuristic_collection_review(collection, &execution);
        let mut verdict = review_response
            .as_deref()
            .and_then(parse_collection_review_verdict)
            .unwrap_or_else(|| heuristic.clone());

        if verdict.status == CollectionReviewStatus::Pass
            && heuristic.status == CollectionReviewStatus::Fail
        {
            verdict = heuristic.clone();
        }

        execution.review_context_window = Some(review_window);

        if verdict.status == CollectionReviewStatus::Fail && verdict.repair_needed {
            let original_summary = execution
                .result
                .as_ref()
                .map(|result| result.summary.clone())
                .unwrap_or_else(|| "<missing summary>".to_string());
            let repair_summary =
                repair_collection_summary(collection, &execution, prompt, &verdict);
            execution.repair_diagnostic = Some(format!(
                "Initial review failed. Original summary: {}",
                truncate_chars(&original_summary, 240)
            ));
            if let Some(result) = execution.result.as_mut() {
                result.summary = repair_summary;
            } else if let Some(original_result) = execution.original_result.clone() {
                execution.result = Some(LlmSearchResult {
                    summary: repair_summary,
                    ..original_result
                });
            }

            let post_repair_verdict = heuristic_collection_review(collection, &execution);
            if post_repair_verdict.status == CollectionReviewStatus::Pass {
                execution.review_verdict = Some(CollectionReviewVerdict {
                    status: CollectionReviewStatus::Pass,
                    reason: format!(
                        "Initial review failed but the repaired summary is now grounded in the selected records. Original reason: {}",
                        verdict.reason
                    ),
                    repair_needed: false,
                    repair_instructions: None,
                });
            } else {
                execution.review_verdict = Some(post_repair_verdict.clone());
                execution.repair_diagnostic = Some(format!(
                    "{} Repair attempt still failed review.",
                    execution.repair_diagnostic.as_deref().unwrap_or_default()
                ));
                execution.result = None;
            }
        } else {
            execution.review_verdict = Some(verdict.clone());
            if verdict.status == CollectionReviewStatus::Fail {
                execution.result = None;
            }
        }

        if let Some(observer) = observer.as_ref() {
            let status = execution
                .review_verdict
                .as_ref()
                .map(|verdict| verdict.status.as_str())
                .unwrap_or("pass");
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: format!("collection review: {}", collection.label),
                depth: 3,
                content: format!(
                    "collection_id: {}\nstatus: {}\n\nsummary:\n{}",
                    collection.id,
                    status,
                    summarize_progress_text(&render_review_summary(
                        execution.review_verdict.as_ref(),
                        execution.repair_diagnostic.as_deref()
                    ))
                ),
            });
        }

        Ok(execution)
    }

    async fn hydrate_actor_scope(
        &self,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        actor_did: &Did,
        args: &Value,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let include_profile = optional_bool_arg(args, "include_profile").unwrap_or(false);
        let include_recent_posts = optional_bool_arg(args, "include_recent_posts").unwrap_or(false);
        let include_pinned_posts = optional_bool_arg(args, "include_pinned_posts").unwrap_or(false);
        let include_recent_replies_received =
            optional_bool_arg(args, "include_recent_replies_received").unwrap_or(false);
        let include_clearsky_lists =
            optional_bool_arg(args, "include_clearsky_lists").unwrap_or(false);

        let mut lines = vec![format!("actor_did: {}", actor_did.as_str())];

        if include_profile {
            let collection_id = format!("actor_profile:{}", actor_did.as_str());
            let collection = self
                .resolve_collection_by_id(store, &collection_id)
                .ok_or_else(|| format!("missing profile collection `{collection_id}`"))?;
            lines.push(format!("collection_id: {}", collection.id));
            lines.push(format!("collection_label: {}", collection.label));
        }
        if include_recent_posts {
            ensure_recent_posts_cached(agent, store, actor_did, 20).await?;
            let collection_id = self.recent_posts_collection_id(actor_did);
            let collection = self
                .resolve_collection_by_id(store, &collection_id)
                .ok_or_else(|| format!("missing recent posts collection `{collection_id}`"))?;
            lines.push(format!("collection_id: {}", collection.id));
            lines.push(format!("collection_label: {}", collection.label));
        }
        if include_pinned_posts {
            ensure_pinned_posts_cached(agent, store, actor_did).await?;
            let collection_id = self.pinned_posts_collection_id(actor_did);
            let collection = self
                .resolve_collection_by_id(store, &collection_id)
                .ok_or_else(|| format!("missing pinned posts collection `{collection_id}`"))?;
            lines.push(format!("collection_id: {}", collection.id));
            lines.push(format!("collection_label: {}", collection.label));
        }
        if include_recent_replies_received {
            ensure_recent_replies_received_cached(agent, store, actor_did, 20, 50).await?;
            let collection_id = format!("recent_replies_received:{}", actor_did.as_str());
            let collection = self
                .resolve_collection_by_id(store, &collection_id)
                .ok_or_else(|| format!("missing recent replies collection `{collection_id}`"))?;
            lines.push(format!("collection_id: {}", collection.id));
            lines.push(format!("collection_label: {}", collection.label));
        }
        if include_clearsky_lists {
            let _ = ensure_clearsky_lists_cached(store, actor_did).await;
            let collection_id = self.clearsky_lists_collection_id(actor_did);
            let collection = self
                .resolve_collection_by_id(store, &collection_id)
                .ok_or_else(|| format!("missing clearsky lists collection `{collection_id}`"))?;
            lines.push(format!("collection_id: {}", collection.id));
            lines.push(format!("collection_label: {}", collection.label));
        }

        if lines.len() == 1 {
            lines.push("note: no scope flags were enabled".to_string());
        }

        Ok(lines.join("\n"))
    }

    fn collections_from_ids(
        &self,
        store: &NotificationStore,
        collection_ids: &[String],
    ) -> Result<Vec<LabeledPostCollection>, Box<dyn std::error::Error>> {
        let mut collections = Vec::new();

        for collection_id in collection_ids {
            match self.resolve_collection_by_id(store, collection_id) {
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

    fn resolve_collection_by_id(
        &self,
        store: &NotificationStore,
        collection_id: &str,
    ) -> Option<LabeledPostCollection> {
        if let Some(collection) = store.get_post_collection(collection_id) {
            return Some(collection.clone());
        }

        let did = collection_id
            .strip_prefix("replies_to_actor:")
            .and_then(|value| value.parse::<Did>().ok())?;
        store
            .get_recent_replies_received_collection(&did)
            .cloned()
            .or_else(|| self.build_replies_to_actor_collection(store, &did))
    }

    fn find_collection_for_item_uri(
        &self,
        store: &NotificationStore,
        item_uri: &str,
    ) -> Option<LabeledPostCollection> {
        let mut matches = self
            .all_collections(store)
            .into_iter()
            .filter(|collection| collection.posts.iter().any(|post| post.uri == item_uri));
        let first = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(first)
    }

    fn build_replies_to_actor_collection(
        &self,
        store: &NotificationStore,
        actor_did: &Did,
    ) -> Option<LabeledPostCollection> {
        let posts = store
            .get_pinned_posts(actor_did)?
            .iter()
            .flat_map(|post| {
                store
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

fn dedupe_collections_by_id(collections: &mut Vec<LabeledPostCollection>) {
    let mut seen_collection_ids = HashSet::new();
    collections.retain(|collection| seen_collection_ids.insert(collection.id.clone()));
}

fn detect_actor_refs(query: &str) -> Option<Vec<String>> {
    let mut actor_refs = Vec::new();

    for raw in query.split_whitespace() {
        let trimmed = raw.trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | '.'
                    | '!'
                    | '?'
                    | ':'
                    | ';'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
                    | '"'
                    | '\''
            )
        });
        if trimmed.is_empty() {
            continue;
        }

        let candidate = trimmed.strip_prefix('@').unwrap_or(trimmed);
        let candidate = candidate.trim_end_matches("'s");
        let looks_like_did = candidate.starts_with("did:");
        let looks_like_handle = candidate.contains('.')
            && candidate
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'));
        if !looks_like_did && !looks_like_handle {
            continue;
        }

        if !actor_refs.iter().any(|existing| existing == candidate) {
            actor_refs.push(candidate.to_string());
        }
    }

    if actor_refs.is_empty() {
        None
    } else {
        Some(actor_refs)
    }
}

fn detect_post_uri(query: &str) -> Option<String> {
    query.split_whitespace().find_map(|raw| {
        let trimmed = raw.trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | '.'
                    | '!'
                    | '?'
                    | ':'
                    | ';'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
                    | '"'
                    | '\''
            )
        });
        (trimmed.starts_with("at://") && trimmed.contains("/app.bsky.feed.post/"))
            .then(|| trimmed.to_string())
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum InternalToolResponse {
    FinalSummary(String),
    ToolCall(PromptToolCall),
    Invalid(String),
}

fn classify_search_intent(query: &str) -> SearchIntent {
    let lower = query.to_ascii_lowercase();
    if [
        "reputation",
        "sentiment",
        "known",
        "known for",
        "how are",
        "positive",
        "negative",
        "list",
        "lists",
        "accusation",
        "accusations",
        "dispute",
        "disputes",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        SearchIntent::ReputationLists
    } else {
        SearchIntent::General
    }
}

fn preferred_collection_order_hint(intent: SearchIntent) -> &'static str {
    match intent {
        SearchIntent::ReputationLists => {
            "clearsky_lists -> recent_replies_received -> actor_profile -> recent_posts_unaddressed"
        }
        SearchIntent::General => "narrowest sufficient collection first",
    }
}

fn validate_internal_tool_response(response: &str) -> InternalToolResponse {
    let trimmed = response.trim();
    if !trimmed.contains("TOOL_CALL") {
        return InternalToolResponse::FinalSummary(response.to_string());
    }

    let Some(tool_call) = parse_prompt_tool_call(trimmed) else {
        return InternalToolResponse::Invalid(
            "response contained `TOOL_CALL` but did not parse into a valid tool request"
                .to_string(),
        );
    };

    let canonical = format!(
        "TOOL_CALL\nname: {}\nargs: {}",
        tool_call.name,
        serde_json::to_string(&tool_call.args).unwrap_or_else(|_| "{}".to_string())
    );
    if trimmed != canonical {
        return InternalToolResponse::Invalid(
            "strict internal mode requires exactly one TOOL_CALL block with no surrounding prose"
                .to_string(),
        );
    }

    InternalToolResponse::ToolCall(tool_call)
}

fn validate_internal_llm_search_tool_call(
    tool_call: &PromptToolCall,
) -> Result<PromptToolCall, String> {
    match tool_call.name.as_str() {
        "resolve_actor_refs" => {
            require_string_arg(&tool_call.args, "query").map_err(|err| err.to_string())?;
        }
        "hydrate_actor_scope" => {
            parse_did_arg(&tool_call.args, "actor_did").map_err(|err| err.to_string())?;
        }
        "collection_search" => {
            let collection_id = require_string_arg(&tool_call.args, "collection_id")
                .map_err(|err| err.to_string())?;
            require_string_arg(&tool_call.args, "prompt").map_err(|err| err.to_string())?;
            validate_collection_id(&collection_id)?;
        }
        "search_global_posts" => {
            require_string_arg(&tool_call.args, "query").map_err(|err| err.to_string())?;
        }
        other => return Err(format!("unknown internal tool `{other}`")),
    }

    Ok(tool_call.clone())
}

fn validate_collection_id(collection_id: &str) -> Result<(), String> {
    let (kind, rest) = collection_id
        .split_once(':')
        .ok_or_else(|| format!("collection id `{collection_id}` is malformed"))?;
    if rest.is_empty() {
        return Err(format!("collection id `{collection_id}` is malformed"));
    }

    match kind {
        "actor_profile"
        | "pinned_posts"
        | "recent_posts"
        | "recent_posts_unaddressed"
        | "recent_replies_sent"
        | "recent_replies_received"
        | "clearsky_lists"
        | "replies_to_actor" => {
            rest.parse::<Did>().map_err(|_| {
                format!("collection id `{collection_id}` has an invalid DID segment")
            })?;
        }
        "global_search_posts" | "post_replies" => {
            if !rest.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Err(format!(
                    "collection id `{collection_id}` has an invalid hashed suffix"
                ));
            }
        }
        _ => {
            return Err(format!(
                "collection id `{collection_id}` uses an unknown kind"
            ));
        }
    }

    Ok(())
}

fn build_collection_review_context(
    collection: &LabeledPostCollection,
    prompt: &str,
    execution: &LlmSearchExecution,
) -> LLMContext {
    let mut context = LLMContext::new(AgentKind::CollectionReview.system_prompt());
    context.push_section("Search Prompt", prompt);
    context.push_section(
        "Collection Evidence",
        render_review_collection_evidence(collection, execution),
    );
    context.push_section(
        "Proposed Summary",
        render_llm_result(execution.original_result.as_ref()),
    );
    context
}

fn render_review_collection_evidence(
    collection: &LabeledPostCollection,
    execution: &LlmSearchExecution,
) -> String {
    let selected_uris = execution
        .original_result
        .as_ref()
        .map(|result| {
            result
                .search_results
                .iter()
                .map(|item| item.uri.as_str())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    let posts = if selected_uris.is_empty() {
        collection.posts.iter().take(4).collect::<Vec<_>>()
    } else {
        collection
            .posts
            .iter()
            .filter(|post| selected_uris.contains(post.uri.as_str()))
            .collect::<Vec<_>>()
    };

    let mut lines = vec![
        format!("collection_id: {}", collection.id),
        format!("collection_label: {}", collection.label),
        format!("collection_kind: {}", collection.collection_kind),
    ];
    for (index, post) in posts.into_iter().enumerate() {
        lines.push(String::new());
        lines.push(format!("matched_item[{index}] uri: {}", post.uri));
        lines.extend(render_search_post_fields(post));
    }
    lines.join("\n")
}

fn parse_collection_review_verdict(response: &str) -> Option<CollectionReviewVerdict> {
    let status = response.lines().find_map(|line| {
        line.trim()
            .strip_prefix("status:")
            .map(str::trim)
            .and_then(|value| match value {
                "pass" => Some(CollectionReviewStatus::Pass),
                "fail" => Some(CollectionReviewStatus::Fail),
                _ => None,
            })
    })?;
    let reason = response
        .lines()
        .find_map(|line| line.trim().strip_prefix("reason:").map(str::trim))
        .filter(|value| !value.is_empty())
        .unwrap_or("No reason provided.")
        .to_string();
    let repair_needed = response
        .lines()
        .find_map(|line| line.trim().strip_prefix("repair_needed:").map(str::trim))
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(matches!(status, CollectionReviewStatus::Fail));
    let repair_instructions = response
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("repair_instructions:")
                .map(str::trim)
        })
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    Some(CollectionReviewVerdict {
        status,
        reason,
        repair_needed,
        repair_instructions,
    })
}

fn heuristic_collection_review(
    collection: &LabeledPostCollection,
    execution: &LlmSearchExecution,
) -> CollectionReviewVerdict {
    let Some(result) = execution
        .result
        .as_ref()
        .or(execution.original_result.as_ref())
    else {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            reason: "No usable `summary:` paragraph exists.".to_string(),
            repair_needed: false,
            repair_instructions: None,
        };
    };

    let summary = result.summary.trim();
    if summary.is_empty() {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            reason: "No usable `summary:` paragraph exists.".to_string(),
            repair_needed: true,
            repair_instructions: Some(
                "Rewrite the summary as one paragraph grounded in the selected records."
                    .to_string(),
            ),
        };
    }

    if summary.contains("\n\n") {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            reason: "The summary is not a single paragraph.".to_string(),
            repair_needed: true,
            repair_instructions: Some(
                "Condense the output into one grounded paragraph.".to_string(),
            ),
        };
    }

    let metadata_markers = [
        "source_post_uri",
        "collection_id:",
        "metadata.",
        "did:",
        "item[",
    ];
    let metadata_hits = metadata_markers
        .iter()
        .map(|marker| summary.matches(marker).count())
        .sum::<usize>();
    if metadata_hits >= 3 {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            reason: "The summary is dominated by identifiers or metadata placeholders.".to_string(),
            repair_needed: true,
            repair_instructions: Some(
                "Replace metadata-heavy text with actual list names, descriptions, or post text."
                    .to_string(),
            ),
        };
    }

    let evidence_hints = selected_record_hints(collection, result);
    let summary_lower = summary.to_ascii_lowercase();
    let hint_matches = evidence_hints
        .iter()
        .filter(|hint| {
            let hint = hint.trim();
            hint.len() >= 4 && summary_lower.contains(&hint.to_ascii_lowercase())
        })
        .count();

    if !evidence_hints.is_empty() && hint_matches == 0 {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            reason: "The summary omits meaningful text that was available in the matched records."
                .to_string(),
            repair_needed: true,
            repair_instructions: Some(
                "Include exact short phrases, list names, list descriptions, or matched reply text from the selected records."
                    .to_string(),
            ),
        };
    }

    CollectionReviewVerdict {
        status: CollectionReviewStatus::Pass,
        reason:
            "The summary is grounded in the selected records and contains substantive evidence."
                .to_string(),
        repair_needed: false,
        repair_instructions: None,
    }
}

fn selected_record_hints(
    collection: &LabeledPostCollection,
    result: &LlmSearchResult,
) -> Vec<String> {
    let mut hints = Vec::new();
    for item in &result.search_results {
        let Some(post) = collection.posts.iter().find(|post| post.uri == item.uri) else {
            continue;
        };
        for hint in candidate_hints(post) {
            let hint = hint.trim();
            if hint.len() >= 4 && !hints.iter().any(|seen| seen == hint) {
                hints.push(hint.to_string());
            }
        }
    }
    hints
}

fn repair_collection_summary(
    collection: &LabeledPostCollection,
    execution: &LlmSearchExecution,
    _prompt: &str,
    verdict: &CollectionReviewVerdict,
) -> String {
    if let Some(original_result) = execution.original_result.as_ref() {
        let summary = fallback_llm_search_summary(collection, &original_result.search_results);
        if !summary.trim().is_empty() {
            return summary;
        }
    }
    verdict.repair_instructions.clone().unwrap_or_else(|| {
        "The selected records did not support a grounded repaired summary.".to_string()
    })
}

fn render_review_summary(
    verdict: Option<&CollectionReviewVerdict>,
    repair_diagnostic: Option<&str>,
) -> String {
    let Some(verdict) = verdict else {
        return "status: pass".to_string();
    };
    let mut lines = vec![
        format!("status: {}", verdict.status.as_str()),
        format!("reason: {}", verdict.reason),
        format!("repair_needed: {}", verdict.repair_needed),
    ];
    if let Some(instructions) = verdict.repair_instructions.as_deref() {
        lines.push(format!("repair_instructions: {instructions}"));
    }
    if let Some(diagnostic) = repair_diagnostic {
        lines.push(format!("repair_diagnostic: {diagnostic}"));
    }
    lines.join("\n")
}

fn render_internal_llm_search_tool_protocol() -> String {
    let tools = [
        InternalLlmSearchToolSpec {
            name: "resolve_actor_refs",
            description: "Resolve actor refs from a natural-language query and return both handle and DID.",
            arguments: &[
                "query (string, required): natural-language query containing one or more actor refs",
            ],
        },
        InternalLlmSearchToolSpec {
            name: "hydrate_actor_scope",
            description: "Ensure specific actor-backed collections exist and report their collection IDs.",
            arguments: &[
                "actor_did (string, required): actor DID to hydrate",
                "include_profile (boolean, optional)",
                "include_recent_posts (boolean, optional)",
                "include_pinned_posts (boolean, optional)",
                "include_recent_replies_received (boolean, optional)",
                "include_clearsky_lists (boolean, optional)",
            ],
        },
        InternalLlmSearchToolSpec {
            name: "collection_search",
            description: "Run a narrow grounded search over one exact validated cached collection.",
            arguments: &[
                "collection_id (string, required): exact cached collection ID",
                "prompt (string, required): what to look for in that collection",
            ],
        },
        InternalLlmSearchToolSpec {
            name: "search_global_posts",
            description: "Run a Bluesky-wide topical search and cache the results as a collection.",
            arguments: &["query (string, required): topical search query"],
        },
    ];

    let inventory = tools
        .iter()
        .map(|tool| {
            format!(
                "Tool: {}\nDescription: {}\nArguments:\n{}",
                tool.name,
                tool.description,
                tool.arguments
                    .iter()
                    .map(|arg| format!("- {arg}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        "If one internal tool would help, emit exactly one tool request block in this format and nothing else:\n\nTOOL_CALL\nname: <tool_name>\nargs: {{...}}\n\nStrict mode rules:\n- emit exactly one TOOL_CALL block and no surrounding prose\n- use exact valid DIDs\n- use exact valid cached collection IDs only\n- for reputation, sentiment, or list questions, prefer `clearsky_lists` first and only expand to `recent_replies_received`, `actor_profile`, or `recent_posts_unaddressed` when needed for contrast or missing evidence\n\nAvailable internal tools:\n\n{inventory}"
    )
}

fn render_resolved_actor_refs(resolved: &[ResolvedActorRef]) -> String {
    if resolved.is_empty() {
        return "No actor refs were resolved.".to_string();
    }

    resolved
        .iter()
        .map(|entry| {
            format!(
                "actor_ref: {}\nresolved_handle: {}\nresolved_did: {}",
                entry.actor_ref,
                entry.handle,
                entry.did.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn post_replies_collection_id(post_uri: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    post_uri.hash(&mut hasher);
    format!("post_replies:{:x}", hasher.finish())
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
        .map(|post| {
            (
                candidate_match_score(post, &response_lower),
                post.uri.clone(),
            )
        })
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
    let mut descriptions = Vec::new();
    for search_result in search_results {
        let Some(post) = collection
            .posts
            .iter()
            .find(|post| post.uri == search_result.uri)
        else {
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
        let fields = body_field_map(&post.body);
        if let Some(description) = fields.get("list_description") {
            let description = description.trim();
            if !description.is_empty() && !descriptions.iter().any(|seen| seen == description) {
                descriptions.push(description.to_string());
            }
        }
        if hints.len() >= 3 && descriptions.len() >= 2 {
            break;
        }
    }

    if hints.is_empty() {
        format!(
            "Grounded evidence was found in {} selected search result(s), but the model did not return a usable structured summary paragraph. The selected records should be treated as the strongest available anchors from this collection, and a follow-up pass may be needed to restate their themes more completely.",
            search_results.len()
        )
    } else {
        let lead = if collection.collection_kind == "clearsky_lists" {
            format!(
                "The strongest grounded evidence in this moderation-list collection centers on {} selected records, with repeated signals around {}.",
                search_results.len(),
                hints.join(", ")
            )
        } else {
            format!(
                "The strongest grounded evidence in this collection centers on {} selected records, with repeated signals around {}.",
                search_results.len(),
                hints.join(", ")
            )
        };
        let support = if descriptions.is_empty() {
            "The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.".to_string()
        } else {
            format!(
                "The matched record text also includes descriptions such as: {}. This fallback summary is derived directly from those matched records because the model response did not yield a usable structured `summary:` field.",
                descriptions
                    .iter()
                    .take(2)
                    .map(|text| format!("\"{}\"", text))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        format!("{lead} {support}")
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
    if collection.posts.iter().any(|post| post.uri == uri) && !uris.iter().any(|seen| seen == uri) {
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
    let start = chars
        .iter()
        .find(|(_, ch)| *ch == '{')
        .map(|(idx, _)| *idx)?;
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
                lines.push(format!(
                    "search_result_{}_uri: {}",
                    index + 1,
                    search_result.uri
                ));
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

fn render_llm_execution_result(execution: &LlmSearchExecution) -> String {
    let mut lines = Vec::new();
    if let Some(diagnostic) = execution.diagnostic.as_deref() {
        lines.push(format!("diagnostic: {diagnostic}"));
    }
    if let Some(verdict) = execution.review_verdict.as_ref() {
        lines.push(format!("review_status: {}", verdict.status.as_str()));
        lines.push(format!("review_reason: {}", verdict.reason));
        lines.push(format!("review_repair_needed: {}", verdict.repair_needed));
    }
    if let Some(repair_diagnostic) = execution.repair_diagnostic.as_deref() {
        lines.push(format!("repair_diagnostic: {repair_diagnostic}"));
    }
    lines.push(render_llm_result(
        execution
            .result
            .as_ref()
            .or(execution.original_result.as_ref()),
    ));
    lines.join("\n")
}

fn render_collection_outcome_result(
    collection_id: &str,
    collection_label: &str,
    execution: &LlmSearchExecution,
) -> String {
    let mut lines = vec![
        format!("collection_id: {collection_id}"),
        format!("collection_label: {collection_label}"),
        format!(
            "status: {}",
            if execution.is_usable() {
                "ok"
            } else {
                "failed"
            }
        ),
    ];
    lines.push(render_llm_execution_result(execution));
    lines.join("\n")
}

fn render_llm_result_compact(result: Option<&LlmSearchResult>) -> String {
    match result {
        Some(result) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
            ];
            for (index, search_result) in result.search_results.iter().enumerate() {
                lines.push(format!(
                    "search_result_{}_uri: {}",
                    index + 1,
                    search_result.uri
                ));
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
    observer: Option<UnboundedSender<ToolProgressEvent>>,
) -> String {
    if outcomes.len() <= 1 {
        return render_combined_llm_search_results(outcomes);
    }

    if let Some(observer) = observer.as_ref() {
        let _ = observer.send(ToolProgressEvent::AgentUpdate {
            label: "llm_search synthesis".to_string(),
            depth: 2,
            content: "status: running".to_string(),
        });
    }

    let context = build_llm_search_parent_context(prompt, outcomes);
    let context_window = build_context_window_report(&context, &llm_client.context_limits());
    let rendered_context = context_window.rendered.clone();
    let rendered = match llm_client
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
        Ok(summary) => {
            render_combined_llm_search_results_with_summary(outcomes, Some(summary.trim()), &[])
        }
        Err(_) => render_combined_llm_search_results_with_summary(outcomes, None, &[]),
    };
    if let Some(observer) = observer.as_ref() {
        let _ = observer.send(ToolProgressEvent::AgentUpdate {
            label: "llm_search synthesis".to_string(),
            depth: 2,
            content: format!(
                "status: completed\n\nsummary:\n{}",
                summarize_progress_text(&rendered)
            ),
        });
    }
    rendered
}

fn build_llm_search_agent_node(
    prompt: &str,
    outcomes: &[CollectionSearchOutcome],
    llm_client: &LlmApiClient,
) -> AgentNodeTemplate {
    AgentNodeTemplate {
        agent_type: AgentNodeKind::ToolAgent,
        agent_kind: Some(AgentKind::LlmSearch),
        label: "llm_search tool agent".to_string(),
        status: if outcomes.iter().any(|outcome| {
            outcome
                .execution
                .as_ref()
                .map(|execution| !execution.is_usable())
                .unwrap_or(true)
        }) {
            AgentNodeStatus::Failed
        } else {
            AgentNodeStatus::Completed
        },
        tool_name: Some("llm_search".to_string()),
        collection_id: None,
        context_window_report: Some(build_llm_search_tool_context_window(
            prompt, outcomes, llm_client,
        )),
        result_summary: Some(render_combined_llm_search_results(outcomes)),
        children: outcomes
            .iter()
            .map(build_collection_search_agent_node)
            .collect(),
    }
}

fn build_collection_search_agent_node(outcome: &CollectionSearchOutcome) -> AgentNodeTemplate {
    let (status, context_window_report, result_summary) = match outcome.execution.as_ref() {
        Ok(execution) => (
            if execution.is_usable() {
                AgentNodeStatus::Completed
            } else {
                AgentNodeStatus::Failed
            },
            Some(execution.context_window.clone()),
            Some(render_llm_execution_result(execution)),
        ),
        Err(err) => (
            AgentNodeStatus::Failed,
            None,
            Some(format!("Tool execution failed: {err}")),
        ),
    };

    AgentNodeTemplate {
        agent_type: AgentNodeKind::CollectionSearchAgent,
        agent_kind: Some(AgentKind::CollectionSearch),
        label: format!("collection search: {}", outcome.collection_label),
        status,
        tool_name: None,
        collection_id: Some(outcome.collection_id.clone()),
        context_window_report,
        result_summary,
        children: outcome
            .execution
            .as_ref()
            .ok()
            .and_then(build_collection_review_agent_node)
            .into_iter()
            .collect(),
    }
}

fn build_collection_review_agent_node(execution: &LlmSearchExecution) -> Option<AgentNodeTemplate> {
    let verdict = execution.review_verdict.as_ref()?;
    Some(AgentNodeTemplate {
        agent_type: AgentNodeKind::CollectionReviewAgent,
        agent_kind: Some(AgentKind::CollectionReview),
        label: "collection review".to_string(),
        status: match verdict.status {
            CollectionReviewStatus::Pass => AgentNodeStatus::Completed,
            CollectionReviewStatus::Fail => AgentNodeStatus::Failed,
        },
        tool_name: None,
        collection_id: None,
        context_window_report: execution.review_context_window.clone(),
        result_summary: Some(render_review_summary(
            execution.review_verdict.as_ref(),
            execution.repair_diagnostic.as_deref(),
        )),
        children: Vec::new(),
    })
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
    let mut context = LLMContext::new(AgentKind::LlmSearch.system_prompt());
    context.push_section("Original Search Query", prompt);
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
                        lines.push(format!(
                            "status: {}",
                            if execution.is_usable() {
                                "ok"
                            } else {
                                "failed"
                            }
                        ));
                        if let Some(diagnostic) = execution.diagnostic.as_deref() {
                            lines.push(format!("diagnostic: {diagnostic}"));
                        }
                        if let Some(verdict) = execution.review_verdict.as_ref() {
                            lines.push(format!("review_status: {}", verdict.status.as_str()));
                            lines.push(format!("review_reason: {}", verdict.reason));
                        }
                        lines.push(render_llm_result_compact(
                            execution
                                .result
                                .as_ref()
                                .or(execution.original_result.as_ref()),
                        ));
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
    render_combined_llm_search_results_with_summary(outcomes, None, &[])
}

fn render_combined_llm_search_results_with_summary(
    outcomes: &[CollectionSearchOutcome],
    parent_summary: Option<&str>,
    diagnostics: &[String],
) -> String {
    if outcomes.len() == 1 {
        return match outcomes.first() {
            Some(outcome) => match outcome.execution.as_ref() {
                Ok(execution) => render_collection_outcome_result(
                    &outcome.collection_id,
                    &outcome.collection_label,
                    execution,
                ),
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
    for diagnostic in diagnostics {
        lines.push(format!("diagnostic: {diagnostic}"));
    }
    if let Some(summary) = normalize_parent_summary(parent_summary) {
        lines.push(format!("summary: {summary}"));
    } else if let Some(summary) = fallback_parent_summary(outcomes) {
        lines.push(format!("summary: {summary}"));
    }

    if let Some((outcome, result)) =
        outcomes
            .iter()
            .find_map(|outcome| match outcome.execution.as_ref() {
                Ok(execution) if execution.is_usable() => {
                    execution.result.as_ref().map(|result| (outcome, result))
                }
                Err(_) => None,
                _ => None,
            })
    {
        if let Some(search_result) = result.search_results.first() {
            lines.push(format!("selected_result_uri: {}", search_result.uri));
            if let Some(source_collection_id) = search_result.source_collection_id.as_deref() {
                lines.push(format!(
                    "selected_result_source_collection_id: {source_collection_id}"
                ));
            }
        }
        lines.push(format!(
            "selected_result_collection_id: {}",
            outcome.collection_id
        ));
        lines.push(format!(
            "selected_result_collection_label: {}",
            outcome.collection_label
        ));
    }

    for outcome in outcomes {
        lines.push(String::new());
        match outcome.execution.as_ref() {
            Ok(execution) => lines.push(render_collection_outcome_result(
                &outcome.collection_id,
                &outcome.collection_label,
                execution,
            )),
            Err(err) => lines.push(format!(
                "collection_id: {}\ncollection_label: {}\nstatus: failed\nerror: {}",
                outcome.collection_id, outcome.collection_label, err
            )),
        }
    }

    lines.join("\n")
}

fn summarize_progress_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join("\n")
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
    .with_refresh_state(collection.last_refreshed_at, collection.refresh_ttl_seconds)
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

fn optional_bool_arg(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
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
        BlueskyTools, CollectionReviewStatus, LlmSearchExecution, ToolRegistry, detect_actor_refs,
        detect_post_uri, fallback_llm_search_summary, heuristic_collection_review,
        merged_collection_from_refs, parse_collection_review_verdict, parse_llm_search_result,
        parse_prompt_tool_call, reduced_search_collection,
        render_internal_llm_search_tool_protocol, render_llm_result, render_post_details,
        serialize_collection, source_collection_id_from_post, validate_collection_id,
        validate_internal_tool_response,
    };
    use crate::harness::context_window::{BuiltContextWindow, ProviderContextLimits};
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
    fn fallback_llm_search_summary_for_lists_is_paragraph_like() {
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
                    body: "list_name: engagement bait\nlist_description: repetitive clout-seeking posting"
                        .to_string(),
                },
            ],
        )
        .with_collection_kind("clearsky_lists");

        let summary = fallback_llm_search_summary(
            &collection,
            &[
                crate::harness::tools::LlmSearchResultItem {
                    uri: "https://example.com/list-a".to_string(),
                    source_collection_id: Some("clearsky:test".to_string()),
                },
                crate::harness::tools::LlmSearchResultItem {
                    uri: "https://example.com/list-b".to_string(),
                    source_collection_id: Some("clearsky:test".to_string()),
                },
            ],
        );

        assert!(summary.contains("moderation-list collection"));
        assert!(summary.contains("accused troll"));
        assert!(summary.contains("engagement bait"));
        assert!(summary.contains("structured `summary:` field"));
    }

    #[test]
    fn renders_tool_inventory_from_registry() {
        let rendered = ToolRegistry::harness_defaults().render_for_prompt();
        assert!(rendered.contains("Tool: llm_search"));
    }

    #[test]
    fn parses_prompt_tool_call_block() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: llm_search\nargs: {\"query\":\"who is rei-cast.xyz?\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(tool_call.args["query"], "who is rei-cast.xyz?");
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
            "TOOL_CALL\nname: llm_search\nargs:\n{query:<|\"|>what are people on Bluesky saying about topic x<|\"|>}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(
            tool_call.args["query"],
            "what are people on Bluesky saying about topic x"
        );
    }

    #[test]
    fn parses_first_tool_call_when_trailing_thought_continues() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: list_collections\nargs: {actor_did: \"did:plc:testactor\"}\n\n<|channel>thought\nextra commentary\n<channel|>TOOL_CALL\nname: llm_search\nargs: {query:\"who is rei-cast.xyz?\"}",
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
        let tools = BlueskyTools::new();

        let collections = tools
            .collections_from_ids(
                &store,
                &[
                    "recent:test".to_string(),
                    "replies_to_actor:did:plc:testactor".to_string(),
                ],
            )
            .expect("expected recent collection to survive optional replies_to_actor miss");

        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, "recent:test");
    }

    #[test]
    fn collections_from_ids_uses_recent_replies_received_for_legacy_replies_to_actor_id() {
        let mut store = NotificationStore::new();
        store.cache_post_collection(
            LabeledPostCollection::new(
                "recent_replies_received:did:plc:testactor",
                "Recent replies received by did:plc:testactor",
                vec![PostRecord {
                    uri: "at://reply".to_string(),
                    author_handle: "other.test".to_string(),
                    body: "reply_text: hello".to_string(),
                }],
            )
            .with_collection_kind("recent_replies_received")
            .with_actor_did("did:plc:testactor"),
        );
        let tools = BlueskyTools::new();

        let collections = tools
            .collections_from_ids(&store, &["replies_to_actor:did:plc:testactor".to_string()])
            .expect("expected legacy replies_to_actor to resolve");

        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].collection_kind, "recent_replies_received");
    }

    #[test]
    fn detect_actor_refs_trims_possessive_handle_suffix() {
        let refs = detect_actor_refs("what are people saying about elsyluna.bsky.social's post?")
            .expect("expected actor ref");

        assert_eq!(refs, vec!["elsyluna.bsky.social"]);
    }

    #[test]
    fn validates_collection_id_rejects_truncated_actor_profile_did() {
        let err = validate_collection_id("actor_profile:did:3de...")
            .expect_err("expected invalid collection id");
        assert!(err.contains("invalid DID segment"));
    }

    #[test]
    fn strict_internal_tool_response_rejects_surrounding_prose() {
        let response = "I will search now.\n\nTOOL_CALL\nname: collection_search\nargs: {\"collection_id\":\"clearsky_lists:did:plc:testactor\",\"prompt\":\"check lists\"}";
        let result = validate_internal_tool_response(response);
        assert!(matches!(result, super::InternalToolResponse::Invalid(_)));
    }

    #[test]
    fn parses_collection_review_verdict_block() {
        let verdict = parse_collection_review_verdict(
            "status: fail\nreason: summary is metadata-heavy\nrepair_needed: true\nrepair_instructions: cite the actual list names",
        )
        .expect("expected verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(verdict.repair_needed);
    }

    #[test]
    fn heuristic_collection_review_fails_metadata_heavy_summary() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky",
            vec![PostRecord {
                uri: "https://example.com/list-a".to_string(),
                author_handle: "clearsky".to_string(),
                body: "list_name: AI Fanatics\nlist_description: magical thinking".to_string(),
            }],
        );
        let result = super::LlmSearchResult {
            title: "weak".to_string(),
            summary:
                "collection_id: clearsky:test did:plc:testactor source_post_uri: at://foo item[0]"
                    .to_string(),
            search_results: vec![super::LlmSearchResultItem {
                uri: "https://example.com/list-a".to_string(),
                source_collection_id: Some("clearsky:test".to_string()),
            }],
        };
        let execution = LlmSearchExecution {
            result: Some(result.clone()),
            original_result: Some(result),
            context_window: empty_test_window(),
            diagnostic: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(&collection, &execution);
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
    }

    #[test]
    fn heuristic_collection_review_passes_grounded_list_summary() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky",
            vec![PostRecord {
                uri: "https://example.com/list-a".to_string(),
                author_handle: "clearsky".to_string(),
                body: "list_name: AI Fanatics\nlist_description: magical thinking".to_string(),
            }],
        );
        let result = parse_llm_search_result(
            &collection,
            "title: grounded\nsummary: The matched list names the actor under \"AI Fanatics\" and the description says \"magical thinking,\" which is explicit negative list evidence.\nuri: https://example.com/list-a",
        )
        .expect("expected result");
        let execution = LlmSearchExecution {
            result: Some(result.clone()),
            original_result: Some(result),
            context_window: empty_test_window(),
            diagnostic: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(&collection, &execution);
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
    }

    fn empty_test_window() -> BuiltContextWindow {
        BuiltContextWindow {
            rendered: String::new(),
            sections: Vec::new(),
            header_tokens: 0,
            used_input_tokens: 0,
            truncated: false,
            limits: ProviderContextLimits {
                provider_name: "test".to_string(),
                model_name: "test".to_string(),
                max_context_tokens: 0,
                reserved_output_tokens: 0,
            },
        }
    }

    #[test]
    fn detect_post_uri_finds_bsky_post_uri_inside_query() {
        let post_uri = detect_post_uri(
            "what are people saying about this post (at://did:plc:test/app.bsky.feed.post/3abc)?",
        )
        .expect("expected post uri");

        assert_eq!(post_uri, "at://did:plc:test/app.bsky.feed.post/3abc");
    }

    #[test]
    fn internal_llm_search_tool_protocol_lists_planner_tools() {
        let rendered = render_internal_llm_search_tool_protocol();

        assert!(rendered.contains("resolve_actor_refs"));
        assert!(rendered.contains("hydrate_actor_scope"));
        assert!(rendered.contains("collection_search"));
        assert!(rendered.contains("search_global_posts"));
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
