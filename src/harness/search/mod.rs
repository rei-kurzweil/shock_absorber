use crate::harness::agents::{AgentNodeStatus, AgentNodeTemplate};
use crate::harness::context_window::{
    BuiltContextWindow, ContextSectionKind, LLMContext, build_context_window_report,
};
use crate::harness::llm_api::LlmApiClient;
use crate::harness::prompts::AgentKind;
use crate::harness::tools::{
    CollectionReviewStatus, CollectionToolOutcome, render_collection_outcome_result,
    render_llm_execution_result, render_llm_result_compact, render_review_summary,
};
use crate::harness::units::{
    UnitDefinition, UnitInstanceState, UnitInstanceStatus, UnitKind, UnitLocalStateSchema,
};

pub(crate) fn build_search_agent_node(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> AgentNodeTemplate {
    AgentNodeTemplate {
        agent_type: UnitKind::PublicToolOrchestration.compatibility_agent_node_kind(),
        agent_kind: Some(AgentKind::Search),
        label: "search tool agent".to_string(),
        status: parent_status(outcomes),
        tool_name: Some("search".to_string()),
        collection_id: None,
        context_window_report: Some(build_search_tool_context_window(
            prompt, outcomes, llm_client,
        )),
        result_summary: Some(render_combined_search_results(outcomes)),
        children: outcomes.iter().map(build_collection_tool_node).collect(),
    }
}

pub(crate) fn build_search_unit_instance(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> UnitInstanceState {
    let mut unit = UnitInstanceState::new(UnitDefinition {
        id: "search.public_tool".to_string(),
        label: "search tool unit".to_string(),
        kind: UnitKind::PublicToolOrchestration,
        graph: None,
        local_state_schema: UnitLocalStateSchema::None,
    });
    unit.instance_label = "search tool unit".to_string();
    unit.status = unit_status(parent_status(outcomes));
    unit.tool_name = Some("search".to_string());
    unit.context_window = Some(build_search_tool_context_window(prompt, outcomes, llm_client));
    unit.result_summary = Some(render_combined_search_results(outcomes));
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
        agent_type: if verdict.status == CollectionReviewStatus::Pass {
            UnitKind::SearchReview.compatibility_agent_node_kind()
        } else {
            UnitKind::SearchReview.compatibility_agent_node_kind()
        },
        agent_kind: Some(AgentKind::SearchReview),
        label: "search review".to_string(),
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
    let mut unit = UnitInstanceState::new(UnitDefinition {
        id: format!("search.collection.{}", outcome.collection_id),
        label: format!(
            "{}: {}",
            outcome.tool_kind.label_prefix(),
            outcome.collection_label
        ),
        kind: UnitKind::CollectionSearch,
        graph: None,
        local_state_schema: UnitLocalStateSchema::None,
    });
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
    let mut unit = UnitInstanceState::new(UnitDefinition {
        id: "search.review".to_string(),
        label: "search review".to_string(),
        kind: UnitKind::SearchReview,
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

fn build_search_tool_context_window(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
    llm_client: &LlmApiClient,
) -> BuiltContextWindow {
    let context = build_search_parent_context(prompt, outcomes);
    build_context_window_report(&context, &llm_client.context_limits())
}

pub(crate) fn build_search_parent_context(
    prompt: &str,
    outcomes: &[CollectionToolOutcome],
) -> LLMContext {
    let mut context = LLMContext::new(AgentKind::Search.system_prompt());
    context.push_section_with_kind(
        "Original Search Query",
        ContextSectionKind::CurrentTask,
        prompt,
    );
    context.push_section_with_kind(
        "Per-Collection Results",
        ContextSectionKind::ParentSearchResults,
        outcomes
            .iter()
            .map(|outcome| {
                let mut lines = vec![
                    format!("tool_name: {}", outcome.tool_kind.tool_name()),
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
                            lines.push(format!("review_grounded: {}", verdict.grounded));
                            lines.push(format!("review_sufficient: {}", verdict.sufficient));
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

pub(crate) fn render_combined_search_results(outcomes: &[CollectionToolOutcome]) -> String {
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
    for outcome in outcomes {
        match outcome.execution.as_ref() {
            Ok(execution) => lines.push(render_collection_outcome_result(
                outcome.tool_kind,
                &outcome.collection_id,
                &outcome.collection_label,
                execution,
            )),
            Err(err) => lines.push(format!(
                "collection_id: {}\nlabel: {}\nstatus: failed\nerror: {}",
                outcome.collection_id, outcome.collection_label, err
            )),
        }
    }
    lines.join("\n\n")
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
