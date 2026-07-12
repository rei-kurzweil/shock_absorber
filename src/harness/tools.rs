use crate::harness::agents::{AgentNodeKind, AgentNodeTemplate};
use crate::harness::context_window::{
    BuiltContextWindow, ContextSectionKind, LLMContext, build_context_window_report,
};
use crate::harness::context_window_logger::append_debug_trace;
use crate::harness::llm_api::{ChatCompletionResponseFormat, ChatMessage, LlmApiClient};
use crate::harness::r#loop::collection_summary::render_summary_collection_loop_result;
use crate::harness::prompts::{AgentKind, tool_prompt};
use crate::harness::search as harness_search;
use crate::harness::summary as harness_summary;
use crate::harness::tool_call_parser::{
    extract_leading_tool_call_block, parse_prompt_tool_call as parse_prompt_tool_call_result,
};
use crate::harness::units::UnitInstanceState;
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
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

pub use crate::harness::tool_call_parser::PromptToolCall;

fn append_summary_trace(entry: impl AsRef<str>) {
    let debug_base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = append_debug_trace(&debug_base_dir, "summary_trace.md", entry.as_ref());
}

fn summary_result_presence(result: Option<&CollectionLeafResult>) -> &'static str {
    match result {
        Some(CollectionLeafResult::Summary(_)) => "summary",
        Some(CollectionLeafResult::Search(_)) => "search",
        Some(CollectionLeafResult::RawWindow(_)) => "raw_window",
        None => "none",
    }
}

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
                name: "search".to_string(),
                description: "Search Bluesky at a high level for selective grounded evidence about handles/users or broader topics.".to_string(),
                arguments: vec![
                    ToolArgumentSpec {
                        name: "query".to_string(),
                        value_type: "string".to_string(),
                        description: "The user's search request in natural language, such as who a handle is or what Bluesky posts say about a topic.".to_string(),
                        required: true,
                    },
                ],
                when_to_use: "Use when you need selective Bluesky-grounded evidence about one or more handles/users, or about a broader topic that requires searching posts.".to_string(),
                notes: vec![
                    "The root agent only supplies the high-level query; the harness decides whether to do handle lookup, actor-centric collection search, or broader Bluesky post search.".to_string(),
                    "If the query names a handle or user, the search should anchor on that actor's profile and may inspect posts for grounding.".to_string(),
                    "If the query is topical rather than person-centric, the search may use Bluesky-wide post search and normalize the results into a collection before running narrower LLM search.".to_string(),
                    "When a collection contains structured fields such as `list_name` or `list_description`, use those exact fields as evidence instead of inventing new labels or categories.".to_string(),
                    "Returns one synthesized block with a chosen URI plus grounded evidence snippets or repeated themes from the matching items.".to_string(),
                ],
            },
            ToolSpec {
                name: "summary".to_string(),
                description: "Summarize an actor-backed Bluesky collection with broad grounded coverage, such as the last 50 posts by a handle.".to_string(),
                arguments: vec![ToolArgumentSpec {
                    name: "query".to_string(),
                    value_type: "string".to_string(),
                    description: "The user's coverage-oriented summary request in natural language.".to_string(),
                    required: true,
                }],
                when_to_use: "Use when the user explicitly asks for broad coverage such as summarizing recent posts, replies, pages, or the last N posts by an actor.".to_string(),
                notes: vec![
                    "The root agent only supplies the high-level query; the harness resolves the actor, hydrates the actor scope, picks the target collection, and delegates coverage work to `collection_summary`.".to_string(),
                    "The first summary slice is actor-centric and defaults to the actor's `recent_posts` collection unless the query explicitly asks for replies or another collection target.".to_string(),
                    "Returns a grounded coverage summary with covered item URIs and source-exhaustion metadata when applicable.".to_string(),
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
pub struct LlmSummaryResult {
    pub title: String,
    pub summary: String,
    pub covered_item_uris: Vec<String>,
    pub omitted_item_uris: Vec<String>,
    pub concatenated_window_summaries: Option<String>,
    pub window_offset: Option<usize>,
    pub window_size: Option<usize>,
    pub page_index: Option<usize>,
    pub page_size: Option<usize>,
    pub collection_total_items: Option<usize>,
    pub has_more: Option<bool>,
    pub source_exhausted: Option<bool>,
    pub window_start: Option<usize>,
    pub window_total_items: Option<usize>,
}

impl LlmSummaryResult {
    pub(crate) fn processed_window_offset(&self) -> Option<usize> {
        self.window_offset.or(self.window_start)
    }

    pub(crate) fn processed_window_size(&self) -> Option<usize> {
        self.window_size.or(self.window_total_items)
    }

    pub(crate) fn processed_page_size(&self) -> Option<usize> {
        self.page_size.or(Some(COLLECTION_SEARCH_PAGE_SIZE))
    }

    pub(crate) fn processed_page_index(&self) -> Option<usize> {
        self.page_index.or_else(|| {
            self.processed_window_offset()
                .zip(self.processed_page_size())
                .map(|(offset, page_size)| offset / page_size.max(1))
        })
    }

    pub(crate) fn processed_collection_total_items(&self) -> Option<usize> {
        self.collection_total_items
            .or_else(|| self.processed_window_size())
    }

    pub(crate) fn concatenated_window_summaries(&self) -> Option<&str> {
        self.concatenated_window_summaries.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionRawWindowResult {
    pub title: String,
    pub summary: String,
    pub window_offset: usize,
    pub window_size: usize,
    pub page_index: usize,
    pub page_size: usize,
    pub collection_total_items: usize,
    pub has_more: bool,
    pub failure_reason: String,
    pub raw_summary_response: Option<String>,
    pub records: Vec<PostRecord>,
}

impl CollectionRawWindowResult {
    pub(crate) fn processed_window_offset(&self) -> usize {
        self.window_offset
    }

    pub(crate) fn processed_window_size(&self) -> usize {
        self.window_size
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionLeafResult {
    Search(LlmSearchResult),
    Summary(LlmSummaryResult),
    RawWindow(CollectionRawWindowResult),
}

impl CollectionLeafResult {
    fn title(&self) -> &str {
        match self {
            Self::Search(result) => &result.title,
            Self::Summary(result) => &result.title,
            Self::RawWindow(result) => &result.title,
        }
    }

    fn summary(&self) -> &str {
        match self {
            Self::Search(result) => &result.summary,
            Self::Summary(result) => &result.summary,
            Self::RawWindow(result) => &result.summary,
        }
    }

    fn as_search(&self) -> Option<&LlmSearchResult> {
        match self {
            Self::Search(result) => Some(result),
            Self::Summary(_) => None,
            Self::RawWindow(_) => None,
        }
    }

    pub(crate) fn as_summary(&self) -> Option<&LlmSummaryResult> {
        match self {
            Self::Search(_) => None,
            Self::Summary(result) => Some(result),
            Self::RawWindow(_) => None,
        }
    }

    pub(crate) fn as_raw_window(&self) -> Option<&CollectionRawWindowResult> {
        match self {
            Self::Search(_) => None,
            Self::Summary(_) => None,
            Self::RawWindow(result) => Some(result),
        }
    }

    fn selected_item_uris(&self) -> Vec<&str> {
        match self {
            Self::Search(result) => result
                .search_results
                .iter()
                .map(|item| item.uri.as_str())
                .collect(),
            Self::Summary(result) => result
                .covered_item_uris
                .iter()
                .map(String::as_str)
                .collect(),
            Self::RawWindow(result) => result
                .records
                .iter()
                .map(|post| post.uri.as_str())
                .collect(),
        }
    }

    pub(crate) fn processed_window_offset(&self) -> Option<usize> {
        match self {
            Self::Search(_) => None,
            Self::Summary(result) => result.processed_window_offset(),
            Self::RawWindow(result) => Some(result.processed_window_offset()),
        }
    }

    pub(crate) fn processed_window_size(&self) -> Option<usize> {
        match self {
            Self::Search(_) => None,
            Self::Summary(result) => result.processed_window_size(),
            Self::RawWindow(result) => Some(result.processed_window_size()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionReviewStatus {
    Pass,
    Fail,
}

impl CollectionReviewStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionReviewVerdict {
    pub status: CollectionReviewStatus,
    pub grounded: bool,
    pub sufficient: bool,
    pub reason: String,
    pub repair_needed: bool,
    pub repair_instructions: Option<String>,
    pub additional_pages_needed: bool,
    pub next_page: Option<usize>,
    pub next_offset: Option<usize>,
    pub required_total_items: Option<usize>,
}

pub struct LlmSearchExecution {
    pub result: Option<CollectionLeafResult>,
    pub original_result: Option<CollectionLeafResult>,
    pub context_window: BuiltContextWindow,
    pub diagnostic: Option<String>,
    pub raw_response: Option<String>,
    pub review_verdict: Option<CollectionReviewVerdict>,
    pub review_context_window: Option<BuiltContextWindow>,
    pub repair_diagnostic: Option<String>,
}

impl LlmSearchExecution {
    pub(crate) fn is_usable(&self) -> bool {
        self.result.is_some()
            && !matches!(
                self.review_verdict.as_ref().map(|verdict| &verdict.status),
                Some(CollectionReviewStatus::Fail)
            )
    }

    pub(crate) fn has_warnings(&self) -> bool {
        self.diagnostic
            .as_deref()
            .is_some_and(diagnostic_counts_as_warning)
            || result_uses_fallback_summary(self)
    }
}

pub struct ToolExecutionOutput {
    pub rendered: String,
    pub context_windows: Vec<ToolContextWindow>,
    pub agent_node: Option<AgentNodeTemplate>,
    pub unit_node: Option<UnitInstanceState>,
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

struct ParsedCollectionToolResult {
    result: Option<CollectionLeafResult>,
    diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedTaggedSummaryFields {
    title: Option<String>,
    summary: Option<String>,
    covered_item_uris: Vec<String>,
    omitted_item_uris: Vec<String>,
    window_offset: Option<usize>,
    window_size: Option<usize>,
    page_index: Option<usize>,
    page_size: Option<usize>,
    collection_total_items: Option<usize>,
    has_more: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TaggedSummaryBody {
    fields: ParsedTaggedSummaryFields,
    missing_end_marker: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollectionLeafToolKind {
    Search,
    Summary,
}

impl CollectionLeafToolKind {
    pub(crate) fn tool_name(self) -> &'static str {
        match self {
            Self::Search => "collection_search",
            Self::Summary => "collection_summary",
        }
    }

    pub(crate) fn label_prefix(self) -> &'static str {
        match self {
            Self::Search => "collection search",
            Self::Summary => "collection summary",
        }
    }

    pub(crate) fn agent_kind(self) -> AgentKind {
        match self {
            Self::Search => AgentKind::CollectionSearch,
            Self::Summary => AgentKind::CollectionSummary,
        }
    }

    pub(crate) fn node_kind(self) -> AgentNodeKind {
        match self {
            Self::Search => AgentNodeKind::CollectionSearchTool,
            Self::Summary => AgentNodeKind::CollectionSummaryTool,
        }
    }

    pub(crate) fn review_agent_kind(self) -> AgentKind {
        match self {
            Self::Search => AgentKind::SearchReview,
            Self::Summary => AgentKind::SummaryReview,
        }
    }

    pub(crate) fn review_node_kind(self) -> AgentNodeKind {
        match self {
            Self::Search => AgentNodeKind::SearchReviewAgent,
            Self::Summary => AgentNodeKind::SummaryReviewAgent,
        }
    }

    pub(crate) fn review_label(self) -> &'static str {
        match self {
            Self::Search => "search review",
            Self::Summary => "summary review",
        }
    }

    fn is_coverage_oriented(self) -> bool {
        matches!(self, Self::Summary)
    }

    fn response_format(self) -> Option<ChatCompletionResponseFormat> {
        match self {
            Self::Search => Some(ChatCompletionResponseFormat::JsonObject),
            Self::Summary => None,
        }
    }
}

#[derive(Clone, Debug)]
struct ResolvedActorRef {
    actor_ref: String,
    handle: String,
    did: Did,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ActorAnchorSource {
    ExplicitQueryRef,
    SelectedActorFallback,
}

impl ActorAnchorSource {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::ExplicitQueryRef => "explicit_query_ref",
            Self::SelectedActorFallback => "selected_actor_fallback",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedActorAnchor {
    pub(crate) actor_ref: Option<String>,
    pub(crate) actor_did: Did,
    pub(crate) actor_handle: Option<String>,
    pub(crate) source: ActorAnchorSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PreparedSummaryActor {
    Resolved(PreparedActorAnchor),
    ExplicitRef(String),
}

impl PreparedSummaryActor {
    pub(crate) fn source(&self) -> ActorAnchorSource {
        match self {
            Self::Resolved(anchor) => anchor.source.clone(),
            Self::ExplicitRef(_) => ActorAnchorSource::ExplicitQueryRef,
        }
    }

    pub(crate) fn explicit_actor_ref(&self) -> Option<&str> {
        match self {
            Self::Resolved(anchor) => anchor.actor_ref.as_deref(),
            Self::ExplicitRef(actor_ref) => Some(actor_ref.as_str()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SummaryCollectionTargetHint {
    RecentPosts,
    Replies,
    PinnedPosts,
    Profile,
    Lists,
}

impl SummaryCollectionTargetHint {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::RecentPosts => "recent_posts",
            Self::Replies => "recent_replies_received",
            Self::PinnedPosts => "pinned_posts",
            Self::Profile => "actor_profile",
            Self::Lists => "clearsky_lists",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedSearchInput {
    pub(crate) original_query: String,
    pub(crate) actor_anchor: Option<PreparedActorAnchor>,
    pub(crate) scope_hints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedSummaryInput {
    pub(crate) original_query: String,
    pub(crate) actor: PreparedSummaryActor,
    pub(crate) requested_summary_scope: RequestedSummaryScope,
    pub(crate) collection_target_hint: SummaryCollectionTargetHint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PreparedPromptToolInput {
    Search(PreparedSearchInput),
    Summary(PreparedSummaryInput),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ToolPrepMissingPrerequisite {
    Query,
    ActorAnchor,
    SummaryScope,
    CollectionTarget,
}

impl ToolPrepMissingPrerequisite {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::ActorAnchor => "actor_anchor",
            Self::SummaryScope => "summary_scope",
            Self::CollectionTarget => "collection_target",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ToolPrepFailure {
    pub(crate) tool_name: String,
    pub(crate) attempt_count: usize,
    pub(crate) missing: Vec<ToolPrepMissingPrerequisite>,
    pub(crate) actor_anchor_source: Option<ActorAnchorSource>,
    pub(crate) tried: Vec<String>,
}

pub(crate) struct CollectionToolOutcome {
    pub(crate) tool_kind: CollectionLeafToolKind,
    pub(crate) collection_id: String,
    pub(crate) collection_label: String,
    pub(crate) execution: Result<LlmSearchExecution, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestedSummaryScope {
    CurrentWindow,
    Count { requested_items: usize },
    Page { page_index: usize },
    PageRange { start_page: usize, end_page: usize },
}

#[derive(Debug, Deserialize)]
struct SummaryPrepResolution {
    requested_summary_scope: SummaryPrepScopePayload,
    collection_target_hint: SummaryPrepCollectionTargetPayload,
}

struct PreparedSummaryRequest {
    requested_summary_scope: RequestedSummaryScope,
    collection_target_hint: SummaryCollectionTargetHint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SummarySelectionReviewStatus {
    Accepted,
    Repaired,
    Rejected,
}

impl SummarySelectionReviewStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Repaired => "repaired",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExplicitRepliesTarget {
    Sent,
    Received,
    Any,
}

impl ExplicitRepliesTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sent => "recent_replies_sent",
            Self::Received => "recent_replies_received",
            Self::Any => "replies",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExplicitSummaryCollectionIntent {
    RecentPosts,
    Replies(ExplicitRepliesTarget),
    PinnedPosts,
    Profile,
    Lists,
    Ambiguous,
}

impl ExplicitSummaryCollectionIntent {
    fn as_str(self) -> &'static str {
        match self {
            Self::RecentPosts => "recent_posts",
            Self::Replies(target) => target.as_str(),
            Self::PinnedPosts => "pinned_posts",
            Self::Profile => "actor_profile",
            Self::Lists => "clearsky_lists",
            Self::Ambiguous => "ambiguous",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SummaryCollectionSelectionReview {
    status: SummarySelectionReviewStatus,
    reason: String,
    original_collection_id: String,
    original_collection_kind: String,
    final_collection_id: String,
    final_collection_kind: String,
    deterministic_repair_applied: bool,
}

#[derive(Debug, Deserialize)]
struct SummaryCollectionSelectionReviewPayload {
    status: String,
    #[serde(default)]
    final_collection_id: Option<String>,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct SummaryPrepScopePayload {
    kind: String,
    #[serde(default)]
    requested_items: Option<usize>,
    #[serde(default)]
    page_index: Option<usize>,
    #[serde(default)]
    start_page: Option<usize>,
    #[serde(default)]
    end_page: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SummaryPrepCollectionTargetPayload {
    RecentPosts,
    Replies,
    PinnedPosts,
    Profile,
    Lists,
}

impl RequestedSummaryScope {
    pub(crate) fn render_for_planner(self) -> String {
        match self {
            Self::CurrentWindow => "kind: current_window".to_string(),
            Self::Count { requested_items } => {
                format!("kind: count\nrequested_items: {requested_items}")
            }
            Self::Page { page_index } => format!("kind: page\npage_index: {page_index}"),
            Self::PageRange {
                start_page,
                end_page,
            } => {
                format!("kind: page_range\nstart_page: {start_page}\nend_page: {end_page}")
            }
        }
    }
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

struct LlmSearchComparator<'a> {
    tool_kind: CollectionLeafToolKind,
    prompt: &'a str,
    llm_client: &'a LlmApiClient,
    max_output_tokens: usize,
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
                raw_response: None,
                review_verdict: None,
                review_context_window: None,
                repair_diagnostic: None,
            });
        }

        let mut context = LLMContext::new(self.tool_kind.agent_kind().system_prompt());
        context.push_section_with_kind(
            "Collection",
            ContextSectionKind::CollectionEvidence,
            serialize_collection(collection),
        );
        context.push_section_with_kind(
            "Search Prompt",
            ContextSectionKind::CurrentTask,
            self.prompt,
        );

        let context_window =
            build_context_window_report(&context, &self.llm_client.context_limits());
        let rendered_context = context_window.rendered.clone();
        let response = self
            .llm_client
            .complete_chat_with_response_format(
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
                self.tool_kind.response_format(),
            )
            .await?;

        let parsed = parse_collection_tool_result(
            collection,
            &response,
            self.tool_kind,
            collection_window_offset(collection),
        );
        if self.tool_kind == CollectionLeafToolKind::Summary {
            append_summary_trace(format!(
                "[summary_leaf_parse]\ncollection_id: {}\nwindow_offset: {}\nresult_present: {}\noriginal_result_kind: {}\ndiagnostic: {}\nraw_response:\n{}",
                collection.id,
                collection_window_offset(collection).unwrap_or(0),
                parsed.result.is_some(),
                summary_result_presence(parsed.result.as_ref()),
                parsed.diagnostic.as_deref().unwrap_or("<none>"),
                truncate_diagnostic_block(&response, 4000)
            ));
        }
        Ok(LlmSearchExecution {
            result: parsed.result.clone(),
            original_result: parsed.result,
            context_window,
            diagnostic: parsed.diagnostic,
            raw_response: Some(response),
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        })
    }
}

pub struct BlueskyTools;

pub(crate) const COLLECTION_SEARCH_PAGE_SIZE: usize = 50;
const MAX_COLLECTION_SEARCH_RESULTS: usize = 10;
const ACTOR_SEARCH_POST_TARGET: usize = 25;
const ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT: usize = 100;
const ACTOR_SCOPE_REPLY_FETCH_LIMIT: usize = 100;
const MAX_INVALID_INTERNAL_TOOL_CALLS: usize = 2;
const INTERNAL_TOOL_REPAIR_MAX_OUTPUT_TOKENS: usize = 384;

impl BlueskyTools {
    pub fn new() -> Self {
        Self
    }

    async fn run_collection_tool(
        &self,
        tool_kind: CollectionLeafToolKind,
        collection: &LabeledPostCollection,
        prompt: &str,
        llm_client: &LlmApiClient,
    ) -> Result<LlmSearchExecution, Box<dyn std::error::Error>> {
        let primary = LlmSearchComparator {
            tool_kind,
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
                    tool_kind,
                    prompt,
                    llm_client,
                    max_output_tokens: 512,
                };
                let mut retried = retry.compare(&reduced).await.map_err(|retry_err| -> Box<dyn std::error::Error> {
                    format!(
                        "{} failed on full collection ({primary_err}) and reduced retry ({retry_err})",
                        tool_kind.tool_name(),
                    )
                    .into()
                })?;
                retried.diagnostic = Some(format!(
                    "Primary full-collection {} failed and a reduced retry view was used instead. Primary failure: {primary_err}",
                    tool_kind.tool_name(),
                ));
                Ok(retried)
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

    pub(crate) async fn prepare_root_tool_input(
        &self,
        tool_call: &PromptToolCall,
        selected_actor_did: Option<&Did>,
        store: &NotificationStore,
        llm_client: &LlmApiClient,
    ) -> Result<PreparedPromptToolInput, ToolPrepFailure> {
        match tool_call.name.as_str() {
            "search" => {
                let query =
                    require_string_arg(&tool_call.args, "query").map_err(|_| ToolPrepFailure {
                        tool_name: "search".to_string(),
                        attempt_count: 1,
                        missing: vec![ToolPrepMissingPrerequisite::Query],
                        actor_anchor_source: None,
                        tried: vec!["validate_query".to_string()],
                    })?;
                let actor_anchor = resolve_prepared_actor_anchor(&query, selected_actor_did, store);
                let mut scope_hints = Vec::new();
                if let Some(anchor) = actor_anchor.as_ref() {
                    scope_hints.push(format!("prepared_actor_did: {}", anchor.actor_did.as_str()));
                    if let Some(handle) = anchor.actor_handle.as_deref() {
                        scope_hints.push(format!("prepared_actor_handle: {handle}"));
                    }
                    scope_hints.push(format!(
                        "prepared_actor_anchor_source: {}",
                        anchor.source.as_str()
                    ));
                }
                Ok(PreparedPromptToolInput::Search(PreparedSearchInput {
                    original_query: query,
                    actor_anchor,
                    scope_hints,
                }))
            }
            "summary" => {
                let query =
                    require_string_arg(&tool_call.args, "query").map_err(|_| ToolPrepFailure {
                        tool_name: "summary".to_string(),
                        attempt_count: 1,
                        missing: vec![ToolPrepMissingPrerequisite::Query],
                        actor_anchor_source: None,
                        tried: vec!["validate_query".to_string()],
                    })?;
                let resolved_summary_request = self
                    .resolve_summary_prep_request(&query, llm_client)
                    .await
                    .map_err(|message| ToolPrepFailure {
                        tool_name: "summary".to_string(),
                        attempt_count: 1,
                        missing: vec![ToolPrepMissingPrerequisite::SummaryScope],
                        actor_anchor_source: None,
                        tried: vec![format!("resolve_summary_request: {message}")],
                    })?;
                let Some(actor) = resolve_prepared_summary_actor(&query, selected_actor_did, store)
                else {
                    return Err(ToolPrepFailure {
                        tool_name: "summary".to_string(),
                        attempt_count: 1,
                        missing: vec![ToolPrepMissingPrerequisite::ActorAnchor],
                        actor_anchor_source: None,
                        tried: vec![
                            "resolve_explicit_actor_ref".to_string(),
                            "resolve_selected_actor_fallback".to_string(),
                        ],
                    });
                };
                Ok(PreparedPromptToolInput::Summary(PreparedSummaryInput {
                    original_query: query,
                    actor,
                    requested_summary_scope: resolved_summary_request.requested_summary_scope,
                    collection_target_hint: resolved_summary_request.collection_target_hint,
                }))
            }
            other => Err(ToolPrepFailure {
                tool_name: other.to_string(),
                attempt_count: 1,
                missing: Vec::new(),
                actor_anchor_source: None,
                tried: vec!["unsupported_tool".to_string()],
            }),
        }
    }

    pub(crate) fn prepared_actor_anchor(
        prepared_input: &PreparedPromptToolInput,
    ) -> Option<&PreparedActorAnchor> {
        match prepared_input {
            PreparedPromptToolInput::Search(prepared) => prepared.actor_anchor.as_ref(),
            PreparedPromptToolInput::Summary(prepared) => match &prepared.actor {
                PreparedSummaryActor::Resolved(anchor) => Some(anchor),
                PreparedSummaryActor::ExplicitRef(_) => None,
            },
        }
    }

    pub(crate) fn prepared_actor_anchor_source(
        prepared_input: &PreparedPromptToolInput,
    ) -> Option<ActorAnchorSource> {
        match prepared_input {
            PreparedPromptToolInput::Search(prepared) => {
                prepared.actor_anchor.as_ref().map(|anchor| anchor.source.clone())
            }
            PreparedPromptToolInput::Summary(prepared) => Some(prepared.actor.source()),
        }
    }

    pub(crate) fn prepared_recent_post_requirements(
        prepared_input: &PreparedPromptToolInput,
    ) -> Option<(usize, usize)> {
        match prepared_input {
            PreparedPromptToolInput::Summary(prepared) => Some(summary_recent_post_requirements(
                prepared.requested_summary_scope,
            )),
            PreparedPromptToolInput::Search(_) => None,
        }
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
        self.execute_prepared_prompt_tool_call(
            tool_call,
            None,
            selected_notification,
            agent,
            store,
            llm_client,
            observer,
        )
        .await
    }

    pub async fn execute_prepared_prompt_tool_call(
        &self,
        tool_call: &PromptToolCall,
        prepared_input: Option<&PreparedPromptToolInput>,
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
                unit_node: None,
            }),
            "search" => {
                let query = require_string_arg(&tool_call.args, "query")?;
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "search tool agent".to_string(),
                        depth: 1,
                        content: format!("query:\n{query}\n\nstatus: running"),
                    });
                }
                let prepared_search = match prepared_input {
                    Some(PreparedPromptToolInput::Search(prepared)) => Some(prepared.clone()),
                    _ => None,
                };
                let (rendered, outcomes) = self
                    .execute_internal_search(
                        &query,
                        prepared_search.as_ref(),
                        selected_notification,
                        agent,
                        store,
                        llm_client,
                        observer.clone(),
                    )
                    .await?;
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "search tool agent".to_string(),
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
                                title: format!("search: {}", outcome.collection_label),
                                window: execution.context_window.clone(),
                            }),
                            Err(_) => None,
                        })
                        .collect(),
                    agent_node: Some(build_search_agent_node(&query, &outcomes, llm_client)),
                    unit_node: Some(harness_search::build_search_unit_instance(
                        &query, &outcomes, llm_client,
                    )),
                })
            }
            "summary" => {
                let query = require_string_arg(&tool_call.args, "query")?;
                let prepared_summary = match prepared_input {
                    Some(PreparedPromptToolInput::Summary(prepared)) => Some(prepared.clone()),
                    _ => None,
                };
                let summary_input = if let Some(prepared) = prepared_summary.as_ref() {
                    prepared
                } else {
                    return Err("summary requires a prepared actor-backed input".into());
                };
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "summary tool agent".to_string(),
                        depth: 1,
                        content: format!("query:\n{query}\n\nstatus: running"),
                    });
                }
                let (rendered, outcomes) = self
                    .execute_public_summary(
                        summary_input,
                        agent,
                        store,
                        llm_client,
                        observer.clone(),
                    )
                    .await?;
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "summary tool agent".to_string(),
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
                                title: format!("summary: {}", outcome.collection_label),
                                window: execution.context_window.clone(),
                            }),
                            Err(_) => None,
                        })
                        .collect(),
                    agent_node: Some(build_summary_agent_node(&query, &outcomes, llm_client)),
                    unit_node: Some(harness_summary::build_summary_unit_instance(
                        &query, &outcomes, llm_client,
                    )),
                })
            }
            other => Err(format!("unknown tool `{other}`").into()),
        }
    }

    async fn execute_internal_search(
        &self,
        query: &str,
        prepared_input: Option<&PreparedSearchInput>,
        selected_notification: Option<&Notification>,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<(String, Vec<CollectionToolOutcome>), Box<dyn std::error::Error>> {
        let search_intent = classify_search_intent(query);
        let requested_summary_scope = detect_requested_summary_scope(query);
        let resolved_actor_refs = if let Some(prepared_actor) =
            prepared_input.and_then(|input| input.actor_anchor.as_ref())
        {
            Some(vec![ResolvedActorRef {
                actor_ref: prepared_actor
                    .actor_ref
                    .clone()
                    .unwrap_or_else(|| prepared_actor.actor_did.as_str().to_string()),
                handle: prepared_actor
                    .actor_handle
                    .clone()
                    .unwrap_or_else(|| prepared_actor.actor_did.as_str().to_string()),
                did: prepared_actor.actor_did.clone(),
            }])
        } else if let Some(actor_refs) = detect_actor_refs(query) {
            Some(self.resolve_actor_refs(agent, store, &actor_refs).await?)
        } else {
            None
        };
        let initial_scope_hints = if let Some(resolved_actor_refs) = resolved_actor_refs.as_deref()
        {
            format!(
                "{}\npreferred_search_intent: {}\npreferred_search_order: {}",
                render_resolved_actor_refs(resolved_actor_refs),
                search_intent.as_str(),
                preferred_collection_order_hint(search_intent)
            )
        } else if let Some(post_uri) = detect_post_uri(query) {
            format!("detected_post_uri: {post_uri}")
        } else if let Some(prepared_input) = prepared_input {
            if prepared_input.scope_hints.is_empty() {
                "No actor refs detected from the query. Use `search_global_posts` if you need a Bluesky-wide topical search.".to_string()
            } else {
                prepared_input.scope_hints.join("\n")
            }
        } else {
            "No actor refs detected from the query. Use `search_global_posts` if you need a Bluesky-wide topical search.".to_string()
        };

        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "{}\n\n{}",
                    AgentKind::Search.system_prompt(),
                    render_internal_search_tool_protocol()
                ),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "User query:\n{query}\n\nInitial scope hints:\n{initial_scope_hints}\n\nHarness-owned requested summary scope:\n{}\n\nWhen you call `summary`, the harness will own paging for that collection automatically. Do not try to manage summary paging manually.\n\nUse tools when needed, then finish with a direct grounded synthesis.",
                    requested_summary_scope.render_for_planner()
                ),
            },
        ];

        let mut outcomes = Vec::new();
        let mut searched_collection_keys = HashSet::new();
        let mut final_summary = None;
        let mut diagnostics = Vec::new();
        let mut consecutive_invalid_tool_responses = 0usize;
        let mut consecutive_invalid_tool_calls = 0usize;

        for round in 0..6usize {
            let response = llm_client.complete_chat(messages.clone(), 768).await?;
            let accepted_tool_call = match validate_internal_tool_response(&response) {
                InternalToolResponse::FinalSummary(summary) => {
                    final_summary = Some(summary);
                    break;
                }
                InternalToolResponse::Invalid(reason) => {
                    consecutive_invalid_tool_responses += 1;
                    let diagnostic = format!(
                        "status: invalid_tool_call\nreason: {reason}\ninstruction: re-emit exactly one valid internal TOOL_CALL block with no extra prose\nraw_response:\n{}",
                        truncate_diagnostic_block(&response, 1200)
                    );
                    diagnostics.push(format!(
                        "internal planner validation failed: {reason}\nraw_response:\n{}",
                        truncate_diagnostic_block(&response, 500)
                    ));
                    if let Some(observer) = observer.as_ref() {
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: "search planner".to_string(),
                            depth: 2,
                            content: diagnostic.clone(),
                        });
                    }
                    if consecutive_invalid_tool_responses >= 2 {
                        diagnostics.push(
                            "internal planner produced repeated invalid tool-call formatting; falling back to harness-side collection resolution"
                                .to_string(),
                        );
                        break;
                    }
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
            consecutive_invalid_tool_responses = 0;

            if let Some(trailing) = accepted_tool_call.discarded_trailing.as_deref() {
                let diagnostic = format!(
                    "status: accepted_tool_call\nwarning: trailing planner output was discarded after the first valid TOOL_CALL block\naccepted_tool_call:\n{}\n\ndiscarded_trailing_output:\n{}",
                    accepted_tool_call.accepted_block,
                    truncate_diagnostic_block(trailing, 800)
                );
                diagnostics.push(format!(
                    "internal planner trailing output discarded after accepted tool call\naccepted_tool_call:\n{}\ndiscarded_trailing_output:\n{}",
                    accepted_tool_call.accepted_block,
                    truncate_diagnostic_block(trailing, 300)
                ));
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "search planner".to_string(),
                        depth: 2,
                        content: diagnostic,
                    });
                }
            }

            let accepted_tool_call = match validate_internal_search_tool_call(
                &accepted_tool_call.tool_call,
            ) {
                Ok(_) => accepted_tool_call,
                Err(reason) => {
                    let tool_args =
                        serde_json::to_string_pretty(&accepted_tool_call.tool_call.args)?;
                    let diagnostic = format!(
                        "status: invalid_tool_call\nname: {}\nreason: {}\ninstruction: re-emit exactly one valid tool call with corrected arguments\nraw_response:\n{}\nparsed_args:\n{}",
                        accepted_tool_call.tool_call.name,
                        reason,
                        truncate_diagnostic_block(&accepted_tool_call.accepted_block, 800),
                        tool_args
                    );
                    diagnostics.push(format!(
                        "internal planner validation failed: {reason}\nname: {}\nparsed_args: {}",
                        accepted_tool_call.tool_call.name,
                        truncate_diagnostic_block(&tool_args, 300)
                    ));
                    if let Some(observer) = observer.as_ref() {
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: format!(
                                "internal tool validation: {}",
                                accepted_tool_call.tool_call.name
                            ),
                            depth: 2,
                            content: diagnostic.clone(),
                        });
                    }
                    let repair_collections = relevant_repair_inventory_collections(
                        store,
                        resolved_actor_refs.as_deref(),
                    );
                    match self
                        .repair_invalid_internal_search_tool_call(
                            query,
                            search_intent,
                            requested_summary_scope,
                            resolved_actor_refs.as_deref().unwrap_or(&[]),
                            &repair_collections,
                            &accepted_tool_call,
                            &reason,
                            llm_client,
                            observer.clone(),
                        )
                        .await?
                    {
                        Some(repaired_tool_call) => {
                            diagnostics.push(format!(
                                "internal tool validation repaired invalid planner call\noriginal_name: {}\nreason: {}\nrepaired_name: {}\nrepaired_args: {}",
                                accepted_tool_call.tool_call.name,
                                reason,
                                repaired_tool_call.tool_call.name,
                                truncate_diagnostic_block(
                                    &serde_json::to_string(&repaired_tool_call.tool_call.args)?,
                                    300
                                )
                            ));
                            repaired_tool_call
                        }
                        None => {
                            consecutive_invalid_tool_calls += 1;
                            if let Some(repaired_tool_call) =
                                deterministic_repair_internal_search_tool_call(
                                    &accepted_tool_call.tool_call,
                                    query,
                                    search_intent,
                                    &repair_collections,
                                )
                            {
                                diagnostics.push(format!(
                                    "internal tool validation applied deterministic repair after repair model declined or failed\noriginal_name: {}\nreason: {}\nrepaired_name: {}\nrepaired_args: {}",
                                    accepted_tool_call.tool_call.name,
                                    reason,
                                    repaired_tool_call.name,
                                    truncate_diagnostic_block(
                                        &serde_json::to_string(&repaired_tool_call.args)?,
                                        300
                                    )
                                ));
                                AcceptedInternalToolCall {
                                    accepted_block: render_tool_call_block(&repaired_tool_call)?,
                                    discarded_trailing: None,
                                    tool_call: repaired_tool_call,
                                }
                            } else {
                                if consecutive_invalid_tool_calls >= MAX_INVALID_INTERNAL_TOOL_CALLS
                                {
                                    diagnostics.push(
                                        "internal planner produced repeated invalid tool arguments; falling back to harness-side collection resolution"
                                            .to_string(),
                                    );
                                    break;
                                }
                                messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: accepted_tool_call.accepted_block.clone(),
                                });
                                messages.push(ChatMessage {
                                    role: "user".to_string(),
                                    content: format!(
                                        "Tool Result\nname: {}\nargs: {}\n\n{}\n\nRe-emit exactly one valid tool call.",
                                        accepted_tool_call.tool_call.name,
                                        serde_json::to_string(&accepted_tool_call.tool_call.args)?,
                                        diagnostic
                                    ),
                                });
                                continue;
                            }
                        }
                    }
                }
            };
            consecutive_invalid_tool_calls = 0;
            let tool_call = accepted_tool_call.tool_call.clone();

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
                    let tool_kind = CollectionLeafToolKind::Search;
                    let collection_id = require_string_arg(&tool_call.args, "collection_id")?;
                    let prompt = require_string_arg(&tool_call.args, "prompt")?;
                    let offset = collection_search_offset(&tool_call.args)?;
                    let dedupe_key = format!("{}:{collection_id}:{offset}", tool_kind.tool_name());
                    if !searched_collection_keys.insert(dedupe_key) {
                        format!(
                            "collection_id: {collection_id}\nstatus: skipped\nreason: this exact collection window was already processed in this search run"
                        )
                    } else {
                        let collection = self
                            .resolve_collection_by_id(store, &collection_id)
                            .ok_or_else(|| format!("unknown collection `{collection_id}`"))?;
                        let collection = paged_search_collection(
                            &collection,
                            offset,
                            COLLECTION_SEARCH_PAGE_SIZE,
                        );
                        let mut collection_outcomes = self
                            .run_collection_tools(
                                tool_kind,
                                &[collection],
                                &prompt,
                                requested_summary_scope,
                                llm_client,
                                observer.clone(),
                            )
                            .await;
                        let outcome = collection_outcomes
                            .pop()
                            .ok_or("missing collection tool outcome")?;
                        let rendered = match outcome.execution.as_ref() {
                            Ok(execution) => render_collection_outcome_result(
                                outcome.tool_kind,
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
                    format!("Tool execution failed: unknown internal search tool `{other}`")
                }
            };

            let tool_args = serde_json::to_string(&tool_call.args)?;
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: accepted_tool_call.accepted_block,
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
                .run_collection_tools(
                    CollectionLeafToolKind::Search,
                    &collections,
                    query,
                    RequestedSummaryScope::CurrentWindow,
                    llm_client,
                    observer.clone(),
                )
                .await;
        }

        let rendered = render_combined_search_results_with_summary(
            &outcomes,
            final_summary.as_deref(),
            &diagnostics,
        );
        Ok((rendered, outcomes))
    }

    async fn execute_public_summary(
        &self,
        prepared_input: &PreparedSummaryInput,
        agent: &BskyAgent,
        store: &mut NotificationStore,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<(String, Vec<CollectionToolOutcome>), Box<dyn std::error::Error>> {
        let query = prepared_input.original_query.as_str();
        let actor_anchor = self
            .resolve_prepared_summary_actor_anchor(&prepared_input.actor, agent, store)
            .await?;
        append_summary_trace(format!(
            "[execute_public_summary]\nstatus: start\nquery: {query}\nactor_anchor_did: {}\nactor_anchor_source: {}",
            actor_anchor.actor_did.as_str(),
            actor_anchor.source.as_str()
        ));
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "summary_scope_resolution".to_string(),
                depth: 2,
                content: format!(
                    "status: running\nquery: {query}\nactor_anchor_did: {}\nactor_anchor_source: {}",
                    actor_anchor.actor_did.as_str(),
                    actor_anchor.source.as_str()
                ),
            });
        }
        append_summary_trace(format!(
            "[execute_public_summary]\nstatus: actor_resolved\nactor_handle: {}\nactor_did: {}",
            actor_anchor
                .actor_handle
                .as_deref()
                .unwrap_or(actor_anchor.actor_did.as_str()),
            actor_anchor.actor_did.as_str()
        ));
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "resolve_actor_refs".to_string(),
                depth: 2,
                content: format!(
                    "status: completed\n\nsummary:\n{}",
                    summarize_progress_text(&format!(
                        "actor_ref: {}\nhandle: {}\ndid: {}",
                        actor_anchor
                            .actor_ref
                            .as_deref()
                            .unwrap_or(actor_anchor.actor_did.as_str()),
                        actor_anchor
                            .actor_handle
                            .as_deref()
                            .unwrap_or(actor_anchor.actor_did.as_str()),
                        actor_anchor.actor_did.as_str()
                    ))
                ),
            });
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "summary_scope_resolution".to_string(),
                depth: 2,
                content: format!(
                    "status: resolved\nactor_handle: {}\nactor_did: {}",
                    actor_anchor
                        .actor_handle
                        .as_deref()
                        .unwrap_or(actor_anchor.actor_did.as_str()),
                    actor_anchor.actor_did.as_str()
                ),
            });
        }

        let hydrate_args = summary_hydration_args_for_hint(
            &prepared_input.collection_target_hint,
            prepared_input.requested_summary_scope,
        );
        append_summary_trace(format!(
            "[execute_public_summary]\nstatus: hydrate_start\nactor_did: {}\nhydrate_args: {}",
            actor_anchor.actor_did.as_str(),
            serde_json::to_string_pretty(&hydrate_args)
                .unwrap_or_else(|_| hydrate_args.to_string())
        ));
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "summary_scope_hydration".to_string(),
                depth: 2,
                content: format!(
                    "status: running\nactor_did: {}\nhydrate_args: {}",
                    actor_anchor.actor_did.as_str(),
                    serde_json::to_string_pretty(&hydrate_args)
                        .unwrap_or_else(|_| hydrate_args.to_string())
                ),
            });
        }
        let _ = self
            .execute_internal_hydrate_actor_scope(
                &actor_anchor.actor_did,
                agent,
                store,
                &hydrate_args,
                observer.clone(),
            )
            .await?;
        let actor_collections = self.collections_for_actor(store, &actor_anchor.actor_did);
        append_summary_trace(format!(
            "[execute_public_summary]\nstatus: hydrate_complete\nactor_did: {}\ncollection_count: {}\ncollections:\n{}",
            actor_anchor.actor_did.as_str(),
            actor_collections.len(),
            if actor_collections.is_empty() {
                "<none>".to_string()
            } else {
                actor_collections
                    .iter()
                    .map(|collection| {
                        format!(
                            "{} | kind={} | posts={}",
                            collection.id,
                            collection.collection_kind,
                            collection.posts.len()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        ));
        if let Some(observer) = observer.as_ref() {
            let collection_summary = if actor_collections.is_empty() {
                "<none>".to_string()
            } else {
                actor_collections
                    .iter()
                    .map(|collection| {
                        format!(
                            "{} | kind={} | posts={}",
                            collection.id,
                            collection.collection_kind,
                            collection.posts.len()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "summary_scope_hydration".to_string(),
                depth: 2,
                content: format!(
                    "status: completed\nactor_did: {}\ncollection_count: {}\ncollections:\n{}",
                    actor_anchor.actor_did.as_str(),
                    actor_collections.len(),
                    collection_summary
                ),
            });
        }

        let collection = pick_summary_collection_for_hint(
            &actor_collections,
            &prepared_input.collection_target_hint,
        )
        .ok_or("summary could not find a hydrated actor-backed collection to summarize")?;
        let requested_summary_scope = prepared_input.requested_summary_scope;
        let selection_review = review_summary_collection_selection(
            query,
            requested_summary_scope,
            &actor_collections,
            &collection,
        );
        append_summary_trace(format!(
            "[execute_public_summary]\nstatus: collection_selected\ncollection_id: {}\ncollection_label: {}\ncollection_kind: {}\npost_count: {}\nrequested_scope: {:?}",
            collection.id,
            collection.label,
            collection.collection_kind,
            collection.posts.len(),
            requested_summary_scope
        ));
        append_summary_trace(format!(
            "[summary_collection_selection_review]\nquery: {}\nrequested_scope: {:?}\nrequested_target: {}\nhydrated_candidate_collections:\n{}\noriginal_collection_id: {}\noriginal_collection_kind: {}\nreview_status: {}\nfinal_collection_id: {}\nfinal_collection_kind: {}\ndeterministic_repair_applied: {}\nreason: {}",
            query,
            requested_summary_scope,
            detect_explicit_summary_collection_intent(query, requested_summary_scope).as_str(),
            render_summary_selection_inventory(&actor_collections),
            selection_review.original_collection_id,
            selection_review.original_collection_kind,
            selection_review.status.as_str(),
            selection_review.final_collection_id,
            selection_review.final_collection_kind,
            selection_review.deterministic_repair_applied,
            selection_review.reason
        ));
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "summary_collection_selection".to_string(),
                depth: 2,
                content: format!(
                    "status: selected\ncollection_id: {}\ncollection_label: {}\ncollection_kind: {}\npost_count: {}\nrequested_scope: {:?}",
                    collection.id,
                    collection.label,
                    collection.collection_kind,
                    collection.posts.len(),
                    requested_summary_scope
                ),
            });
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "summary_collection_selection_review".to_string(),
                depth: 2,
                content: format!(
                    "status: {}\noriginal_collection_id: {}\noriginal_collection_kind: {}\nfinal_collection_id: {}\nfinal_collection_kind: {}\ndeterministic_repair_applied: {}\nreason: {}",
                    selection_review.status.as_str(),
                    selection_review.original_collection_id,
                    selection_review.original_collection_kind,
                    selection_review.final_collection_id,
                    selection_review.final_collection_kind,
                    selection_review.deterministic_repair_applied,
                    selection_review.reason
                ),
            });
        }
        if selection_review.status == SummarySelectionReviewStatus::Rejected {
            return Err(selection_review.reason.into());
        }
        let collection = if selection_review.final_collection_id == collection.id {
            collection
        } else {
            let repaired = actor_collections
                .iter()
                .find(|candidate| candidate.id == selection_review.final_collection_id)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "summary selection repair chose missing collection `{}`",
                        selection_review.final_collection_id
                    )
                })?;
            append_summary_trace(format!(
                "[summary_collection_selection_repair]\nstatus: applied\noriginal_collection_id: {}\noriginal_collection_kind: {}\nfinal_collection_id: {}\nfinal_collection_kind: {}\nreason: {}",
                selection_review.original_collection_id,
                selection_review.original_collection_kind,
                repaired.id,
                repaired.collection_kind,
                selection_review.reason
            ));
            repaired
        };
        let collection = if selection_review.status == SummarySelectionReviewStatus::Accepted {
            let llm_review = llm_review_summary_collection_selection(
                query,
                requested_summary_scope,
                &actor_collections,
                &collection,
                llm_client,
            )
            .await?;
            append_summary_trace(format!(
                "[summary_collection_selection_llm_review]\nquery: {}\nrequested_scope: {:?}\nproposed_collection_id: {}\nproposed_collection_kind: {}\nreview_status: {}\nfinal_collection_id: {}\nfinal_collection_kind: {}\nreason: {}",
                query,
                requested_summary_scope,
                llm_review.original_collection_id,
                llm_review.original_collection_kind,
                llm_review.status.as_str(),
                llm_review.final_collection_id,
                llm_review.final_collection_kind,
                llm_review.reason
            ));
            if let Some(observer) = observer.as_ref() {
                let _ = observer.send(ToolProgressEvent::AgentUpdate {
                    label: "summary_collection_selection_llm_review".to_string(),
                    depth: 2,
                    content: format!(
                        "status: {}\noriginal_collection_id: {}\noriginal_collection_kind: {}\nfinal_collection_id: {}\nfinal_collection_kind: {}\nreason: {}",
                        llm_review.status.as_str(),
                        llm_review.original_collection_id,
                        llm_review.original_collection_kind,
                        llm_review.final_collection_id,
                        llm_review.final_collection_kind,
                        llm_review.reason
                    ),
                });
            }
            if llm_review.status == SummarySelectionReviewStatus::Rejected {
                return Err(llm_review.reason.into());
            }
            let llm_selected = actor_collections
                .iter()
                .find(|candidate| candidate.id == llm_review.final_collection_id)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "summary selection llm review chose missing collection `{}`",
                        llm_review.final_collection_id
                    )
                })?;
            let final_review = review_summary_collection_selection(
                query,
                requested_summary_scope,
                &actor_collections,
                &llm_selected,
            );
            append_summary_trace(format!(
                "[summary_collection_selection_llm_enforcement]\nreview_status: {}\noriginal_collection_id: {}\noriginal_collection_kind: {}\nfinal_collection_id: {}\nfinal_collection_kind: {}\ndeterministic_repair_applied: {}\nreason: {}",
                final_review.status.as_str(),
                final_review.original_collection_id,
                final_review.original_collection_kind,
                final_review.final_collection_id,
                final_review.final_collection_kind,
                final_review.deterministic_repair_applied,
                final_review.reason
            ));
            if final_review.status == SummarySelectionReviewStatus::Rejected {
                return Err(final_review.reason.into());
            }
            if final_review.final_collection_id == collection.id {
                collection
            } else {
                actor_collections
                    .iter()
                    .find(|candidate| candidate.id == final_review.final_collection_id)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "summary selection enforcement chose missing collection `{}`",
                            final_review.final_collection_id
                        )
                    })?
            }
        } else {
            collection
        };
        let outcomes = self
            .run_collection_summary_loop(
                &collection,
                query,
                requested_summary_scope,
                llm_client,
                observer,
            )
            .await;
        append_summary_trace(format!(
            "[execute_public_summary]\nstatus: loop_finished\noutcome_count: {}\nrendered:\n{}",
            outcomes.len(),
            render_summary_collection_loop_result(&outcomes)
        ));
        let rendered = render_public_summary_outcomes(&outcomes);
        Ok((rendered, outcomes))
    }

    async fn repair_invalid_internal_search_tool_call(
        &self,
        query: &str,
        search_intent: SearchIntent,
        requested_summary_scope: RequestedSummaryScope,
        resolved_actor_refs: &[ResolvedActorRef],
        available_collections: &[LabeledPostCollection],
        invalid_tool_call: &AcceptedInternalToolCall,
        validation_error: &str,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<Option<AcceptedInternalToolCall>, Box<dyn std::error::Error>> {
        if let Some(observer) = observer.as_ref() {
            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                label: "tool_call_repair".to_string(),
                depth: 2,
                content: format!(
                    "status: running\ninvalid_name: {}\nvalidation_error: {}\navailable_collection_count: {}",
                    invalid_tool_call.tool_call.name,
                    validation_error,
                    available_collections.len()
                ),
            });
        }

        let repaired = complete_internal_tool_call_repair(
            query,
            search_intent,
            requested_summary_scope,
            resolved_actor_refs,
            available_collections,
            invalid_tool_call,
            validation_error,
            llm_client,
        )
        .await;

        let repaired = match repaired {
            Ok(Some(repaired_tool_call)) => {
                match validate_internal_search_tool_call(&repaired_tool_call.tool_call) {
                    Ok(_) => {
                        if let Some(observer) = observer.as_ref() {
                            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                label: "tool_call_repair".to_string(),
                                depth: 2,
                                content: format!(
                                    "status: repaired\noriginal_name: {}\nrepaired_name: {}\nrepaired_tool_call:\n{}",
                                    invalid_tool_call.tool_call.name,
                                    repaired_tool_call.tool_call.name,
                                    repaired_tool_call.accepted_block
                                ),
                            });
                        }
                        Some(repaired_tool_call)
                    }
                    Err(reason) => {
                        if let Some(observer) = observer.as_ref() {
                            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                label: "tool_call_repair".to_string(),
                                depth: 2,
                                content: format!(
                                    "status: failed\nreason: repaired tool call still failed validation\nvalidation_error: {}",
                                    reason
                                ),
                            });
                        }
                        None
                    }
                }
            }
            Ok(None) => {
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "tool_call_repair".to_string(),
                        depth: 2,
                        content: "status: cannot_repair".to_string(),
                    });
                }
                None
            }
            Err(err) => {
                if let Some(observer) = observer.as_ref() {
                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                        label: "tool_call_repair".to_string(),
                        depth: 2,
                        content: format!("status: failed\nreason: {}", err),
                    });
                }
                None
            }
        };

        Ok(repaired)
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
            .ok_or_else(|| {
                let hint = if collection.collection_kind == "clearsky_lists" {
                    " `clearsky_lists` items are keyed by their exact stored list URL, not by inferred AT URIs. Use `search` for moderation-list questions, or reuse an exact `search_result_*_uri` from a prior tool result."
                } else {
                    " Reuse an exact `search_result_*_uri` or `selected_result_uri` from a prior tool result; do not invent an item URI."
                };
                format!("item `{item_uri}` was not found in `{collection_id}`.{hint}")
            })?;

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
        result: Option<&CollectionLeafResult>,
    ) {
        match result {
            Some(CollectionLeafResult::Search(result)) => {
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
            Some(CollectionLeafResult::Summary(result)) => {
                let mut lines = vec![
                    format!("post: {}", result.title),
                    format!("summary: {}", result.summary),
                ];
                for (index, uri) in result.covered_item_uris.iter().enumerate() {
                    lines.push(format!("covered_item_{}_uri: {}", index + 1, uri));
                }
                context.push_section(title, lines.join("\n"));
            }
            Some(CollectionLeafResult::RawWindow(result)) => {
                let mut lines = vec![
                    format!("post: {}", result.title),
                    format!("summary: {}", result.summary),
                    format!("window_offset: {}", result.window_offset),
                    format!("window_size: {}", result.window_size),
                    format!("page_index: {}", result.page_index),
                ];
                for (index, post) in result.records.iter().enumerate() {
                    lines.push(format!("raw_window_item_{}_uri: {}", index + 1, post.uri));
                    lines.push(format!("raw_window_item_{}_body: {}", index + 1, post.body));
                }
                context.push_section(title, lines.join("\n"));
            }
            None => context.push_section(title, "No matching cached posts."),
        }
    }

    pub(crate) async fn run_collection_tools(
        &self,
        tool_kind: CollectionLeafToolKind,
        collections: &[LabeledPostCollection],
        prompt: &str,
        requested_summary_scope: RequestedSummaryScope,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Vec<CollectionToolOutcome> {
        let mut outcomes = Vec::with_capacity(collections.len());

        for collection in collections {
            let collection = if collection_window_offset(collection).is_some() {
                collection.clone()
            } else {
                paged_search_collection(collection, 0, COLLECTION_SEARCH_PAGE_SIZE)
            };
            if let Some(observer) = observer.as_ref() {
                let _ = observer.send(ToolProgressEvent::AgentUpdate {
                    label: format!("{}: {}", tool_kind.label_prefix(), collection.label),
                    depth: 2,
                    content: format!("collection_id: {}\nstatus: running", collection.id),
                });
            }
            let execution = match self
                .run_collection_tool(tool_kind, &collection, prompt, llm_client)
                .await
            {
                Ok(execution) => self
                    .review_collection_execution(
                        tool_kind,
                        &collection,
                        prompt,
                        requested_summary_scope,
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
                    summarize_progress_text(&render_collection_outcome_progress_result(
                        tool_kind,
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
                    label: format!("{}: {}", tool_kind.label_prefix(), collection.label),
                    depth: 2,
                    content: progress_content,
                });
            }
            outcomes.push(CollectionToolOutcome {
                tool_kind,
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

    fn select_summary_collection(
        &self,
        store: &NotificationStore,
        actor_did: &Did,
        query: &str,
    ) -> Option<LabeledPostCollection> {
        let collections = self.collections_for_actor(store, actor_did);
        pick_summary_fallback_collection(&collections, query)
    }

    fn select_summary_collection_for_hint(
        &self,
        store: &NotificationStore,
        actor_did: &Did,
        hint: &SummaryCollectionTargetHint,
    ) -> Option<LabeledPostCollection> {
        let collections = self.collections_for_actor(store, actor_did);
        pick_summary_collection_for_hint(&collections, hint)
    }

    async fn resolve_prepared_summary_actor_anchor(
        &self,
        prepared_actor: &PreparedSummaryActor,
        agent: &BskyAgent,
        store: &mut NotificationStore,
    ) -> Result<PreparedActorAnchor, Box<dyn std::error::Error>> {
        match prepared_actor {
            PreparedSummaryActor::Resolved(anchor) => Ok(anchor.clone()),
            PreparedSummaryActor::ExplicitRef(actor_ref) => {
                let profile = ensure_actor_profile_cached(agent, store, actor_ref).await?;
                Ok(PreparedActorAnchor {
                    actor_ref: Some(actor_ref.clone()),
                    actor_handle: Some(profile.handle),
                    actor_did: profile.did,
                    source: ActorAnchorSource::ExplicitQueryRef,
                })
            }
        }
    }

    async fn resolve_summary_prep_request(
        &self,
        query: &str,
        llm_client: &LlmApiClient,
    ) -> Result<PreparedSummaryRequest, String> {
        let response = llm_client
            .complete_chat_with_response_format(
                vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: "You convert a natural-language public `summary(query)` request into a structured harness prep object. Return only JSON. Preserve user intent. Use `requested_summary_scope.kind` as one of: `current_window`, `count`, `page`, `page_range`. Use `collection_target_hint` as one of: `recent_posts`, `replies`, `pinned_posts`, `profile`, `lists`. For count requests like 'last 50 posts' or '50 most recent posts', return `kind: count` with `requested_items: 50`. For requests about moderation lists, reputation based on lists, or list membership, return `collection_target_hint: \"lists\"`. Do not invent actors.".to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: format!(
                            "Query:\n{query}\n\nReturn JSON with this shape:\n{{\"requested_summary_scope\":{{\"kind\":\"count\",\"requested_items\":50}},\"collection_target_hint\":\"recent_posts\"}}"
                        ),
                    },
                ],
                256,
                Some(ChatCompletionResponseFormat::JsonObject),
            )
            .await
            .map_err(|err| err.to_string())?;
        let parsed: SummaryPrepResolution =
            serde_json::from_str(&response).map_err(|err| format!("invalid prep json: {err}"))?;
        let requested_summary_scope =
            requested_summary_scope_from_prep_payload(&parsed.requested_summary_scope)?;
        let collection_target_hint =
            summary_collection_target_hint_from_payload(parsed.collection_target_hint);
        Ok(PreparedSummaryRequest {
            requested_summary_scope,
            collection_target_hint,
        })
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

                    ensure_recent_replies_received_cached(
                        agent,
                        store,
                        &profile.did,
                        ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT,
                        ACTOR_SCOPE_REPLY_FETCH_LIMIT,
                    )
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

                    ensure_recent_posts_cached(
                        agent,
                        store,
                        &profile.did,
                        ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT,
                        ACTOR_SEARCH_POST_TARGET,
                    )
                    .await?;
                    if let Some(collection) = self.resolve_collection_by_id(
                        store,
                        &self.recent_posts_collection_id(&profile.did),
                    ) {
                        collections.push(collection);
                    }
                }
                SearchIntent::General => {
                    ensure_recent_posts_cached(
                        agent,
                        store,
                        &profile.did,
                        ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT,
                        ACTOR_SEARCH_POST_TARGET,
                    )
                    .await?;
                    ensure_pinned_posts_cached(agent, store, &profile.did).await?;
                    ensure_recent_replies_received_cached(
                        agent,
                        store,
                        &profile.did,
                        ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT,
                        ACTOR_SCOPE_REPLY_FETCH_LIMIT,
                    )
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

    async fn review_collection_execution(
        &self,
        tool_kind: CollectionLeafToolKind,
        collection: &LabeledPostCollection,
        prompt: &str,
        requested_summary_scope: RequestedSummaryScope,
        mut execution: LlmSearchExecution,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Result<LlmSearchExecution, Box<dyn std::error::Error>> {
        let review_label = format!("{}: {}", tool_kind.review_label(), collection.label);
        if let Some(observer) = observer.as_ref() {
            if !tool_kind.is_coverage_oriented() {
                let _ = observer.send(ToolProgressEvent::AgentUpdate {
                    label: review_label.clone(),
                    depth: 3,
                    content: format!("collection_id: {}\nstatus: running", collection.id),
                });
            }
        }

        let review_context = build_collection_review_context(
            tool_kind,
            collection,
            prompt,
            requested_summary_scope,
            &execution,
        );
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

        let heuristic =
            heuristic_collection_review(tool_kind, collection, requested_summary_scope, &execution);
        let mut verdict = review_response
            .as_deref()
            .and_then(parse_collection_review_verdict)
            .unwrap_or_else(|| heuristic.clone());

        if verdict.status == CollectionReviewStatus::Pass
            && heuristic.status == CollectionReviewStatus::Fail
        {
            verdict = heuristic.clone();
        }

        if tool_kind == CollectionLeafToolKind::Summary {
            append_summary_trace(format!(
                "[summary_leaf_review]\ncollection_id: {}\nwindow_offset: {}\nreview_status: {}\nreview_grounded: {}\nreview_sufficient: {}\nreview_repair_needed: {}\nreview_additional_pages_needed: {}\nreview_reason: {}\nresult_before_review: {}\noriginal_result_before_review: {}\nsummary_before_review:\n{}\nreview_context:\n{}",
                collection.id,
                collection_window_offset(collection).unwrap_or(0),
                verdict.status.as_str(),
                verdict.grounded,
                verdict.sufficient,
                verdict.repair_needed,
                verdict.additional_pages_needed,
                verdict.reason,
                summary_result_presence(execution.result.as_ref()),
                summary_result_presence(execution.original_result.as_ref()),
                truncate_diagnostic_block(
                    &execution
                        .result
                        .as_ref()
                        .or(execution.original_result.as_ref())
                        .map(|result| result.summary().to_string())
                        .unwrap_or_else(|| "<missing summary>".to_string()),
                    4000,
                ),
                truncate_diagnostic_block(&review_window.rendered, 4000),
            ));
        }

        execution.review_context_window = Some(review_window);

        if verdict.status == CollectionReviewStatus::Fail && verdict.repair_needed {
            let original_summary = execution
                .result
                .as_ref()
                .map(|result| result.summary().to_string())
                .unwrap_or_else(|| "<missing summary>".to_string());
            if tool_kind.is_coverage_oriented() {
                execution.review_verdict = Some(verdict.clone());
                execution.result = None;
                append_summary_trace(format!(
                    "[summary_leaf_review]\ncollection_id: {}\nwindow_offset: {}\naction: dropped_result_for_failed_coverage_review\nresult_after_drop: {}\noriginal_result_after_drop: {}",
                    collection.id,
                    collection_window_offset(collection).unwrap_or(0),
                    summary_result_presence(execution.result.as_ref()),
                    summary_result_presence(execution.original_result.as_ref()),
                ));
            } else {
                let repair_summary =
                    repair_collection_summary(collection, &execution, prompt, &verdict);
                execution.repair_diagnostic = Some(format!(
                    "Initial review failed. Applied deterministic cited repair when possible. Original summary: {}",
                    truncate_chars(&original_summary, 240)
                ));
                if let Some(result) = execution.result.as_mut() {
                    match result {
                        CollectionLeafResult::Search(result) => result.summary = repair_summary,
                        CollectionLeafResult::Summary(result) => result.summary = repair_summary,
                        CollectionLeafResult::RawWindow(result) => result.summary = repair_summary,
                    }
                } else if let Some(original_result) = execution.original_result.clone() {
                    execution.result = Some(match original_result {
                        CollectionLeafResult::Search(mut result) => {
                            result.summary = repair_summary;
                            CollectionLeafResult::Search(result)
                        }
                        CollectionLeafResult::Summary(mut result) => {
                            result.summary = repair_summary;
                            CollectionLeafResult::Summary(result)
                        }
                        CollectionLeafResult::RawWindow(mut result) => {
                            result.summary = repair_summary;
                            CollectionLeafResult::RawWindow(result)
                        }
                    });
                }

                let post_repair_verdict = heuristic_collection_review(
                    tool_kind,
                    collection,
                    requested_summary_scope,
                    &execution,
                );
                if post_repair_verdict.status == CollectionReviewStatus::Pass {
                    execution.review_verdict = Some(CollectionReviewVerdict {
                        status: CollectionReviewStatus::Pass,
                        grounded: true,
                        sufficient: true,
                        reason: format!(
                            "Initial review failed but the repaired summary is now grounded in the selected records. Original reason: {}",
                            verdict.reason
                        ),
                        repair_needed: false,
                        repair_instructions: None,
                        additional_pages_needed: false,
                        next_page: None,
                        next_offset: None,
                        required_total_items: None,
                    });
                } else {
                    execution.review_verdict = Some(post_repair_verdict.clone());
                    execution.repair_diagnostic = Some(format!(
                        "{} Repair attempt still failed review.",
                        execution.repair_diagnostic.as_deref().unwrap_or_default()
                    ));
                    execution.result = None;
                }
            }
        } else {
            execution.review_verdict = Some(verdict.clone());
            if verdict.status == CollectionReviewStatus::Fail {
                execution.result = None;
            }
        }

        if let Some(observer) = observer.as_ref() {
            if !tool_kind.is_coverage_oriented() {
                let status = execution
                    .review_verdict
                    .as_ref()
                    .map(|verdict| verdict.status.as_str())
                    .unwrap_or("pass");
                let _ = observer.send(ToolProgressEvent::AgentUpdate {
                    label: review_label,
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
        let recent_posts_feed_fetch_limit =
            optional_usize_arg(args, "recent_posts_feed_fetch_limit")?
                .unwrap_or(ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT);
        let recent_posts_min_top_level_posts =
            optional_usize_arg(args, "recent_posts_min_top_level_posts")?
                .unwrap_or(ACTOR_SEARCH_POST_TARGET);

        let mut lines = vec![format!("actor_did: {}", actor_did.as_str())];

        if include_profile {
            let _ = ensure_actor_profile_cached(agent, store, actor_did.as_str()).await?;
            let collection_id = format!("actor_profile:{}", actor_did.as_str());
            let collection = self
                .resolve_collection_by_id(store, &collection_id)
                .ok_or_else(|| format!("missing profile collection `{collection_id}`"))?;
            lines.push(format!("collection_id: {}", collection.id));
            lines.push(format!("collection_label: {}", collection.label));
        }
        if include_recent_posts {
            ensure_recent_posts_cached(
                agent,
                store,
                actor_did,
                recent_posts_feed_fetch_limit,
                recent_posts_min_top_level_posts,
            )
            .await?;
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
            ensure_recent_replies_received_cached(
                agent,
                store,
                actor_did,
                ACTOR_SCOPE_AUTHOR_FEED_FETCH_LIMIT,
                ACTOR_SCOPE_REPLY_FETCH_LIMIT,
            )
            .await?;
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

fn resolve_prepared_actor_anchor(
    query: &str,
    selected_actor_did: Option<&Did>,
    store: &NotificationStore,
) -> Option<PreparedActorAnchor> {
    if let Some(actor_ref) = detect_actor_refs(query).and_then(|refs| refs.into_iter().next()) {
        return store
            .find_did(&actor_ref)
            .or_else(|| actor_ref.parse().ok())
            .map(|actor_did| PreparedActorAnchor {
                actor_ref: Some(actor_ref),
                actor_handle: store.get_handle(&actor_did).map(str::to_string),
                actor_did,
                source: ActorAnchorSource::ExplicitQueryRef,
            });
    }

    selected_actor_did.map(|actor_did| PreparedActorAnchor {
        actor_ref: store.get_handle(actor_did).map(str::to_string),
        actor_handle: store.get_handle(actor_did).map(str::to_string),
        actor_did: actor_did.clone(),
        source: ActorAnchorSource::SelectedActorFallback,
    })
}

fn resolve_prepared_summary_actor(
    query: &str,
    selected_actor_did: Option<&Did>,
    store: &NotificationStore,
) -> Option<PreparedSummaryActor> {
    if let Some(actor_ref) = detect_actor_refs(query).and_then(|refs| refs.into_iter().next()) {
        if let Some(actor_did) = store.find_did(&actor_ref).or_else(|| actor_ref.parse().ok()) {
            return Some(PreparedSummaryActor::Resolved(PreparedActorAnchor {
                actor_ref: Some(actor_ref),
                actor_handle: store.get_handle(&actor_did).map(str::to_string),
                actor_did,
                source: ActorAnchorSource::ExplicitQueryRef,
            }));
        }
        return Some(PreparedSummaryActor::ExplicitRef(actor_ref));
    }

    selected_actor_did.map(|actor_did| {
        PreparedSummaryActor::Resolved(PreparedActorAnchor {
            actor_ref: store.get_handle(actor_did).map(str::to_string),
            actor_handle: store.get_handle(actor_did).map(str::to_string),
            actor_did: actor_did.clone(),
            source: ActorAnchorSource::SelectedActorFallback,
        })
    })
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
    ToolCall(AcceptedInternalToolCall),
    Invalid(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AcceptedInternalToolCall {
    tool_call: PromptToolCall,
    accepted_block: String,
    discarded_trailing: Option<String>,
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

fn should_prefer_summary_fallback(
    query: &str,
    requested_summary_scope: RequestedSummaryScope,
) -> bool {
    if requested_summary_scope != RequestedSummaryScope::CurrentWindow {
        return true;
    }

    let lower = query.to_ascii_lowercase();
    [
        "summarize",
        "summary",
        "analyze",
        "analysis",
        "blog post",
        "quote",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn pick_summary_fallback_collection(
    collections: &[LabeledPostCollection],
    query: &str,
) -> Option<LabeledPostCollection> {
    let hint = detect_summary_collection_target_hint(query);
    pick_summary_collection_for_hint(collections, &hint).or_else(|| collections.first().cloned())
}

fn summary_hydration_args(query: &str) -> Value {
    let hint = detect_summary_collection_target_hint(query);
    summary_hydration_args_for_hint(&hint, detect_requested_summary_scope(query))
}

fn requested_summary_scope_from_prep_payload(
    payload: &SummaryPrepScopePayload,
) -> Result<RequestedSummaryScope, String> {
    match payload.kind.as_str() {
        "current_window" => Ok(RequestedSummaryScope::CurrentWindow),
        "count" => payload
            .requested_items
            .filter(|value| *value > 0)
            .map(|requested_items| RequestedSummaryScope::Count { requested_items })
            .ok_or_else(|| "count scope is missing a positive `requested_items`".to_string()),
        "page" => payload
            .page_index
            .map(|page_index| RequestedSummaryScope::Page { page_index })
            .ok_or_else(|| "page scope is missing `page_index`".to_string()),
        "page_range" => {
            let start_page = payload
                .start_page
                .ok_or_else(|| "page_range scope is missing `start_page`".to_string())?;
            let end_page = payload
                .end_page
                .ok_or_else(|| "page_range scope is missing `end_page`".to_string())?;
            Ok(RequestedSummaryScope::PageRange {
                start_page: start_page.min(end_page),
                end_page: start_page.max(end_page),
            })
        }
        other => Err(format!("unknown summary scope kind `{other}`")),
    }
}

fn summary_collection_target_hint_from_payload(
    payload: SummaryPrepCollectionTargetPayload,
) -> SummaryCollectionTargetHint {
    match payload {
        SummaryPrepCollectionTargetPayload::RecentPosts => SummaryCollectionTargetHint::RecentPosts,
        SummaryPrepCollectionTargetPayload::Replies => SummaryCollectionTargetHint::Replies,
        SummaryPrepCollectionTargetPayload::PinnedPosts => SummaryCollectionTargetHint::PinnedPosts,
        SummaryPrepCollectionTargetPayload::Profile => SummaryCollectionTargetHint::Profile,
        SummaryPrepCollectionTargetPayload::Lists => SummaryCollectionTargetHint::Lists,
    }
}

fn detect_summary_collection_target_hint(query: &str) -> SummaryCollectionTargetHint {
    let lower = query.to_ascii_lowercase();
    if lower.contains("repl") {
        SummaryCollectionTargetHint::Replies
    } else if [
        " list",
        "lists",
        "listed",
        "placed on",
        "reputation",
        "sentiment",
        "known for",
        "known on",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        SummaryCollectionTargetHint::Lists
    } else if lower.contains("pinned") {
        SummaryCollectionTargetHint::PinnedPosts
    } else if lower.contains("profile") || lower.contains("bio") || lower.contains("who is") {
        SummaryCollectionTargetHint::Profile
    } else {
        SummaryCollectionTargetHint::RecentPosts
    }
}

fn pick_summary_collection_for_hint(
    collections: &[LabeledPostCollection],
    hint: &SummaryCollectionTargetHint,
) -> Option<LabeledPostCollection> {
    let preferred_kinds: &[&str] = match hint {
        SummaryCollectionTargetHint::Replies => &[
            "recent_replies_received",
            "recent_replies_sent",
            "replies_to_actor",
            "recent_posts",
        ],
        SummaryCollectionTargetHint::PinnedPosts => &["pinned_posts", "recent_posts"],
        SummaryCollectionTargetHint::Profile => &["actor_profile", "recent_posts"],
        SummaryCollectionTargetHint::Lists => &[
            "clearsky_lists",
            "recent_replies_received",
            "actor_profile",
            "recent_posts",
        ],
        SummaryCollectionTargetHint::RecentPosts => {
            &["recent_posts", "recent_posts_unaddressed", "pinned_posts"]
        }
    };

    preferred_kinds
        .iter()
        .find_map(|kind| {
            collections.iter().find(|collection| {
                collection.collection_kind == *kind || collection.id.starts_with(kind)
            })
        })
        .cloned()
}

fn render_summary_selection_inventory(collections: &[LabeledPostCollection]) -> String {
    if collections.is_empty() {
        return "<none>".to_string();
    }

    collections
        .iter()
        .map(|collection| {
            format!(
                "{} | kind={} | posts={}",
                collection.id,
                collection.collection_kind,
                collection.posts.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn llm_review_summary_collection_selection(
    query: &str,
    requested_summary_scope: RequestedSummaryScope,
    collections: &[LabeledPostCollection],
    selected: &LabeledPostCollection,
    llm_client: &LlmApiClient,
) -> Result<SummaryCollectionSelectionReview, Box<dyn std::error::Error>> {
    let response = llm_client
        .complete_chat_with_response_format(
            vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You review a proposed public `summary` collection selection. Return only JSON. Use `status` as one of `accepted`, `repaired`, or `rejected`. Use `final_collection_id` as an exact id from the provided collection inventory for accepted or repaired results. Reject when none of the available collections fit the query. Do not invent collection ids.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!(
                        "Query:\n{query}\n\nRequested summary scope:\n{}\n\nProposed collection:\ncollection_id: {}\ncollection_label: {}\ncollection_kind: {}\nitem_count: {}\n\nAvailable hydrated collections:\n{}\n\nReturn JSON with this shape:\n{{\"status\":\"accepted\",\"final_collection_id\":\"{}\",\"reason\":\"...\"}}",
                        requested_summary_scope.render_for_planner(),
                        selected.id,
                        selected.label,
                        selected.collection_kind,
                        selected.posts.len(),
                        render_summary_selection_inventory(collections),
                        selected.id
                    ),
                },
            ],
            256,
            Some(ChatCompletionResponseFormat::JsonObject),
        )
        .await?;
    parse_llm_summary_collection_selection_review(collections, selected, &response)
}

fn parse_llm_summary_collection_selection_review(
    collections: &[LabeledPostCollection],
    selected: &LabeledPostCollection,
    response: &str,
) -> Result<SummaryCollectionSelectionReview, Box<dyn std::error::Error>> {
    let payload: SummaryCollectionSelectionReviewPayload = serde_json::from_str(response)
        .map_err(|err| format!("invalid summary selection review json: {err}"))?;
    let status = match payload.status.trim() {
        "accepted" => SummarySelectionReviewStatus::Accepted,
        "repaired" => SummarySelectionReviewStatus::Repaired,
        "rejected" => SummarySelectionReviewStatus::Rejected,
        other => {
            return Err(
                format!("summary selection review returned unknown status `{other}`").into(),
            );
        }
    };

    let (final_collection_id, final_collection_kind) = match status {
        SummarySelectionReviewStatus::Rejected => {
            (selected.id.clone(), selected.collection_kind.clone())
        }
        SummarySelectionReviewStatus::Accepted | SummarySelectionReviewStatus::Repaired => {
            let final_collection_id = payload
                .final_collection_id
                .filter(|value| !value.trim().is_empty())
                .ok_or("summary selection review omitted final_collection_id")?;
            let final_collection = collections
                .iter()
                .find(|candidate| candidate.id == final_collection_id)
                .ok_or_else(|| {
                    format!(
                        "summary selection review chose unknown collection `{final_collection_id}`"
                    )
                })?;
            (
                final_collection.id.clone(),
                final_collection.collection_kind.clone(),
            )
        }
    };

    Ok(SummaryCollectionSelectionReview {
        status,
        reason: payload.reason,
        original_collection_id: selected.id.clone(),
        original_collection_kind: selected.collection_kind.clone(),
        final_collection_id,
        final_collection_kind,
        deterministic_repair_applied: false,
    })
}

fn review_summary_collection_selection(
    query: &str,
    requested_summary_scope: RequestedSummaryScope,
    collections: &[LabeledPostCollection],
    selected: &LabeledPostCollection,
) -> SummaryCollectionSelectionReview {
    let intent = detect_explicit_summary_collection_intent(query, requested_summary_scope);
    let original_collection_id = selected.id.clone();
    let original_collection_kind = selected.collection_kind.clone();

    if matches!(intent, ExplicitSummaryCollectionIntent::Ambiguous) {
        return SummaryCollectionSelectionReview {
            status: SummarySelectionReviewStatus::Accepted,
            reason: "query leaves the summary collection target ambiguous, so the initial selection was kept".to_string(),
            original_collection_id: original_collection_id.clone(),
            original_collection_kind: original_collection_kind.clone(),
            final_collection_id: original_collection_id,
            final_collection_kind: original_collection_kind,
            deterministic_repair_applied: false,
        };
    }

    let preferred_kinds = explicit_summary_collection_preferred_kinds(intent);
    if preferred_kinds
        .iter()
        .any(|kind| selected.collection_kind == *kind || selected.id.starts_with(kind))
    {
        return SummaryCollectionSelectionReview {
            status: SummarySelectionReviewStatus::Accepted,
            reason: format!(
                "selected collection kind `{}` matches explicit request target `{}`",
                selected.collection_kind,
                intent.as_str()
            ),
            original_collection_id: original_collection_id.clone(),
            original_collection_kind: original_collection_kind.clone(),
            final_collection_id: original_collection_id,
            final_collection_kind: original_collection_kind,
            deterministic_repair_applied: false,
        };
    }

    if let Some(repaired) = preferred_kinds.iter().find_map(|kind| {
        collections
            .iter()
            .find(|candidate| candidate.collection_kind == *kind || candidate.id.starts_with(kind))
    }) {
        return SummaryCollectionSelectionReview {
            status: SummarySelectionReviewStatus::Repaired,
            reason: format!(
                "replaced incompatible collection kind `{}` with explicit request target `{}`",
                selected.collection_kind, repaired.collection_kind
            ),
            original_collection_id,
            original_collection_kind,
            final_collection_id: repaired.id.clone(),
            final_collection_kind: repaired.collection_kind.clone(),
            deterministic_repair_applied: true,
        };
    }

    SummaryCollectionSelectionReview {
        status: SummarySelectionReviewStatus::Rejected,
        reason: format!(
            "summary query explicitly targets `{}`, but no compatible hydrated collection was available",
            intent.as_str()
        ),
        original_collection_id: original_collection_id.clone(),
        original_collection_kind: original_collection_kind.clone(),
        final_collection_id: original_collection_id,
        final_collection_kind: original_collection_kind,
        deterministic_repair_applied: false,
    }
}

fn explicit_summary_collection_preferred_kinds(
    intent: ExplicitSummaryCollectionIntent,
) -> &'static [&'static str] {
    match intent {
        ExplicitSummaryCollectionIntent::RecentPosts => {
            &["recent_posts", "recent_posts_unaddressed", "pinned_posts"]
        }
        ExplicitSummaryCollectionIntent::Replies(ExplicitRepliesTarget::Sent) => &[
            "recent_replies_sent",
            "recent_replies_received",
            "replies_to_actor",
        ],
        ExplicitSummaryCollectionIntent::Replies(ExplicitRepliesTarget::Received) => &[
            "recent_replies_received",
            "replies_to_actor",
            "recent_replies_sent",
        ],
        ExplicitSummaryCollectionIntent::Replies(ExplicitRepliesTarget::Any) => &[
            "recent_replies_received",
            "recent_replies_sent",
            "replies_to_actor",
        ],
        ExplicitSummaryCollectionIntent::PinnedPosts => &["pinned_posts", "recent_posts"],
        ExplicitSummaryCollectionIntent::Profile => &["actor_profile"],
        ExplicitSummaryCollectionIntent::Lists => &["clearsky_lists"],
        ExplicitSummaryCollectionIntent::Ambiguous => &[],
    }
}

fn detect_explicit_summary_collection_intent(
    query: &str,
    requested_summary_scope: RequestedSummaryScope,
) -> ExplicitSummaryCollectionIntent {
    let lower = query.to_ascii_lowercase();

    if [
        " list",
        "lists",
        "listed",
        "placed on",
        "moderation",
        "block list",
        "blocklist",
        "follow graph",
        "follow list",
        "reputation",
        "sentiment",
        "known for",
        "known on",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return ExplicitSummaryCollectionIntent::Lists;
    }

    if lower.contains("pinned") {
        return ExplicitSummaryCollectionIntent::PinnedPosts;
    }

    if lower.contains("repl") {
        if lower.contains("sent") || lower.contains("from them") || lower.contains("by them") {
            return ExplicitSummaryCollectionIntent::Replies(ExplicitRepliesTarget::Sent);
        }
        if lower.contains("received")
            || lower.contains("to them")
            || lower.contains("replies to")
            || lower.contains("people replying")
        {
            return ExplicitSummaryCollectionIntent::Replies(ExplicitRepliesTarget::Received);
        }
        return ExplicitSummaryCollectionIntent::Replies(ExplicitRepliesTarget::Any);
    }

    if lower.contains("profile")
        || lower.contains("bio")
        || lower.contains("who is")
        || lower.contains("who are they")
        || lower.contains("what does their profile say")
    {
        return ExplicitSummaryCollectionIntent::Profile;
    }

    let explicit_post_count =
        matches!(requested_summary_scope, RequestedSummaryScope::Count { .. })
            && lower.contains("post");
    let explicit_recent_posts = [
        "recent posts",
        "recent post",
        "most recent posts",
        "latest posts",
        "last posts",
    ]
    .iter()
    .any(|needle| lower.contains(needle));

    if explicit_post_count || explicit_recent_posts {
        return ExplicitSummaryCollectionIntent::RecentPosts;
    }

    ExplicitSummaryCollectionIntent::Ambiguous
}

fn summary_hydration_args_for_hint(
    hint: &SummaryCollectionTargetHint,
    requested_summary_scope: RequestedSummaryScope,
) -> Value {
    let (recent_posts_feed_fetch_limit, recent_posts_min_top_level_posts) =
        summary_recent_post_requirements(requested_summary_scope);
    match hint {
        SummaryCollectionTargetHint::Replies => serde_json::json!({
            "include_recent_replies_received": true,
            "include_recent_posts": true,
            "include_profile": true,
            "recent_posts_feed_fetch_limit": recent_posts_feed_fetch_limit,
            "recent_posts_min_top_level_posts": recent_posts_min_top_level_posts
        }),
        SummaryCollectionTargetHint::PinnedPosts => serde_json::json!({
            "include_recent_posts": true,
            "include_pinned_posts": true,
            "include_profile": true,
            "recent_posts_feed_fetch_limit": recent_posts_feed_fetch_limit,
            "recent_posts_min_top_level_posts": recent_posts_min_top_level_posts
        }),
        SummaryCollectionTargetHint::Profile => serde_json::json!({
            "include_profile": true,
            "include_recent_posts": true,
            "recent_posts_feed_fetch_limit": recent_posts_feed_fetch_limit,
            "recent_posts_min_top_level_posts": recent_posts_min_top_level_posts
        }),
        SummaryCollectionTargetHint::Lists => serde_json::json!({
            "include_clearsky_lists": true,
            "include_profile": true,
            "include_recent_posts": true,
            "recent_posts_feed_fetch_limit": recent_posts_feed_fetch_limit,
            "recent_posts_min_top_level_posts": recent_posts_min_top_level_posts
        }),
        SummaryCollectionTargetHint::RecentPosts => serde_json::json!({
            "include_recent_posts": true,
            "include_pinned_posts": true,
            "include_profile": true,
            "recent_posts_feed_fetch_limit": recent_posts_feed_fetch_limit,
            "recent_posts_min_top_level_posts": recent_posts_min_top_level_posts
        }),
    }
}

fn summary_recent_post_requirements(
    requested_summary_scope: RequestedSummaryScope,
) -> (usize, usize) {
    let requested_posts = match requested_summary_scope {
        RequestedSummaryScope::CurrentWindow => COLLECTION_SEARCH_PAGE_SIZE,
        RequestedSummaryScope::Count { requested_items } => requested_items,
        RequestedSummaryScope::Page { page_index } => page_index
            .saturating_add(1)
            .saturating_mul(COLLECTION_SEARCH_PAGE_SIZE),
        RequestedSummaryScope::PageRange { end_page, .. } => end_page
            .saturating_add(1)
            .saturating_mul(COLLECTION_SEARCH_PAGE_SIZE),
    }
    .max(COLLECTION_SEARCH_PAGE_SIZE);

    let feed_fetch_limit = requested_posts.saturating_mul(2).max(100).min(400);
    (feed_fetch_limit, requested_posts)
}

fn preferred_collection_order_hint(intent: SearchIntent) -> &'static str {
    match intent {
        SearchIntent::ReputationLists => {
            "clearsky_lists -> recent_replies_received -> actor_profile -> recent_posts"
        }
        SearchIntent::General => "narrowest sufficient collection first",
    }
}

fn validate_internal_tool_response(response: &str) -> InternalToolResponse {
    let trimmed = response.trim();
    if !trimmed.contains("TOOL_CALL") {
        return InternalToolResponse::FinalSummary(response.to_string());
    }

    let accepted = match extract_leading_tool_call_block(trimmed) {
        Ok(accepted) => accepted,
        Err(err) => return InternalToolResponse::Invalid(err.diagnostic().to_string()),
    };

    let tool_call = match parse_prompt_tool_call_result(&accepted.accepted_block) {
        Ok(tool_call) => tool_call,
        Err(err) => return InternalToolResponse::Invalid(err.diagnostic().to_string()),
    };

    InternalToolResponse::ToolCall(AcceptedInternalToolCall {
        tool_call,
        accepted_block: accepted.accepted_block,
        discarded_trailing: accepted.discarded_trailing,
    })
}

fn validate_internal_search_tool_call(
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
            optional_usize_arg(&tool_call.args, "page").map_err(|err| err.to_string())?;
            optional_usize_arg(&tool_call.args, "offset").map_err(|err| err.to_string())?;
            validate_collection_id(&collection_id)?;
        }
        "search_global_posts" => {
            require_string_arg(&tool_call.args, "query").map_err(|err| err.to_string())?;
        }
        other => return Err(format!("unknown internal tool `{other}`")),
    }

    Ok(tool_call.clone())
}

async fn complete_internal_tool_call_repair(
    query: &str,
    search_intent: SearchIntent,
    requested_summary_scope: RequestedSummaryScope,
    resolved_actor_refs: &[ResolvedActorRef],
    available_collections: &[LabeledPostCollection],
    invalid_tool_call: &AcceptedInternalToolCall,
    validation_error: &str,
    llm_client: &LlmApiClient,
) -> Result<Option<AcceptedInternalToolCall>, Box<dyn std::error::Error>> {
    let rendered_tool_inventory = render_internal_search_tool_protocol();
    let rendered_collections = render_repair_collection_inventory(available_collections);
    let rendered_actors = render_resolved_actor_refs(resolved_actor_refs);
    let parsed_args = serde_json::to_string_pretty(&invalid_tool_call.tool_call.args)?;
    let response = llm_client
        .complete_chat(
            vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You repair one invalid internal `search` tool call. Use the supplied runtime context to either return exactly one corrected TOOL_CALL block or the exact token `CANNOT_REPAIR`. Do not add prose. Do not invent actors, collection ids, tool names, or paging semantics. Only use exact collection ids from the provided inventory.".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: format!(
                        "Root query:\n{query}\n\nSearch intent:\n{}\n\nRequested summary scope:\n{}\n\nInvalid TOOL_CALL block:\n{}\n\nParsed invalid args:\n{}\n\nValidation error:\n{}\n\nResolved actor refs:\n{}\n\nAvailable cached collections:\n{}\n\nInternal tool inventory:\n{}\n\nReturn exactly one corrected TOOL_CALL block or `CANNOT_REPAIR`.",
                        search_intent.as_str(),
                        requested_summary_scope.render_for_planner(),
                        invalid_tool_call.accepted_block,
                        parsed_args,
                        validation_error,
                        rendered_actors,
                        rendered_collections,
                        rendered_tool_inventory
                    ),
                },
            ],
            INTERNAL_TOOL_REPAIR_MAX_OUTPUT_TOKENS,
        )
        .await?;

    parse_internal_tool_repair_response(&response)
}

fn parse_internal_tool_repair_response(
    response: &str,
) -> Result<Option<AcceptedInternalToolCall>, Box<dyn std::error::Error>> {
    if response.trim() == "CANNOT_REPAIR" {
        return Ok(None);
    }

    match validate_internal_tool_response(response) {
        InternalToolResponse::ToolCall(tool_call) => Ok(Some(tool_call)),
        InternalToolResponse::FinalSummary(_) => Ok(None),
        InternalToolResponse::Invalid(reason) => {
            Err(format!("tool-call repair returned invalid output: {reason}").into())
        }
    }
}

fn relevant_repair_inventory_collections(
    store: &NotificationStore,
    resolved_actor_refs: Option<&[ResolvedActorRef]>,
) -> Vec<LabeledPostCollection> {
    let mut collections = if let Some(resolved_actor_refs) = resolved_actor_refs {
        resolved_actor_refs
            .iter()
            .flat_map(|actor| {
                store
                    .actor_post_collections(&actor.did)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    } else {
        store
            .post_collections()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    };
    dedupe_collections_by_id(&mut collections);
    collections.sort_by(|left, right| left.id.cmp(&right.id));
    collections
}

fn render_repair_collection_inventory(collections: &[LabeledPostCollection]) -> String {
    if collections.is_empty() {
        return "No cached collections are currently available.".to_string();
    }

    collections
        .iter()
        .map(|collection| {
            let mut lines = vec![
                format!("collection_id: {}", collection.id),
                format!("collection_label: {}", collection.label),
                format!("collection_kind: {}", collection.collection_kind),
                format!("item_count: {}", collection.posts.len()),
            ];
            if let Some(actor_did) = collection.actor_did.as_deref() {
                lines.push(format!("actor_did: {actor_did}"));
            }
            lines.join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn deterministic_repair_internal_search_tool_call(
    tool_call: &PromptToolCall,
    query: &str,
    search_intent: SearchIntent,
    available_collections: &[LabeledPostCollection],
) -> Option<PromptToolCall> {
    if tool_call.name != "collection_search" {
        return None;
    }

    let collection_id = require_string_arg(&tool_call.args, "collection_id").ok()?;
    if !collection_id.starts_with("did:") {
        return None;
    }

    let repaired_collection_id = choose_deterministic_collection_id_for_actor(
        &collection_id,
        query,
        search_intent,
        available_collections,
    )?;
    let mut repaired_tool_call = tool_call.clone();
    repaired_tool_call.args["collection_id"] = Value::String(repaired_collection_id);
    validate_internal_search_tool_call(&repaired_tool_call).ok()
}

fn choose_deterministic_collection_id_for_actor(
    actor_did: &str,
    query: &str,
    search_intent: SearchIntent,
    available_collections: &[LabeledPostCollection],
) -> Option<String> {
    let actor_collections = available_collections
        .iter()
        .filter(|collection| collection.actor_did.as_deref() == Some(actor_did))
        .collect::<Vec<_>>();

    if actor_collections.len() == 1 {
        return Some(actor_collections[0].id.clone());
    }

    let preferred_kinds = deterministic_actor_collection_kind_preferences(query, search_intent);
    for kind in preferred_kinds {
        let matches = actor_collections
            .iter()
            .filter(|collection| collection.collection_kind == *kind)
            .collect::<Vec<_>>();
        if matches.len() == 1 {
            return Some(matches[0].id.clone());
        }
    }

    None
}

fn deterministic_actor_collection_kind_preferences(
    query: &str,
    search_intent: SearchIntent,
) -> &'static [&'static str] {
    let lower = query.to_ascii_lowercase();
    if lower.contains("repl") {
        return &[
            "recent_replies_sent",
            "recent_posts",
            "recent_replies_received",
            "replies_to_actor",
        ];
    }
    if lower.contains("profile") || lower.contains("bio") || lower.contains("who is") {
        return &["actor_profile"];
    }
    if matches!(search_intent, SearchIntent::ReputationLists) {
        return &[
            "clearsky_lists",
            "recent_replies_received",
            "actor_profile",
            "recent_posts",
            "recent_posts_unaddressed",
        ];
    }
    &["recent_posts", "recent_posts_unaddressed", "pinned_posts"]
}

fn render_tool_call_block(tool_call: &PromptToolCall) -> Result<String, serde_json::Error> {
    Ok(format!(
        "TOOL_CALL\nname: {}\nargs: {}",
        tool_call.name,
        serde_json::to_string(&tool_call.args)?
    ))
}

fn parse_requested_summary_scope_args(args: &Value) -> Result<RequestedSummaryScope, String> {
    let kind = require_string_arg(args, "kind").map_err(|err| err.to_string())?;
    match kind.as_str() {
        "current_window" => Ok(RequestedSummaryScope::CurrentWindow),
        "count" => {
            let requested_items =
                require_usize_arg(args, "requested_items").map_err(|err| err.to_string())?;
            Ok(RequestedSummaryScope::Count { requested_items })
        }
        "page" => {
            let page_index =
                require_usize_arg(args, "page_index").map_err(|err| err.to_string())?;
            Ok(RequestedSummaryScope::Page { page_index })
        }
        "page_range" => {
            let start_page =
                require_usize_arg(args, "start_page").map_err(|err| err.to_string())?;
            let end_page = require_usize_arg(args, "end_page").map_err(|err| err.to_string())?;
            Ok(RequestedSummaryScope::PageRange {
                start_page: start_page.min(end_page),
                end_page: start_page.max(end_page),
            })
        }
        other => Err(format!(
            "invalid summary scope kind `{other}`; expected `current_window`, `count`, `page`, or `page_range`"
        )),
    }
}

fn validate_collection_id(collection_id: &str) -> Result<(), String> {
    if collection_id.starts_with("did:") {
        return Err(format!(
            "collection id `{collection_id}` is a bare DID, not a cached collection id; use an exact collection id such as `recent_posts:{collection_id}` or `actor_profile:{collection_id}`"
        ));
    }

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
    tool_kind: CollectionLeafToolKind,
    collection: &LabeledPostCollection,
    prompt: &str,
    requested_summary_scope: RequestedSummaryScope,
    execution: &LlmSearchExecution,
) -> LLMContext {
    let mut context = LLMContext::new(tool_kind.review_agent_kind().system_prompt());
    context.push_section_with_kind("Search Prompt", ContextSectionKind::CurrentTask, prompt);
    if tool_kind.is_coverage_oriented() {
        context.push_section_with_kind(
            "Harness Scope Assessment",
            ContextSectionKind::CurrentTask,
            render_summary_scope_assessment(requested_summary_scope, collection),
        );
    }
    context.push_section_with_kind(
        "Collection Evidence",
        ContextSectionKind::ReviewEvidence,
        render_review_collection_evidence(tool_kind, collection, execution),
    );
    context.push_section_with_kind(
        "Proposed Summary",
        ContextSectionKind::ParentSearchResults,
        render_llm_result(execution.original_result.as_ref()),
    );
    context
}

fn render_review_collection_evidence(
    tool_kind: CollectionLeafToolKind,
    collection: &LabeledPostCollection,
    execution: &LlmSearchExecution,
) -> String {
    let selected_uris = execution
        .original_result
        .as_ref()
        .map(|result| {
            result
                .selected_item_uris()
                .into_iter()
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    let posts = if tool_kind.is_coverage_oriented() {
        collection.posts.iter().collect::<Vec<_>>()
    } else if selected_uris.is_empty() {
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
    if tool_kind.is_coverage_oriented() {
        lines.push(format!(
            "search_window_offset: {}",
            collection_window_offset(collection).unwrap_or(0)
        ));
        lines.push(format!(
            "search_window_total_items: {}",
            collection_window_size(collection)
        ));
    }
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
    let additional_pages_needed = response
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("additional_pages_needed:")
                .map(str::trim)
        })
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let grounded = response
        .lines()
        .find_map(|line| line.trim().strip_prefix("grounded:").map(str::trim))
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(matches!(status, CollectionReviewStatus::Pass) || additional_pages_needed);
    let sufficient = response
        .lines()
        .find_map(|line| line.trim().strip_prefix("sufficient:").map(str::trim))
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(matches!(status, CollectionReviewStatus::Pass));
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
    let next_offset = response.lines().find_map(|line| {
        line.trim()
            .strip_prefix("next_offset:")
            .map(str::trim)
            .and_then(|value| value.parse::<usize>().ok())
    });
    let next_page = response.lines().find_map(|line| {
        line.trim()
            .strip_prefix("next_page:")
            .map(str::trim)
            .and_then(|value| value.parse::<usize>().ok())
    });
    let required_total_items = response.lines().find_map(|line| {
        line.trim()
            .strip_prefix("required_total_items:")
            .map(str::trim)
            .and_then(|value| value.parse::<usize>().ok())
    });

    Some(CollectionReviewVerdict {
        status,
        grounded,
        sufficient,
        reason,
        repair_needed,
        repair_instructions,
        additional_pages_needed,
        next_page,
        next_offset,
        required_total_items,
    })
}

fn heuristic_collection_review(
    tool_kind: CollectionLeafToolKind,
    collection: &LabeledPostCollection,
    requested_summary_scope: RequestedSummaryScope,
    execution: &LlmSearchExecution,
) -> CollectionReviewVerdict {
    let Some(result) = execution
        .result
        .as_ref()
        .or(execution.original_result.as_ref())
    else {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            grounded: false,
            sufficient: false,
            reason: "No usable `summary:` paragraph exists.".to_string(),
            repair_needed: false,
            repair_instructions: None,
            additional_pages_needed: false,
            next_page: None,
            next_offset: None,
            required_total_items: None,
        };
    };

    let summary = result.summary().trim();
    if summary.is_empty() {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            grounded: false,
            sufficient: false,
            reason: "No usable `summary:` paragraph exists.".to_string(),
            repair_needed: !tool_kind.is_coverage_oriented(),
            repair_instructions: Some(
                "Rewrite the summary as one paragraph grounded in the selected records."
                    .to_string(),
            ),
            additional_pages_needed: false,
            next_page: None,
            next_offset: None,
            required_total_items: None,
        };
    }

    if summary.contains("\n\n") {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            grounded: false,
            sufficient: false,
            reason: "The summary is not a single paragraph.".to_string(),
            repair_needed: !tool_kind.is_coverage_oriented(),
            repair_instructions: Some(
                "Condense the output into one grounded paragraph.".to_string(),
            ),
            additional_pages_needed: false,
            next_page: None,
            next_offset: None,
            required_total_items: None,
        };
    }

    let fallback_markers = [
        "model response did not yield a usable structured `summary:` field",
        "model did not return a fully structured summary paragraph",
        "Grounded evidence was found in",
    ];
    if fallback_markers
        .iter()
        .any(|marker| summary.contains(marker))
    {
        return CollectionReviewVerdict {
            status: CollectionReviewStatus::Fail,
            grounded: false,
            sufficient: false,
            reason: "The summary is fallback diagnostic text rather than a grounded collection summary.".to_string(),
            repair_needed: !tool_kind.is_coverage_oriented(),
            repair_instructions: Some(
                "Rewrite the summary from the selected records, emphasizing repeated names, exact phrases, and the strongest versus secondary evidence."
                    .to_string(),
            ),
            additional_pages_needed: false,
            next_page: None,
            next_offset: None,
            required_total_items: None,
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
            grounded: false,
            sufficient: false,
            reason: "The summary is dominated by identifiers or metadata placeholders.".to_string(),
            repair_needed: !tool_kind.is_coverage_oriented(),
            repair_instructions: Some(
                "Replace metadata-heavy text with actual list names, descriptions, or post text."
                    .to_string(),
            ),
            additional_pages_needed: false,
            next_page: None,
            next_offset: None,
            required_total_items: None,
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
            grounded: false,
            sufficient: false,
            reason: "The summary omits meaningful text that was available in the matched records."
                .to_string(),
            repair_needed: !tool_kind.is_coverage_oriented(),
            repair_instructions: Some(
                "Include exact short phrases, list names, list descriptions, or matched reply text from the selected records."
                    .to_string(),
            ),
            additional_pages_needed: false,
            next_page: None,
            next_offset: None,
            required_total_items: None,
        };
    }

    if tool_kind.is_coverage_oriented() {
        let Some(result) = result.as_summary() else {
            return fail_summary_review(
                false,
                "Expected a summary result for coverage-oriented review.",
            );
        };
        return heuristic_summary_review(collection, requested_summary_scope, result);
    }

    CollectionReviewVerdict {
        status: CollectionReviewStatus::Pass,
        grounded: true,
        sufficient: true,
        reason: if tool_kind.is_coverage_oriented() {
            "The summary is grounded and the coverage accounting matches the requested collection window."
                .to_string()
        } else {
            "The summary is grounded in the selected records and contains substantive evidence."
                .to_string()
        },
        repair_needed: false,
        repair_instructions: None,
        additional_pages_needed: false,
        next_page: None,
        next_offset: None,
        required_total_items: None,
    }
}

fn heuristic_summary_review(
    collection: &LabeledPostCollection,
    requested_summary_scope: RequestedSummaryScope,
    result: &LlmSummaryResult,
) -> CollectionReviewVerdict {
    let valid_window_uris = collection
        .posts
        .iter()
        .map(|post| post.uri.as_str())
        .collect::<HashSet<_>>();
    let covered = &result.covered_item_uris;
    let omitted = &result.omitted_item_uris;
    let expected_window_items = collection_window_size(collection);
    let expected_offset = collection_window_offset(collection).unwrap_or(0);
    let processed_window_size = result.processed_window_size();
    let processed_window_offset = result.processed_window_offset();

    if processed_window_size != Some(expected_window_items) {
        return fail_summary_review(
            false,
            "The claimed `window_size` does not match the actual collection window size.",
        );
    }

    if processed_window_offset != Some(expected_offset) {
        return fail_summary_review(
            false,
            "The claimed `window_offset` does not match the requested collection window.",
        );
    }

    let expected_page_index = expected_offset / COLLECTION_SEARCH_PAGE_SIZE;
    if result.processed_page_index() != Some(expected_page_index) {
        return fail_summary_review(
            false,
            "The claimed `page_index` does not match the requested collection window.",
        );
    }

    let available_total_items = collection_available_total_items(collection);
    if result.processed_collection_total_items() != Some(available_total_items) {
        return fail_summary_review(
            false,
            "The claimed `collection_total_items` does not match the available collection size.",
        );
    }

    let expected_has_more =
        expected_offset.saturating_add(expected_window_items) < available_total_items;
    if result.has_more != Some(expected_has_more) {
        return fail_summary_review(
            false,
            "The claimed `has_more` flag does not match the processed collection window.",
        );
    }

    let mut seen = HashSet::new();
    for uri in covered.iter().chain(omitted.iter()) {
        if !valid_window_uris.contains(uri.as_str()) {
            return fail_summary_review(
                false,
                "Coverage accounting includes URIs that are not part of the requested collection window.",
            );
        }
        if !seen.insert(uri.as_str()) {
            return fail_summary_review(
                false,
                "Coverage accounting repeats the same URI more than once.",
            );
        }
    }

    if seen.len() != collection.posts.len() {
        return fail_summary_review(
            false,
            "Coverage accounting does not fully match the actual items in the requested window.",
        );
    }

    let ranges = vec![(expected_offset, expected_window_items)];
    summary_scope_verdict(requested_summary_scope, &ranges, available_total_items)
}

fn fail_summary_review(grounded: bool, reason: impl Into<String>) -> CollectionReviewVerdict {
    CollectionReviewVerdict {
        status: CollectionReviewStatus::Fail,
        grounded,
        sufficient: false,
        reason: reason.into(),
        repair_needed: false,
        repair_instructions: None,
        additional_pages_needed: false,
        next_page: None,
        next_offset: None,
        required_total_items: None,
    }
}

fn scope_required_total_items(
    scope: RequestedSummaryScope,
    available_total_items: usize,
) -> Option<usize> {
    match scope {
        RequestedSummaryScope::CurrentWindow => None,
        RequestedSummaryScope::Count { requested_items } => {
            Some(requested_items.min(available_total_items))
        }
        RequestedSummaryScope::Page { page_index } => {
            required_page_window_size(page_index, available_total_items)
        }
        RequestedSummaryScope::PageRange {
            start_page,
            end_page,
        } => {
            let start_offset = start_page.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE);
            let end_offset = end_page
                .saturating_add(1)
                .saturating_mul(COLLECTION_SEARCH_PAGE_SIZE)
                .min(available_total_items);
            Some(end_offset.saturating_sub(start_offset))
        }
    }
}

fn detect_requested_summary_scope(prompt: &str) -> RequestedSummaryScope {
    let tokens = prompt
        .split_whitespace()
        .map(normalize_scope_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        if token == "pages" {
            if let Some((start, end)) = parse_page_range_tokens(&tokens, index + 1) {
                return RequestedSummaryScope::PageRange {
                    start_page: start,
                    end_page: end,
                };
            }
        }
        if token == "page" {
            if let Some(raw_page) = tokens
                .get(index + 1)
                .and_then(|token| parse_scope_number(token))
            {
                return RequestedSummaryScope::Page {
                    page_index: interpret_user_page_number(raw_page),
                };
            }
        }
        if token == "count" {
            if let Some(requested_items) = tokens
                .get(index + 1)
                .and_then(|token| parse_scope_number(token))
            {
                return RequestedSummaryScope::Count { requested_items };
            }
        }
    }

    for (index, token) in tokens.iter().enumerate() {
        let Some(value) = parse_scope_number(token) else {
            continue;
        };
        let prev = index
            .checked_sub(1)
            .and_then(|i| tokens.get(i))
            .map(String::as_str);
        let next = tokens.get(index + 1).map(String::as_str);
        if matches!(prev, Some("page")) || matches!(next, Some("page")) {
            return RequestedSummaryScope::Page {
                page_index: interpret_user_page_number(value),
            };
        }
        if matches!(prev, Some("pages")) {
            if let Some(end) = next.and_then(parse_scope_number) {
                let zero_based = interpret_user_page_range(value, end);
                return RequestedSummaryScope::PageRange {
                    start_page: zero_based.0,
                    end_page: zero_based.1,
                };
            }
        }
        if matches!(prev, Some("count" | "first" | "last"))
            || matches!(
                next,
                Some("post" | "posts" | "thing" | "things" | "item" | "items")
            )
        {
            return RequestedSummaryScope::Count {
                requested_items: value,
            };
        }
    }

    RequestedSummaryScope::CurrentWindow
}

pub(crate) fn summary_scope_initial_offset(scope: RequestedSummaryScope) -> usize {
    match scope {
        RequestedSummaryScope::CurrentWindow | RequestedSummaryScope::Count { .. } => 0,
        RequestedSummaryScope::Page { page_index } => {
            page_index.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE)
        }
        RequestedSummaryScope::PageRange { start_page, .. } => {
            start_page.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE)
        }
    }
}

pub(crate) fn summary_scope_max_pages(
    scope: RequestedSummaryScope,
    available_total_items: usize,
) -> usize {
    match scope {
        RequestedSummaryScope::CurrentWindow => 1,
        RequestedSummaryScope::Count { requested_items } => requested_items
            .min(available_total_items)
            .div_ceil(COLLECTION_SEARCH_PAGE_SIZE)
            .max(1),
        RequestedSummaryScope::Page { .. } => 1,
        RequestedSummaryScope::PageRange {
            start_page,
            end_page,
        } => end_page.saturating_sub(start_page).saturating_add(1).max(1),
    }
}

fn render_summary_scope_assessment(
    requested_summary_scope: RequestedSummaryScope,
    collection: &LabeledPostCollection,
) -> String {
    let available_total_items = collection_available_total_items(collection);
    match requested_summary_scope {
        RequestedSummaryScope::CurrentWindow => format!(
            "requested_scope: current_window\npage_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page\navailable_total_items: {}\ncurrent_window_offset: {}\ncurrent_window_size: {}",
            available_total_items,
            collection_window_offset(collection).unwrap_or(0),
            collection_window_size(collection)
        ),
        RequestedSummaryScope::Count { requested_items } => format!(
            "requested_scope: count {}\nrequired_total_items: {}\npage_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page\navailable_total_items: {}\ncurrent_window_offset: {}\ncurrent_window_size: {}",
            requested_items,
            requested_items.min(available_total_items),
            available_total_items,
            collection_window_offset(collection).unwrap_or(0),
            collection_window_size(collection)
        ),
        RequestedSummaryScope::Page { page_index } => format!(
            "requested_scope: page {}\nrequested_offset: {}\npage_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page\navailable_total_items: {}\ncurrent_window_offset: {}\ncurrent_window_size: {}",
            page_index.saturating_add(1),
            page_index.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE),
            available_total_items,
            collection_window_offset(collection).unwrap_or(0),
            collection_window_size(collection)
        ),
        RequestedSummaryScope::PageRange {
            start_page,
            end_page,
        } => format!(
            "requested_scope: pages {}-{}\nrequired_total_items: {}\npage_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page\navailable_total_items: {}\ncurrent_window_offset: {}\ncurrent_window_size: {}",
            start_page.saturating_add(1),
            end_page.saturating_add(1),
            scope_required_total_items(
                RequestedSummaryScope::PageRange {
                    start_page,
                    end_page,
                },
                available_total_items,
            )
            .unwrap_or(0),
            available_total_items,
            collection_window_offset(collection).unwrap_or(0),
            collection_window_size(collection)
        ),
    }
}

fn normalize_scope_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .to_ascii_lowercase()
}

fn parse_scope_number(token: &str) -> Option<usize> {
    token.parse::<usize>().ok()
}

fn parse_page_range_tokens(tokens: &[String], start_index: usize) -> Option<(usize, usize)> {
    let token = tokens.get(start_index)?;
    if let Some((left, right)) = token.split_once('-') {
        let start = parse_scope_number(left)?;
        let end = parse_scope_number(right)?;
        return Some(interpret_user_page_range(start, end));
    }
    let start = parse_scope_number(token)?;
    let separator = tokens.get(start_index + 1).map(String::as_str);
    let end_token_index = match separator {
        Some("-" | "to") => start_index + 2,
        _ => start_index + 1,
    };
    let end = tokens
        .get(end_token_index)
        .and_then(|token| parse_scope_number(token))?;
    Some(interpret_user_page_range(start, end))
}

fn interpret_user_page_number(page_number: usize) -> usize {
    page_number.saturating_sub(1)
}

fn interpret_user_page_range(start: usize, end: usize) -> (usize, usize) {
    let zero_based = start == 0 || end == 0;
    let normalize = |value: usize| {
        if zero_based {
            value
        } else {
            value.saturating_sub(1)
        }
    };
    let start_page = normalize(start);
    let end_page = normalize(end);
    (start_page.min(end_page), start_page.max(end_page))
}

fn required_page_window_size(page_index: usize, available_total_items: usize) -> Option<usize> {
    let start_offset = page_index.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE);
    if start_offset >= available_total_items {
        return None;
    }
    Some((available_total_items - start_offset).min(COLLECTION_SEARCH_PAGE_SIZE))
}

fn page_is_covered(
    page_index: usize,
    ranges: &[(usize, usize)],
    available_total_items: usize,
) -> bool {
    let Some(required_window_size) = required_page_window_size(page_index, available_total_items)
    else {
        return false;
    };
    let page_offset = page_index.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE);
    ranges
        .iter()
        .any(|(start, len)| *start == page_offset && *len >= required_window_size)
}

fn contiguous_coverage_end(ranges: &[(usize, usize)]) -> usize {
    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|(start, _)| *start);

    let mut contiguous_coverage = 0usize;
    for (start, len) in sorted {
        if start > contiguous_coverage {
            break;
        }
        contiguous_coverage = contiguous_coverage.max(start.saturating_add(len));
    }
    contiguous_coverage
}

fn summary_scope_verdict(
    scope: RequestedSummaryScope,
    ranges: &[(usize, usize)],
    available_total_items: usize,
) -> CollectionReviewVerdict {
    let contiguous_coverage = contiguous_coverage_end(ranges);
    let grounded = true;

    let (sufficient, additional_pages_needed, next_page, next_offset, required_total_items, reason) =
        match scope {
            RequestedSummaryScope::CurrentWindow => (
                true,
                false,
                None,
                None,
                None,
                "The summary is grounded and the processed window metadata matches the requested current window."
                    .to_string(),
            ),
            RequestedSummaryScope::Count { requested_items } => {
                let required_total_items = requested_items.min(available_total_items);
                let exhausted_available_scope = contiguous_coverage >= available_total_items;
                let sufficient =
                    contiguous_coverage >= required_total_items || exhausted_available_scope;
                (
                    sufficient,
                    !sufficient && !exhausted_available_scope,
                    if sufficient {
                        None
                    } else if exhausted_available_scope {
                        None
                    } else {
                        Some(contiguous_coverage / COLLECTION_SEARCH_PAGE_SIZE)
                    },
                    if sufficient || exhausted_available_scope {
                        None
                    } else {
                        Some(contiguous_coverage)
                    },
                    Some(required_total_items),
                    if sufficient {
                        if required_total_items < requested_items {
                            format!(
                                "Grounded summary coverage reaches all {} available item(s), exhausting the available collection even though {} item(s) were requested.",
                                required_total_items, requested_items
                            )
                        } else {
                            format!(
                                "Grounded summary coverage reaches {} item(s), satisfying the requested {} item scope.",
                                contiguous_coverage, required_total_items
                            )
                        }
                    } else {
                        format!(
                            "Grounded summary coverage currently reaches {} item(s), but {} item(s) are required before parent synthesis is sufficient.",
                            contiguous_coverage, required_total_items
                        )
                    },
                )
            }
            RequestedSummaryScope::Page { page_index } => {
                let required_offset = page_index.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE);
                let sufficient = page_is_covered(page_index, ranges, available_total_items);
                (
                    sufficient,
                    false,
                    if sufficient { None } else { Some(page_index) },
                    if sufficient { None } else { Some(required_offset) },
                    required_page_window_size(page_index, available_total_items),
                    if sufficient {
                        format!(
                            "The requested page {} was summarized with grounded full-window accounting.",
                            page_index.saturating_add(1)
                        )
                    } else {
                        format!(
                            "The prompt requested page {}, but the processed windows do not include offset {}.",
                            page_index.saturating_add(1),
                            required_offset
                        )
                    },
                )
            }
            RequestedSummaryScope::PageRange {
                start_page,
                end_page,
            } => {
                let first_missing_page = (start_page..=end_page)
                    .find(|page_index| !page_is_covered(*page_index, ranges, available_total_items));
                let required_total_items = scope_required_total_items(
                    RequestedSummaryScope::PageRange {
                        start_page,
                        end_page,
                    },
                    available_total_items,
                );
                (
                    first_missing_page.is_none(),
                    first_missing_page.is_some(),
                    first_missing_page,
                    first_missing_page.map(|page_index| {
                        page_index.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE)
                    }),
                    required_total_items,
                    if let Some(page_index) = first_missing_page {
                        format!(
                            "The requested page range {}-{} is still missing page {}.",
                            start_page.saturating_add(1),
                            end_page.saturating_add(1),
                            page_index.saturating_add(1)
                        )
                    } else {
                        format!(
                            "The requested page range {}-{} was summarized with grounded full-window accounting.",
                            start_page.saturating_add(1),
                            end_page.saturating_add(1)
                        )
                    },
                )
            }
        };

    CollectionReviewVerdict {
        status: if grounded && sufficient {
            CollectionReviewStatus::Pass
        } else {
            CollectionReviewStatus::Fail
        },
        grounded,
        sufficient,
        reason,
        repair_needed: false,
        repair_instructions: None,
        additional_pages_needed,
        next_page,
        next_offset,
        required_total_items,
    }
}

fn selected_record_hints(
    collection: &LabeledPostCollection,
    result: &CollectionLeafResult,
) -> Vec<String> {
    let mut hints = Vec::new();
    for uri in result.selected_item_uris() {
        let Some(post) = collection.posts.iter().find(|post| post.uri == uri) else {
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
        let summary =
            deterministic_repair_summary(collection, &original_result.selected_item_uris());
        if !summary.trim().is_empty() {
            return summary;
        }
    }
    verdict.repair_instructions.clone().unwrap_or_else(|| {
        "The selected records did not support a grounded repaired summary.".to_string()
    })
}

fn deterministic_repair_summary(
    collection: &LabeledPostCollection,
    selected_uris: &[&str],
) -> String {
    let selected_posts = selected_uris
        .iter()
        .filter_map(|uri| collection.posts.iter().find(|post| post.uri == *uri))
        .collect::<Vec<_>>();

    if selected_posts.is_empty() {
        return String::new();
    }

    if collection.collection_kind == "clearsky_lists" {
        return deterministic_clearsky_list_summary(&selected_posts);
    }

    deterministic_post_summary(&selected_posts)
}

fn deterministic_clearsky_list_summary(selected_posts: &[&PostRecord]) -> String {
    let mut sentences = vec![format!(
        "Selected moderation-list evidence is drawn from {} cited record(s).",
        selected_posts.len()
    )];

    let evidence = selected_posts
        .iter()
        .enumerate()
        .map(|(index, post)| render_repaired_list_evidence_item(index, post))
        .collect::<Vec<_>>();

    let lower_names = selected_posts
        .iter()
        .filter_map(|post| body_field_map(&post.body).get("list_name").cloned())
        .map(|name| name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let has_negative = lower_names.iter().any(|name| {
        name.contains("ai")
            || name.contains("crypto")
            || name.contains("cull")
            || name.contains("shithead")
            || name.contains("cunts")
    });
    let has_neutral_or_positive = lower_names.iter().any(|name| {
        name.starts_with("follows of @")
            || name.contains("insightful")
            || name.contains("unmissable")
            || name.contains("commentary")
    });
    if has_negative && has_neutral_or_positive {
        sentences.push(
            "The cited items mix neutral or positive labels with more judgmental AI/crypto-themed labels."
                .to_string(),
        );
    }
    sentences.push(evidence.join(" "));
    sentences.join(" ")
}

fn deterministic_post_summary(selected_posts: &[&PostRecord]) -> String {
    let source_post_count = selected_posts
        .iter()
        .filter_map(|post| body_field_map(&post.body).get("source_post_uri").cloned())
        .collect::<HashSet<_>>()
        .len();
    let reply_like_count = selected_posts
        .iter()
        .filter(|post| {
            body_field_map(&post.body)
                .get("reply_text")
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        })
        .count();
    let mut sentences = vec![format!(
        "Selected evidence is drawn from {} cited record(s){}.",
        selected_posts.len(),
        if source_post_count > 0 {
            format!(" across {} source post(s)", source_post_count)
        } else {
            String::new()
        }
    )];
    if reply_like_count > 0 {
        sentences.push(format!(
            "{} of those cited records include captured reply text.",
            reply_like_count
        ));
    }

    let evidence = selected_posts
        .iter()
        .enumerate()
        .map(|(index, post)| render_repaired_post_evidence_item(index, post))
        .collect::<Vec<_>>();
    sentences.push(evidence.join(" "));
    sentences.join(" ")
}

fn render_repaired_list_evidence_item(index: usize, post: &PostRecord) -> String {
    let fields = body_field_map(&post.body);
    let name = fields
        .get("list_name")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("<missing list name>");
    let description = fields
        .get("list_description")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| truncate_chars(&collapse_whitespace(value), 140));

    match description {
        Some(description) => format!("[{index}] \"{name}\" - \"{description}\"."),
        None => format!("[{index}] \"{name}\" - no description."),
    }
}

fn render_repaired_post_evidence_item(index: usize, post: &PostRecord) -> String {
    let snippet = extract_repaired_post_snippet(post)
        .unwrap_or_else(|| "no grounded text was captured for this item".to_string());
    format!(
        "[{index}] @{}: \"{}\".",
        post.author_handle,
        truncate_chars(&collapse_whitespace(&snippet), 160)
    )
}

fn extract_repaired_post_snippet(post: &PostRecord) -> Option<String> {
    let fields = body_field_map(&post.body);
    if let Some(reply_text) = fields.get("reply_text").map(|value| value.trim()) {
        if !reply_text.is_empty() {
            return Some(reply_text.to_string());
        }
    }

    let filtered_lines = post
        .body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !matches_repair_metadata_line(line))
        .collect::<Vec<_>>();

    if filtered_lines.is_empty() {
        None
    } else {
        Some(filtered_lines.join(" "))
    }
}

fn matches_repair_metadata_line(line: &str) -> bool {
    [
        "source_post_uri:",
        "source_collection_id:",
        "source_collection_label:",
        "link:",
        "uri:",
        "did:",
        "author:",
        "reply_text:",
        "list_name:",
        "list_description:",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn join_quoted(items: &[String]) -> String {
    match items.len() {
        0 => "no concrete matched phrases".to_string(),
        1 => format!("\"{}\"", items[0]),
        2 => format!("\"{}\" and \"{}\"", items[0], items[1]),
        _ => {
            let mut rendered = items
                .iter()
                .map(|item| format!("\"{}\"", item))
                .collect::<Vec<_>>();
            let last = rendered.pop().unwrap_or_default();
            format!("{}, and {}", rendered.join(", "), last)
        }
    }
}

pub(crate) fn render_review_summary(
    verdict: Option<&CollectionReviewVerdict>,
    repair_diagnostic: Option<&str>,
) -> String {
    let Some(verdict) = verdict else {
        return "status: pass".to_string();
    };
    let mut lines = vec![
        format!("status: {}", verdict.status.as_str()),
        format!("grounded: {}", verdict.grounded),
        format!("sufficient: {}", verdict.sufficient),
        format!("reason: {}", verdict.reason),
        format!("repair_needed: {}", verdict.repair_needed),
    ];
    if let Some(instructions) = verdict.repair_instructions.as_deref() {
        lines.push(format!("repair_instructions: {instructions}"));
    }
    lines.push(format!(
        "additional_pages_needed: {}",
        verdict.additional_pages_needed
    ));
    if let Some(next_page) = verdict.next_page {
        lines.push(format!("next_page: {next_page}"));
    }
    if let Some(next_offset) = verdict.next_offset {
        lines.push(format!("next_offset: {next_offset}"));
    }
    if let Some(required_total_items) = verdict.required_total_items {
        lines.push(format!("required_total_items: {required_total_items}"));
    }
    if let Some(diagnostic) = repair_diagnostic {
        lines.push(format!("repair_diagnostic: {diagnostic}"));
    }
    lines.join("\n")
}

fn render_internal_search_tool_protocol() -> String {
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
            description: "Use when you need the strongest supporting records from one exact validated cached collection window rather than full coverage.",
            arguments: &[
                "collection_id (string, required): exact cached collection ID",
                "prompt (string, required): what to look for in that collection",
                "page (integer, optional): zero-based 25-item page of the collection to search",
                "offset (integer, optional): zero-based item offset into the collection; takes precedence over `page`",
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
        "If one internal tool would help, emit exactly one tool request block in this format and nothing else:\n\nTOOL_CALL\nname: <tool_name>\nargs: {{...}}\n\nStrict mode rules:\n- emit exactly one TOOL_CALL block and no surrounding prose\n- use exact valid DIDs\n- use exact valid cached collection IDs only\n- for reputation, sentiment, or list questions, prefer `clearsky_lists` first and only expand to `recent_replies_received`, `actor_profile`, or `recent_posts` when needed for contrast or missing evidence\n- prefer `recent_posts` over `recent_posts_unaddressed` unless the task explicitly needs top-level-only posts\n- use `collection_search` when you need only the strongest evidence from a window\n\nAvailable internal tools:\n\n{inventory}"
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

fn parse_collection_tool_result(
    collection: &LabeledPostCollection,
    response: &str,
    tool_kind: CollectionLeafToolKind,
    window_start: Option<usize>,
) -> ParsedCollectionToolResult {
    if tool_kind.is_coverage_oriented() {
        match parse_collection_summary_result_tool_call(collection, response) {
            Ok(result) => {
                return ParsedCollectionToolResult {
                    result: Some(CollectionLeafResult::Summary(result)),
                    diagnostic: None,
                };
            }
            Err(tool_call_err) => match parse_collection_summary_result_tagged(response) {
                Ok(result) => {
                    return ParsedCollectionToolResult {
                        result: Some(CollectionLeafResult::Summary(normalize_summary_result(
                            collection, result,
                        ))),
                        diagnostic: Some(format!(
                            "summary result used legacy tagged parsing after tool-call parsing failed ({tool_call_err})"
                        )),
                    };
                }
                Err(tagged_err) => match parse_collection_summary_result_json(collection, response)
                {
                    Ok(result) => {
                        return ParsedCollectionToolResult {
                            result: Some(CollectionLeafResult::Summary(result)),
                            diagnostic: Some(format!(
                                "summary result used json parsing after tool-call parsing failed ({tool_call_err}) and tagged parser failed ({tagged_err})"
                            )),
                        };
                    }
                    Err(json_err) => {
                        match parse_collection_summary_result_tagged_partial(
                            collection,
                            response,
                            window_start,
                        ) {
                            Ok((result, repair_diagnostic)) => {
                                return ParsedCollectionToolResult {
                                    result: Some(CollectionLeafResult::Summary(result)),
                                    diagnostic: Some(format!(
                                        "summary result parsing needed partial recovery: tool-call parser failed ({tool_call_err}); tagged parser failed ({tagged_err}); json parser failed ({json_err}); {repair_diagnostic}"
                                    )),
                                };
                            }
                            Err(partial_err) => {
                                return ParsedCollectionToolResult {
                                    result: None,
                                    diagnostic: Some(format!(
                                        "summary result parsing failed: tool-call parser failed ({tool_call_err}); tagged parser failed ({tagged_err}); json parser failed ({json_err}); partial tagged repair failed ({partial_err})"
                                    )),
                                };
                            }
                        }
                    }
                },
            },
        }
    }

    if let Some(result) = parse_collection_tool_result_json(collection, response, tool_kind) {
        return ParsedCollectionToolResult {
            result: Some(result),
            diagnostic: None,
        };
    }

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
        return ParsedCollectionToolResult {
            result: None,
            diagnostic: Some(
                "search result parsing failed: no valid matching collection URIs were recovered from the model response"
                    .to_string(),
            ),
        };
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

    ParsedCollectionToolResult {
        result: Some(CollectionLeafResult::Search(LlmSearchResult {
            title,
            summary,
            search_results,
        })),
        diagnostic: None,
    }
}

fn parse_llm_search_result(
    collection: &LabeledPostCollection,
    response: &str,
) -> Option<LlmSearchResult> {
    parse_collection_tool_result(collection, response, CollectionLeafToolKind::Search, None)
        .result
        .and_then(|result| match result {
            CollectionLeafResult::Search(result) => Some(result),
            CollectionLeafResult::Summary(_) => None,
            CollectionLeafResult::RawWindow(_) => None,
        })
}

fn parse_collection_summary_result_tool_call(
    collection: &LabeledPostCollection,
    response: &str,
) -> Result<LlmSummaryResult, String> {
    let trimmed = response.trim();
    let parsed = extract_leading_tool_call_block(trimmed)
        .map_err(|err| err.diagnostic().to_string())
        .and_then(|accepted| {
            parse_prompt_tool_call_result(&accepted.accepted_block)
                .map_err(|err| err.diagnostic().to_string())
        });

    let (title, summary, recovery_note) = match parsed {
        Ok(tool_call) => {
            if tool_call.name != "submit_summary_result" {
                return Err(format!(
                    "unexpected tool call `{}` for summary result",
                    tool_call.name
                ));
            }

            let title = require_string_arg(&tool_call.args, "title")
                .ok()
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("LLM-selected post in {}", collection.label));

            let summary = require_string_arg(&tool_call.args, "summary")
                .map_err(|err| err.to_string())?
                .trim()
                .to_string();
            (title, summary, None)
        }
        Err(primary_err) => {
            if !trimmed.contains("TOOL_CALL") || !trimmed.contains("submit_summary_result") {
                return Err(primary_err);
            }
            let title = extract_json_string_field(trimmed, "title")
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| format!("LLM-selected post in {}", collection.label));
            let summary = extract_json_string_field(trimmed, "summary")
                .or_else(|| fallback_summary_from_raw_output(trimmed))
                .ok_or(primary_err)?;
            (
                title,
                summary,
                Some("recovered summary text from malformed submit_summary_result tool call"),
            )
        }
    };
    if !is_valid_llm_search_summary(&summary) {
        return Err("missing required scalar field `summary`".to_string());
    }

    let covered_item_uris = collection
        .posts
        .iter()
        .map(|post| post.uri.clone())
        .collect::<Vec<_>>();
    let omitted_item_uris = Vec::new();

    let window_offset = collection_window_offset(collection).unwrap_or(0);
    let window_size = collection_window_size(collection);
    let available_total_items = collection_available_total_items(collection);
    let page_size = COLLECTION_SEARCH_PAGE_SIZE;
    let page_index = window_offset / page_size.max(1);
    let has_more = window_offset.saturating_add(window_size) < available_total_items;

    let result = LlmSummaryResult {
        title,
        summary,
        covered_item_uris,
        omitted_item_uris,
        concatenated_window_summaries: None,
        window_offset: Some(window_offset),
        window_size: Some(window_size),
        page_index: Some(page_index),
        page_size: Some(page_size),
        collection_total_items: Some(available_total_items),
        has_more: Some(has_more),
        source_exhausted: None,
        window_start: Some(window_offset),
        window_total_items: Some(window_size),
    };

    let _ = recovery_note;
    Ok(result)
}

fn extract_json_string_field(input: &str, field_name: &str) -> Option<String> {
    let needle = format!("\"{field_name}\"");
    let start = input.find(&needle)?;
    let rest = &input[start + needle.len()..];
    let colon = rest.find(':')?;
    let mut chars = rest[colon + 1..].chars().peekable();
    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }
    if chars.next()? != '"' {
        return None;
    }

    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    Some(out)
}

fn fallback_summary_from_raw_output(input: &str) -> Option<String> {
    let summary_start = input.find("summary")?;
    let fallback = input[summary_start..]
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    if fallback.is_empty() {
        None
    } else {
        Some(fallback)
    }
}

fn parse_tagged_summary_body(
    response: &str,
    require_end_marker: bool,
) -> Result<TaggedSummaryBody, String> {
    let start_marker = "SUMMARY_RESULT_START";
    let end_marker = "SUMMARY_RESULT_END";
    let Some(start_index) = response.find(start_marker) else {
        return Err("missing SUMMARY_RESULT_START marker".to_string());
    };
    let end_index = response.find(end_marker);
    if require_end_marker && end_index.is_none() {
        return Err("missing SUMMARY_RESULT_END marker".to_string());
    }
    if let Some(end_index) = end_index {
        if end_index <= start_index {
            return Err("summary markers are out of order".to_string());
        }
    }

    let body_start = start_index + start_marker.len();
    let body_end = end_index.unwrap_or(response.len());
    let body = &response[body_start..body_end];
    let mut parsed = ParsedTaggedSummaryFields {
        title: None,
        summary: None,
        covered_item_uris: Vec::new(),
        omitted_item_uris: Vec::new(),
        window_offset: None,
        window_size: None,
        page_index: None,
        page_size: None,
        collection_total_items: None,
        has_more: None,
    };

    for raw_line in body.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((raw_key, raw_value)) = line.split_once(':') else {
            continue;
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        match key {
            "title" => {
                if !value.is_empty() {
                    parsed.title = Some(value.to_string());
                }
            }
            "summary" => {
                if !value.is_empty() {
                    parsed.summary = Some(value.to_string());
                }
            }
            "covered_item_uri" => {
                if !value.is_empty() {
                    parsed.covered_item_uris.push(value.to_string());
                }
            }
            "omitted_item_uri" => {
                if !value.is_empty() {
                    parsed.omitted_item_uris.push(value.to_string());
                }
            }
            "window_offset" => {
                parsed.window_offset = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| "malformed integer field `window_offset`".to_string())?,
                );
            }
            "window_size" => {
                parsed.window_size = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| "malformed integer field `window_size`".to_string())?,
                );
            }
            "page_index" => {
                parsed.page_index = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| "malformed integer field `page_index`".to_string())?,
                );
            }
            "page_size" => {
                parsed.page_size = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| "malformed integer field `page_size`".to_string())?,
                );
            }
            "collection_total_items" => {
                parsed.collection_total_items =
                    Some(value.parse::<usize>().map_err(|_| {
                        "malformed integer field `collection_total_items`".to_string()
                    })?);
            }
            "has_more" => {
                parsed.has_more = Some(match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err("malformed bool field `has_more`".to_string()),
                });
            }
            _ => {}
        }
    }

    Ok(TaggedSummaryBody {
        fields: parsed,
        missing_end_marker: end_index.is_none(),
    })
}

