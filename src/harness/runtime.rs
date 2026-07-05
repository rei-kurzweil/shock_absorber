use crate::harness::agents::AgentGraph;
use crate::harness::context_window::BuiltContextWindow;
use crate::harness::llm_api::ChatMessage;
use crate::harness::tools::PromptToolCall;
use crate::visualization::context::ContextVisualizationData;
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT_RUN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RootRunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl RootRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TranscriptEntryKind {
    ToolCall,
    Notice,
}

#[derive(Clone, Debug)]
pub struct TranscriptEntry {
    pub kind: TranscriptEntryKind,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct RootToolCallRecord {
    pub tool_name: String,
    pub normalized_args: String,
    pub fingerprint: String,
    pub first_round_seen: usize,
    pub times_requested: usize,
    pub executed: bool,
}

#[derive(Clone, Debug)]
pub struct RootRunRound {
    pub round_index: usize,
    pub provider_response: String,
    pub tool_call: Option<PromptToolCall>,
    pub tool_executed: bool,
    pub tool_result_summary: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RootRunState {
    run_id: u64,
    query: String,
    status: RootRunStatus,
    cancel_requested: bool,
    root_context_window: BuiltContextWindow,
    messages: Vec<ChatMessage>,
    transcript_entries: Vec<TranscriptEntry>,
    tool_call_history: Vec<RootToolCallRecord>,
    rounds: Vec<RootRunRound>,
    active_tool_entry: Option<String>,
    final_response: Option<String>,
    agent_graph: AgentGraph,
    context_visualization: Option<ContextVisualizationData>,
}

impl RootRunState {
    pub fn new(
        query: impl Into<String>,
        root_context_window: BuiltContextWindow,
        messages: Vec<ChatMessage>,
        agent_graph: AgentGraph,
    ) -> Self {
        Self {
            run_id: NEXT_ROOT_RUN_ID.fetch_add(1, Ordering::Relaxed),
            query: query.into(),
            status: RootRunStatus::Running,
            cancel_requested: false,
            root_context_window,
            messages,
            transcript_entries: Vec::new(),
            tool_call_history: Vec::new(),
            rounds: Vec::new(),
            active_tool_entry: None,
            final_response: None,
            agent_graph,
            context_visualization: None,
        }
    }

    pub fn run_id(&self) -> u64 {
        self.run_id
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn status(&self) -> &RootRunStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: RootRunStatus) {
        self.status = status;
    }

    pub fn cancel_requested(&self) -> bool {
        self.cancel_requested
    }

    pub fn request_cancel(&mut self) {
        self.cancel_requested = true;
    }

    pub fn root_context_window(&self) -> &BuiltContextWindow {
        &self.root_context_window
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut Vec<ChatMessage> {
        &mut self.messages
    }

    pub fn transcript_entries(&self) -> &[TranscriptEntry] {
        &self.transcript_entries
    }

    pub fn push_transcript_entry(
        &mut self,
        kind: TranscriptEntryKind,
        content: impl Into<String>,
    ) {
        self.transcript_entries.push(TranscriptEntry {
            kind,
            content: content.into(),
        });
    }

    pub fn tool_call_history(&self) -> &[RootToolCallRecord] {
        &self.tool_call_history
    }

    pub fn record_tool_call(
        &mut self,
        round_index: usize,
        tool_call: &PromptToolCall,
        executed: bool,
    ) -> Result<(), serde_json::Error> {
        let normalized_args = normalize_json_value(&tool_call.args)?;
        let fingerprint = format!("{}:{normalized_args}", tool_call.name);
        if let Some(existing) = self
            .tool_call_history
            .iter_mut()
            .find(|entry| entry.fingerprint == fingerprint)
        {
            existing.times_requested += 1;
            existing.executed |= executed;
        } else {
            self.tool_call_history.push(RootToolCallRecord {
                tool_name: tool_call.name.clone(),
                normalized_args,
                fingerprint,
                first_round_seen: round_index,
                times_requested: 1,
                executed,
            });
        }
        Ok(())
    }

    pub fn rounds(&self) -> &[RootRunRound] {
        &self.rounds
    }

    pub fn record_round(
        &mut self,
        round_index: usize,
        provider_response: impl Into<String>,
        tool_call: Option<PromptToolCall>,
        tool_executed: bool,
        tool_result_summary: Option<String>,
    ) {
        self.rounds.push(RootRunRound {
            round_index,
            provider_response: provider_response.into(),
            tool_call,
            tool_executed,
            tool_result_summary,
        });
    }

    pub fn set_active_tool_entry(&mut self, entry: Option<String>) {
        self.active_tool_entry = entry;
    }

    pub fn final_response(&self) -> Option<&str> {
        self.final_response.as_deref()
    }

    pub fn set_final_response(&mut self, response: impl Into<String>) {
        self.final_response = Some(response.into());
    }

    pub fn agent_graph(&self) -> &AgentGraph {
        &self.agent_graph
    }

    pub fn agent_graph_mut(&mut self) -> &mut AgentGraph {
        &mut self.agent_graph
    }

    pub fn context_visualization(&self) -> Option<&ContextVisualizationData> {
        self.context_visualization.as_ref()
    }

    pub fn set_context_visualization(&mut self, data: ContextVisualizationData) {
        self.context_visualization = Some(data);
    }

    pub fn render_output_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.transcript_entries.is_empty() || self.active_tool_entry.is_some() {
            lines.push("Tool Transcript:".to_string());
            lines.push(String::new());
            for entry in &self.transcript_entries {
                lines.extend(entry.content.lines().map(str::to_owned));
                lines.push(String::new());
            }
            if let Some(active_entry) = self.active_tool_entry.as_deref() {
                lines.extend(active_entry.lines().map(str::to_owned));
                lines.push(String::new());
            }
        }

        if let Some(response) = self.final_response() {
            if !self.transcript_entries.is_empty() || self.active_tool_entry.is_some() {
                lines.push("Final Answer:".to_string());
                lines.push(String::new());
            }
            if response.trim().is_empty() {
                lines.push("<empty response>".to_string());
            } else {
                lines.extend(response.lines().map(str::to_owned));
            }
        }

        if lines.is_empty() {
            lines.push("Waiting for evil_gemma...".to_string());
        }

        lines
    }
}

fn normalize_json_value(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(&canonicalize_json_value(value))
}

fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_json_value).collect()),
        Value::Object(map) => {
            let sorted = map
                .iter()
                .map(|(key, value)| (key.clone(), canonicalize_json_value(value)))
                .collect::<std::collections::BTreeMap<_, _>>();
            let mut out = Map::new();
            for (key, value) in sorted {
                out.insert(key, value);
            }
            Value::Object(out)
        }
        _ => value.clone(),
    }
}
