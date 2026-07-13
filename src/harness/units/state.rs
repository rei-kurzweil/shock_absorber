use crate::harness::context_window::BuiltContextWindow;
use crate::harness::units::definition::UnitDefinition;
use crate::harness::units::kinds::UnitKind;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_UNIT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitInstanceStatus {
    Ready,
    Running,
    Completed,
    CompletedWithWarnings,
    Failed,
    BlockedOnChild,
}

impl UnitInstanceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::CompletedWithWarnings => "warning",
            Self::Failed => "failed",
            Self::BlockedOnChild => "blocked_on_child",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CollectionSummaryUnitLocalState {
    pub selected_collection_id: Option<String>,
    pub current_offset: Option<usize>,
    pub next_offset: Option<usize>,
    pub pages_processed: usize,
    pub covered_post_count: usize,
    pub source_exhausted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchPlannerUnitLocalState {
    pub current_round: usize,
    pub current_node: Option<String>,
    pub pending_tool_name: Option<String>,
    pub consecutive_invalid_tool_responses: usize,
    pub consecutive_invalid_tool_calls: usize,
    pub searched_collection_windows: usize,
    pub final_summary_present: bool,
    pub fallback_resolution_used: bool,
    pub last_validation_failure: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitLocalState {
    None,
    CollectionSummary(CollectionSummaryUnitLocalState),
    SearchPlanner(SearchPlannerUnitLocalState),
}

impl Default for UnitLocalState {
    fn default() -> Self {
        Self::None
    }
}

impl UnitLocalState {
    pub fn compact_summary(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::CollectionSummary(state) => Some(format!(
                "collection_id: {}\ncurrent_offset: {}\nnext_offset: {}\npages_processed: {}\ncovered_post_count: {}\nsource_exhausted: {}",
                state.selected_collection_id.as_deref().unwrap_or("<none>"),
                state
                    .current_offset
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "<none>".to_string()),
                state
                    .next_offset
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "<none>".to_string()),
                state.pages_processed,
                state.covered_post_count,
                state.source_exhausted
            )),
            Self::SearchPlanner(state) => Some(format!(
                "current_round: {}\ncurrent_node: {}\npending_tool_name: {}\nconsecutive_invalid_tool_responses: {}\nconsecutive_invalid_tool_calls: {}\nsearched_collection_windows: {}\nfinal_summary_present: {}\nfallback_resolution_used: {}\nlast_validation_failure: {}",
                state.current_round,
                state.current_node.as_deref().unwrap_or("<none>"),
                state.pending_tool_name.as_deref().unwrap_or("<none>"),
                state.consecutive_invalid_tool_responses,
                state.consecutive_invalid_tool_calls,
                state.searched_collection_windows,
                state.final_summary_present,
                state.fallback_resolution_used,
                state.last_validation_failure.as_deref().unwrap_or("<none>")
            )),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnitOutputPayload {
    pub port: Option<String>,
    pub payload: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct UnitInstanceState {
    pub instance_id: String,
    pub definition: UnitDefinition,
    pub instance_label: String,
    pub kind: UnitKind,
    pub status: UnitInstanceStatus,
    pub active_node: Option<String>,
    pub visit_count: usize,
    pub visited_nodes: Vec<String>,
    pub last_output_port: Option<String>,
    pub transitions: Vec<ExecutionTransitionState>,
    pub blocked_on_child: Option<String>,
    pub tool_name: Option<String>,
    pub collection_id: Option<String>,
    pub context_window: Option<BuiltContextWindow>,
    pub result_summary: Option<String>,
    pub local_state: UnitLocalState,
    pub output: Option<UnitOutputPayload>,
    pub children: Vec<UnitInstanceState>,
}

impl UnitInstanceState {
    pub fn new(definition: UnitDefinition) -> Self {
        Self {
            instance_id: format!(
                "unit-{}",
                NEXT_UNIT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed)
            ),
            instance_label: definition.label.clone(),
            kind: definition.kind,
            definition,
            status: UnitInstanceStatus::Ready,
            active_node: None,
            visit_count: 0,
            visited_nodes: Vec::new(),
            last_output_port: None,
            transitions: Vec::new(),
            blocked_on_child: None,
            tool_name: None,
            collection_id: None,
            context_window: None,
            result_summary: None,
            local_state: UnitLocalState::None,
            output: None,
            children: Vec::new(),
        }
    }

    pub fn push_child(&mut self, child: UnitInstanceState) {
        self.children.push(child);
    }

    pub fn activate(&mut self, node: impl Into<String>) {
        let node = node.into();
        self.status = UnitInstanceStatus::Running;
        self.visit_count += 1;
        if !self.visited_nodes.contains(&node) {
            self.visited_nodes.push(node.clone());
        }
        self.active_node = Some(node);
    }

    pub fn transition(
        &mut self,
        output_port: impl Into<String>,
        target_instance_id: Option<String>,
        graph_output: Option<String>,
    ) {
        let output_port = output_port.into();
        self.last_output_port = Some(output_port.clone());
        let sequence = self.transitions.len() as u64 + 1;
        self.transitions.push(ExecutionTransitionState {
            source_instance_id: self.instance_id.clone(),
            output_port,
            target_instance_id,
            graph_output,
            sequence,
            traversal_count: 1,
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionTransitionState {
    pub source_instance_id: String,
    pub output_port: String,
    pub target_instance_id: Option<String>,
    pub graph_output: Option<String>,
    pub sequence: u64,
    pub traversal_count: usize,
}

#[derive(Clone, Debug)]
pub enum ExecutionRuntimeEvent {
    Started {
        instance_id: String,
        node_id: Option<String>,
    },
    StateChanged {
        instance_id: String,
        status: UnitInstanceStatus,
        active_node: Option<String>,
    },
    ChildAttached {
        parent_instance_id: String,
        child: UnitInstanceState,
        invoker_instance_id: Option<String>,
    },
    Transition(ExecutionTransitionState),
    Finished {
        instance_id: String,
        output_port: String,
    },
    Failed {
        instance_id: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::units::{UnitDefinition, UnitKind, UnitLocalStateSchema};

    #[test]
    fn transitions_retain_visits_and_only_last_port_as_current() {
        let mut unit = UnitInstanceState::new(UnitDefinition {
            id: "x".into(),
            label: "x".into(),
            kind: UnitKind::Loop,
            graph: None,
            local_state_schema: UnitLocalStateSchema::None,
        });
        unit.activate("plan");
        unit.transition("retry", None, None);
        unit.activate("plan");
        unit.transition("success", None, Some("success".into()));
        assert_eq!(unit.visit_count, 2);
        assert_eq!(unit.visited_nodes, vec!["plan"]);
        assert_eq!(unit.last_output_port.as_deref(), Some("success"));
        assert_eq!(unit.transitions.len(), 2);
    }
}