fn parse_collection_summary_result_tagged(response: &str) -> Result<LlmSummaryResult, String> {
    let parsed = parse_tagged_summary_body(response, true)?.fields;

    if parsed.summary.as_deref().is_none() {
        return Err("missing required scalar field `summary`".to_string());
    }
    if parsed.covered_item_uris.is_empty() && parsed.omitted_item_uris.is_empty() {
        return Err("no usable covered/omitted URI accounting was provided".to_string());
    }
    if parsed.window_offset.is_none()
        || parsed.window_size.is_none()
        || parsed.page_index.is_none()
        || parsed.page_size.is_none()
        || parsed.collection_total_items.is_none()
        || parsed.has_more.is_none()
    {
        let mut missing = Vec::new();
        if parsed.window_offset.is_none() {
            missing.push("window_offset");
        }
        if parsed.window_size.is_none() {
            missing.push("window_size");
        }
        if parsed.page_index.is_none() {
            missing.push("page_index");
        }
        if parsed.page_size.is_none() {
            missing.push("page_size");
        }
        if parsed.collection_total_items.is_none() {
            missing.push("collection_total_items");
        }
        if parsed.has_more.is_none() {
            missing.push("has_more");
        }
        return Err(format!(
            "missing required scalar field(s): {}",
            missing.join(", ")
        ));
    }

    Ok(LlmSummaryResult {
        title: parsed.title.unwrap_or_default(),
        summary: parsed.summary.unwrap_or_default(),
        covered_item_uris: parsed.covered_item_uris,
        omitted_item_uris: parsed.omitted_item_uris,
        concatenated_window_summaries: None,
        window_offset: parsed.window_offset,
        window_size: parsed.window_size,
        page_index: parsed.page_index,
        page_size: parsed.page_size,
        collection_total_items: parsed.collection_total_items,
        has_more: parsed.has_more,
        source_exhausted: None,
        window_start: parsed.window_offset,
        window_total_items: parsed.window_size,
    })
}

