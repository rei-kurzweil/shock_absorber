use crate::harness::agents::{AgentNodeStatus, AgentNodeTemplate};
use crate::harness::context_window::{
    BuiltContextWindow, ContextSectionKind, LLMContext, build_context_window_report,
};
use crate::harness::llm_api::LlmApiClient;
use crate::harness::prompts::AgentKind;
use crate::harness::r#loop::{collection_summary, summary as summary_loop};
use crate::harness::tools::{
    COLLECTION_SEARCH_PAGE_SIZE, CollectionLeafResult, CollectionReviewStatus, CollectionToolOutcome,
    RequestedSummaryScope, render_llm_execution_result, render_review_summary,
};
use crate::harness::units::{
    CollectionSummaryUnitLocalState, UnitInstanceState, UnitInstanceStatus, UnitKind,
    UnitLocalState, UnitLocalStateSchema, unit_definition_from_loop,
};

pub(crate) fn build_summary_agent_node(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> AgentNodeTemplate {
    AgentNodeTemplate {
        agent_type: UnitKind::PublicToolOrchestration.compatibility_agent_node_kind(),
        agent_kind: Some(AgentKind::Summary),
        label: "summary tool agent".to_string(),
        status: parent_status(outcomes),
        tool_name: Some("summary".to_string()),
        collection_id: None,
        context_window_report: Some(build_summary_tool_context_window(
            prompt, outcomes, llm_client,
        )),
        result_summary: Some(collection_summary::render_summary_collection_loop_result(outcomes)),
        children: outcomes.iter().map(build_collection_tool_node).collect(),
    }
}

pub(crate) fn build_summary_unit_instance(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> UnitInstanceState {
    let mut unit = UnitInstanceState::new(unit_definition_from_loop(
        "summary.public_tool",
        "summary tool unit",
        UnitKind::PublicToolOrchestration,
        &summary_loop::DEFINITION,
        UnitLocalStateSchema::None,
    ));
    unit.status = unit_status(parent_status(outcomes));
    unit.tool_name = Some("summary".to_string());
    unit.context_window = Some(build_summary_tool_context_window(prompt, outcomes, llm_client));
    unit.result_summary = Some(collection_summary::render_summary_collection_loop_result(outcomes));
    unit.active_node = Some(if outcomes.is_empty() {
        summary_loop::DEFINITION.entry_node.to_string()
    } else {
        "run_collection_summary".to_string()
    });
    unit.children = outcomes.iter().map(build_collection_tool_unit).collect();
    unit
}

fn parent_status(outcomes: &[CollectionToolOutcome]) -> AgentNodeStatus {
    if outcomes.is_empty() {
        AgentNodeStatus::Failed
    } else if outcomes.iter().any(|outcome| {
        outcome
            .execution
            .as_ref()
            .map(|execution| !execution.is_usable())
            .unwrap_or(true)
    }) {
        AgentNodeStatus::Failed
    } else if outcomes.iter().any(|outcome| {
        outcome
            .execution
            .as_ref()
            .map(|execution| execution.has_warnings())
            .unwrap_or(false)
    }) {
        AgentNodeStatus::CompletedWithWarnings
    } else {
        AgentNodeStatus::Completed
    }
}

fn build_collection_tool_node(outcome: &CollectionToolOutcome) -> AgentNodeTemplate {
    let (status, context_window_report, result_summary) = match outcome.execution.as_ref() {
        Ok(execution) => (
            if execution.is_usable() {
                if execution.has_warnings() {
                    AgentNodeStatus::CompletedWithWarnings
                } else {
                    AgentNodeStatus::Completed
                }
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
        agent_type: outcome.tool_kind.node_kind(),
        agent_kind: Some(outcome.tool_kind.agent_kind()),
        label: format!(
            "{}: {}",
            outcome.tool_kind.label_prefix(),
            outcome.collection_label
        ),
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

fn build_collection_review_agent_node(
    execution: &crate::harness::tools::LlmSearchExecution,
) -> Option<AgentNodeTemplate> {
    let verdict = execution.review_verdict.as_ref()?;
    Some(AgentNodeTemplate {
        agent_type: UnitKind::SummaryReview.compatibility_agent_node_kind(),
        agent_kind: Some(AgentKind::SummaryReview),
        label: "summary review".to_string(),
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

fn build_collection_tool_unit(outcome: &CollectionToolOutcome) -> UnitInstanceState {
    let mut unit = UnitInstanceState::new(unit_definition_from_loop(
        format!("summary.collection.{}", outcome.collection_id),
        format!(
            "{}: {}",
            outcome.tool_kind.label_prefix(),
            outcome.collection_label
        ),
        UnitKind::Loop,
        &collection_summary::DEFINITION,
        UnitLocalStateSchema::CollectionSummaryPagination,
    ));
    let (status, context_window_report, result_summary) = match outcome.execution.as_ref() {
        Ok(execution) => (
            if execution.is_usable() {
                if execution.has_warnings() {
                    UnitInstanceStatus::CompletedWithWarnings
                } else {
                    UnitInstanceStatus::Completed
                }
            } else {
                UnitInstanceStatus::Failed
            },
            Some(execution.context_window.clone()),
            Some(render_llm_execution_result(execution)),
        ),
        Err(err) => (
            UnitInstanceStatus::Failed,
            None,
            Some(format!("Tool execution failed: {err}")),
        ),
    };
    unit.status = status;
    unit.collection_id = Some(outcome.collection_id.clone());
    unit.context_window = context_window_report;
    unit.result_summary = result_summary;
    unit.active_node = Some(final_collection_summary_node(outcome).to_string());
    unit.local_state = UnitLocalState::CollectionSummary(collection_summary_state(outcome));
    if let Some(execution) = outcome.execution.as_ref().ok() {
        if let Some(review_unit) = build_collection_review_unit(execution) {
            unit.children.push(review_unit);
        }
    }
    unit
}

fn build_collection_review_unit(
    execution: &crate::harness::tools::LlmSearchExecution,
) -> Option<UnitInstanceState> {
    let verdict = execution.review_verdict.as_ref()?;
    let mut unit = UnitInstanceState::new(crate::harness::units::UnitDefinition {
        id: "summary.review".to_string(),
        label: "summary review".to_string(),
        kind: UnitKind::SummaryReview,
        graph: None,
        local_state_schema: UnitLocalStateSchema::None,
    });
    unit.status = match verdict.status {
        CollectionReviewStatus::Pass => UnitInstanceStatus::Completed,
        CollectionReviewStatus::Fail => UnitInstanceStatus::Failed,
    };
    unit.context_window = execution.review_context_window.clone();
    unit.result_summary = Some(render_review_summary(
        execution.review_verdict.as_ref(),
        execution.repair_diagnostic.as_deref(),
    ));
    Some(unit)
}

fn final_collection_summary_node(outcome: &CollectionToolOutcome) -> &'static str {
    match outcome.execution.as_ref() {
        Ok(execution) if execution.is_usable() => "return_summary",
        Ok(_) => "collection_summary_notes_review",
        Err(_) => "repair_summary_output",
    }
}

fn collection_summary_state(outcome: &CollectionToolOutcome) -> CollectionSummaryUnitLocalState {
    let Some(execution) = outcome.execution.as_ref().ok() else {
        return CollectionSummaryUnitLocalState {
            selected_collection_id: Some(outcome.collection_id.clone()),
            ..Default::default()
        };
    };
    let summary = execution
        .result
        .as_ref()
        .or(execution.original_result.as_ref())
        .and_then(CollectionLeafResult::as_summary);
    let current_offset = summary.and_then(|result| {
        let covered_post_count = result.processed_window_size().unwrap_or(0);
        let start_offset = result.processed_window_offset().unwrap_or(0);
        if covered_post_count == 0 {
            return Some(start_offset);
        }
        let page_size = result.processed_page_size().unwrap_or(COLLECTION_SEARCH_PAGE_SIZE);
        let last_window_size = covered_post_count.saturating_sub(1) % page_size + 1;
        Some(start_offset + covered_post_count.saturating_sub(last_window_size))
    });
    let covered_post_count = summary
        .and_then(|result| result.processed_window_size())
        .unwrap_or(0);
    let pages_processed = summary
        .and_then(|result| result.processed_page_size())
        .map(|page_size| covered_post_count.div_ceil(page_size.max(1)))
        .unwrap_or(usize::from(covered_post_count > 0));
    let next_offset = summary.and_then(|result| {
        if result.has_more == Some(true) {
            Some(
                result
                    .processed_window_offset()
                    .unwrap_or(0)
                    .saturating_add(result.processed_window_size().unwrap_or(0)),
            )
        } else {
            None
        }
    });

    CollectionSummaryUnitLocalState {
        selected_collection_id: Some(outcome.collection_id.clone()),
        current_offset,
        next_offset,
        pages_processed,
        covered_post_count,
        source_exhausted: summary.and_then(|result| result.source_exhausted).unwrap_or(false),
    }
}

fn build_summary_tool_context_window(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> BuiltContextWindow {
    let mut context = LLMContext::new(AgentKind::Summary.system_prompt());
    context.push_section_with_kind(
        "Original Summary Query",
        ContextSectionKind::CurrentTask,
        prompt,
    );
    context.push_section_with_kind(
        "Summary Result",
        ContextSectionKind::ParentSearchResults,
        collection_summary::render_summary_collection_loop_result(outcomes),
    );
    build_context_window_report(&context, &llm_client.context_limits())
}

pub(crate) fn render_public_summary_outcomes(outcomes: &[CollectionToolOutcome]) -> String {
    if outcomes.len() == 1 {
        if let Some(outcome) = outcomes.first() {
            if let Ok(execution) = outcome.execution.as_ref() {
                if execution.is_usable() {
                    if let Some(result) = execution
                        .result
                        .as_ref()
                        .or(execution.original_result.as_ref())
                        .and_then(CollectionLeafResult::as_summary)
                    {
                        let mut sections = Vec::new();
                        let final_summary = result.summary.trim();
                        let concatenated = result
                            .concatenated_window_summaries()
                            .map(str::trim)
                            .filter(|text| !text.is_empty());

                        if !final_summary.is_empty() {
                            sections.push(format!(
                                "Overall commentary across {}:\n{}",
                                outcome.collection_label, final_summary
                            ));
                        }

                        if let Some(concatenated) = concatenated {
                            sections.push(format!(
                                "Concatenated page summaries for {}:\n{}",
                                outcome.collection_label, concatenated
                            ));
                        }

                        if !sections.is_empty() {
                            return sections.join("\n\n");
                        }
                    }
                }
            }
        }
    }

    collection_summary::render_summary_collection_loop_result(outcomes)
}

pub(crate) fn summary_scope_local_state(
    collection_id: &str,
    requested_summary_scope: RequestedSummaryScope,
) -> CollectionSummaryUnitLocalState {
    let current_offset = match requested_summary_scope {
        RequestedSummaryScope::CurrentWindow | RequestedSummaryScope::Count { .. } => Some(0),
        RequestedSummaryScope::Page { page_index } => Some(page_index * COLLECTION_SEARCH_PAGE_SIZE),
        RequestedSummaryScope::PageRange { start_page, .. } => {
            Some(start_page * COLLECTION_SEARCH_PAGE_SIZE)
        }
    };
    CollectionSummaryUnitLocalState {
        selected_collection_id: Some(collection_id.to_string()),
        current_offset,
        ..Default::default()
    }
}

fn unit_status(status: AgentNodeStatus) -> UnitInstanceStatus {
    match status {
        AgentNodeStatus::Ready => UnitInstanceStatus::Ready,
        AgentNodeStatus::Running => UnitInstanceStatus::Running,
        AgentNodeStatus::Completed => UnitInstanceStatus::Completed,
        AgentNodeStatus::CompletedWithWarnings => UnitInstanceStatus::CompletedWithWarnings,
        AgentNodeStatus::Failed => UnitInstanceStatus::Failed,
    }
}
