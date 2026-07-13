use crate::harness::context_window::{
    BuiltContextWindow, ContextSectionKind, LLMContext, build_context_window_report,
};
use crate::harness::context_window_logger::append_debug_trace;
use crate::harness::llm_api::ChatMessage;
use crate::harness::llm_api::LlmApiClient;
use crate::harness::r#loop::{
    LoopDefinition, LoopExecutor, LoopKind, LoopNodeDefinition, LoopNodeKind, LoopPort,
    LoopPortTarget,
};
use crate::harness::tools::{
    BlueskyTools, COLLECTION_SEARCH_PAGE_SIZE, CollectionLeafResult, CollectionLeafToolKind,
    CollectionRawWindowResult, CollectionReviewStatus, CollectionReviewVerdict,
    CollectionToolOutcome, LlmSearchExecution, LlmSummaryResult, RequestedSummaryScope,
    ToolProgressEvent, apply_summary_sufficiency_gates, paged_search_collection,
    render_collection_outcome_result, summary_scope_initial_offset, summary_scope_max_pages,
};
use crate::model::{LabeledPostCollection, PostRecord};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;

const COLLECTION_SUMMARY_PLANNER_PROMPT: &str =
    include_str!("../prompts/agents/collection_summary_planner.md");
const COLLECTION_SUMMARY_NOTES_PROMPT: &str =
    include_str!("../prompts/agents/collection_summary_notes.md");

fn append_summary_trace(entry: impl AsRef<str>) {
    let debug_base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = append_debug_trace(&debug_base_dir, "summary_trace.md", entry.as_ref());
}