fn parse_collection_summary_result_tagged_partial(
    collection: &LabeledPostCollection,
    response: &str,
    window_start: Option<usize>,
) -> Result<(LlmSummaryResult, String), String> {
    let parsed_body = parse_tagged_summary_body(response, false)?;
    let parsed = parsed_body.fields;

    let Some(summary) = parsed
        .summary
        .as_deref()
        .filter(|summary| is_valid_llm_search_summary(summary))
        .map(str::to_owned)
    else {
        return Err("missing required scalar field `summary`".to_string());
    };

    let mut covered_item_uris = Vec::new();
    for uri in parsed.covered_item_uris {
        push_search_uri(
            collection,
            &mut covered_item_uris,
            &uri,
            collection.posts.len().max(1),
        );
    }
    if covered_item_uris.is_empty() {
        return Err("no usable covered URI accounting was provided".to_string());
    }

    let inferred_window_offset = parsed
        .window_offset
        .or_else(|| {
            parsed
                .page_index
                .map(|page_index| page_index * COLLECTION_SEARCH_PAGE_SIZE)
        })
        .or(window_start);
    if inferred_window_offset.is_none() {
        return Err("missing required scalar field `window_offset` or `page_index`".to_string());
    }

    let Some(window_size) = parsed.window_size else {
        return Err("missing required scalar field `window_size`".to_string());
    };

    if parsed.collection_total_items.is_none() && parsed.has_more.is_none() {
        return Err(
            "missing required scalar field `collection_total_items` or `has_more`".to_string(),
        );
    }

    let page_size = parsed.page_size.unwrap_or(COLLECTION_SEARCH_PAGE_SIZE);
    let page_index = parsed
        .page_index
        .or_else(|| inferred_window_offset.map(|offset| offset / page_size.max(1)));
    let collection_total_items = parsed.collection_total_items.or_else(|| {
        parsed
            .has_more
            .map(|_| collection_available_total_items(collection))
    });
    let has_more = parsed.has_more.or_else(|| {
        inferred_window_offset
            .zip(Some(window_size))
            .map(|(offset, size)| {
                offset.saturating_add(size) < collection_available_total_items(collection)
            })
    });

    let omitted_was_missing = parsed.omitted_item_uris.is_empty();
    let mut omitted_item_uris = Vec::new();
    for uri in parsed.omitted_item_uris {
        push_search_uri(
            collection,
            &mut omitted_item_uris,
            &uri,
            collection.posts.len().max(1),
        );
    }
    if omitted_item_uris.is_empty() {
        let covered = covered_item_uris.iter().cloned().collect::<HashSet<_>>();
        for post in &collection.posts {
            if !covered.contains(&post.uri) {
                omitted_item_uris.push(post.uri.clone());
            }
        }
    }

    let mut repair_notes = Vec::new();
    if parsed_body.missing_end_marker {
        repair_notes.push("missing SUMMARY_RESULT_END marker".to_string());
    }
    if parsed.page_index.is_none() {
        repair_notes.push("inferred page_index from window offset".to_string());
    }
    if parsed.page_size.is_none() {
        repair_notes.push("defaulted page_size to collection page size".to_string());
    }
    if parsed.collection_total_items.is_none() {
        repair_notes.push("filled collection_total_items from collection metadata".to_string());
    }
    if parsed.has_more.is_none() {
        repair_notes.push("filled has_more from collection metadata".to_string());
    }
    if !omitted_item_uris.is_empty() && omitted_was_missing {
        repair_notes
            .push("derived omitted_item_uri values from uncovered window items".to_string());
    }

    Ok((
        normalize_summary_result(
            collection,
            LlmSummaryResult {
                title: parsed.title.unwrap_or_default(),
                summary,
                covered_item_uris,
                omitted_item_uris,
                concatenated_window_summaries: None,
                window_offset: inferred_window_offset,
                window_size: Some(window_size),
                page_index,
                page_size: Some(page_size),
                collection_total_items,
                has_more,
                source_exhausted: None,
                window_start: inferred_window_offset,
                window_total_items: Some(window_size),
            },
        ),
        if repair_notes.is_empty() {
            "summary partial parse repaired".to_string()
        } else {
            format!(
                "summary partial parse repaired: {}",
                repair_notes.join("; ")
            )
        },
    ))
}

