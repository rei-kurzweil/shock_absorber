use crate::harness::llm_api::LlmApiClient;
use crate::harness::r#loop::{
    LoopDefinition, LoopExecutor, LoopKind, LoopNodeDefinition, LoopNodeKind, LoopPort,
    LoopPortTarget,
};
use crate::harness::tools::{
    BlueskyTools, COLLECTION_SEARCH_PAGE_SIZE, CollectionLeafToolKind, CollectionToolOutcome,
    RequestedSummaryScope, ToolProgressEvent, apply_summary_sufficiency_gates,
    paged_search_collection, render_collection_outcome_result, summary_scope_initial_offset,
    summary_scope_max_pages,
};
use crate::model::LabeledPostCollection;
use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedSender;

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
                target: LoopPortTarget::Node("return_summary"),
            },
            LoopPort {
                name: "continue",
                target: LoopPortTarget::Node("advance_cursor"),
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
const NODE_ADVANCE_CURSOR: &str = "advance_cursor";
const NODE_RETURN_SUMMARY: &str = "return_summary";
const PORT_SUCCESS: &str = "success";
const PORT_CONTINUE: &str = "continue";
const PORT_COMPLETE: &str = "complete";

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

pub(crate) fn render_summary_collection_loop_result(outcomes: &[CollectionToolOutcome]) -> String {
    if outcomes.is_empty() {
        return "status: failed\nreason: no summary pages were processed".to_string();
    }

    let mut lines = vec![format!("summary_pages_processed: {}", outcomes.len())];
    for outcome in outcomes {
        match outcome.execution.as_ref() {
            Ok(execution) => lines.push(render_collection_outcome_result(
                outcome.tool_kind,
                &outcome.collection_id,
                &outcome.collection_label,
                execution,
            )),
            Err(err) => lines.push(format!(
                "tool_name: {}\ncollection_id: {}\ncollection_label: {}\nstatus: failed\nerror: {}",
                outcome.tool_kind.tool_name(),
                outcome.collection_id,
                outcome.collection_label,
                err
            )),
        }
    }
    lines.join("\n\n")
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
        let mut outcomes = Vec::new();
        let max_pages = summary_scope_max_pages(requested_summary_scope, collection.posts.len());
        let mut pages_processed = 0usize;
        let mut seen_offsets = HashSet::new();
        let mut offset = 0usize;
        let mut current_node = DEFINITION.entry_node;

        loop {
            match current_node {
                NODE_INIT_WINDOW => {
                    offset = summary_scope_initial_offset(requested_summary_scope);
                    let Some(next) = next_node(NODE_INIT_WINDOW, PORT_SUCCESS) else {
                        break;
                    };
                    current_node = next;
                }
                NODE_SUMMARIZE_PAGE => {
                    if pages_processed >= max_pages
                        || offset >= collection.posts.len()
                        || !seen_offsets.insert(offset)
                    {
                        break;
                    }

                    let paged =
                        paged_search_collection(collection, offset, COLLECTION_SEARCH_PAGE_SIZE);
                    let mut page_outcomes = self
                        .run_collection_tools(
                            CollectionLeafToolKind::Summary,
                            &[paged],
                            prompt,
                            requested_summary_scope,
                            llm_client,
                            observer.clone(),
                        )
                        .await;
                    let Some(mut outcome) = page_outcomes.pop() else {
                        break;
                    };
                    if let Ok(execution) = outcome.execution.as_mut() {
                        apply_summary_sufficiency_gates(
                            requested_summary_scope,
                            &outcome.collection_id,
                            &outcomes,
                            execution,
                        );
                        if execution.diagnostic.is_none() {
                            execution.diagnostic = Some(format!(
                                "summary cursor processed offset {offset} (page {} of at most {max_pages})",
                                pages_processed + 1
                            ));
                        }
                    }

                    let next_offset = outcome
                        .execution
                        .as_ref()
                        .ok()
                        .and_then(|execution| execution.review_verdict.as_ref())
                        .and_then(|verdict| {
                            verdict
                                .additional_pages_needed
                                .then_some(verdict.next_offset)
                        })
                        .flatten();
                    outcomes.push(outcome);
                    pages_processed += 1;

                    current_node = if next_offset.is_some() {
                        offset = next_offset.unwrap_or(offset);
                        next_node(NODE_REVIEW_PAGE, PORT_CONTINUE).unwrap_or(NODE_ADVANCE_CURSOR)
                    } else {
                        next_node(NODE_REVIEW_PAGE, PORT_COMPLETE).unwrap_or(NODE_RETURN_SUMMARY)
                    };
                }
                NODE_ADVANCE_CURSOR => {
                    let Some(next) = next_node(NODE_ADVANCE_CURSOR, PORT_SUCCESS) else {
                        break;
                    };
                    current_node = next;
                }
                NODE_RETURN_SUMMARY => break,
                _ => break,
            }
        }

        outcomes
    }
}
