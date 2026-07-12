use crate::harness::context_window::BuiltContextWindow;
use crate::harness::units::definition::UnitDefinition;
use crate::harness::units::kinds::UnitKind;
use serde_json::Value;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitLocalState {
    None,
    CollectionSummary(CollectionSummaryUnitLocalState),
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
    pub definition: UnitDefinition,
    pub instance_label: String,
    pub kind: UnitKind,
    pub status: UnitInstanceStatus,
    pub active_node: Option<String>,
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
            instance_label: definition.label.clone(),
            kind: definition.kind,
            definition,
            status: UnitInstanceStatus::Ready,
            active_node: None,
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
}