fn normalize_summary_result(
    collection: &LabeledPostCollection,
    mut result: LlmSummaryResult,
) -> LlmSummaryResult {
    if result.title.trim().is_empty() {
        result.title = format!("LLM-selected post in {}", collection.label);
    }
    result
}

fn parse_collection_summary_result_json(
    collection: &LabeledPostCollection,
    response: &str,
) -> Result<LlmSummaryResult, String> {
    let value = serde_json::from_str::<Value>(response)
        .map_err(|err| format!("invalid JSON object: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "response was not a JSON object".to_string())?;

    let title = object
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("LLM-selected post in {}", collection.label));

    let summary = object
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|summary| is_valid_llm_search_summary(summary))
        .map(str::to_owned)
        .ok_or_else(|| "missing required scalar field `summary`".to_string())?;

    let covered_item_uris = parse_collection_window_uris(
        collection,
        object.get("covered_item_uris"),
        collection.posts.len(),
    );
    let omitted_item_uris = parse_collection_window_uris(
        collection,
        object.get("omitted_item_uris"),
        collection.posts.len(),
    );
    if covered_item_uris.is_empty() && omitted_item_uris.is_empty() {
        return Err("no usable covered/omitted URI accounting was provided".to_string());
    }

    Ok(LlmSummaryResult {
        title,
        summary,
        covered_item_uris,
        omitted_item_uris,
        concatenated_window_summaries: None,
        window_offset: object
            .get("window_offset")
            .or_else(|| object.get("window_start"))
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        window_size: object
            .get("window_size")
            .or_else(|| object.get("window_total_items"))
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        page_index: object
            .get("page_index")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        page_size: object
            .get("page_size")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        collection_total_items: object
            .get("collection_total_items")
            .or_else(|| object.get("search_window_total_items"))
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        has_more: object.get("has_more").and_then(Value::as_bool),
        source_exhausted: None,
        window_start: object
            .get("window_start")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        window_total_items: object
            .get("window_total_items")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
    })
}