const COLLECTION_SUMMARY_NODES: &[LoopNodeDefinition] = &[
    LoopNodeDefinition {
        id: "init_window",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Harness,
        handler_key: "initialize_summary_window",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("summarize_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "summarize_page",
        kind: LoopNodeKind::Tool,
        executor: LoopExecutor::Llm,
        handler_key: "run_summary_page_compaction",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("review_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Node("repair_summary_output"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "repair_summary_output",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "repair_summary_output",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("review_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "review_page",
        kind: LoopNodeKind::Review,
        executor: LoopExecutor::Harness,
        handler_key: "review_summary_page",
        ports: &[
            LoopPort {
                name: "complete",
                target: LoopPortTarget::Node("collection_summary_planner"),
            },
            LoopPort {
                name: "continue",
                target: LoopPortTarget::Node("collection_summary_planner"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "collection_summary_planner",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Harness,
        handler_key: "collection_summary_planner",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("collection_summary_planner_review"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Node("collection_summary_planner_repair"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "collection_summary_planner_review",
        kind: LoopNodeKind::Review,
        executor: LoopExecutor::Harness,
        handler_key: "collection_summary_planner_review",
        ports: &[
            LoopPort {
                name: "continue",
                target: LoopPortTarget::Node("advance_cursor"),
            },
            LoopPort {
                name: "complete",
                target: LoopPortTarget::Node("collection_summary_notes"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Node("collection_summary_planner_repair"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "collection_summary_planner_repair",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "collection_summary_planner_repair",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("collection_summary_planner_review"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "advance_cursor",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Harness,
        handler_key: "advance_summary_cursor",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("summarize_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "collection_summary_notes",
        kind: LoopNodeKind::Synthesize,
        executor: LoopExecutor::Harness,
        handler_key: "collection_summary_notes",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("collection_summary_notes_review"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Node("collection_summary_notes_repair"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "collection_summary_notes_review",
        kind: LoopNodeKind::Review,
        executor: LoopExecutor::Harness,
        handler_key: "collection_summary_notes_review",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("return_summary"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Node("collection_summary_notes_repair"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "collection_summary_notes_repair",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "collection_summary_notes_repair",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("collection_summary_notes_review"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "return_summary",
        kind: LoopNodeKind::Return,
        executor: LoopExecutor::Harness,
        handler_key: "return_collection_summary",
        ports: &[],
    },
];

pub const DEFINITION: LoopDefinition = LoopDefinition {
    kind: LoopKind::CollectionSummary,
    entry_node: "init_window",
    nodes: COLLECTION_SUMMARY_NODES,
};

const NODE_INIT_WINDOW: &str = "init_window";
const NODE_SUMMARIZE_PAGE: &str = "summarize_page";
const NODE_REVIEW_PAGE: &str = "review_page";
const NODE_COLLECTION_SUMMARY_PLANNER: &str = "collection_summary_planner";
const NODE_COLLECTION_SUMMARY_PLANNER_REVIEW: &str = "collection_summary_planner_review";
const NODE_COLLECTION_SUMMARY_PLANNER_REPAIR: &str = "collection_summary_planner_repair";
const NODE_ADVANCE_CURSOR: &str = "advance_cursor";
const NODE_COLLECTION_SUMMARY_NOTES: &str = "collection_summary_notes";
const NODE_COLLECTION_SUMMARY_NOTES_REVIEW: &str = "collection_summary_notes_review";
const NODE_COLLECTION_SUMMARY_NOTES_REPAIR: &str = "collection_summary_notes_repair";
const NODE_RETURN_SUMMARY: &str = "return_summary";
const PORT_SUCCESS: &str = "success";
const PORT_CONTINUE: &str = "continue";
const PORT_COMPLETE: &str = "complete";
const PORT_FAILURE: &str = "failure";

#[derive(Default)]
struct SummaryLoopAccumulator {
    page_outcomes: Vec<CollectionToolOutcome>,
    page_states: Vec<SummaryPageState>,
    accepted_page_summaries: Vec<String>,
    planner_updates: Vec<PlannerUpdate>,
    covered_window_offsets: Vec<usize>,
    covered_post_count: usize,
    collection_total_items: Option<usize>,
    source_exhausted: bool,
    planner_context_window: Option<BuiltContextWindow>,
    notes_context_window: Option<BuiltContextWindow>,
    planner_raw_response: Option<String>,
    notes_raw_response: Option<String>,
    pending_planner_update: Option<String>,
    pending_notes_summary: Option<String>,
    pending_planner_context_window: Option<BuiltContextWindow>,
    pending_notes_context_window: Option<BuiltContextWindow>,
    pending_planner_raw_response: Option<String>,
    pending_notes_raw_response: Option<String>,
    pending_planner_failure_reason: Option<String>,
    pending_notes_failure_reason: Option<String>,
    planner_repair_attempted: bool,
    notes_repair_attempted: bool,
    final_notes_summary: Option<String>,
    accepted_windows: Vec<(usize, usize)>,
}

#[derive(Clone)]
struct PlannerUpdate {
    text: String,
    covered_post_count: usize,
}

#[derive(Clone)]
enum SummaryPagePayload {
    Summary { text: String },
    RawWindow { fallback: CollectionRawWindowResult },
}

#[derive(Clone)]
struct SummaryPageState {
    offset: usize,
    page_index: usize,
    window_size: usize,
    next_offset: Option<usize>,
    source_exhausted: bool,
    repair_attempted: bool,
    fallback_applied: bool,
    failure_reason: Option<String>,
    payload: SummaryPagePayload,
}

impl SummaryLoopAccumulator {
    fn record_page_outcome(&mut self, outcome: CollectionToolOutcome) {
        self.page_outcomes.push(outcome);
    }

    fn record_page_state(&mut self, page_state: SummaryPageState, collection_total_items: usize) {
        self.covered_window_offsets.push(page_state.offset);
        self.covered_post_count += page_state.window_size;
        self.collection_total_items = Some(collection_total_items);
        self.source_exhausted = page_state.source_exhausted;
        self.accepted_windows
            .push((page_state.offset, page_state.window_size));
        if let SummaryPagePayload::Summary { text } = &page_state.payload {
            self.accepted_page_summaries.push(text.trim().to_string());
        }
        self.page_states.push(page_state);
    }

    fn concatenated_window_summaries(&self) -> String {
        self.accepted_page_summaries.join("\n\n")
    }

    fn coherent_page_count(&self) -> usize {
        self.page_states.len()
    }

    fn fallback_page_count(&self) -> usize {
        self.page_states
            .iter()
            .filter(|page| page.fallback_applied)
            .count()
    }

    fn planner_notes(&self) -> String {
        self.planner_updates
            .iter()
            .map(|update| update.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn final_summary_text(&self) -> String {
        if let Some(notes) = self
            .final_notes_summary
            .as_deref()
            .map(str::trim)
            .filter(|summary| !summary.is_empty())
        {
            return notes.to_string();
        }

        if let Some(notes) = self
            .planner_updates
            .last()
            .filter(|update| update.covered_post_count >= self.covered_post_count)
            .map(|update| update.text.trim())
            .filter(|summary| !summary.is_empty())
        {
            return notes.to_string();
        }

        let concatenated = self.concatenated_window_summaries();
        if !concatenated.trim().is_empty() && self.accepted_page_summaries.len() == self.page_states.len() {
            concatenated
        } else if self.page_states.is_empty() {
            "No grounded summary pages were accepted.".to_string()
        } else {
            self.deterministic_scope_summary()
        }
    }

    fn deterministic_scope_summary(&self) -> String {
        let mut paragraphs = Vec::new();
        for page_state in &self.page_states {
            match &page_state.payload {
                SummaryPagePayload::Summary { text } => {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        paragraphs.push(trimmed.to_string());
                    }
                }
                SummaryPagePayload::RawWindow { fallback } => {
                    let snippets = representative_body_snippets(&fallback.records, 3);
                    let paragraph = if snippets.is_empty() {
                        format!(
                            "Page {} preserved grounded raw evidence, but the window did not yield concise representative snippets.",
                            page_state.page_index + 1
                        )
                    } else {
                        format!(
                            "Page {} preserved grounded raw evidence around {}.",
                            page_state.page_index + 1,
                            snippets.join("; ")
                        )
                    };
                    paragraphs.push(paragraph);
                }
            }
        }
        if paragraphs.is_empty() {
            format!(
                "Grounded raw-window fallback preserved {} page(s), but no final synthesis was produced.",
                self.page_states.len()
            )
        } else {
            paragraphs.join("\n\n")
        }
    }

    fn into_final_outcome(
        self,
        collection: &LabeledPostCollection,
        requested_summary_scope: RequestedSummaryScope,
    ) -> Option<CollectionToolOutcome> {
        if self.page_states.is_empty() {
            return None;
        }

        let concatenated_window_summaries = self.concatenated_window_summaries();
        let final_summary = self.final_summary_text();
        let notes_summary_accepted = self
            .final_notes_summary
            .as_deref()
            .map(str::trim)
            .is_some_and(|summary| !summary.is_empty());
        let planner_summary_accepted = self
            .planner_updates
            .last()
            .map(|update| update.text.trim())
            .is_some_and(|summary| !summary.is_empty());
        let required_post_count = match requested_summary_scope {
            RequestedSummaryScope::Count { requested_items } => Some(requested_items),
            _ => None,
        };
        let coverage_complete = required_post_count
            .map(|required| self.covered_post_count >= required)
            .unwrap_or(!self.covered_window_offsets.is_empty());
        let source_exhausted = self.source_exhausted
            || self
                .collection_total_items
                .map(|total| self.covered_post_count >= total)
                .unwrap_or(false);
        let diagnostic = format!(
            "collection_summary_planner accepted {} page summaries and {} raw-window fallbacks; final_notes_summary_accepted: {}; planner_summary_accepted: {}",
            self.accepted_page_summaries.len(),
            self.fallback_page_count(),
            notes_summary_accepted,
            planner_summary_accepted
        );
        let review_reason = if notes_summary_accepted {
            if coverage_complete || source_exhausted {
                format!(
                    "collection_summary_notes produced a final scope summary after considering {} posts.",
                    self.covered_post_count
                )
            } else {
                format!(
                    "collection_summary_notes produced a partial scope summary after considering {} posts before exhaustion.",
                    self.covered_post_count
                )
            }
        } else if planner_summary_accepted {
            if coverage_complete || source_exhausted {
                format!(
                    "collection_summary_planner produced the best accepted synthesis after considering {} posts.",
                    self.covered_post_count
                )
            } else {
                format!(
                    "collection_summary_planner produced the best partial accepted synthesis after considering {} posts before exhaustion.",
                    self.covered_post_count
                )
            }
        } else if source_exhausted {
            format!(
                "No final planner or notes synthesis was accepted, but grounded raw-window fallback preserved the full available scope across {} posts.",
                self.covered_post_count
            )
        } else if coverage_complete {
            format!(
                "No final planner or notes synthesis was accepted after considering {} posts.",
                self.covered_post_count
            )
        } else {
            format!(
                "No final planner or notes synthesis was accepted before the requested scope stopped at {} grounded posts.",
                self.covered_post_count
            )
        };
        let rendered_summary = if coverage_complete || source_exhausted {
            final_summary
        } else {
            format!(
                "Partial coverage only. The loop preserved grounded evidence for {} post(s), but the requested scope remains incomplete.\n\n{}",
                self.covered_post_count, final_summary
            )
        };

        Some(CollectionToolOutcome {
            tool_kind: CollectionLeafToolKind::Summary,
            collection_id: collection.id.clone(),
            collection_label: collection.label.clone(),
            execution: Ok(LlmSearchExecution {
                result: Some(CollectionLeafResult::Summary(LlmSummaryResult {
                    title: format!("Summary of {}", collection.label),
                    summary: rendered_summary,
                    covered_item_uris: Vec::new(),
                    omitted_item_uris: Vec::new(),
                    concatenated_window_summaries: Some(concatenated_window_summaries),
                    window_offset: self.covered_window_offsets.first().copied(),
                    window_size: Some(self.covered_post_count),
                    page_index: self
                        .covered_window_offsets
                        .first()
                        .map(|offset| offset / COLLECTION_SEARCH_PAGE_SIZE.max(1)),
                    page_size: Some(COLLECTION_SEARCH_PAGE_SIZE),
                    collection_total_items: self.collection_total_items,
                    has_more: Some(!coverage_complete && !source_exhausted),
                    source_exhausted: Some(source_exhausted),
                    window_start: self.covered_window_offsets.first().copied(),
                    window_total_items: Some(self.covered_post_count),
                })),
                original_result: None,
                context_window: self
                    .notes_context_window
                    .clone()
                    .or_else(|| self.planner_context_window.clone())
                    .or_else(|| {
                        self.page_outcomes
                            .iter()
                            .rev()
                            .find_map(|outcome| outcome.execution.as_ref().ok())
                            .map(|execution| execution.context_window.clone())
                    })?,
                diagnostic: Some(format!(
                    "{diagnostic}\ncovered_window_offsets: {}\ncovered_post_count: {}\nplanner_updates: {}\ncoherent_pages: {}",
                    self.covered_window_offsets
                        .iter()
                        .map(|offset| offset.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.covered_post_count,
                    self.planner_updates.len(),
                    self.coherent_page_count()
                )),
                raw_response: self.notes_raw_response.or(self.planner_raw_response),
                review_verdict: Some(CollectionReviewVerdict {
                    status: if coverage_complete || source_exhausted {
                        CollectionReviewStatus::Pass
                    } else {
                        CollectionReviewStatus::Fail
                    },
                    grounded: true,
                    sufficient: coverage_complete || source_exhausted,
                    reason: review_reason,
                    repair_needed: false,
                    repair_instructions: None,
                    additional_pages_needed: !(coverage_complete || source_exhausted),
                    next_page: None,
                    next_offset: None,
                    required_total_items: required_post_count,
                }),
                review_context_window: self.planner_context_window.clone(),
                repair_diagnostic: None,
            }),
        })
    }
}

fn next_node(current: &'static str, port: &str) -> Option<&'static str> {
    let node = DEFINITION.nodes.iter().find(|node| node.id == current)?;
    let target = node
        .ports
        .iter()
        .find(|candidate| candidate.name == port)?
        .target;
    match target {
        LoopPortTarget::Node(node_id) => Some(node_id),
        LoopPortTarget::Return | LoopPortTarget::Fail => None,
    }
}

fn route_after_planner_attempt(pending_next_offset: Option<usize>) -> &'static str {
    if pending_next_offset.is_some() {
        next_node(NODE_COLLECTION_SUMMARY_PLANNER_REVIEW, PORT_CONTINUE)
            .unwrap_or(NODE_ADVANCE_CURSOR)
    } else {
        next_node(NODE_COLLECTION_SUMMARY_PLANNER_REVIEW, PORT_COMPLETE)
            .unwrap_or(NODE_COLLECTION_SUMMARY_NOTES)
    }
}

fn summarize_progress_excerpt(text: &str, limit: usize) -> String {
    let _ = limit;
    let trimmed = text.trim();
    trimmed.to_string()
}

fn normalize_synthesis_response(text: String) -> String {
    text.replace("\\\"", "\"")
}

fn render_raw_window_records(records: &[PostRecord]) -> String {
    let mut lines = Vec::new();
    for (index, post) in records.iter().enumerate() {
        lines.push(format!("item[{index}]"));
        lines.push(format!("uri: {}", post.uri));
        lines.push(format!("author: {}", post.author_handle));
        lines.push(format!("body: {}", post.body));
        lines.push(String::new());
    }
    lines.join("\n")
}

fn render_page_evidence(page_state: &SummaryPageState) -> String {
    let mut lines = vec![
        format!("window_offset: {}", page_state.offset),
        format!("window_size: {}", page_state.window_size),
        format!("page_index: {}", page_state.page_index),
        format!("repair_attempted: {}", page_state.repair_attempted),
        format!("fallback_applied: {}", page_state.fallback_applied),
        format!(
            "failure_reason: {}",
            page_state.failure_reason.as_deref().unwrap_or("<none>")
        ),
    ];
    if let Some(next_offset) = page_state.next_offset {
        lines.push(format!("next_offset: {next_offset}"));
    }
    match &page_state.payload {
        SummaryPagePayload::Summary { text } => {
            lines.push("page_status: accepted_summary".to_string());
            lines.push(format!("summary: {text}"));
        }
        SummaryPagePayload::RawWindow { fallback } => {
            lines.push("page_status: raw_window_fallback".to_string());
            lines.push(format!("failure_reason: {}", fallback.failure_reason));
            if let Some(raw_summary_response) = fallback.raw_summary_response.as_deref() {
                lines.push(format!("raw_summary_response:\n{raw_summary_response}"));
            }
            lines.push("records:".to_string());
            lines.push(render_raw_window_records(&fallback.records));
        }
    }
    lines.join("\n")
}

fn next_offset_for_scope(
    requested_summary_scope: RequestedSummaryScope,
    collection_total_items: usize,
    offset: usize,
    window_size: usize,
) -> Option<usize> {
    let next_offset = offset.saturating_add(window_size);
    if next_offset >= collection_total_items {
        return None;
    }

    match requested_summary_scope {
        RequestedSummaryScope::CurrentWindow | RequestedSummaryScope::Page { .. } => None,
        RequestedSummaryScope::Count { requested_items } => {
            (next_offset < requested_items).then_some(next_offset)
        }
        RequestedSummaryScope::PageRange {
            start_page: _,
            end_page,
        } => {
            let next_page_index = next_offset / COLLECTION_SEARCH_PAGE_SIZE.max(1);
            (next_page_index <= end_page).then_some(next_offset)
        }
    }
}

fn build_page_state(
    collection_total_items: usize,
    requested_summary_scope: RequestedSummaryScope,
    offset: usize,
    page_index: usize,
    window_records: &[PostRecord],
    fallback_context_window: BuiltContextWindow,
    outcome: &mut CollectionToolOutcome,
) -> Option<SummaryPageState> {
    let window_size = window_records.len();
    let has_more = offset.saturating_add(window_size) < collection_total_items;
    let next_offset = next_offset_for_scope(
        requested_summary_scope,
        collection_total_items,
        offset,
        window_size,
    );

    if let Err(err) = &outcome.execution {
        let failure_reason = format!("summary execution failed: {err}");
        let fallback = CollectionRawWindowResult {
            title: format!("Raw window fallback for page {}", page_index + 1),
            summary: format!(
                "This raw-window fallback preserves the exact page records after summary execution failed at offset {}.",
                offset
            ),
            window_offset: offset,
            window_size,
            page_index,
            page_size: COLLECTION_SEARCH_PAGE_SIZE,
            collection_total_items,
            has_more,
            failure_reason: failure_reason.clone(),
            raw_summary_response: None,
            records: window_records.to_vec(),
        };
        outcome.execution = Ok(LlmSearchExecution {
            result: Some(CollectionLeafResult::RawWindow(fallback.clone())),
            original_result: None,
            context_window: fallback_context_window,
            diagnostic: Some(format!("raw-window fallback preserved offset {offset}")),
            raw_response: None,
            review_verdict: Some(CollectionReviewVerdict {
                status: CollectionReviewStatus::Pass,
                grounded: true,
                sufficient: next_offset.is_none(),
                reason: format!(
                    "Raw-window fallback preserved grounded evidence for page {} after summary execution failed: {}",
                    page_index + 1,
                    failure_reason
                ),
                repair_needed: false,
                repair_instructions: None,
                additional_pages_needed: next_offset.is_some(),
                next_page: next_offset.map(|value| value / COLLECTION_SEARCH_PAGE_SIZE.max(1)),
                next_offset,
                required_total_items: match requested_summary_scope {
                    RequestedSummaryScope::Count { requested_items } => Some(requested_items),
                    _ => None,
                },
            }),
            review_context_window: None,
            repair_diagnostic: None,
        });

        return Some(SummaryPageState {
            offset,
            page_index,
            window_size,
            next_offset,
            source_exhausted: !has_more,
            repair_attempted: false,
            fallback_applied: true,
            failure_reason: Some(failure_reason),
            payload: SummaryPagePayload::RawWindow { fallback },
        });
    }

    let execution = outcome.execution.as_mut().ok()?;
    if execution
        .review_verdict
        .as_ref()
        .map(|verdict| verdict.grounded)
        .unwrap_or(false)
    {
        if let Some(result) = execution
            .result
            .as_ref()
            .or(execution.original_result.as_ref())
            .and_then(CollectionLeafResult::as_summary)
        {
            return Some(SummaryPageState {
                offset,
                page_index,
                window_size: result
                    .processed_window_size()
                    .unwrap_or(window_records.len()),
                next_offset: execution
                    .review_verdict
                    .as_ref()
                    .and_then(|verdict| verdict.next_offset)
                    .or_else(|| {
                        next_offset_for_scope(
                            requested_summary_scope,
                            collection_total_items,
                            offset,
                            result
                                .processed_window_size()
                                .unwrap_or(window_records.len()),
                        )
                    }),
                source_exhausted: result.has_more == Some(false),
                repair_attempted: execution.repair_diagnostic.is_some(),
                fallback_applied: false,
                failure_reason: None,
                payload: SummaryPagePayload::Summary {
                    text: result.summary.trim().to_string(),
                },
            });
        }
    }

    let failure_reason = execution
        .review_verdict
        .as_ref()
        .map(|verdict| verdict.reason.clone())
        .unwrap_or_else(|| "summary review did not return a usable verdict".to_string());
    let raw_summary_response = execution.raw_response.clone();
    let fallback = CollectionRawWindowResult {
        title: format!("Raw window fallback for page {}", page_index + 1),
        summary: format!(
            "This raw-window fallback preserves the exact page records after summary processing failed at offset {}.",
            offset
        ),
        window_offset: offset,
        window_size,
        page_index,
        page_size: COLLECTION_SEARCH_PAGE_SIZE,
        collection_total_items,
        has_more,
        failure_reason: failure_reason.clone(),
        raw_summary_response,
        records: window_records.to_vec(),
    };
    execution.result = Some(CollectionLeafResult::RawWindow(fallback.clone()));
    execution.review_verdict = Some(CollectionReviewVerdict {
        status: CollectionReviewStatus::Pass,
        grounded: true,
        sufficient: next_offset.is_none(),
        reason: format!(
            "Raw-window fallback preserved grounded evidence for page {} after summary failure: {}",
            page_index + 1,
            failure_reason
        ),
        repair_needed: false,
        repair_instructions: None,
        additional_pages_needed: next_offset.is_some(),
        next_page: next_offset.map(|value| value / COLLECTION_SEARCH_PAGE_SIZE.max(1)),
        next_offset,
        required_total_items: match requested_summary_scope {
            RequestedSummaryScope::Count { requested_items } => Some(requested_items),
            _ => None,
        },
    });
    execution.diagnostic = Some(match execution.diagnostic.take() {
        Some(diagnostic) => format!("{diagnostic}; raw-window fallback preserved offset {offset}"),
        None => format!("raw-window fallback preserved offset {offset}"),
    });

    Some(SummaryPageState {
        offset,
        page_index,
        window_size,
        next_offset,
        source_exhausted: !has_more,
        repair_attempted: execution.repair_diagnostic.is_some(),
        fallback_applied: true,
        failure_reason: Some(failure_reason),
        payload: SummaryPagePayload::RawWindow { fallback },
    })
}

#[derive(Clone, Copy)]
enum SummarySynthesisKind {
    Planner,
    Notes,
}

fn paragraph_count(text: &str) -> usize {
    text.split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .count()
}

fn contains_metadata_lines(text: &str) -> bool {
    [
        "status:",
        "summary:",
        "collection_id:",
        "collection_label:",
        "covered_window_offsets:",
        "covered_post_count:",
        "window_offset:",
        "window_size:",
        "page_index:",
        "page_size:",
        "collection_total_items:",
        "source_exhausted:",
        "TOOL_CALL",
        "args:",
        "name:",
    ]
    .iter()
    .any(|prefix| {
        text.lines()
            .map(str::trim_start)
            .any(|line| line.starts_with(prefix))
    })
}

fn ends_with_suspicious_truncation(text: &str) -> bool {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.ends_with(':') || trimmed.ends_with(',') || trimmed.ends_with('(') {
        return true;
    }
    trimmed.matches('"').count() % 2 == 1
}

fn extract_quoted_snippets(text: &str) -> Vec<String> {
    let mut snippets = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for ch in text.chars() {
        if ch == '"' {
            if in_quote {
                let snippet = current.trim();
                if !snippet.is_empty() {
                    snippets.push(snippet.to_string());
                }
                current.clear();
            }
            in_quote = !in_quote;
            continue;
        }

        if in_quote {
            current.push(ch);
        }
    }

    snippets
}

fn covered_post_bodies<'a>(
    collection: &'a LabeledPostCollection,
    accepted_windows: &[(usize, usize)],
) -> Vec<&'a str> {
    let mut bodies = Vec::new();
    for (offset, window_size) in accepted_windows {
        let end = offset
            .saturating_add(*window_size)
            .min(collection.posts.len());
        for post in collection
            .posts
            .iter()
            .skip(*offset)
            .take(end.saturating_sub(*offset))
        {
            bodies.push(post.body.as_str());
        }
    }
    bodies
}

fn representative_body_snippets(records: &[PostRecord], limit: usize) -> Vec<String> {
    let mut snippets = Vec::new();
    for post in records {
        let snippet = post
            .body
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("");
        if snippet.is_empty() {
            continue;
        }
        let condensed = if snippet.chars().count() > 140 {
            let truncated: String = snippet.chars().take(140).collect();
            format!("{truncated}...")
        } else {
            snippet.to_string()
        };
        if snippets.iter().any(|existing| existing == &condensed) {
            continue;
        }
        snippets.push(format!("{:?}", condensed));
        if snippets.len() >= limit {
            break;
        }
    }
    snippets
}

fn review_synthesis_text(
    kind: SummarySynthesisKind,
    text: &str,
    collection: &LabeledPostCollection,
    accumulator: &SummaryLoopAccumulator,
) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("response was empty".to_string());
    }
    if trimmed.contains("```") {
        return Err("response contained markdown fences".to_string());
    }
    if contains_metadata_lines(trimmed) {
        return Err("response included harness metadata instead of plain prose".to_string());
    }
    if ends_with_suspicious_truncation(trimmed) {
        return Err("response appears truncated".to_string());
    }

    let paragraphs = paragraph_count(trimmed);
    match kind {
        SummarySynthesisKind::Planner => {
            if paragraphs != 1 {
                return Err(format!(
                    "planner synthesis must be exactly one paragraph, got {paragraphs}"
                ));
            }
        }
        SummarySynthesisKind::Notes => {
            if !(1..=3).contains(&paragraphs) {
                return Err(format!(
                    "final notes synthesis must be 1-3 paragraphs, got {paragraphs}"
                ));
            }

            let covered_bodies = covered_post_bodies(collection, &accumulator.accepted_windows);
            for snippet in extract_quoted_snippets(trimmed) {
                if !covered_bodies.iter().any(|body| body.contains(&snippet)) {
                    return Err(format!(
                        "quoted snippet {:?} was not found in the covered posts",
                        snippet
                    ));
                }
            }
        }
    }

    Ok(())
}

fn build_repair_context(
    base_prompt: &str,
    prior_context: &BuiltContextWindow,
    prior_response: &str,
    failure_reason: &str,
) -> LLMContext {
    let mut context = LLMContext::new(base_prompt);
    context.push_section_with_kind(
        "Original Context",
        ContextSectionKind::ParentSearchResults,
        prior_context.rendered.clone(),
    );
    context.push_section("Invalid Prior Response", prior_response);
    context.push_section(
        "Repair Instructions",
        format!(
            "Rewrite the response as valid plain prose only. Fix this specific problem: {failure_reason}"
        ),
    );
    context
}

fn build_collection_summary_planner_context(
    collection: &LabeledPostCollection,
    prompt: &str,
    requested_summary_scope: RequestedSummaryScope,
    accumulator: &SummaryLoopAccumulator,
) -> LLMContext {
    let mut context = LLMContext::new(COLLECTION_SUMMARY_PLANNER_PROMPT);
    context.push_section("Task", prompt);
    context.push_section("Collection", render_collection_summary_stub(collection));
    context.push_section(
        "Requested Scope",
        requested_summary_scope.render_for_planner(),
    );
    context.push_section(
        "Coverage State",
        format!(
            "covered_window_offsets: {}\ncovered_post_count: {}\ncollection_total_items: {}\nsource_exhausted: {}",
            accumulator
                .covered_window_offsets
                .iter()
                .map(|offset| offset.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            accumulator.covered_post_count,
            accumulator.collection_total_items.unwrap_or(collection.posts.len()),
            accumulator.source_exhausted
        ),
    );
    context.push_section_with_kind(
        "Accepted Window Summaries",
        ContextSectionKind::CollectionEvidence,
        accumulator.concatenated_window_summaries(),
    );
    let fallback_pages = accumulator
        .page_states
        .iter()
        .filter(|page| page.fallback_applied)
        .map(render_page_evidence)
        .collect::<Vec<_>>()
        .join("\n\n");
    if !fallback_pages.trim().is_empty() {
        context.push_section_with_kind(
            "Raw Window Fallbacks",
            ContextSectionKind::CollectionEvidence,
            fallback_pages,
        );
    }
    context
}

fn build_collection_summary_notes_context(
    collection: &LabeledPostCollection,
    prompt: &str,
    requested_summary_scope: RequestedSummaryScope,
    accumulator: &SummaryLoopAccumulator,
) -> LLMContext {
    let mut context = LLMContext::new(COLLECTION_SUMMARY_NOTES_PROMPT);
    context.push_section("Task", prompt);
    context.push_section("Collection", render_collection_summary_stub(collection));
    context.push_section(
        "Requested Scope",
        requested_summary_scope.render_for_planner(),
    );
    context.push_section(
        "Coverage State",
        format!(
            "covered_window_offsets: {}\ncovered_post_count: {}\ncollection_total_items: {}\nsource_exhausted: {}",
            accumulator
                .covered_window_offsets
                .iter()
                .map(|offset| offset.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            accumulator.covered_post_count,
            accumulator.collection_total_items.unwrap_or(collection.posts.len()),
            accumulator.source_exhausted
        ),
    );
    context.push_section_with_kind(
        "Accepted Window Summaries",
        ContextSectionKind::CollectionEvidence,
        accumulator.concatenated_window_summaries(),
    );
    let fallback_pages = accumulator
        .page_states
        .iter()
        .filter(|page| page.fallback_applied)
        .map(render_page_evidence)
        .collect::<Vec<_>>()
        .join("\n\n");
    if !fallback_pages.trim().is_empty() {
        context.push_section_with_kind(
            "Raw Window Fallbacks",
            ContextSectionKind::CollectionEvidence,
            fallback_pages,
        );
    }
    if !accumulator.planner_updates.is_empty() {
        context.push_section_with_kind(
            "Planner Notes",
            ContextSectionKind::ParentSearchResults,
            accumulator.planner_notes(),
        );
    }
    context
}

fn render_collection_summary_stub(collection: &LabeledPostCollection) -> String {
    let mut lines = vec![
        format!("collection_id: {}", collection.id),
        format!("collection_label: {}", collection.label),
        format!("item_count: {}", collection.posts.len()),
    ];
    if let Some(actor_did) = collection.actor_did.as_deref() {
        lines.push(format!("actor_did: {actor_did}"));
    }
    lines.join("\n")
}

pub(crate) fn render_summary_collection_loop_result(outcomes: &[CollectionToolOutcome]) -> String {
    if outcomes.is_empty() {
        return "status: failed\nreason: no summary pages were processed".to_string();
    }

    if let Some(outcome) = outcomes.last() {
        return match outcome.execution.as_ref() {
            Ok(execution) => render_collection_outcome_result(
                outcome.tool_kind,
                &outcome.collection_id,
                &outcome.collection_label,
                execution,
            ),
            Err(err) => format!(
                "tool_name: {}\ncollection_id: {}\ncollection_label: {}\nstatus: failed\nerror: {}",
                outcome.tool_kind.tool_name(),
                outcome.collection_id,
                outcome.collection_label,
                err
            ),
        };
    }

    "status: failed\nreason: no summary pages were processed".to_string()
}

impl BlueskyTools {
    pub(crate) async fn run_collection_summary_loop(
        &self,
        collection: &LabeledPostCollection,
        prompt: &str,
        requested_summary_scope: RequestedSummaryScope,
        llm_client: &LlmApiClient,
        observer: Option<UnboundedSender<ToolProgressEvent>>,
    ) -> Vec<CollectionToolOutcome> {
        let mut accumulator = SummaryLoopAccumulator::default();
        let max_pages = summary_scope_max_pages(requested_summary_scope, collection.posts.len());
        let mut pages_processed = 0usize;
        let mut seen_offsets = HashSet::new();
        let mut offset = 0usize;
        let mut current_node = DEFINITION.entry_node;
        let mut pending_next_offset = None;

        loop {
            match current_node {
                NODE_INIT_WINDOW => {
                    offset = summary_scope_initial_offset(requested_summary_scope);
                    append_summary_trace(format!(
                        "[collection_summary_loop]\nnode: init_window\ncollection_id: {}\ncollection_posts: {}\ninitial_offset: {}\nmax_pages: {}\nrequested_scope: {:?}",
                        collection.id,
                        collection.posts.len(),
                        offset,
                        max_pages,
                        requested_summary_scope
                    ));
                    if let Some(observer) = observer.as_ref() {
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: "collection_summary_trace".to_string(),
                            depth: 2,
                            content: format!(
                                "node: init_window\nstatus: completed\ncollection_id: {}\ncollection_posts: {}\ninitial_offset: {}\nmax_pages: {}\nrequested_scope: {:?}",
                                collection.id,
                                collection.posts.len(),
                                offset,
                                max_pages,
                                requested_summary_scope
                            ),
                        });
                    }
                    let Some(next) = next_node(NODE_INIT_WINDOW, PORT_SUCCESS) else {
                        break;
                    };
                    current_node = next;
                }
                NODE_SUMMARIZE_PAGE => {
                    if pages_processed >= max_pages {
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: summarize_page\nstatus: break\nreason: reached_max_pages\npages_processed: {}\nmax_pages: {}\ncurrent_offset: {}",
                            pages_processed, max_pages, offset
                        ));
                        if let Some(observer) = observer.as_ref() {
                            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                label: "collection_summary_trace".to_string(),
                                depth: 2,
                                content: format!(
                                    "node: summarize_page\nstatus: break\nreason: reached_max_pages\npages_processed: {}\nmax_pages: {}\ncurrent_offset: {}",
                                    pages_processed,
                                    max_pages,
                                    offset
                                ),
                            });
                        }
                        break;
                    }
                    if offset >= collection.posts.len() {
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: summarize_page\nstatus: break\nreason: offset_out_of_range\ncurrent_offset: {}\ncollection_posts: {}\npages_processed: {}\nmax_pages: {}",
                            offset,
                            collection.posts.len(),
                            pages_processed,
                            max_pages
                        ));
                        if let Some(observer) = observer.as_ref() {
                            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                label: "collection_summary_trace".to_string(),
                                depth: 2,
                                content: format!(
                                    "node: summarize_page\nstatus: break\nreason: offset_out_of_range\ncurrent_offset: {}\ncollection_posts: {}\npages_processed: {}\nmax_pages: {}",
                                    offset,
                                    collection.posts.len(),
                                    pages_processed,
                                    max_pages
                                ),
                            });
                        }
                        break;
                    }
                    if !seen_offsets.insert(offset) {
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: summarize_page\nstatus: break\nreason: repeated_offset\ncurrent_offset: {}\npages_processed: {}",
                            offset, pages_processed
                        ));
                        if let Some(observer) = observer.as_ref() {
                            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                label: "collection_summary_trace".to_string(),
                                depth: 2,
                                content: format!(
                                    "node: summarize_page\nstatus: break\nreason: repeated_offset\ncurrent_offset: {}\npages_processed: {}",
                                    offset,
                                    pages_processed
                                ),
                            });
                        }
                        break;
                    }
                    append_summary_trace(format!(
                        "[collection_summary_loop]\nnode: summarize_page\nstatus: running\ncollection_id: {}\npage_index: {}\noffset: {}\nwindow_size: {}",
                        collection.id, pages_processed, offset, COLLECTION_SEARCH_PAGE_SIZE
                    ));
                    if let Some(observer) = observer.as_ref() {
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: "collection_summary_trace".to_string(),
                            depth: 2,
                            content: format!(
                                "node: summarize_page\nstatus: running\ncollection_id: {}\npage_index: {}\noffset: {}\nwindow_size: {}",
                                collection.id,
                                pages_processed,
                                offset,
                                COLLECTION_SEARCH_PAGE_SIZE
                            ),
                        });
                    }

                    let paged =
                        paged_search_collection(collection, offset, COLLECTION_SEARCH_PAGE_SIZE);
                    let paged_collection_id = paged.id.clone();
                    let paged_post_count = paged.posts.len();
                    let mut fallback_context = LLMContext::new("Raw window fallback context");
                    fallback_context.push_section("Task", prompt);
                    fallback_context
                        .push_section("Collection", render_collection_summary_stub(&paged));
                    let fallback_context_window = build_context_window_report(
                        &fallback_context,
                        &llm_client.context_limits(),
                    );
                    let mut page_outcomes = self
                        .run_collection_tools(
                            CollectionLeafToolKind::Summary,
                            &[paged.clone()],
                            prompt,
                            requested_summary_scope,
                            llm_client,
                            observer.clone(),
                        )
                        .await;
                    let Some(mut outcome) = page_outcomes.pop() else {
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: summarize_page\nstatus: break\nreason: missing_page_outcome\ncollection_id: {}\noffset: {}\nwindow_post_count: {}",
                            paged_collection_id, offset, paged_post_count
                        ));
                        if let Some(observer) = observer.as_ref() {
                            let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                label: "collection_summary_trace".to_string(),
                                depth: 2,
                                content: format!(
                                    "node: summarize_page\nstatus: break\nreason: missing_page_outcome\ncollection_id: {}\noffset: {}\nwindow_post_count: {}",
                                    paged_collection_id,
                                    offset,
                                    paged_post_count
                                ),
                            });
                        }
                        break;
                    };
                    if let Ok(execution) = outcome.execution.as_mut() {
                        apply_summary_sufficiency_gates(
                            requested_summary_scope,
                            &outcome.collection_id,
                            &accumulator.page_outcomes,
                            execution,
                        );
                        if execution.diagnostic.is_none() {
                            execution.diagnostic = Some(format!(
                                "summary cursor processed offset {offset} (page {} of at most {max_pages})",
                                pages_processed + 1
                            ));
                        }
                    }
                    let page_state = build_page_state(
                        collection.posts.len(),
                        requested_summary_scope,
                        offset,
                        pages_processed,
                        &paged.posts,
                        fallback_context_window,
                        &mut outcome,
                    );
                    append_summary_trace(match outcome.execution.as_ref() {
                        Ok(execution) => format!(
                            "[collection_summary_loop]\nnode: summarize_page\nstatus: page_outcome\ncollection_id: {}\noffset: {}\nresult_present: {}\nreview_status: {}\nreview_reason: {}\ndiagnostic: {}",
                            outcome.collection_id,
                            offset,
                            execution.result.is_some(),
                            execution
                                .review_verdict
                                .as_ref()
                                .map(|verdict| match verdict.status {
                                    CollectionReviewStatus::Pass => "pass",
                                    CollectionReviewStatus::Fail => "fail",
                                })
                                .unwrap_or("<none>"),
                            execution
                                .review_verdict
                                .as_ref()
                                .map(|verdict| verdict.reason.as_str())
                                .unwrap_or("<none>"),
                            execution.diagnostic.as_deref().unwrap_or("<none>")
                        ),
                        Err(err) => format!(
                            "[collection_summary_loop]\nnode: summarize_page\nstatus: page_outcome_failed\ncollection_id: {}\noffset: {}\nerror: {}",
                            outcome.collection_id, offset, err
                        ),
                    });
                    if let Some(observer) = observer.as_ref() {
                        let execution_summary = match outcome.execution.as_ref() {
                            Ok(execution) => format!(
                                "status: page_outcome\nusable: {}\nreview: {}\ndiagnostic: {}",
                                execution.result.is_some()
                                    && !matches!(
                                        execution
                                            .review_verdict
                                            .as_ref()
                                            .map(|verdict| &verdict.status),
                                        Some(CollectionReviewStatus::Fail)
                                    ),
                                execution
                                    .review_verdict
                                    .as_ref()
                                    .map(|verdict| verdict.reason.as_str())
                                    .unwrap_or("<none>"),
                                execution.diagnostic.as_deref().unwrap_or("<none>")
                            ),
                            Err(err) => format!("status: page_outcome_failed\nerror: {err}"),
                        };
                        let _ = observer.send(ToolProgressEvent::AgentUpdate {
                            label: "collection_summary_trace".to_string(),
                            depth: 2,
                            content: format!(
                                "node: summarize_page\ncollection_id: {}\noffset: {}\n{}",
                                outcome.collection_id, offset, execution_summary
                            ),
                        });
                    }
                    if let Some(page_state) = page_state {
                        append_summary_trace(format!(
                            "[collection_summary_loop_page_state]\ncollection_id: {}\npage_offset: {}\npage_index: {}\npage_status: {}\nrepair_attempted: {}\nfallback_applied: {}\nnext_offset: {}\naccumulated_covered_count: {}\nfailure_reason: {}",
                            outcome.collection_id,
                            page_state.offset,
                            page_state.page_index,
                            if page_state.fallback_applied {
                                "raw_window_fallback"
                            } else {
                                "accepted_summary"
                            },
                            page_state.repair_attempted,
                            page_state.fallback_applied,
                            page_state
                                .next_offset
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "<none>".to_string()),
                            accumulator
                                .covered_post_count
                                .saturating_add(page_state.window_size),
                            page_state.failure_reason.as_deref().unwrap_or("<none>")
                        ));
                        if page_state.fallback_applied {
                            append_summary_trace(format!(
                                "[summary_leaf_raw_window_fallback]\ncollection_id: {}\npage_offset: {}\npage_index: {}\nwindow_size: {}\nfailure_reason: {}",
                                outcome.collection_id,
                                page_state.offset,
                                page_state.page_index,
                                page_state.window_size,
                                page_state.failure_reason.as_deref().unwrap_or("<none>")
                            ));
                        }
                        pending_next_offset = page_state.next_offset;
                        accumulator.record_page_state(page_state, collection.posts.len());
                    } else {
                        pending_next_offset = None;
                    }
                    accumulator.record_page_outcome(outcome);
                    pages_processed += 1;

                    current_node = if pending_next_offset.is_some() {
                        next_node(NODE_REVIEW_PAGE, PORT_CONTINUE)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER)
                    } else {
                        next_node(NODE_REVIEW_PAGE, PORT_COMPLETE)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER)
                    };
                }
                NODE_COLLECTION_SUMMARY_PLANNER => {
                    let Some(_page_state) = accumulator.page_states.last() else {
                        current_node = next_node(NODE_COLLECTION_SUMMARY_PLANNER, PORT_FAILURE)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER_REPAIR);
                        continue;
                    };
                    let planner_context = build_collection_summary_planner_context(
                        collection,
                        prompt,
                        requested_summary_scope,
                        &accumulator,
                    );
                    let planner_window =
                        build_context_window_report(&planner_context, &llm_client.context_limits());
                    let planner_response = llm_client
                        .complete_chat(
                            vec![
                                ChatMessage {
                                    role: "system".to_string(),
                                    content: planner_context.header().to_string(),
                                },
                                ChatMessage {
                                    role: "user".to_string(),
                                    content: planner_window.rendered.clone(),
                                },
                            ],
                            256,
                        )
                        .await
                        .ok()
                        .map(|response| normalize_synthesis_response(response.trim().to_string()))
                        .filter(|response| !response.is_empty());
                    accumulator.pending_planner_update = planner_response.clone();
                    accumulator.pending_planner_context_window = Some(planner_window);
                    accumulator.pending_planner_raw_response = planner_response;
                    accumulator.pending_planner_failure_reason = None;
                    accumulator.planner_repair_attempted = false;
                    current_node = if accumulator.pending_planner_update.is_some() {
                        next_node(NODE_COLLECTION_SUMMARY_PLANNER, PORT_SUCCESS)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER_REVIEW)
                    } else {
                        next_node(NODE_COLLECTION_SUMMARY_PLANNER, PORT_FAILURE)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER_REPAIR)
                    };
                }
                NODE_COLLECTION_SUMMARY_PLANNER_REVIEW => {
                    let review_result = accumulator
                        .pending_planner_update
                        .as_deref()
                        .ok_or_else(|| "planner response missing".to_string())
                        .and_then(|response| {
                            review_synthesis_text(
                                SummarySynthesisKind::Planner,
                                response,
                                collection,
                                &accumulator,
                            )
                        });
                    match review_result {
                        Ok(()) => {
                            if let Some(response) = accumulator.pending_planner_update.take() {
                                accumulator.planner_updates.push(PlannerUpdate {
                                    text: response.clone(),
                                    covered_post_count: accumulator.covered_post_count,
                                });
                                accumulator.planner_raw_response = Some(
                                    accumulator
                                        .pending_planner_raw_response
                                        .take()
                                        .unwrap_or_else(|| response.clone()),
                                );
                                accumulator.pending_planner_failure_reason = None;
                                accumulator.planner_context_window =
                                    accumulator.pending_planner_context_window.take();
                                if let Some(observer) = observer.as_ref() {
                                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                        label: "collection_summary_planner".to_string(),
                                        depth: 3,
                                        content: format!(
                                            "collection_id: {}\nstatus: completed\ncovered_post_count: {}\nsummary:\n{}",
                                            collection.id,
                                            accumulator.covered_post_count,
                                            summarize_progress_excerpt(&response, 2000)
                                        ),
                                    });
                                }
                            }
                            current_node = route_after_planner_attempt(pending_next_offset);
                        }
                        Err(reason) => {
                            if accumulator.planner_repair_attempted {
                                append_summary_trace(format!(
                                    "[collection_summary_loop]\nnode: collection_summary_planner_review\nstatus: planner_optional_failure\nreason: {}\nroute: {}",
                                    reason,
                                    if pending_next_offset.is_some() {
                                        NODE_ADVANCE_CURSOR
                                    } else {
                                        NODE_COLLECTION_SUMMARY_NOTES
                                    }
                                ));
                                accumulator.pending_planner_failure_reason = Some(reason);
                                current_node = route_after_planner_attempt(pending_next_offset);
                            } else {
                                accumulator.pending_planner_failure_reason = Some(reason);
                                current_node =
                                    next_node(NODE_COLLECTION_SUMMARY_PLANNER_REVIEW, PORT_FAILURE)
                                        .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER_REPAIR);
                            }
                        }
                    }
                }
                NODE_COLLECTION_SUMMARY_PLANNER_REPAIR => {
                    if accumulator.planner_repair_attempted {
                        let reason = accumulator
                            .pending_planner_failure_reason
                            .clone()
                            .unwrap_or_else(|| {
                                "planner synthesis failed after repair attempt".to_string()
                            });
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: collection_summary_planner_repair\nstatus: planner_optional_failure\nreason: {}\nroute: {}",
                            reason,
                            if pending_next_offset.is_some() {
                                NODE_ADVANCE_CURSOR
                            } else {
                                NODE_COLLECTION_SUMMARY_NOTES
                            }
                        ));
                        current_node = route_after_planner_attempt(pending_next_offset);
                        continue;
                    }
                    let Some(prior_context) = accumulator.pending_planner_context_window.clone()
                    else {
                        accumulator.pending_planner_failure_reason = Some(
                            "planner repair context missing after validation failure".to_string(),
                        );
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: collection_summary_planner_repair\nstatus: planner_optional_failure\nreason: {}\nroute: {}",
                            accumulator
                                .pending_planner_failure_reason
                                .as_deref()
                                .unwrap_or("<none>"),
                            if pending_next_offset.is_some() {
                                NODE_ADVANCE_CURSOR
                            } else {
                                NODE_COLLECTION_SUMMARY_NOTES
                            }
                        ));
                        current_node = route_after_planner_attempt(pending_next_offset);
                        continue;
                    };
                    let prior_response = accumulator
                        .pending_planner_update
                        .clone()
                        .unwrap_or_default();
                    let failure_reason = accumulator
                        .pending_planner_failure_reason
                        .clone()
                        .unwrap_or_else(|| "planner synthesis failed validation".to_string());
                    let repair_context = build_repair_context(
                        COLLECTION_SUMMARY_PLANNER_PROMPT,
                        &prior_context,
                        &prior_response,
                        &failure_reason,
                    );
                    let repair_window =
                        build_context_window_report(&repair_context, &llm_client.context_limits());
                    let repair_response = llm_client
                        .complete_chat(
                            vec![
                                ChatMessage {
                                    role: "system".to_string(),
                                    content: repair_context.header().to_string(),
                                },
                                ChatMessage {
                                    role: "user".to_string(),
                                    content: repair_window.rendered.clone(),
                                },
                            ],
                            256,
                        )
                        .await
                        .ok()
                        .map(|response| normalize_synthesis_response(response.trim().to_string()))
                        .filter(|response| !response.is_empty());
                    accumulator.pending_planner_update = repair_response.clone();
                    accumulator.pending_planner_context_window = Some(repair_window);
                    accumulator.pending_planner_raw_response = repair_response;
                    accumulator.pending_planner_failure_reason = None;
                    accumulator.planner_repair_attempted = true;
                    current_node = if accumulator.pending_planner_update.is_some() {
                        next_node(NODE_COLLECTION_SUMMARY_PLANNER_REPAIR, PORT_SUCCESS)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_PLANNER_REVIEW)
                    } else {
                        accumulator.pending_planner_failure_reason =
                            Some(failure_reason.clone());
                        append_summary_trace(format!(
                            "[collection_summary_loop]\nnode: collection_summary_planner_repair\nstatus: planner_optional_failure\nreason: {}\nroute: {}",
                            failure_reason,
                            if pending_next_offset.is_some() {
                                NODE_ADVANCE_CURSOR
                            } else {
                                NODE_COLLECTION_SUMMARY_NOTES
                            }
                        ));
                        route_after_planner_attempt(pending_next_offset)
                    };
                }
                NODE_ADVANCE_CURSOR => {
                    if let Some(next_offset) = pending_next_offset.take() {
                        offset = next_offset;
                    }
                    let Some(next) = next_node(NODE_ADVANCE_CURSOR, PORT_SUCCESS) else {
                        break;
                    };
                    current_node = next;
                }
                NODE_COLLECTION_SUMMARY_NOTES => {
                    let notes_context = build_collection_summary_notes_context(
                        collection,
                        prompt,
                        requested_summary_scope,
                        &accumulator,
                    );
                    let notes_window =
                        build_context_window_report(&notes_context, &llm_client.context_limits());
                    let notes_response = llm_client
                        .complete_chat(
                            vec![
                                ChatMessage {
                                    role: "system".to_string(),
                                    content: notes_context.header().to_string(),
                                },
                                ChatMessage {
                                    role: "user".to_string(),
                                    content: notes_window.rendered.clone(),
                                },
                            ],
                            384,
                        )
                        .await
                        .ok()
                        .map(|response| normalize_synthesis_response(response.trim().to_string()))
                        .filter(|response| !response.is_empty());
                    accumulator.pending_notes_summary = notes_response.clone();
                    accumulator.pending_notes_context_window = Some(notes_window);
                    accumulator.pending_notes_raw_response = notes_response;
                    accumulator.pending_notes_failure_reason = None;
                    accumulator.notes_repair_attempted = false;
                    current_node = if accumulator.pending_notes_summary.is_some() {
                        next_node(NODE_COLLECTION_SUMMARY_NOTES, PORT_SUCCESS)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_NOTES_REVIEW)
                    } else {
                        next_node(NODE_COLLECTION_SUMMARY_NOTES, PORT_FAILURE)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_NOTES_REPAIR)
                    };
                }
                NODE_COLLECTION_SUMMARY_NOTES_REVIEW => {
                    let review_result = accumulator
                        .pending_notes_summary
                        .as_deref()
                        .ok_or_else(|| "notes response missing".to_string())
                        .and_then(|response| {
                            review_synthesis_text(
                                SummarySynthesisKind::Notes,
                                response,
                                collection,
                                &accumulator,
                            )
                        });
                    match review_result {
                        Ok(()) => {
                            if let Some(response) = accumulator.pending_notes_summary.take() {
                                accumulator.final_notes_summary = Some(response.clone());
                                accumulator.notes_raw_response = Some(
                                    accumulator
                                        .pending_notes_raw_response
                                        .take()
                                        .unwrap_or_else(|| response.clone()),
                                );
                                accumulator.pending_notes_failure_reason = None;
                                accumulator.notes_context_window =
                                    accumulator.pending_notes_context_window.take();
                                if let Some(observer) = observer.as_ref() {
                                    let _ = observer.send(ToolProgressEvent::AgentUpdate {
                                        label: "collection_summary_notes".to_string(),
                                        depth: 3,
                                        content: format!(
                                            "collection_id: {}\nstatus: completed\nsummary:\n{}",
                                            collection.id,
                                            summarize_progress_excerpt(&response, 2000)
                                        ),
                                    });
                                }
                            }
                            let Some(next) =
                                next_node(NODE_COLLECTION_SUMMARY_NOTES_REVIEW, PORT_SUCCESS)
                            else {
                                break;
                            };
                            current_node = next;
                        }
                        Err(reason) => {
                            if accumulator.notes_repair_attempted {
                                break;
                            }
                            accumulator.pending_notes_failure_reason = Some(reason);
                            current_node =
                                next_node(NODE_COLLECTION_SUMMARY_NOTES_REVIEW, PORT_FAILURE)
                                    .unwrap_or(NODE_COLLECTION_SUMMARY_NOTES_REPAIR);
                        }
                    }
                }
                NODE_COLLECTION_SUMMARY_NOTES_REPAIR => {
                    if accumulator.notes_repair_attempted {
                        break;
                    }
                    let Some(prior_context) = accumulator.pending_notes_context_window.clone()
                    else {
                        break;
                    };
                    let prior_response = accumulator
                        .pending_notes_summary
                        .clone()
                        .unwrap_or_default();
                    let failure_reason = accumulator
                        .pending_notes_failure_reason
                        .clone()
                        .unwrap_or_else(|| "notes synthesis failed validation".to_string());
                    let repair_context = build_repair_context(
                        COLLECTION_SUMMARY_NOTES_PROMPT,
                        &prior_context,
                        &prior_response,
                        &failure_reason,
                    );
                    let repair_window =
                        build_context_window_report(&repair_context, &llm_client.context_limits());
                    let repair_response = llm_client
                        .complete_chat(
                            vec![
                                ChatMessage {
                                    role: "system".to_string(),
                                    content: repair_context.header().to_string(),
                                },
                                ChatMessage {
                                    role: "user".to_string(),
                                    content: repair_window.rendered.clone(),
                                },
                            ],
                            384,
                        )
                        .await
                        .ok()
                        .map(|response| normalize_synthesis_response(response.trim().to_string()))
                        .filter(|response| !response.is_empty());
                    accumulator.pending_notes_summary = repair_response.clone();
                    accumulator.pending_notes_context_window = Some(repair_window);
                    accumulator.pending_notes_raw_response = repair_response;
                    accumulator.pending_notes_failure_reason = None;
                    accumulator.notes_repair_attempted = true;
                    current_node = if accumulator.pending_notes_summary.is_some() {
                        next_node(NODE_COLLECTION_SUMMARY_NOTES_REPAIR, PORT_SUCCESS)
                            .unwrap_or(NODE_COLLECTION_SUMMARY_NOTES_REVIEW)
                    } else {
                        break;
                    };
                }
                NODE_RETURN_SUMMARY => break,
                _ => break,
            }
        }

        accumulator
            .into_final_outcome(collection, requested_summary_scope)
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::llm_api::{LlmApiClient, OpenAiRestConfig};
    use crate::model::PostRecord;
    use tokio::sync::mpsc::unbounded_channel;

    fn start_mock_llm_client(responses: Vec<String>) -> LlmApiClient {
        LlmApiClient::scripted_for_tests(
            OpenAiRestConfig::llama_cpp("http://scripted.test", "test-model"),
            responses,
        )
    }

    fn test_collection(post_count: usize) -> LabeledPostCollection {
        let posts = (0..post_count)
            .map(|index| PostRecord {
                uri: format!("at://post/{index}"),
                author_handle: "alpha.test".to_string(),
                body: format!(
                    "post {index}: theme {}\nquote: \"snippet {index}\"",
                    if index < 25 { "alpha" } else { "beta" }
                ),
            })
            .collect::<Vec<_>>();
        LabeledPostCollection::new("recent_posts:did:plc:test", "Recent posts", posts)
            .with_collection_kind("recent_posts")
    }

    #[tokio::test]
    async fn run_collection_summary_loop_returns_aggregated_summary_payload() {
        let llm_client = start_mock_llm_client(vec![
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 0\",\n  \"summary\": \"The first window repeatedly returns to \\\"theme alpha\\\" posts, with lines like \\\"post 0: theme alpha\\\" and \\\"quote: \\\"snippet 12\\\"\\\" showing a steady, narrow focus across the opening page.\"\n}".to_string(),
            "status: fail\ngrounded: true\nsufficient: false\nreason: Grounded summary coverage currently reaches 50 item(s), but 100 item(s) are required before parent synthesis is sufficient.\nrepair_needed: false\nadditional_pages_needed: true\nnext_page: 1\nnext_offset: 50\nrequired_total_items: 100".to_string(),
            "Across the covered windows so far, the posts repeatedly circle the \\\"alpha\\\" theme, with terse updates and quoted snippets showing a steady, narrow focus rather than abrupt topic changes.".to_string(),
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 1\",\n  \"summary\": \"The second window shifts toward \\\"theme beta\\\" posts, with lines like \\\"post 50: theme beta\\\" and \\\"quote: \\\"snippet 79\\\"\\\" showing a broader follow-on page with a visible change in emphasis.\"\n}".to_string(),
            "status: pass\ngrounded: true\nsufficient: true\nreason: Grounded summary coverage reaches 100 item(s), satisfying the requested 100 item scope.\nrepair_needed: false\nadditional_pages_needed: false\nrequired_total_items: 100".to_string(),
            "Taken together, the covered windows move from a concentrated \\\"alpha\\\" run into a broader \\\"beta\\\" run, so the collection now shows both continuity and a visible topic shift across the requested scope.".to_string(),
            "The first 100 posts split into two clear phases. The opening half repeatedly returns to \\\"alpha\\\" updates, with short quoted snippets like \\\"snippet 0\\\" and \\\"snippet 12\\\" reinforcing a steady, narrow focus.\n\nThe second half broadens into recurring \\\"beta\\\" updates, with lines like \\\"snippet 50\\\" and \\\"snippet 79\\\" marking a visible shift in emphasis. Across the whole requested scope, the dominant pattern is continuity in tone with a clear change in subject emphasis between the two windows.".to_string(),
        ]);
        let tools = BlueskyTools::new();
        let collection = test_collection(100);
        let (observer, mut receiver) = unbounded_channel();

        let outcomes = tools
            .run_collection_summary_loop(
                &collection,
                "summarize the last 100 posts by alpha.test",
                RequestedSummaryScope::Count {
                    requested_items: 100,
                },
                &llm_client,
                Some(observer),
            )
            .await;

        let mut progress = Vec::new();
        while let Ok(event) = receiver.try_recv() {
            match event {
                ToolProgressEvent::AgentUpdate { label, content, .. } => {
                    progress.push(format!("{label}\n{content}"));
                }
            }
        }

        assert_eq!(
            outcomes.len(),
            1,
            "progress:\n{}",
            progress.join("\n\n---\n\n")
        );
        let outcome = outcomes.first().expect("aggregated outcome");
        let execution = outcome.execution.as_ref().expect("successful execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert_eq!(result.window_offset, Some(0));
        assert_eq!(
            result.window_size,
            Some(100),
            "diagnostic: {:?}\nconcatenated: {:?}\nsummary: {:?}\nprogress:\n{}",
            execution.diagnostic,
            result.concatenated_window_summaries(),
            result.summary,
            progress.join("\n\n---\n\n")
        );
        assert_eq!(result.collection_total_items, Some(100));
        assert_eq!(result.source_exhausted, Some(true));

        let concatenated = result
            .concatenated_window_summaries()
            .expect("concatenated summaries");
        assert!(
            concatenated.contains("The first window repeatedly returns to \"theme alpha\" posts")
        );
        assert!(concatenated.contains("The second window shifts toward \"theme beta\" posts"));

        assert!(
            result
                .summary
                .contains("The first 100 posts split into two clear phases."),
            "final summary: {:?}",
            result.summary
        );
        assert!(
            result.summary.contains("\"alpha\""),
            "final summary: {:?}",
            result.summary
        );
        assert!(
            result.summary.contains("\"beta\""),
            "final summary: {:?}",
            result.summary
        );

        let verdict = execution.review_verdict.as_ref().expect("review verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.grounded);
        assert!(verdict.sufficient);
        assert!(!verdict.additional_pages_needed);
        assert_eq!(verdict.required_total_items, Some(100));

        let diagnostic = execution.diagnostic.as_deref().unwrap_or_default();
        assert!(diagnostic.contains("collection_summary_planner accepted 2 page summaries"));
        assert!(diagnostic.contains("covered_post_count: 100"));
    }

    #[tokio::test]
    async fn run_collection_summary_loop_repairs_invalid_planner_and_notes_synthesis() {
        let llm_client = start_mock_llm_client(vec![
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 0\",\n  \"summary\": \"The first window repeatedly returns to \\\"theme alpha\\\" posts, with lines like \\\"post 0: theme alpha\\\" and \\\"quote: \\\"snippet 12\\\"\\\" showing a steady, narrow focus across the opening page.\"\n}".to_string(),
            "status: fail\ngrounded: true\nsufficient: false\nreason: Grounded summary coverage currently reaches 50 item(s), but 100 item(s) are required before parent synthesis is sufficient.\nrepair_needed: false\nadditional_pages_needed: true\nnext_page: 1\nnext_offset: 50\nrequired_total_items: 100".to_string(),
            "status: completed\nsummary: invalid planner metadata".to_string(),
            "Across the covered windows so far, the posts repeatedly circle the \\\"alpha\\\" theme, with terse updates and quoted snippets showing a steady, narrow focus rather than abrupt topic changes.".to_string(),
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 1\",\n  \"summary\": \"The second window shifts toward \\\"theme beta\\\" posts, with lines like \\\"post 50: theme beta\\\" and \\\"quote: \\\"snippet 79\\\"\\\" showing a broader follow-on page with a visible change in emphasis.\"\n}".to_string(),
            "status: pass\ngrounded: true\nsufficient: true\nreason: Grounded summary coverage reaches 100 item(s), satisfying the requested 100 item scope.\nrepair_needed: false\nadditional_pages_needed: false\nrequired_total_items: 100".to_string(),
            "Taken together, the covered windows move from a concentrated \\\"alpha\\\" run into a broader \\\"beta\\\" run, so the collection now shows both continuity and a visible topic shift across the requested scope.".to_string(),
            "summary: The first 100 posts split into two clear phases, with \\\"snippet 0\\\" showing the early focus and \\\"invented quote\\\"".to_string(),
            "The first 100 posts split into two clear phases. The opening half repeatedly returns to \\\"alpha\\\" updates, with short quoted snippets like \\\"snippet 0\\\" and \\\"snippet 12\\\" reinforcing a steady, narrow focus.\n\nThe second half broadens into recurring \\\"beta\\\" updates, with lines like \\\"snippet 50\\\" and \\\"snippet 79\\\" marking a visible shift in emphasis. Across the whole requested scope, the dominant pattern is continuity in tone with a clear change in subject emphasis between the two windows.".to_string(),
        ]);
        let tools = BlueskyTools::new();
        let collection = test_collection(100);

        let outcomes = tools
            .run_collection_summary_loop(
                &collection,
                "summarize the last 100 posts by alpha.test",
                RequestedSummaryScope::Count {
                    requested_items: 100,
                },
                &llm_client,
                None,
            )
            .await;

        assert_eq!(outcomes.len(), 1);
        let outcome = outcomes.first().expect("aggregated outcome");
        let execution = outcome.execution.as_ref().expect("successful execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert!(
            result
                .summary
                .contains("The first 100 posts split into two clear phases.")
        );
        assert!(result.summary.contains("\"snippet 79\""));
        assert!(!result.summary.contains("invented quote"));
    }

    #[tokio::test]
    async fn run_collection_summary_loop_continues_when_partial_review_omits_grounded_and_next_offset()
     {
        let llm_client = start_mock_llm_client(vec![
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 0\",\n  \"summary\": \"The first window repeatedly returns to \\\"theme alpha\\\" posts, with lines like \\\"post 0: theme alpha\\\" and \\\"quote: \\\"snippet 12\\\"\\\" showing a steady, narrow focus across the opening page.\"\n}".to_string(),
            "status: fail\nsufficient: false\nreason: need more pages\nrepair_needed: false\nadditional_pages_needed: true\nrequired_total_items: 100".to_string(),
            "Across the covered windows so far, the posts repeatedly circle the \\\"alpha\\\" theme, with terse updates and quoted snippets showing a steady, narrow focus rather than abrupt topic changes.".to_string(),
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 1\",\n  \"summary\": \"The second window shifts toward \\\"theme beta\\\" posts, with lines like \\\"post 50: theme beta\\\" and \\\"quote: \\\"snippet 79\\\"\\\" showing a broader follow-on page with a visible change in emphasis.\"\n}".to_string(),
            "status: pass\ngrounded: true\nsufficient: true\nreason: scope complete\nrepair_needed: false\nadditional_pages_needed: false\nrequired_total_items: 100".to_string(),
            "Taken together, the covered windows move from a concentrated \\\"alpha\\\" run into a broader \\\"beta\\\" run, so the collection now shows both continuity and a visible topic shift across the requested scope.".to_string(),
            "The first 100 posts split into two clear phases. The opening half repeatedly returns to \\\"alpha\\\" updates, with short quoted snippets like \\\"snippet 0\\\" and \\\"snippet 12\\\" reinforcing a steady, narrow focus.\n\nThe second half broadens into recurring \\\"beta\\\" updates, with lines like \\\"snippet 50\\\" and \\\"snippet 79\\\" marking a visible shift in emphasis. Across the whole requested scope, the dominant pattern is continuity in tone with a clear change in subject emphasis between the two windows.".to_string(),
        ]);
        let tools = BlueskyTools::new();
        let collection = test_collection(100);

        let outcomes = tools
            .run_collection_summary_loop(
                &collection,
                "summarize the last 100 posts by alpha.test",
                RequestedSummaryScope::Count {
                    requested_items: 100,
                },
                &llm_client,
                None,
            )
            .await;

        assert_eq!(outcomes.len(), 1);
        let outcome = outcomes.first().expect("aggregated outcome");
        let execution = outcome.execution.as_ref().expect("successful execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert_eq!(result.window_size, Some(100));
        assert!(
            result
                .summary
                .contains("The first 100 posts split into two clear phases.")
        );

        let diagnostic = execution.diagnostic.as_deref().unwrap_or_default();
        assert!(diagnostic.contains("collection_summary_planner accepted 2 page summaries"));
    }

    #[tokio::test]
    async fn run_collection_summary_loop_preserves_later_page_as_raw_window_fallback() {
        let llm_client = start_mock_llm_client(vec![
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 0\",\n  \"summary\": \"The first window repeatedly returns to \\\"theme alpha\\\" posts, with lines like \\\"post 0: theme alpha\\\" and \\\"quote: \\\"snippet 12\\\"\\\" showing a steady, narrow focus across the opening page.\"\n}".to_string(),
            "status: fail\ngrounded: true\nsufficient: false\nreason: Grounded summary coverage currently reaches 50 item(s), but 100 item(s) are required before parent synthesis is sufficient.\nrepair_needed: false\nadditional_pages_needed: true\nnext_page: 1\nnext_offset: 50\nrequired_total_items: 100".to_string(),
            "Across the covered windows so far, the posts repeatedly circle the \\\"alpha\\\" theme, with terse updates and quoted snippets showing a steady, narrow focus rather than abrupt topic changes.".to_string(),
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 1\",\n  \"summary\": \"The second window shifts toward \\\"theme beta\\\" posts, with lines like \\\"post 50: theme beta\\\" and \\\"quote: \\\"snippet 79\\\"\\\" showing a broader follow-on page with a visible change in emphasis.\"\n}".to_string(),
            "status: fail\ngrounded: false\nsufficient: false\nreason: The response appears truncated and cannot be trusted as a grounded page summary.\nrepair_needed: false\nadditional_pages_needed: false\nrequired_total_items: 100".to_string(),
            "The covered scope now includes both the accepted alpha summary and one raw fallback window whose exact records show the later shift into recurring beta updates.".to_string(),
            "Across the first 100 posts, the collection starts with a concentrated run of alpha updates and then shifts into recurring beta posts. The later window was preserved as raw fallback evidence, and its exact records still show the same change in emphasis across the requested scope.".to_string(),
        ]);
        let tools = BlueskyTools::new();
        let collection = test_collection(100);

        let outcomes = tools
            .run_collection_summary_loop(
                &collection,
                "summarize the last 100 posts by alpha.test",
                RequestedSummaryScope::Count {
                    requested_items: 100,
                },
                &llm_client,
                None,
            )
            .await;

        assert_eq!(outcomes.len(), 1);
        let outcome = outcomes.first().expect("aggregated outcome");
        let execution = outcome.execution.as_ref().expect("successful execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert_eq!(result.window_size, Some(100));
        assert!(result.summary.contains("alpha updates"));
        assert!(result.summary.contains("beta posts"));
        assert!(result.summary.contains("raw fallback evidence"));

        let diagnostic = execution.diagnostic.as_deref().unwrap_or_default();
        assert!(diagnostic.contains("accepted 1 page summaries and 1 raw-window fallbacks"));
        assert!(diagnostic.contains("covered_post_count: 100"));

        let verdict = execution.review_verdict.as_ref().expect("review verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.grounded);
        assert!(verdict.sufficient);
    }

    #[tokio::test]
    async fn run_collection_summary_loop_continues_after_planner_and_repair_fail() {
        let llm_client = start_mock_llm_client(vec![
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 0\",\n  \"summary\": \"The first window repeatedly returns to \\\"theme alpha\\\" posts, with lines like \\\"post 0: theme alpha\\\" and \\\"quote: \\\"snippet 12\\\"\\\" showing a steady, narrow focus across the opening page.\"\n}".to_string(),
            "status: fail\ngrounded: true\nsufficient: false\nreason: Grounded summary coverage currently reaches 50 item(s), but 100 item(s) are required before parent synthesis is sufficient.\nrepair_needed: false\nadditional_pages_needed: true\nnext_page: 1\nnext_offset: 50\nrequired_total_items: 100".to_string(),
            "status: completed\nsummary: invalid planner metadata".to_string(),
            "still invalid:\nsummary: planner repair also returned metadata".to_string(),
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 1\",\n  \"summary\": \"The second window shifts toward \\\"theme beta\\\" posts, with lines like \\\"post 50: theme beta\\\" and \\\"quote: \\\"snippet 79\\\"\\\" showing a broader follow-on page with a visible change in emphasis.\"\n}".to_string(),
            "status: pass\ngrounded: true\nsufficient: true\nreason: Grounded summary coverage reaches 100 item(s), satisfying the requested 100 item scope.\nrepair_needed: false\nadditional_pages_needed: false\nrequired_total_items: 100".to_string(),
            "Taken together, the covered windows move from a concentrated \\\"alpha\\\" run into a broader \\\"beta\\\" run, so the collection now shows both continuity and a visible topic shift across the requested scope.".to_string(),
            "The first 100 posts split into two clear phases. The opening half repeatedly returns to \\\"alpha\\\" updates, with short quoted snippets like \\\"snippet 0\\\" and \\\"snippet 12\\\" reinforcing a steady, narrow focus.\n\nThe second half broadens into recurring \\\"beta\\\" updates, with lines like \\\"snippet 50\\\" and \\\"snippet 79\\\" marking a visible shift in emphasis across the requested scope.".to_string(),
        ]);
        let tools = BlueskyTools::new();
        let collection = test_collection(100);

        let outcomes = tools
            .run_collection_summary_loop(
                &collection,
                "summarize the last 100 posts by alpha.test",
                RequestedSummaryScope::Count {
                    requested_items: 100,
                },
                &llm_client,
                None,
            )
            .await;

        assert_eq!(outcomes.len(), 1);
        let outcome = outcomes.first().expect("aggregated outcome");
        let execution = outcome.execution.as_ref().expect("successful execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert_eq!(result.window_size, Some(100));
        assert!(result.summary.contains("The first 100 posts split into two clear phases."));
        assert!(result.summary.contains("\"beta\""));

        let diagnostic = execution.diagnostic.as_deref().unwrap_or_default();
        assert!(diagnostic.contains("covered_post_count: 100"));
    }

    #[tokio::test]
    async fn run_collection_summary_loop_falls_back_when_review_output_is_missing() {
        let llm_client = start_mock_llm_client(vec![
            "TOOL_CALL\nname: submit_summary_result\nargs: {\n  \"title\": \"page 0\",\n  \"summary\": \"The opening window contains only alpha posts:\"\n}".to_string(),
            "this is not a parseable review verdict".to_string(),
            "The requested scope is covered by one raw fallback window whose exact records remain focused on alpha posts throughout the page.".to_string(),
            "Across the requested 50 posts, the collection stays tightly focused on recurring alpha updates. The final answer relies on a raw fallback window because the page review verdict was missing, but the preserved records still show a consistent single-theme run.".to_string(),
        ]);
        let tools = BlueskyTools::new();
        let collection = test_collection(50);

        let outcomes = tools
            .run_collection_summary_loop(
                &collection,
                "summarize the last 50 posts by alpha.test",
                RequestedSummaryScope::Count {
                    requested_items: 50,
                },
                &llm_client,
                None,
            )
            .await;

        assert_eq!(outcomes.len(), 1);
        let outcome = outcomes.first().expect("aggregated outcome");
        let execution = outcome.execution.as_ref().expect("successful execution");
        let result = execution
            .result
            .as_ref()
            .and_then(CollectionLeafResult::as_summary)
            .expect("summary result");

        assert_eq!(result.window_size, Some(50));
        assert!(result.summary.contains("raw fallback window"));

        let diagnostic = execution.diagnostic.as_deref().unwrap_or_default();
        assert!(diagnostic.contains("accepted 0 page summaries and 1 raw-window fallbacks"));

        let verdict = execution.review_verdict.as_ref().expect("review verdict");
        assert_eq!(verdict.status, CollectionReviewStatus::Pass);
        assert!(verdict.grounded);
    }
}