fn parse_collection_tool_result_json(
    collection: &LabeledPostCollection,
    response: &str,
    tool_kind: CollectionLeafToolKind,
) -> Option<CollectionLeafResult> {
    let value = serde_json::from_str::<Value>(response).ok()?;
    let object = value.as_object()?;

    let title = object
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("LLM-selected post in {}", collection.label));

    let parsed_summary = object
        .get("summary")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|summary| is_valid_llm_search_summary(summary))
        .map(str::to_owned);

    match tool_kind {
        CollectionLeafToolKind::Search => {
            let uris = object
                .get("uris")
                .and_then(Value::as_array)
                .map(|items| {
                    let mut uris = Vec::new();
                    for item in items {
                        let Some(uri) = item.as_str() else {
                            continue;
                        };
                        push_search_uri(collection, &mut uris, uri, MAX_COLLECTION_SEARCH_RESULTS);
                    }
                    uris
                })
                .unwrap_or_default();

            if uris.is_empty() {
                return None;
            }

            let search_results = uris
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

            Some(CollectionLeafResult::Search(LlmSearchResult {
                title,
                summary: parsed_summary
                    .unwrap_or_else(|| fallback_llm_search_summary(collection, &search_results)),
                search_results,
            }))
        }
        CollectionLeafToolKind::Summary => {
            parse_collection_summary_result_json(collection, response)
                .ok()
                .map(CollectionLeafResult::Summary)
        }
    }
}

fn find_matching_uris_from_response(
    collection: &LabeledPostCollection,
    response: &str,
) -> Vec<String> {
    let mut uris = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();
        if let Some(uri) = trimmed.strip_prefix("uri:") {
            push_search_uri(
                collection,
                &mut uris,
                uri.trim(),
                MAX_COLLECTION_SEARCH_RESULTS,
            );
        }
    }

    for post in &collection.posts {
        if uris.len() >= MAX_COLLECTION_SEARCH_RESULTS {
            break;
        }
        if response.contains(&post.uri) {
            push_search_uri(
                collection,
                &mut uris,
                &post.uri,
                MAX_COLLECTION_SEARCH_RESULTS,
            );
        }
    }

    if uris.len() >= MAX_COLLECTION_SEARCH_RESULTS {
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
        if uris.len() >= MAX_COLLECTION_SEARCH_RESULTS {
            break;
        }
        push_search_uri(collection, &mut uris, &uri, MAX_COLLECTION_SEARCH_RESULTS);
    }

    uris
}

fn parse_collection_window_uris(
    collection: &LabeledPostCollection,
    value: Option<&Value>,
    limit: usize,
) -> Vec<String> {
    let mut uris = Vec::new();
    let Some(items) = value.and_then(Value::as_array) else {
        return uris;
    };
    for item in items {
        let Some(uri) = item.as_str() else {
            continue;
        };
        push_search_uri(collection, &mut uris, uri, limit);
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
        && !trimmed.starts_with("matched_item[")
        && !trimmed.starts_with("{")
        && !trimmed.contains(" item[")
        && !trimmed.contains(" matched_item[")
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

fn candidate_hints(post: &PostRecord) -> Vec<String> {
    let mut hints = Vec::new();
    let body_fields = body_field_map(&post.body);

    if let Some(value) = body_fields.get("reply_text") {
        let value = value.trim();
        if !value.is_empty() {
            hints.push(value.to_string());
        }
    }
    if let Some(value) = body_fields.get("list_name") {
        let value = value.trim();
        if !value.is_empty() {
            hints.push(value.to_string());
        }
    }
    if let Some(value) = body_fields.get("list_description") {
        let value = value.trim();
        if !value.is_empty() {
            hints.push(value.to_string());
        }
    }
    if let Some(value) = body_fields.get("source_collection_label") {
        let value = value.trim();
        if !value.is_empty() {
            hints.push(value.to_string());
        }
    }
    let author = post.author_handle.trim();
    if !author.is_empty() {
        hints.push(author.to_string());
    }

    for line in post.body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if hints.iter().any(|seen| seen == trimmed) {
            continue;
        }
        hints.push(trimmed.to_string());
    }

    hints
}

pub fn prompt_tool_protocol_instructions() -> &'static str {
    tool_prompt()
}

pub fn parse_prompt_tool_call(response: &str) -> Option<PromptToolCall> {
    parse_prompt_tool_call_result(response).ok()
}

fn render_llm_result(result: Option<&CollectionLeafResult>) -> String {
    match result {
        Some(CollectionLeafResult::Search(result)) => {
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
        Some(CollectionLeafResult::Summary(result)) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
            ];
            if let Some(window_offset) = result.processed_window_offset() {
                lines.push(format!("window_offset: {window_offset}"));
            }
            if let Some(window_size) = result.processed_window_size() {
                lines.push(format!("window_size: {window_size}"));
            }
            if let Some(page_index) = result.processed_page_index() {
                lines.push(format!("page_index: {page_index}"));
            }
            if let Some(page_size) = result.processed_page_size() {
                lines.push(format!("page_size: {page_size}"));
            }
            if let Some(collection_total_items) = result.processed_collection_total_items() {
                lines.push(format!("collection_total_items: {collection_total_items}"));
            }
            if let Some(has_more) = result.has_more {
                lines.push(format!("has_more: {has_more}"));
            }
            if let Some(source_exhausted) = result.source_exhausted {
                lines.push(format!("source_exhausted: {source_exhausted}"));
            }
            if let Some(concatenated) = result.concatenated_window_summaries() {
                lines.push(format!(
                    "concatenated_window_summaries:\n{}",
                    truncate_diagnostic_block(concatenated, 4000)
                ));
            }
            for (index, uri) in result.covered_item_uris.iter().enumerate() {
                lines.push(format!("covered_item_{}_uri: {}", index + 1, uri));
            }
            for (index, uri) in result.omitted_item_uris.iter().enumerate() {
                lines.push(format!("omitted_item_{}_uri: {}", index + 1, uri));
            }
            lines.join("\n")
        }
        Some(CollectionLeafResult::RawWindow(result)) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
                format!("window_offset: {}", result.window_offset),
                format!("window_size: {}", result.window_size),
                format!("page_index: {}", result.page_index),
                format!("page_size: {}", result.page_size),
                format!("collection_total_items: {}", result.collection_total_items),
                format!("has_more: {}", result.has_more),
                format!("failure_reason: {}", result.failure_reason),
            ];
            if let Some(raw_summary_response) = result.raw_summary_response.as_deref() {
                lines.push(format!(
                    "raw_summary_response:\n{}",
                    truncate_diagnostic_block(raw_summary_response, 4000)
                ));
            }
            for (index, post) in result.records.iter().enumerate() {
                lines.push(format!("raw_window_item_{}_uri: {}", index + 1, post.uri));
                lines.push(format!(
                    "raw_window_item_{}_author: {}",
                    index + 1,
                    post.author_handle
                ));
                lines.push(format!("raw_window_item_{}_body: {}", index + 1, post.body));
            }
            lines.join("\n")
        }
        None => "No matching cached posts.".to_string(),
    }
}

pub(crate) fn render_llm_execution_result(execution: &LlmSearchExecution) -> String {
    let mut lines = Vec::new();
    if let Some(diagnostic) = execution.diagnostic.as_deref() {
        lines.push(format!("diagnostic: {diagnostic}"));
    }
    if let Some(raw_response) = execution.raw_response.as_deref() {
        lines.push(format!(
            "raw_response:\n{}",
            truncate_diagnostic_block(raw_response, 4000)
        ));
    }
    if let Some(verdict) = execution.review_verdict.as_ref() {
        lines.push(format!("review_status: {}", verdict.status.as_str()));
        lines.push(format!("review_grounded: {}", verdict.grounded));
        lines.push(format!("review_sufficient: {}", verdict.sufficient));
        lines.push(format!("review_reason: {}", verdict.reason));
        lines.push(format!("review_repair_needed: {}", verdict.repair_needed));
        lines.push(format!(
            "review_additional_pages_needed: {}",
            verdict.additional_pages_needed
        ));
        if let Some(next_page) = verdict.next_page {
            lines.push(format!("review_next_page: {next_page}"));
        }
        if let Some(next_offset) = verdict.next_offset {
            lines.push(format!("review_next_offset: {next_offset}"));
        }
        if let Some(required_total_items) = verdict.required_total_items {
            lines.push(format!(
                "review_required_total_items: {required_total_items}"
            ));
        }
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

fn render_public_summary_outcomes(outcomes: &[CollectionToolOutcome]) -> String {
    harness_summary::render_public_summary_outcomes(outcomes)
}

pub(crate) fn render_collection_outcome_result(
    tool_kind: CollectionLeafToolKind,
    collection_id: &str,
    collection_label: &str,
    execution: &LlmSearchExecution,
) -> String {
    let mut lines = vec![
        format!("tool_name: {}", tool_kind.tool_name()),
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

pub(crate) fn apply_summary_sufficiency_gates(
    requested_summary_scope: RequestedSummaryScope,
    collection_id: &str,
    prior_outcomes: &[CollectionToolOutcome],
    execution: &mut LlmSearchExecution,
) {
    let original_summary_result = execution
        .result
        .as_ref()
        .or(execution.original_result.as_ref())
        .and_then(CollectionLeafResult::as_summary)
        .cloned();
    let Some(result) = original_summary_result.as_ref() else {
        append_summary_trace(format!(
            "[summary_sufficiency_gate]\ncollection_id: {}\nstatus: skipped\nreason: missing_summary_result\nresult_present: {}\noriginal_result_present: {}\nreview_status: {}",
            collection_id,
            execution.result.is_some(),
            execution.original_result.is_some(),
            execution
                .review_verdict
                .as_ref()
                .map(|verdict| verdict.status.as_str())
                .unwrap_or("<none>")
        ));
        return;
    };
    let current_start = result.processed_window_offset().unwrap_or(0);
    let current_len = result.processed_window_size().unwrap_or(0);
    let current_page_size = result
        .processed_page_size()
        .unwrap_or(COLLECTION_SEARCH_PAGE_SIZE);

    let mut ranges = prior_outcomes
        .iter()
        .filter(|outcome| {
            outcome.tool_kind == CollectionLeafToolKind::Summary
                && outcome.collection_id == collection_id
        })
        .filter_map(|outcome| outcome.execution.as_ref().ok())
        .filter_map(|prior| {
            prior
                .result
                .as_ref()
                .or(prior.original_result.as_ref())
                .and_then(|result| {
                    Some((
                        result.processed_window_offset()?,
                        result.processed_window_size()?,
                    ))
                })
        })
        .collect::<Vec<_>>();
    ranges.push((current_start, current_len));
    let contiguous_coverage = contiguous_coverage_end(&ranges);
    let current_available_total =
        if result.has_more == Some(false) || current_len < current_page_size {
            current_start.saturating_add(current_len)
        } else {
            result
                .processed_collection_total_items()
                .or_else(|| {
                    execution
                        .review_verdict
                        .as_ref()
                        .and_then(|verdict| verdict.required_total_items)
                })
                .unwrap_or(contiguous_coverage)
        };
    let reviewed = summary_scope_verdict(requested_summary_scope, &ranges, current_available_total);
    let result_present_before_restore = execution.result.is_some();
    let original_result_present_before_restore = execution.original_result.is_some();
    let review_grounded = execution
        .review_verdict
        .as_ref()
        .map(|v| v.grounded)
        .unwrap_or(false);

    let status = if review_grounded && reviewed.sufficient {
        CollectionReviewStatus::Pass
    } else {
        CollectionReviewStatus::Fail
    };
    if let Some(verdict) = execution.review_verdict.as_mut() {
        verdict.status = status.clone();
        verdict.sufficient = reviewed.sufficient;
        verdict.additional_pages_needed = reviewed.additional_pages_needed;
        verdict.next_page = reviewed.next_page;
        verdict.next_offset = reviewed.next_offset;
        verdict.required_total_items = reviewed.required_total_items;
        verdict.reason = reviewed.reason;
    }

    if matches!(status, CollectionReviewStatus::Pass) && execution.result.is_none() {
        execution.result = original_summary_result.map(CollectionLeafResult::Summary);
    }

    append_summary_trace(format!(
        "[summary_sufficiency_gate]\ncollection_id: {}\nwindow_offset: {}\ncontiguous_coverage: {}\navailable_total_items: {}\nstatus_after_gate: {}\nreview_grounded: {}\nreview_sufficient: {}\nreview_additional_pages_needed: {}\nreview_next_offset: {}\nresult_before_restore: {}\noriginal_result_before_restore: {}\nresult_after_gate: {}\noriginal_result_after_gate: {}",
        collection_id,
        current_start,
        contiguous_coverage,
        current_available_total,
        status.as_str(),
        review_grounded,
        execution
            .review_verdict
            .as_ref()
            .map(|verdict| verdict.sufficient)
            .unwrap_or(false),
        execution
            .review_verdict
            .as_ref()
            .map(|verdict| verdict.additional_pages_needed)
            .unwrap_or(false),
        execution
            .review_verdict
            .as_ref()
            .and_then(|verdict| verdict.next_offset)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<none>".to_string()),
        result_present_before_restore,
        original_result_present_before_restore,
        execution.result.is_some(),
        execution.original_result.is_some(),
    ));
}

pub(crate) fn render_llm_result_compact(result: Option<&CollectionLeafResult>) -> String {
    match result {
        Some(CollectionLeafResult::Search(result)) => {
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
        Some(CollectionLeafResult::Summary(result)) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
            ];
            if let Some(window_offset) = result.processed_window_offset() {
                lines.push(format!("window_offset: {window_offset}"));
            }
            if let Some(window_size) = result.processed_window_size() {
                lines.push(format!("window_size: {window_size}"));
            }
            if let Some(page_index) = result.processed_page_index() {
                lines.push(format!("page_index: {page_index}"));
            }
            if let Some(page_size) = result.processed_page_size() {
                lines.push(format!("page_size: {page_size}"));
            }
            if let Some(collection_total_items) = result.processed_collection_total_items() {
                lines.push(format!("collection_total_items: {collection_total_items}"));
            }
            if let Some(has_more) = result.has_more {
                lines.push(format!("has_more: {has_more}"));
            }
            for (index, uri) in result.covered_item_uris.iter().enumerate() {
                lines.push(format!("covered_item_{}_uri: {}", index + 1, uri));
            }
            lines.join("\n")
        }
        Some(CollectionLeafResult::RawWindow(result)) => {
            let mut lines = vec![
                format!("post: {}", result.title),
                format!("summary: {}", result.summary),
                format!("window_offset: {}", result.window_offset),
                format!("window_size: {}", result.window_size),
                format!("page_index: {}", result.page_index),
                format!("page_size: {}", result.page_size),
                format!("collection_total_items: {}", result.collection_total_items),
                format!("has_more: {}", result.has_more),
                format!("failure_reason: {}", result.failure_reason),
            ];
            for (index, post) in result.records.iter().enumerate() {
                lines.push(format!("raw_window_item_{}_uri: {}", index + 1, post.uri));
            }
            lines.join("\n")
        }
        None => "No matching cached posts.".to_string(),
    }
}

async fn synthesize_search_results(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
    observer: Option<UnboundedSender<ToolProgressEvent>>,
) -> String {
    if outcomes.len() <= 1 {
        return render_combined_search_results(outcomes);
    }

    if let Some(observer) = observer.as_ref() {
        let _ = observer.send(ToolProgressEvent::AgentUpdate {
            label: "search synthesis".to_string(),
            depth: 2,
            content: "status: running".to_string(),
        });
    }

    let context = build_search_parent_context(prompt, outcomes);
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
            render_combined_search_results_with_summary(outcomes, Some(summary.trim()), &[])
        }
        Err(_) => render_combined_search_results_with_summary(outcomes, None, &[]),
    };
    if let Some(observer) = observer.as_ref() {
        let _ = observer.send(ToolProgressEvent::AgentUpdate {
            label: "search synthesis".to_string(),
            depth: 2,
            content: format!(
                "status: completed\n\nsummary:\n{}",
                summarize_progress_text(&rendered)
            ),
        });
    }
    rendered
}

fn build_search_agent_node(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> AgentNodeTemplate {
    harness_search::build_search_agent_node(prompt, outcomes, llm_client)
}

fn build_search_parent_context(prompt: &str, outcomes: &[CollectionToolOutcome]) -> LLMContext {
    harness_search::build_search_parent_context(prompt, outcomes)
}

fn build_summary_agent_node(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> AgentNodeTemplate {
    harness_summary::build_summary_agent_node(prompt, outcomes, llm_client)
}

fn render_combined_search_results(outcomes: &[CollectionToolOutcome]) -> String {
    harness_search::render_combined_search_results(outcomes)
}

fn render_combined_search_results_with_summary(
    outcomes: &[CollectionToolOutcome],
    parent_summary: Option<&str>,
    diagnostics: &[String],
) -> String {
    if outcomes.len() == 1 {
        return match outcomes.first() {
            Some(outcome) => match outcome.execution.as_ref() {
                Ok(execution) => render_collection_outcome_result(
                    outcome.tool_kind,
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
        "search searched collections independently and combined the grounded results below."
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
        if let Some(search_result) = result
            .as_search()
            .and_then(|result| result.search_results.first())
        {
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
                outcome.tool_kind,
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

fn render_collection_outcome_progress_result(
    tool_kind: CollectionLeafToolKind,
    collection_id: &str,
    collection_label: &str,
    execution: &LlmSearchExecution,
) -> String {
    let mut lines = vec![
        format!("tool_name: {}", tool_kind.tool_name()),
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

    if let Some(verdict) = execution.review_verdict.as_ref() {
        lines.push(format!("review_status: {}", verdict.status.as_str()));
        lines.push(format!("review_reason: {}", verdict.reason));
        if let Some(required_total_items) = verdict.required_total_items {
            lines.push(format!(
                "review_required_total_items: {required_total_items}"
            ));
        }
    }

    lines.push(render_llm_result_compact(
        execution
            .result
            .as_ref()
            .or(execution.original_result.as_ref()),
    ));
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

fn truncate_diagnostic_block(text: &str, max_chars: usize) -> String {
    let _ = max_chars;
    text.trim().to_string()
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

fn fallback_parent_summary(outcomes: &[CollectionToolOutcome]) -> Option<String> {
    let summaries = outcomes
        .iter()
        .filter_map(|outcome| {
            let execution = outcome.execution.as_ref().ok()?;
            let result = execution.result.as_ref()?;
            Some(format!(
                "{}: {}",
                outcome.collection_label,
                result.summary()
            ))
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

pub(crate) fn paged_search_collection(
    collection: &LabeledPostCollection,
    offset: usize,
    page_size: usize,
) -> LabeledPostCollection {
    if collection.posts.len() <= page_size && offset == 0 {
        return collection.clone();
    }

    let start = offset.min(collection.posts.len());
    let end = start.saturating_add(page_size).min(collection.posts.len());
    let posts = collection.posts[start..end].to_vec();
    let start_display = if collection.posts.is_empty() || start == end {
        0
    } else {
        start + 1
    };
    let label = format!(
        "{} (items {}-{} of {})",
        collection.label,
        start_display,
        end,
        collection.posts.len()
    );

    let mut metadata = collection.metadata.clone();
    metadata.insert("search_window_offset".to_string(), start.to_string());
    metadata.insert("search_window_size".to_string(), posts.len().to_string());
    metadata.insert(
        "search_window_total_items".to_string(),
        collection.posts.len().to_string(),
    );

    let paged = LabeledPostCollection::new(collection.id.clone(), label, posts)
        .with_collection_kind(collection.collection_kind.clone())
        .with_refresh_state(collection.last_refreshed_at, collection.refresh_ttl_seconds)
        .with_related_collections(collection.related_collection_ids.clone())
        .with_metadata(metadata);

    if let Some(actor_did) = collection.actor_did.as_ref() {
        paged.with_actor_did(actor_did.clone())
    } else {
        paged
    }
}

fn collection_window_offset(collection: &LabeledPostCollection) -> Option<usize> {
    collection
        .metadata
        .get("search_window_offset")
        .and_then(|value| value.parse::<usize>().ok())
}

fn collection_window_size(collection: &LabeledPostCollection) -> usize {
    collection.posts.len()
}

fn collection_available_total_items(collection: &LabeledPostCollection) -> usize {
    collection
        .metadata
        .get("search_window_total_items")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| collection_window_size(collection))
}

fn collection_search_offset(args: &Value) -> Result<usize, Box<dyn std::error::Error>> {
    if let Some(offset) = optional_usize_arg(args, "offset")? {
        return Ok(offset);
    }
    let page = optional_usize_arg(args, "page")?.unwrap_or(0);
    Ok(page.saturating_mul(COLLECTION_SEARCH_PAGE_SIZE))
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

fn result_uses_fallback_summary(execution: &LlmSearchExecution) -> bool {
    execution
        .result
        .as_ref()
        .map(|result| match result {
            CollectionLeafResult::RawWindow(_) => true,
            _ => {
                result
                    .summary()
                    .contains("The model did not return a fully structured")
                    || result
                        .summary()
                        .contains("This fallback summary is derived directly")
                    || result.summary().contains("a follow-up pass may be needed")
            }
        })
        .unwrap_or(false)
}

fn diagnostic_counts_as_warning(diagnostic: &str) -> bool {
    !diagnostic.starts_with("summary cursor processed offset ")
        && !diagnostic.starts_with("collection_summary_planner accepted ")
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
    let _ = max_chars;
    text.to_string()
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

fn optional_usize_arg(
    args: &Value,
    key: &str,
) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| format!("tool arg `{key}` must be a non-negative integer").into()),
        Some(_) => Err(format!("tool arg `{key}` must be a non-negative integer").into()),
    }
}

fn require_usize_arg(args: &Value, key: &str) -> Result<usize, Box<dyn std::error::Error>> {
    optional_usize_arg(args, key)?
        .ok_or_else(|| format!("tool arg `{key}` must be a non-negative integer").into())
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
        ActorAnchorSource, BlueskyTools, CollectionLeafResult, CollectionLeafToolKind,
        CollectionToolOutcome,
        CollectionReviewStatus, LlmSearchExecution, LlmSearchResult, LlmSearchResultItem,
        LlmSummaryResult, PreparedPromptToolInput, PreparedSummaryActor, PromptToolCall,
        RequestedSummaryScope,
        SearchIntent, SummaryCollectionTargetHint, SummarySelectionReviewStatus,
        ToolPrepMissingPrerequisite, ToolRegistry, build_search_agent_node,
        build_summary_agent_node, choose_deterministic_collection_id_for_actor,
        collection_search_offset, detect_actor_refs, detect_post_uri,
        detect_summary_collection_target_hint, deterministic_repair_internal_search_tool_call,
        deterministic_repair_summary, fallback_llm_search_summary, heuristic_collection_review,
        llm_review_summary_collection_selection, merged_collection_from_refs,
        paged_search_collection, parse_collection_review_verdict, parse_collection_tool_result,
        parse_internal_tool_repair_response, parse_llm_search_result,
        parse_llm_summary_collection_selection_review, parse_prompt_tool_call,
        parse_requested_summary_scope_args, pick_summary_collection_for_hint,
        reduced_search_collection, render_internal_search_tool_protocol, render_llm_result,
        render_public_summary_outcomes,
        render_post_details, review_summary_collection_selection, serialize_collection,
        source_collection_id_from_post, summary_hydration_args_for_hint, validate_collection_id,
        validate_internal_tool_response,
    };
    use crate::app::EvilGemmaConfig;
    use crate::harness::context_window::{
        BuiltContextSection, BuiltContextWindow, ProviderContextLimits,
    };
    use crate::harness::llm_api::{ChatCompletionResponseFormat, LlmApiClient, OpenAiRestConfig};
    use crate::model::{LabeledPostCollection, PostRecord};
    use crate::net_backend::{
        NotificationStore, PostDetails, PostFacet, ensure_actor_profile_cached,
        ensure_recent_posts_cached,
    };
    use bsky_sdk::BskyAgent;
    use bsky_sdk::api::types::string::Did;
    use std::env;
    use std::time::Duration;
    use tokio::time::timeout;

    async fn start_live_bsky_agent() -> Result<(BskyAgent, String), Box<dyn std::error::Error>> {
        let _ = dotenvy::dotenv();
        let handle = env::var("BSKY_HANDLE")?;
        let password = env::var("BSKY_APP_PASSWORD")?;
        let agent = BskyAgent::builder().build().await?;
        agent.login(&handle, &password).await?;
        Ok((agent, handle))
    }

    fn scripted_llm_for_tests(responses: Vec<&str>) -> LlmApiClient {
        LlmApiClient::scripted_for_tests(
            OpenAiRestConfig::llama_cpp("http://example.test", "test-model"),
            responses.into_iter().map(str::to_string).collect(),
        )
    }

    fn summary_prep_llm_for_tests(response: &str) -> LlmApiClient {
        scripted_llm_for_tests(vec![response])
    }

    fn search_leaf_result(result: LlmSearchResult) -> CollectionLeafResult {
        CollectionLeafResult::Search(result)
    }

    fn summary_leaf_result(result: LlmSummaryResult) -> CollectionLeafResult {
        CollectionLeafResult::Summary(result)
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
    fn paged_search_collection_limits_window_to_twenty_five_items() {
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent test posts",
            (0..40)
                .map(|index| PostRecord {
                    uri: format!("at://{index}"),
                    author_handle: "alpha.test".to_string(),
                    body: format!("body {index}"),
                })
                .collect(),
        );

        let paged = paged_search_collection(&collection, 25, 25);
        assert_eq!(paged.posts.len(), 15);
        assert_eq!(paged.posts[0].uri, "at://25");
        assert!(paged.label.contains("items 26-40 of 40"));
    }

    #[test]
    fn collection_search_offset_uses_page_when_offset_absent() {
        let args =
            serde_json::json!({"collection_id": "recent:test", "prompt": "check", "page": 2});
        let offset = collection_search_offset(&args).expect("expected offset");
        assert_eq!(offset, 100);
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

        let rendered = render_llm_result(Some(&search_leaf_result(result)));
        assert!(rendered.contains("summary: quote and context"));
        assert!(rendered.contains("search_result_1_uri: at://one"));
        assert!(rendered.contains("search_result_1_source_collection_id: recent:test"));
    }

    #[test]
    fn render_public_summary_outcomes_prefers_commentary_plus_concatenated_pages() {
        let outcome = CollectionToolOutcome {
            tool_kind: CollectionLeafToolKind::Summary,
            collection_id: "recent:test".to_string(),
            collection_label: "Recent test posts".to_string(),
            execution: Ok(LlmSearchExecution {
                result: Some(summary_leaf_result(LlmSummaryResult {
                    title: "Summary of Recent test posts".to_string(),
                    summary: "Final cross-page commentary.".to_string(),
                    covered_item_uris: Vec::new(),
                    omitted_item_uris: Vec::new(),
                    concatenated_window_summaries: Some(
                        "Page one summary.\n\nPage two summary.".to_string(),
                    ),
                    window_offset: Some(0),
                    window_size: Some(100),
                    page_index: Some(0),
                    page_size: Some(50),
                    collection_total_items: Some(100),
                    has_more: Some(false),
                    source_exhausted: Some(true),
                    window_start: Some(0),
                    window_total_items: Some(100),
                })),
                original_result: None,
                context_window: BuiltContextWindow {
                    rendered: "test".to_string(),
                    limits: ProviderContextLimits {
                        provider_name: "llama.cpp".to_string(),
                        model_name: "test-model".to_string(),
                        max_context_tokens: 1024,
                        reserved_output_tokens: 128,
                    },
                    header_tokens: 1,
                    used_input_tokens: 1,
                    truncated: false,
                    sections: Vec::<BuiltContextSection>::new(),
                },
                diagnostic: Some("ok".to_string()),
                raw_response: None,
                review_verdict: None,
                review_context_window: None,
                repair_diagnostic: None,
            }),
        };

        let rendered = render_public_summary_outcomes(&[outcome]);
        assert!(rendered.contains("Overall commentary across Recent test posts:"));
        assert!(rendered.contains("Final cross-page commentary."));
        assert!(rendered.contains("Concatenated page summaries for Recent test posts:"));
        assert!(rendered.contains("Page one summary."));
        assert!(rendered.contains("Page two summary."));
        assert!(!rendered.contains("diagnostic:"));
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
    fn parse_llm_search_result_accepts_json_object_response() {
        let collection = LabeledPostCollection::new(
            "recent:test",
            "Recent test posts",
            vec![PostRecord {
                uri: "at://one".to_string(),
                author_handle: "alpha.test".to_string(),
                body: "reply_text: This 3D render looks so cool! Such precise lines.".to_string(),
            }],
        )
        .with_collection_kind("recent_replies_received");

        let result = parse_llm_search_result(
            &collection,
            r#"{"title":"grounded","summary":"Replies are enthusiastic and visually focused, with praise like \"This 3D render looks so cool! Such precise lines.\" pointing to direct appreciation of polished technical visuals.","uris":["at://one"]}"#,
        )
        .expect("expected parsed result");

        assert_eq!(result.title, "grounded");
        assert_eq!(result.search_results.len(), 1);
        assert_eq!(result.search_results[0].uri, "at://one");
        assert!(result.summary.contains("visually focused"));
        assert!(!result.summary.contains("item["));
    }

    #[test]
    fn parse_collection_summary_result_accepts_json_object_response() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post one".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post two".to_string(),
                    },
                ],
            ),
            0,
            25,
        );

        let result = parse_collection_tool_result(
            &collection,
            r#"{"title":"window summary","summary":"Both posts focus on test content, with \"post one\" and \"post two\" forming the full window.","covered_item_uris":["at://one","at://two"],"omitted_item_uris":[],"window_start":0,"window_total_items":2}"#,
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        )
        .result
        .expect("expected parsed result");
        let result = result.as_summary().expect("expected summary result");

        assert_eq!(result.covered_item_uris.len(), 2);
        assert_eq!(result.omitted_item_uris.len(), 0);
        assert_eq!(result.window_start, Some(0));
        assert_eq!(result.window_total_items, Some(2));
    }

    #[tokio::test]
    #[ignore = "requires live Bluesky credentials and live LLM API access"]
    async fn execute_internal_search_summarizes_last_50_posts_end_to_end() {
        let (agent, handle) = start_live_bsky_agent()
            .await
            .expect("live Bluesky login from .env");
        let evil_gemma = EvilGemmaConfig::from_env()
            .await
            .expect("live llm client from .env");
        let tools = BlueskyTools::new();
        let mut store = NotificationStore::new();

        let profile = ensure_actor_profile_cached(&agent, &mut store, &handle)
            .await
            .expect("actor profile should hydrate");
        ensure_recent_posts_cached(&agent, &mut store, &profile.did, 75, 50)
            .await
            .expect("recent posts should hydrate");

        let query = format!("summarize the last 50 posts by {handle}");
        let (rendered, outcomes) = timeout(
            Duration::from_secs(180),
            tools.execute_internal_search(
                &query,
                None,
                None,
                &agent,
                &mut store,
                &evil_gemma.client,
                None,
            ),
        )
        .await
        .expect("llm_search live test timed out after 180 seconds")
        .expect("llm_search should complete");

        let outcome = outcomes
            .iter()
            .find(|outcome| outcome.tool_kind == CollectionLeafToolKind::Summary)
            .expect(&format!("expected summary outcome\nrendered:\n{rendered}"));
        let execution = outcome
            .execution
            .as_ref()
            .expect("successful summary execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert!(
            result.window_size.unwrap_or_default() >= 50,
            "expected at least 50 covered posts\nrendered:\n{rendered}\ndiagnostic: {:?}",
            execution.diagnostic
        );
        assert!(
            result
                .concatenated_window_summaries()
                .map(str::trim)
                .map(|text| !text.is_empty())
                .unwrap_or(false),
            "expected concatenated window summaries\nrendered:\n{rendered}"
        );
        assert!(
            !result.summary.trim().is_empty(),
            "expected non-empty final summary\nrendered:\n{rendered}"
        );
        let paragraph_count = result
            .summary
            .split("\n\n")
            .map(str::trim)
            .filter(|paragraph| !paragraph.is_empty())
            .count();
        assert!(
            (1..=3).contains(&paragraph_count),
            "expected 1-3 summary paragraphs, got {paragraph_count}\nsummary:\n{}",
            result.summary
        );
    }

    #[test]
    fn parse_collection_summary_result_accepts_tool_call_response() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post one".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post two".to_string(),
                    },
                ],
            ),
            0,
            25,
        );

        let response = "TOOL_CALL\nname: submit_summary_result\nargs: {\"title\":\"window summary\",\"summary\":\"Both posts focus on test content, with \\\"post one\\\" and \\\"post two\\\" forming the full window.\"}";
        let parsed = parse_collection_tool_result(
            &collection,
            response,
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        );

        assert!(parsed.diagnostic.is_none());
        let result = parsed
            .result
            .expect("expected parsed result")
            .as_summary()
            .expect("expected summary result")
            .clone();

        assert_eq!(result.covered_item_uris, vec!["at://one", "at://two"]);
        assert!(result.omitted_item_uris.is_empty());
        assert_eq!(result.window_offset, Some(0));
        assert_eq!(result.window_size, Some(2));
        assert_eq!(result.page_index, Some(0));
        assert_eq!(result.page_size, Some(50));
        assert_eq!(result.collection_total_items, Some(2));
        assert_eq!(result.has_more, Some(false));
    }

    #[test]
    fn parse_collection_summary_result_recovers_truncated_tool_call_summary_text() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post one".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post two".to_string(),
                    },
                ],
            ),
            0,
            25,
        );

        let response = "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"window summary\",\n  \"summary\": \"Both posts focus on test content, with \\\"post one\\\" and \\\"post two\\\" forming the full window.\",\n  \"covered_item_uris\": [\n    \"at://one\",\n    \"at://";
        let parsed = parse_collection_tool_result(
            &collection,
            response,
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        );

        assert!(parsed.diagnostic.is_none());
        let result = parsed
            .result
            .expect("expected parsed result")
            .as_summary()
            .expect("expected summary result")
            .clone();
        assert!(result.summary.contains("Both posts focus on test content"));
        assert_eq!(result.covered_item_uris, vec!["at://one", "at://two"]);
        assert!(result.omitted_item_uris.is_empty());
    }

    #[test]
    fn parse_collection_summary_result_accepts_tagged_response() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post one".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post two".to_string(),
                    },
                ],
            ),
            0,
            25,
        );

        let response = "SUMMARY_RESULT_START\nsummary: Both posts stay grounded in direct text, including \"post one\" and \"post two\", so the paragraph accounts for the whole window without JSON escaping problems.\ncovered_item_uri: at://one\ncovered_item_uri: at://two\nwindow_offset: 0\nwindow_size: 2\npage_index: 0\npage_size: 50\ncollection_total_items: 2\nhas_more: false\nSUMMARY_RESULT_END";
        let result = parse_collection_tool_result(
            &collection,
            response,
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        )
        .result
        .expect("expected parsed result");
        let result = result.as_summary().expect("expected summary result");

        assert_eq!(result.covered_item_uris, vec!["at://one", "at://two"]);
        assert!(result.omitted_item_uris.is_empty());
        assert_eq!(result.window_offset, Some(0));
        assert_eq!(result.window_size, Some(2));
        assert_eq!(result.page_index, Some(0));
        assert_eq!(result.page_size, Some(50));
        assert_eq!(result.collection_total_items, Some(2));
        assert_eq!(result.has_more, Some(false));
        assert_eq!(result.title, "LLM-selected post in Recent test posts");
    }

    #[test]
    fn parse_collection_summary_result_collects_repeated_omitted_tagged_fields() {
        let parsed = super::parse_collection_summary_result_tagged(
            "SUMMARY_RESULT_START\nsummary: grounded paragraph\ncovered_item_uri: at://one\nomitted_item_uri: at://two\nomitted_item_uri: at://three\nwindow_offset: 0\nwindow_size: 3\npage_index: 0\npage_size: 25\ncollection_total_items: 3\nhas_more: false\nSUMMARY_RESULT_END",
        )
        .expect("expected tagged parse");

        assert_eq!(parsed.covered_item_uris, vec!["at://one"]);
        assert_eq!(parsed.omitted_item_uris, vec!["at://two", "at://three"]);
    }

    #[test]
    fn parse_collection_summary_result_reports_missing_tagged_markers() {
        let err = super::parse_collection_summary_result_tagged(
            "summary: grounded paragraph\ncovered_item_uri: at://one",
        )
        .expect_err("expected missing marker failure");

        assert!(err.contains("missing SUMMARY_RESULT_START marker"));
    }

    #[test]
    fn parse_collection_summary_result_reports_malformed_tagged_scalar() {
        let err = super::parse_collection_summary_result_tagged(
            "SUMMARY_RESULT_START\nsummary: grounded paragraph\ncovered_item_uri: at://one\nwindow_offset: nope\nwindow_size: 1\npage_index: 0\npage_size: 25\ncollection_total_items: 1\nhas_more: false\nSUMMARY_RESULT_END",
        )
        .expect_err("expected malformed scalar failure");

        assert!(err.contains("malformed integer field `window_offset`"));
    }

    #[test]
    fn parse_collection_summary_result_falls_back_to_json_when_tagged_parse_fails() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post one".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post two".to_string(),
                    },
                ],
            ),
            0,
            25,
        );

        let parsed = parse_collection_tool_result(
            &collection,
            r#"{"title":"window summary","summary":"Both posts focus on test content, with \"post one\" and \"post two\" forming the full window.","covered_item_uris":["at://one","at://two"],"omitted_item_uris":[],"window_start":0,"window_total_items":2}"#,
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        );

        let diagnostic = parsed.diagnostic.expect("expected fallback diagnostic");
        assert!(
            diagnostic.contains("tool-call parsing failed")
                || diagnostic.contains("tool-call parser failed")
        );
        let result = parsed
            .result
            .expect("expected parsed result")
            .as_summary()
            .expect("expected summary result")
            .clone();
        assert_eq!(result.covered_item_uris.len(), 2);
        assert_eq!(result.window_start, Some(0));
    }

    #[test]
    fn parse_collection_summary_result_includes_parser_failure_reasons() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![PostRecord {
                    uri: "at://one".to_string(),
                    author_handle: "alpha.test".to_string(),
                    body: "post one".to_string(),
                }],
            ),
            0,
            25,
        );

        let parsed = parse_collection_tool_result(
            &collection,
            "SUMMARY_RESULT_START\nsummary: grounded paragraph\ncovered_item_uri: at://one\nwindow_offset: nope\nSUMMARY_RESULT_END",
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        );

        let diagnostic = parsed.diagnostic.expect("expected diagnostic");
        assert!(diagnostic.contains("tagged parser failed"));
        assert!(diagnostic.contains("malformed integer field `window_offset`"));
        assert!(diagnostic.contains("json parser failed"));
    }

    #[test]
    fn parse_collection_summary_result_recovers_partial_tagged_response_without_end_marker() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post one".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "post two".to_string(),
                    },
                ],
            ),
            0,
            25,
        );

        let parsed = parse_collection_tool_result(
            &collection,
            "SUMMARY_RESULT_START\nsummary: Both posts stay grounded in direct text, including \"post one\" and \"post two\".\ncovered_item_uri: at://one\nwindow_offset: 0\nwindow_size: 2\ncollection_total_items: 2\nhas_more: false",
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        );

        let diagnostic = parsed.diagnostic.expect("expected repair diagnostic");
        assert!(diagnostic.contains("partial recovery"));
        assert!(diagnostic.contains("missing SUMMARY_RESULT_END marker"));

        let result = parsed
            .result
            .expect("expected parsed result")
            .as_summary()
            .expect("expected summary result")
            .clone();
        assert_eq!(result.covered_item_uris, vec!["at://one"]);
        assert_eq!(result.omitted_item_uris, vec!["at://two"]);
        assert_eq!(result.page_index, Some(0));
        assert_eq!(result.page_size, Some(50));
        assert_eq!(result.collection_total_items, Some(2));
        assert_eq!(result.has_more, Some(false));
    }

    #[test]
    fn parse_collection_summary_result_partial_repair_still_rejects_missing_minimum_fields() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent test posts",
                vec![PostRecord {
                    uri: "at://one".to_string(),
                    author_handle: "alpha.test".to_string(),
                    body: "post one".to_string(),
                }],
            ),
            0,
            25,
        );

        let parsed = parse_collection_tool_result(
            &collection,
            "SUMMARY_RESULT_START\nsummary: grounded paragraph\ncovered_item_uri: at://one\nwindow_offset: 0",
            CollectionLeafToolKind::Summary,
            super::collection_window_offset(&collection),
        );

        assert!(parsed.result.is_none());
        let diagnostic = parsed.diagnostic.expect("expected diagnostic");
        assert!(diagnostic.contains("partial tagged repair failed"));
        assert!(diagnostic.contains("window_size"));
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
        assert!(rendered.contains("Tool: search"));
        assert!(rendered.contains("Tool: summary"));
    }

    #[test]
    fn parses_prompt_tool_call_block() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: search\nargs: {\"query\":\"who is rei-cast.xyz?\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "search");
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
            "TOOL_CALL\nname: search\nargs:\n{query:<|\"|>what are people on Bluesky saying about topic x<|\"|>}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "search");
        assert_eq!(
            tool_call.args["query"],
            "what are people on Bluesky saying about topic x"
        );
    }

    #[test]
    fn parses_first_tool_call_when_trailing_thought_continues() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: list_collections\nargs: {actor_did: \"did:plc:testactor\"}\n\n<|channel>thought\nextra commentary\n<channel|>TOOL_CALL\nname: search\nargs: {query:\"who is rei-cast.xyz?\"}",
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

    #[tokio::test]
    async fn prepared_summary_uses_selected_actor_fallback_when_query_is_relative() {
        let tools = BlueskyTools::new();
        let llm_client = summary_prep_llm_for_tests(
            "{\"requested_summary_scope\":{\"kind\":\"count\",\"requested_items\":50},\"collection_target_hint\":\"recent_posts\"}",
        );
        let selected_did: Did = "did:plc:selectedactor".parse().expect("invalid did");
        let mut store = NotificationStore::new();
        store.cache_actor(&selected_did, "selected.test", None);
        store.cache_post_collection(
            LabeledPostCollection::new(
                "recent_posts:did:plc:selectedactor",
                "Recent posts",
                vec![],
            )
            .with_collection_kind("recent_posts")
            .with_actor_did(selected_did.as_str()),
        );
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: serde_json::json!({"query":"summarize the last 50 things this actor has posted"}),
        };

        let prepared = tools
            .prepare_root_tool_input(&tool_call, Some(&selected_did), &store, &llm_client)
            .await
            .expect("expected prepared summary input");

        match prepared {
            PreparedPromptToolInput::Summary(summary) => {
                let PreparedSummaryActor::Resolved(anchor) = &summary.actor else {
                    panic!("expected resolved actor anchor");
                };
                assert_eq!(anchor.actor_did, selected_did);
                assert_eq!(anchor.source, ActorAnchorSource::SelectedActorFallback);
                assert_eq!(
                    summary.collection_target_hint,
                    SummaryCollectionTargetHint::RecentPosts
                );
            }
            other => panic!("expected summary prep, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn prepared_summary_fails_without_explicit_or_selected_actor() {
        let tools = BlueskyTools::new();
        let llm_client = summary_prep_llm_for_tests(
            "{\"requested_summary_scope\":{\"kind\":\"count\",\"requested_items\":50},\"collection_target_hint\":\"recent_posts\"}",
        );
        let store = NotificationStore::new();
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: serde_json::json!({"query":"summarize the last 50 things this actor has posted"}),
        };

        let failure = tools
            .prepare_root_tool_input(&tool_call, None, &store, &llm_client)
            .await
            .expect_err("expected prep failure");

        assert_eq!(
            failure.missing,
            vec![ToolPrepMissingPrerequisite::ActorAnchor]
        );
    }

    #[tokio::test]
    async fn prepared_search_uses_selected_actor_fallback_for_relative_query() {
        let tools = BlueskyTools::new();
        let llm_client = summary_prep_llm_for_tests(
            "{\"requested_summary_scope\":{\"kind\":\"count\",\"requested_items\":25},\"collection_target_hint\":\"recent_posts\"}",
        );
        let selected_did: Did = "did:plc:selectedactor".parse().expect("invalid did");
        let mut store = NotificationStore::new();
        store.cache_actor(&selected_did, "selected.test", None);
        let tool_call = PromptToolCall {
            name: "search".to_string(),
            args: serde_json::json!({"query":"what have they been posting about lately?"}),
        };

        let prepared = tools
            .prepare_root_tool_input(&tool_call, Some(&selected_did), &store, &llm_client)
            .await
            .expect("expected prepared search input");

        match prepared {
            PreparedPromptToolInput::Search(search) => {
                let anchor = search.actor_anchor.expect("expected actor anchor");
                assert_eq!(anchor.actor_did, selected_did);
                assert_eq!(anchor.source, ActorAnchorSource::SelectedActorFallback);
            }
            other => panic!("expected search prep, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn prepared_summary_prefers_explicit_query_actor_over_selected_actor() {
        let tools = BlueskyTools::new();
        let llm_client = summary_prep_llm_for_tests(
            "{\"requested_summary_scope\":{\"kind\":\"count\",\"requested_items\":50},\"collection_target_hint\":\"recent_posts\"}",
        );
        let explicit_did: Did = "did:plc:explicitactor".parse().expect("invalid did");
        let selected_did: Did = "did:plc:selectedactor".parse().expect("invalid did");
        let mut store = NotificationStore::new();
        store.cache_actor(&explicit_did, "explicit.test", None);
        store.cache_actor(&selected_did, "selected.test", None);
        store.cache_post_collection(
            LabeledPostCollection::new(
                "recent_posts:did:plc:explicitactor",
                "Recent posts",
                vec![],
            )
            .with_collection_kind("recent_posts")
            .with_actor_did(explicit_did.as_str()),
        );
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: serde_json::json!({"query":"summarize the last 50 posts by explicit.test"}),
        };

        let prepared = tools
            .prepare_root_tool_input(&tool_call, Some(&selected_did), &store, &llm_client)
            .await
            .expect("expected prepared summary input");

        match prepared {
            PreparedPromptToolInput::Summary(summary) => {
                let PreparedSummaryActor::Resolved(anchor) = &summary.actor else {
                    panic!("expected resolved actor anchor");
                };
                assert_eq!(anchor.actor_did, explicit_did);
                assert_eq!(anchor.source, ActorAnchorSource::ExplicitQueryRef);
            }
            other => panic!("expected summary prep, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn prepared_summary_accepts_uncached_explicit_query_actor() {
        let tools = BlueskyTools::new();
        let llm_client = summary_prep_llm_for_tests(
            "{\"requested_summary_scope\":{\"kind\":\"count\",\"requested_items\":150},\"collection_target_hint\":\"recent_posts\"}",
        );
        let store = NotificationStore::new();
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: serde_json::json!({"query":"summarize 150 posts by segyges.bsky.social into 3 paragraphs"}),
        };

        let prepared = tools
            .prepare_root_tool_input(&tool_call, None, &store, &llm_client)
            .await
            .expect("expected prepared summary input");

        match prepared {
            PreparedPromptToolInput::Summary(summary) => {
                assert_eq!(
                    &summary.actor,
                    &PreparedSummaryActor::ExplicitRef("segyges.bsky.social".to_string())
                );
                assert_eq!(
                    summary.collection_target_hint,
                    SummaryCollectionTargetHint::RecentPosts
                );
            }
            other => panic!("expected summary prep, got {other:?}"),
        }
    }

    #[test]
    fn validates_collection_id_rejects_truncated_actor_profile_did() {
        let err = validate_collection_id("actor_profile:did:3de...")
            .expect_err("expected invalid collection id");
        assert!(err.contains("invalid DID segment"));
    }

    #[test]
    fn strict_internal_tool_response_rejects_surrounding_prose() {
        let response = "I will search now.\n\nTOOL_CALL\nname: search\nargs: {\"collection_id\":\"clearsky_lists:did:plc:testactor\",\"prompt\":\"check lists\"}";
        let result = validate_internal_tool_response(response);
        assert!(matches!(result, super::InternalToolResponse::Invalid(_)));
    }

    #[test]
    fn strict_internal_tool_response_accepts_trailing_prose_by_discarding_it() {
        let response = "TOOL_CALL\nname: search\nargs: {\"collection_id\":\"clearsky_lists:did:plc:testactor\",\"prompt\":\"check lists\"}\n\nSelf-correction: hypothetical results go here";
        let result = validate_internal_tool_response(response);
        match result {
            super::InternalToolResponse::ToolCall(accepted) => {
                assert_eq!(accepted.tool_call.name, "search");
                assert!(accepted.discarded_trailing.is_some());
            }
            other => panic!("expected accepted tool call, got {other:?}"),
        }
    }

    #[test]
    fn strict_internal_tool_response_accepts_first_tool_block_and_discards_later_blocks() {
        let response = "TOOL_CALL\nname: search\nargs: {\"collection_id\":\"clearsky_lists:did:plc:testactor\",\"prompt\":\"check lists\"}\n\nTOOL_CALL\nname: search_global_posts\nargs: {\"query\":\"duplicate\"}";
        let result = validate_internal_tool_response(response);
        match result {
            super::InternalToolResponse::ToolCall(accepted) => {
                assert_eq!(accepted.tool_call.name, "search");
                assert!(
                    accepted
                        .discarded_trailing
                        .as_deref()
                        .is_some_and(|trailing| trailing.contains("search_global_posts"))
                );
            }
            other => panic!("expected accepted tool call, got {other:?}"),
        }
    }

    #[test]
    fn strict_internal_tool_response_accepts_whitespace_only_formatting_differences() {
        let response = "TOOL_CALL\nname: search\nargs:\n{\n  \"collection_id\": \"clearsky_lists:did:plc:testactor\",\n  \"prompt\": \"check lists\"\n}\n";
        let result = validate_internal_tool_response(response);
        assert!(matches!(result, super::InternalToolResponse::ToolCall(_)));
    }

    #[test]
    fn parse_internal_tool_repair_response_accepts_cannot_repair() {
        let repaired = parse_internal_tool_repair_response("CANNOT_REPAIR")
            .expect("expected successful parse");
        assert!(repaired.is_none());
    }

    #[test]
    fn deterministic_collection_repair_picks_recent_posts_for_post_query() {
        let did = "did:plc:testactor";
        let collections = vec![
            LabeledPostCollection::new("actor_profile:did:plc:testactor", "Profile", vec![])
                .with_collection_kind("actor_profile")
                .with_actor_did(did),
            LabeledPostCollection::new(
                "recent_replies_received:did:plc:testactor",
                "Recent replies received",
                vec![],
            )
            .with_collection_kind("recent_replies_received")
            .with_actor_did(did),
            LabeledPostCollection::new("recent_posts:did:plc:testactor", "Recent posts", vec![])
                .with_collection_kind("recent_posts")
                .with_actor_did(did),
            LabeledPostCollection::new(
                "recent_posts_unaddressed:did:plc:testactor",
                "Recent top-level posts",
                vec![],
            )
            .with_collection_kind("recent_posts_unaddressed")
            .with_actor_did(did),
        ];

        let picked = choose_deterministic_collection_id_for_actor(
            did,
            "summarize the 25 most recent posts by rei-cast.xyz",
            SearchIntent::General,
            &collections,
        )
        .expect("expected deterministic collection id");

        assert_eq!(picked, "recent_posts:did:plc:testactor");
    }

    #[test]
    fn deterministic_tool_call_repair_rewrites_bare_did_collection_id() {
        let tool_call = PromptToolCall {
            name: "collection_search".to_string(),
            args: serde_json::json!({
                "collection_id": "did:plc:testactor",
                "prompt": "inspect the latest posts",
                "page": 0
            }),
        };
        let collections = vec![
            LabeledPostCollection::new("recent_posts:did:plc:testactor", "Recent posts", vec![])
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:testactor"),
        ];

        let repaired = deterministic_repair_internal_search_tool_call(
            &tool_call,
            "inspect the 25 most recent posts by rei-cast.xyz",
            SearchIntent::General,
            &collections,
        )
        .expect("expected repaired tool call");

        assert_eq!(repaired.name, "collection_search");
        assert_eq!(
            repaired.args["collection_id"],
            serde_json::json!("recent_posts:did:plc:testactor")
        );
    }

    #[test]
    fn truncate_diagnostic_block_preserves_full_text() {
        let truncated = super::truncate_diagnostic_block("abcdefghij", 6);
        assert_eq!(truncated, "abcdefghij");
    }

    #[test]
    fn parses_collection_review_verdict_block() {
        let verdict = parse_collection_review_verdict(
            "status: fail\ngrounded: false\nsufficient: false\nreason: summary is metadata-heavy\nrepair_needed: true\nrepair_instructions: cite the actual list names",
        )
        .expect("expected verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(verdict.repair_needed);
        assert!(!verdict.grounded);
        assert!(!verdict.sufficient);
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
        let result = LlmSearchResult {
            title: "weak".to_string(),
            summary:
                "collection_id: clearsky:test did:plc:testactor source_post_uri: at://foo item[0]"
                    .to_string(),
            search_results: vec![LlmSearchResultItem {
                uri: "https://example.com/list-a".to_string(),
                source_collection_id: Some("clearsky:test".to_string()),
            }],
        };
        let execution = LlmSearchExecution {
            result: Some(search_leaf_result(result.clone())),
            original_result: Some(search_leaf_result(result)),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(
            CollectionLeafToolKind::Search,
            &collection,
            RequestedSummaryScope::CurrentWindow,
            &execution,
        );
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
            result: Some(search_leaf_result(result.clone())),
            original_result: Some(search_leaf_result(result)),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(
            CollectionLeafToolKind::Search,
            &collection,
            RequestedSummaryScope::CurrentWindow,
            &execution,
        );
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.grounded);
        assert!(verdict.sufficient);
    }

    #[test]
    fn heuristic_collection_review_fails_fallback_diagnostic_summary() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky",
            vec![PostRecord {
                uri: "https://example.com/list-a".to_string(),
                author_handle: "clearsky".to_string(),
                body: "list_name: AI Fanatics\nlist_description: magical thinking".to_string(),
            }],
        );
        let result = LlmSearchResult {
            title: "fallback".to_string(),
            summary: "The strongest grounded evidence in this moderation-list collection centers on 1 selected records, with repeated signals around AI Fanatics. The matched record text also includes descriptions such as: \"magical thinking\". This fallback summary is derived directly from those matched records because the model response did not yield a usable structured `summary:` field.".to_string(),
            search_results: vec![LlmSearchResultItem {
                uri: "https://example.com/list-a".to_string(),
                source_collection_id: Some("clearsky:test".to_string()),
            }],
        };
        let execution = LlmSearchExecution {
            result: Some(search_leaf_result(result.clone())),
            original_result: Some(search_leaf_result(result)),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(
            CollectionLeafToolKind::Search,
            &collection,
            RequestedSummaryScope::CurrentWindow,
            &execution,
        );
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(verdict.repair_needed);
    }

    #[test]
    fn heuristic_collection_review_fails_collection_summary_with_incomplete_coverage() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "first topic".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "second topic".to_string(),
                    },
                ],
            ),
            0,
            25,
        );
        let result = LlmSummaryResult {
            title: "window".to_string(),
            summary: "The window mostly discusses first-topic material.".to_string(),
            covered_item_uris: vec!["at://one".to_string()],
            omitted_item_uris: Vec::new(),
            concatenated_window_summaries: None,
            window_offset: Some(0),
            window_size: Some(2),
            page_index: Some(0),
            page_size: Some(25),
            collection_total_items: Some(2),
            has_more: Some(false),
            source_exhausted: None,
            window_start: Some(0),
            window_total_items: Some(2),
        };
        let execution = LlmSearchExecution {
            result: Some(summary_leaf_result(result.clone())),
            original_result: Some(summary_leaf_result(result)),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(
            CollectionLeafToolKind::Summary,
            &collection,
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
            &execution,
        );
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(!verdict.repair_needed);
        assert!(!verdict.grounded);
        assert!(!verdict.sufficient);
    }

    #[test]
    fn heuristic_collection_review_marks_grounded_summary_as_insufficient_for_larger_scope() {
        let posts = (0..30)
            .map(|index| PostRecord {
                uri: format!("at://{index}"),
                author_handle: "alpha.test".to_string(),
                body: format!("topic {index}"),
            })
            .collect::<Vec<_>>();
        let full_collection = LabeledPostCollection::new("recent:test", "Recent", posts);
        let collection = paged_search_collection(&full_collection, 0, 25);
        let covered_item_uris = collection
            .posts
            .iter()
            .map(|post| post.uri.clone())
            .collect::<Vec<_>>();
        let result = LlmSummaryResult {
            title: "window".to_string(),
            summary: "The first 25 posts form a grounded page summary, with repeated short updates like \"topic 0,\" \"topic 12,\" and \"topic 24\" showing the full opening window."
                .to_string(),
            covered_item_uris,
            omitted_item_uris: Vec::new(),
            concatenated_window_summaries: None,
            window_offset: Some(0),
            window_size: Some(25),
            page_index: Some(0),
            page_size: Some(25),
            collection_total_items: Some(30),
            has_more: Some(true),
            source_exhausted: None,
            window_start: Some(0),
            window_total_items: Some(25),
        };
        let execution = LlmSearchExecution {
            result: Some(summary_leaf_result(result.clone())),
            original_result: Some(summary_leaf_result(result)),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(
            CollectionLeafToolKind::Summary,
            &collection,
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
            &execution,
        );
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(verdict.grounded);
        assert!(!verdict.sufficient);
        assert!(verdict.additional_pages_needed);
        assert_eq!(verdict.next_offset, Some(25));
        assert_eq!(verdict.required_total_items, Some(30));
    }

    #[test]
    fn heuristic_collection_review_passes_collection_summary_with_full_coverage() {
        let collection = paged_search_collection(
            &LabeledPostCollection::new(
                "recent:test",
                "Recent",
                vec![
                    PostRecord {
                        uri: "at://one".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "first topic".to_string(),
                    },
                    PostRecord {
                        uri: "at://two".to_string(),
                        author_handle: "alpha.test".to_string(),
                        body: "second topic".to_string(),
                    },
                ],
            ),
            0,
            25,
        );
        let result = LlmSummaryResult {
            title: "window".to_string(),
            summary: "The two-post window splits between \"first topic\" and \"second topic,\" so the summary accounts for the entire page rather than only one standout item.".to_string(),
            covered_item_uris: vec!["at://one".to_string(), "at://two".to_string()],
            omitted_item_uris: Vec::new(),
            concatenated_window_summaries: None,
            window_offset: Some(0),
            window_size: Some(2),
            page_index: Some(0),
            page_size: Some(25),
            collection_total_items: Some(2),
            has_more: Some(false),
            source_exhausted: None,
            window_start: Some(0),
            window_total_items: Some(2),
        };
        let execution = LlmSearchExecution {
            result: Some(summary_leaf_result(result.clone())),
            original_result: Some(summary_leaf_result(result)),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: None,
            review_context_window: None,
            repair_diagnostic: None,
        };

        let verdict = heuristic_collection_review(
            CollectionLeafToolKind::Summary,
            &collection,
            RequestedSummaryScope::Page { page_index: 0 },
            &execution,
        );
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.grounded);
        assert!(verdict.sufficient);
    }

    #[test]
    fn summary_sufficiency_gates_pass_after_contiguous_multi_page_coverage() {
        let page0 = LlmSearchExecution {
            result: Some(summary_leaf_result(LlmSummaryResult {
                title: "page 0".to_string(),
                summary: "page 0".to_string(),
                covered_item_uris: vec!["at://0".to_string()],
                omitted_item_uris: Vec::new(),
                concatenated_window_summaries: None,
                window_offset: Some(0),
                window_size: Some(25),
                page_index: Some(0),
                page_size: Some(25),
                collection_total_items: Some(50),
                has_more: Some(true),
                source_exhausted: None,
                window_start: Some(0),
                window_total_items: Some(25),
            })),
            original_result: None,
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: Some(super::CollectionReviewVerdict {
                status: CollectionReviewStatus::Fail,
                grounded: true,
                sufficient: false,
                reason: "need more pages".to_string(),
                repair_needed: false,
                repair_instructions: None,
                additional_pages_needed: true,
                next_page: Some(1),
                next_offset: Some(25),
                required_total_items: Some(50),
            }),
            review_context_window: None,
            repair_diagnostic: None,
        };
        let mut page1 = LlmSearchExecution {
            result: Some(summary_leaf_result(LlmSummaryResult {
                title: "page 1".to_string(),
                summary: "page 1".to_string(),
                covered_item_uris: vec!["at://25".to_string()],
                omitted_item_uris: Vec::new(),
                concatenated_window_summaries: None,
                window_offset: Some(25),
                window_size: Some(25),
                page_index: Some(1),
                page_size: Some(25),
                collection_total_items: Some(50),
                has_more: Some(false),
                source_exhausted: None,
                window_start: Some(25),
                window_total_items: Some(25),
            })),
            original_result: None,
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: Some(super::CollectionReviewVerdict {
                status: CollectionReviewStatus::Fail,
                grounded: true,
                sufficient: false,
                reason: "need more pages".to_string(),
                repair_needed: false,
                repair_instructions: None,
                additional_pages_needed: true,
                next_page: Some(2),
                next_offset: Some(50),
                required_total_items: Some(50),
            }),
            review_context_window: None,
            repair_diagnostic: None,
        };
        let prior_outcomes = vec![super::CollectionToolOutcome {
            tool_kind: CollectionLeafToolKind::Summary,
            collection_id: "recent:test".to_string(),
            collection_label: "Recent".to_string(),
            execution: Ok(page0),
        }];

        super::apply_summary_sufficiency_gates(
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
            "recent:test",
            &prior_outcomes,
            &mut page1,
        );

        let verdict = page1.review_verdict.expect("verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.sufficient);
        assert!(!verdict.additional_pages_needed);
        assert_eq!(verdict.required_total_items, Some(50));
        let preserved = page1
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("page result should remain available once coverage becomes sufficient");
        assert_eq!(preserved.window_offset, Some(25));
        assert_eq!(preserved.window_size, Some(25));
    }

    #[test]
    fn apply_summary_sufficiency_gates_preserves_grounded_page_result_while_more_pages_are_needed()
    {
        let original_result = LlmSummaryResult {
            title: "page 0".to_string(),
            summary: "grounded page 0".to_string(),
            covered_item_uris: vec!["at://0".to_string()],
            omitted_item_uris: Vec::new(),
            concatenated_window_summaries: None,
            window_offset: Some(0),
            window_size: Some(25),
            page_index: Some(0),
            page_size: Some(25),
            collection_total_items: Some(50),
            has_more: Some(true),
            source_exhausted: None,
            window_start: Some(0),
            window_total_items: Some(25),
        };
        let mut execution = LlmSearchExecution {
            result: Some(summary_leaf_result(original_result.clone())),
            original_result: None,
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: Some(super::CollectionReviewVerdict {
                status: CollectionReviewStatus::Fail,
                grounded: true,
                sufficient: false,
                reason: "need more pages".to_string(),
                repair_needed: false,
                repair_instructions: None,
                additional_pages_needed: true,
                next_page: Some(1),
                next_offset: Some(25),
                required_total_items: Some(50),
            }),
            review_context_window: None,
            repair_diagnostic: None,
        };

        super::apply_summary_sufficiency_gates(
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
            "recent:test",
            &[],
            &mut execution,
        );

        let verdict = execution.review_verdict.as_ref().expect("verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(verdict.grounded);
        assert!(!verdict.sufficient);
        assert!(verdict.additional_pages_needed);
        assert_eq!(verdict.next_offset, Some(25));

        let preserved = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("preserved grounded page result");
        assert_eq!(preserved.summary, original_result.summary);
        assert_eq!(preserved.window_offset, Some(0));
        assert_eq!(preserved.window_size, Some(25));
    }

    #[test]
    fn apply_summary_sufficiency_gates_treats_short_final_window_as_exhausted_available_scope() {
        let mut execution = LlmSearchExecution {
            result: Some(summary_leaf_result(LlmSummaryResult {
                title: "only page".to_string(),
                summary: "only page".to_string(),
                covered_item_uris: (0..7).map(|index| format!("at://{index}")).collect(),
                omitted_item_uris: Vec::new(),
                concatenated_window_summaries: None,
                window_offset: Some(0),
                window_size: Some(7),
                page_index: Some(0),
                page_size: Some(25),
                collection_total_items: Some(73),
                has_more: Some(true),
                source_exhausted: None,
                window_start: Some(0),
                window_total_items: Some(7),
            })),
            original_result: None,
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: Some(super::CollectionReviewVerdict {
                status: CollectionReviewStatus::Fail,
                grounded: true,
                sufficient: false,
                reason: "need more pages".to_string(),
                repair_needed: false,
                repair_instructions: None,
                additional_pages_needed: true,
                next_page: Some(0),
                next_offset: Some(7),
                required_total_items: Some(50),
            }),
            review_context_window: None,
            repair_diagnostic: None,
        };

        super::apply_summary_sufficiency_gates(
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
            "recent:test",
            &[],
            &mut execution,
        );

        let verdict = execution.review_verdict.expect("verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.grounded);
        assert!(verdict.sufficient);
        assert!(!verdict.additional_pages_needed);
        assert_eq!(verdict.next_page, None);
        assert_eq!(verdict.next_offset, None);
        assert_eq!(verdict.required_total_items, Some(7));
        assert!(verdict.reason.contains("all 7 available item(s)"));
    }

    #[test]
    fn apply_summary_sufficiency_gates_preserves_ungrounded_failure_reason() {
        let mut execution = LlmSearchExecution {
            result: None,
            original_result: Some(summary_leaf_result(LlmSummaryResult {
                title: "page 0".to_string(),
                summary: "generic paraphrase without grounded snippets".to_string(),
                covered_item_uris: (0..50).map(|index| format!("at://{index}")).collect(),
                omitted_item_uris: Vec::new(),
                concatenated_window_summaries: None,
                window_offset: Some(0),
                window_size: Some(50),
                page_index: Some(0),
                page_size: Some(50),
                collection_total_items: Some(400),
                has_more: Some(true),
                source_exhausted: None,
                window_start: Some(0),
                window_total_items: Some(50),
            })),
            context_window: empty_test_window(),
            diagnostic: None,
            raw_response: None,
            review_verdict: Some(super::CollectionReviewVerdict {
                status: CollectionReviewStatus::Fail,
                grounded: false,
                sufficient: false,
                reason: "The summary omits meaningful text that was available in the matched records.".to_string(),
                repair_needed: false,
                repair_instructions: None,
                additional_pages_needed: false,
                next_page: None,
                next_offset: None,
                required_total_items: None,
            }),
            review_context_window: None,
            repair_diagnostic: None,
        };

        super::apply_summary_sufficiency_gates(
            RequestedSummaryScope::Count {
                requested_items: 400,
            },
            "recent:test",
            &[],
            &mut execution,
        );

        let verdict = execution.review_verdict.expect("verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Fail);
        assert!(!verdict.grounded);
        assert!(!verdict.sufficient);
        assert_eq!(
            verdict.reason,
            "Grounded summary coverage currently reaches 50 item(s), but 400 item(s) are required before parent synthesis is sufficient."
        );
        assert!(verdict.additional_pages_needed);
        assert_eq!(verdict.next_offset, Some(50));
        assert_eq!(verdict.next_page, Some(1));
        assert_eq!(verdict.required_total_items, Some(400));
        assert!(execution.result.is_none());
        assert!(execution.original_result.is_some());
    }

    #[test]
    fn deterministic_repair_summary_for_clearsky_lists_uses_real_list_names() {
        let collection = LabeledPostCollection::new(
            "clearsky:test",
            "Clearsky test lists",
            vec![
                PostRecord {
                    uri: "https://example.com/list-a".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: Follows of @norvid-studies.bsky.social\nlist_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.".to_string(),
                },
                PostRecord {
                    uri: "https://example.com/list-b".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: Follows of @norvid-studies.bsky.social\nlist_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.".to_string(),
                },
                PostRecord {
                    uri: "https://example.com/list-c".to_string(),
                    author_handle: "clearsky".to_string(),
                    body: "list_name: The Great AI - NFT - CRYPTO Cull\nlist_description: If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will.".to_string(),
                },
            ],
        )
        .with_collection_kind("clearsky_lists");

        let summary = deterministic_repair_summary(
            &collection,
            &[
                "https://example.com/list-a",
                "https://example.com/list-b",
                "https://example.com/list-c",
            ],
        );

        assert!(summary.contains("Follows of @norvid-studies.bsky.social"));
        assert!(summary.contains("The Great AI - NFT - CRYPTO Cull"));
        assert!(summary.contains("[0]"));
        assert!(summary.contains("[1]"));
        assert!(!summary.contains("usable structured `summary:` field"));
    }

    #[test]
    fn deterministic_repair_summary_for_replies_uses_handles_and_reply_text_without_metadata() {
        let collection = LabeledPostCollection::new(
            "recent_replies_received:did:plc:testactor",
            "Recent replies received",
            vec![
                PostRecord {
                    uri: "at://reply-one".to_string(),
                    author_handle: "bot-tan.suibari.com".to_string(),
                    body: "source_post_uri: at://did:plc:testactor/app.bsky.feed.post/root-one\nreply_text: This 3D render looks so cool! Such precise lines.".to_string(),
                },
                PostRecord {
                    uri: "at://reply-two".to_string(),
                    author_handle: "technobaboo.bsky.social".to_string(),
                    body: "source_post_uri: at://did:plc:testactor/app.bsky.feed.post/root-two\nreply_text: omg saaaaaame".to_string(),
                },
            ],
        )
        .with_collection_kind("recent_replies_received");

        let summary =
            deterministic_repair_summary(&collection, &["at://reply-one", "at://reply-two"]);

        assert!(summary.contains("[0] @bot-tan.suibari.com"));
        assert!(summary.contains("[1] @technobaboo.bsky.social"));
        assert!(summary.contains("This 3D render looks so cool! Such precise lines."));
        assert!(summary.contains("omg saaaaaame"));
        assert!(!summary.contains("source_post_uri:"));
        assert!(!summary.contains("reply_text:"));
    }

    #[test]
    fn parses_reply_collection_result_from_reply_text_hint_without_uri() {
        let collection = LabeledPostCollection::new(
            "recent_replies_received:did:plc:testactor",
            "Recent replies received",
            vec![PostRecord {
                uri: "at://did:plc:reply-author/app.bsky.feed.post/abc".to_string(),
                author_handle: "reply.author".to_string(),
                body: "source_post_uri: at://did:plc:testactor/app.bsky.feed.post/root\nreply_text: i asked for help with fixing performance first".to_string(),
            }],
        )
        .with_collection_kind("recent_replies_received");

        let result = parse_llm_search_result(
            &collection,
            "title: grounded\nsummary: The replies include \"i asked for help with fixing performance first,\" which reads as engaged product feedback rather than hostility.",
        )
        .expect("expected reply-text hint to anchor a result");

        assert_eq!(result.search_results.len(), 1);
        assert_eq!(
            result.search_results[0].uri,
            "at://did:plc:reply-author/app.bsky.feed.post/abc"
        );
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
    fn internal_search_tool_protocol_lists_planner_tools() {
        let rendered = render_internal_search_tool_protocol();

        assert!(rendered.contains("resolve_actor_refs"));
        assert!(rendered.contains("hydrate_actor_scope"));
        assert!(rendered.contains("Tool: collection_search"));
        assert!(!rendered.contains("Tool: summary"));
        assert!(rendered.contains("search_global_posts"));
        assert!(!rendered.contains("set_summary_scope"));
    }

    #[test]
    fn summary_leaf_requests_do_not_force_json_object_response_format() {
        assert!(CollectionLeafToolKind::Summary.response_format().is_none());
    }

    #[test]
    fn search_leaf_requests_still_force_json_object_response_format() {
        assert!(matches!(
            CollectionLeafToolKind::Search.response_format(),
            Some(ChatCompletionResponseFormat::JsonObject)
        ));
    }

    #[test]
    fn validates_collection_id_rejects_bare_did_with_collection_hint() {
        let err = validate_collection_id("did:plc:testactor").expect_err("expected invalid id");
        assert!(err.contains("bare DID"));
        assert!(err.contains("recent_posts:did:plc:testactor"));
    }

    #[test]
    fn parse_requested_summary_scope_args_accepts_page_range() {
        let scope = parse_requested_summary_scope_args(&serde_json::json!({
            "kind": "page_range",
            "start_page": 2,
            "end_page": 1
        }))
        .expect("expected scope");

        assert_eq!(
            scope,
            RequestedSummaryScope::PageRange {
                start_page: 1,
                end_page: 2,
            }
        );
    }

    #[test]
    fn detect_summary_collection_target_hint_prefers_lists_for_list_queries() {
        assert_eq!(
            detect_summary_collection_target_hint(
                "summarize the most recent 50 lists that destiny.gg has been placed on"
            ),
            SummaryCollectionTargetHint::Lists
        );
        assert_eq!(
            detect_summary_collection_target_hint(
                "what can we infer about his reputation based on the lists he's on?"
            ),
            SummaryCollectionTargetHint::Lists
        );
    }

    #[test]
    fn pick_summary_collection_for_hint_prefers_clearsky_lists() {
        let collections = vec![
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
            LabeledPostCollection::new(
                "clearsky_lists:did:plc:test",
                "Clearsky moderation lists",
                Vec::new(),
            )
            .with_collection_kind("clearsky_lists")
            .with_actor_did("did:plc:test"),
        ];

        let selected =
            pick_summary_collection_for_hint(&collections, &SummaryCollectionTargetHint::Lists)
                .expect("expected lists collection");

        assert_eq!(selected.collection_kind, "clearsky_lists");
        assert_eq!(selected.id, "clearsky_lists:did:plc:test");
    }

    #[test]
    fn pick_summary_collection_for_hint_prefers_recent_posts_over_profile() {
        let collections = vec![
            LabeledPostCollection::new("actor_profile:did:plc:test", "Profile", Vec::new())
                .with_collection_kind("actor_profile")
                .with_actor_did("did:plc:test"),
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
        ];

        let selected = pick_summary_collection_for_hint(
            &collections,
            &SummaryCollectionTargetHint::RecentPosts,
        )
        .expect("expected recent posts collection");

        assert_eq!(selected.collection_kind, "recent_posts");
        assert_eq!(selected.id, "recent_posts:did:plc:test");
    }

    #[test]
    fn review_summary_collection_selection_repairs_post_query_to_recent_posts() {
        let collections = vec![
            LabeledPostCollection::new("actor_profile:did:plc:test", "Profile", Vec::new())
                .with_collection_kind("actor_profile")
                .with_actor_did("did:plc:test"),
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
        ];

        let review = review_summary_collection_selection(
            "summarize the last 300 posts by schizanon.bsky.social",
            RequestedSummaryScope::Count {
                requested_items: 300,
            },
            &collections,
            &collections[0],
        );

        assert_eq!(review.status, SummarySelectionReviewStatus::Repaired);
        assert!(review.deterministic_repair_applied);
        assert_eq!(review.original_collection_kind, "actor_profile");
        assert_eq!(review.final_collection_kind, "recent_posts");
        assert_eq!(review.final_collection_id, "recent_posts:did:plc:test");
    }

    #[test]
    fn review_summary_collection_selection_repairs_replies_query_to_sent_replies() {
        let collections = vec![
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
            LabeledPostCollection::new(
                "recent_replies_sent:did:plc:test",
                "Recent replies sent",
                Vec::new(),
            )
            .with_collection_kind("recent_replies_sent")
            .with_actor_did("did:plc:test"),
        ];

        let review = review_summary_collection_selection(
            "summarize the last 50 replies sent by schizanon.bsky.social",
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
            &collections,
            &collections[0],
        );

        assert_eq!(review.status, SummarySelectionReviewStatus::Repaired);
        assert!(review.deterministic_repair_applied);
        assert_eq!(review.final_collection_kind, "recent_replies_sent");
        assert_eq!(
            review.final_collection_id,
            "recent_replies_sent:did:plc:test"
        );
    }

    #[test]
    fn review_summary_collection_selection_accepts_profile_query_for_actor_profile() {
        let collections = vec![
            LabeledPostCollection::new("actor_profile:did:plc:test", "Profile", Vec::new())
                .with_collection_kind("actor_profile")
                .with_actor_did("did:plc:test"),
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
        ];

        let review = review_summary_collection_selection(
            "what does their profile say?",
            RequestedSummaryScope::CurrentWindow,
            &collections,
            &collections[0],
        );

        assert_eq!(review.status, SummarySelectionReviewStatus::Accepted);
        assert!(!review.deterministic_repair_applied);
        assert_eq!(review.final_collection_kind, "actor_profile");
    }

    #[test]
    fn review_summary_collection_selection_rejects_explicit_post_query_without_posts_collection() {
        let collections = vec![
            LabeledPostCollection::new("actor_profile:did:plc:test", "Profile", Vec::new())
                .with_collection_kind("actor_profile")
                .with_actor_did("did:plc:test"),
        ];

        let review = review_summary_collection_selection(
            "summarize the last 300 posts by schizanon.bsky.social",
            RequestedSummaryScope::Count {
                requested_items: 300,
            },
            &collections,
            &collections[0],
        );

        assert_eq!(review.status, SummarySelectionReviewStatus::Rejected);
        assert!(!review.deterministic_repair_applied);
        assert!(review.reason.contains("recent_posts"));
    }

    #[test]
    fn build_summary_agent_node_marks_empty_outcomes_as_failed() {
        let llm_client = summary_prep_llm_for_tests(
            r#"{"status":"accepted","final_collection_id":"recent_posts:did:plc:test","reason":"ok"}"#,
        );

        let node =
            build_summary_agent_node("summarize the last 50 posts by test", &[], &llm_client);

        assert_eq!(node.status, crate::harness::agents::AgentNodeStatus::Failed);
        assert_eq!(
            node.result_summary.as_deref(),
            Some("status: failed\nreason: no summary pages were processed")
        );
    }

    #[test]
    fn build_search_agent_node_marks_empty_outcomes_as_failed() {
        let llm_client = summary_prep_llm_for_tests(
            r#"{"status":"accepted","final_collection_id":"recent_posts:did:plc:test","reason":"ok"}"#,
        );

        let node = build_search_agent_node("search for mentions of test", &[], &llm_client);

        assert_eq!(node.status, crate::harness::agents::AgentNodeStatus::Failed);
    }

    #[tokio::test]
    async fn llm_review_summary_collection_selection_can_repair_ambiguous_query() {
        let llm_client = scripted_llm_for_tests(vec![
            r#"{"status":"repaired","final_collection_id":"recent_posts:did:plc:test","reason":"The query asks about posting activity lately, so recent posts are the better fit."}"#,
        ]);
        let collections = vec![
            LabeledPostCollection::new("actor_profile:did:plc:test", "Profile", Vec::new())
                .with_collection_kind("actor_profile")
                .with_actor_did("did:plc:test"),
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
        ];

        let review = llm_review_summary_collection_selection(
            "what have they been posting about lately?",
            RequestedSummaryScope::CurrentWindow,
            &collections,
            &collections[0],
            &llm_client,
        )
        .await
        .expect("expected llm selection review");

        assert_eq!(review.status, SummarySelectionReviewStatus::Repaired);
        assert_eq!(review.final_collection_id, "recent_posts:did:plc:test");
        assert_eq!(review.final_collection_kind, "recent_posts");
    }

    #[test]
    fn parse_llm_summary_collection_selection_review_rejects_unknown_collection_id() {
        let collections = vec![
            LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", Vec::new())
                .with_collection_kind("recent_posts")
                .with_actor_did("did:plc:test"),
        ];

        let err = parse_llm_summary_collection_selection_review(
            &collections,
            &collections[0],
            r#"{"status":"repaired","final_collection_id":"actor_profile:did:plc:test","reason":"profile is better"}"#,
        )
        .expect_err("expected unknown collection id failure");

        assert!(
            err.to_string()
                .contains("summary selection review chose unknown collection")
        );
    }

    #[test]
    fn summary_hydration_args_include_clearsky_lists_for_lists_hint() {
        let args = summary_hydration_args_for_hint(
            &SummaryCollectionTargetHint::Lists,
            RequestedSummaryScope::Count {
                requested_items: 50,
            },
        );

        assert_eq!(args["include_clearsky_lists"], serde_json::json!(true));
        assert_eq!(args["include_profile"], serde_json::json!(true));
        assert_eq!(args["include_recent_posts"], serde_json::json!(true));
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
