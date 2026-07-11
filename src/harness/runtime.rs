use crate::harness::agents::AgentGraph;
use crate::harness::context_window::BuiltContextWindow;
use crate::harness::llm_api::ChatMessage;
use crate::harness::tools::PromptToolCall;
use crate::visualization::context::ContextVisualizationData;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT_RUN_ID: AtomicU64 = AtomicU64::new(1);
const TOOL_PANEL_BG: Color = Color::Rgb(230, 230, 230);
const TOOL_PANEL_FG: Color = Color::Black;
const AGENT_PANEL_BG: Color = Color::Rgb(70, 70, 70);
const AGENT_PANEL_FG: Color = Color::Rgb(210, 210, 210);

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
    UserInput,
    ToolCall,
    Notice,
    AgentEvent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextMessageKind {
    InitialSystem,
    InitialUserContext,
    ToolRequest,
    ToolResult,
    AssistantReply,
    UserFollowUp,
    RoundLimitPrompt,
    RepairPrompt,
}

#[derive(Clone, Debug)]
pub struct ContextMessage {
    pub kind: ContextMessageKind,
    pub message: ChatMessage,
}

#[derive(Clone, Debug)]
pub struct TranscriptEntry {
    pub kind: TranscriptEntryKind,
    pub content: String,
    pub agent_label: Option<String>,
    pub depth: usize,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuccessfulRootToolRun {
    pub tool_name: String,
    pub query: String,
    pub rendered_result: String,
    pub summary: String,
    pub actor_refs: Vec<String>,
    pub collection_ids: Vec<String>,
    pub intent: String,
    pub requested_post_count: Option<usize>,
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
    messages: Vec<ContextMessage>,
    transcript_entries: Vec<TranscriptEntry>,
    tool_call_history: Vec<RootToolCallRecord>,
    rounds: Vec<RootRunRound>,
    active_tool_entry: Option<String>,
    final_response: Option<String>,
    latest_successful_tool_runs: HashMap<String, SuccessfulRootToolRun>,
    agent_graph: AgentGraph,
    context_visualization: Option<ContextVisualizationData>,
}

impl RootRunState {
    pub fn new(
        query: impl Into<String>,
        root_context_window: BuiltContextWindow,
        messages: Vec<ContextMessage>,
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
            latest_successful_tool_runs: HashMap::new(),
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

    pub fn messages(&self) -> &[ContextMessage] {
        &self.messages
    }

    pub fn llm_messages(&self) -> Vec<ChatMessage> {
        self.messages
            .iter()
            .map(|entry| entry.message.clone())
            .collect()
    }

    pub fn push_message(
        &mut self,
        kind: ContextMessageKind,
        role: impl Into<String>,
        content: impl Into<String>,
    ) {
        self.messages.push(ContextMessage {
            kind,
            message: ChatMessage {
                role: role.into(),
                content: content.into(),
            },
        });
    }

    pub fn transcript_entries(&self) -> &[TranscriptEntry] {
        &self.transcript_entries
    }

    pub fn push_transcript_entry(&mut self, kind: TranscriptEntryKind, content: impl Into<String>) {
        self.transcript_entries.push(TranscriptEntry {
            kind,
            content: content.into(),
            agent_label: None,
            depth: 0,
        });
    }

    pub fn push_agent_entry(
        &mut self,
        kind: TranscriptEntryKind,
        agent_label: impl Into<String>,
        depth: usize,
        content: impl Into<String>,
    ) {
        self.transcript_entries.push(TranscriptEntry {
            kind,
            content: content.into(),
            agent_label: Some(agent_label.into()),
            depth,
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

    pub fn active_tool_entry(&self) -> Option<&str> {
        self.active_tool_entry.as_deref()
    }

    pub fn final_response(&self) -> Option<&str> {
        self.final_response.as_deref()
    }

    pub fn set_final_response(&mut self, response: impl Into<String>) {
        self.final_response = Some(response.into());
    }

    pub fn latest_successful_tool_run(&self, tool_name: &str) -> Option<&SuccessfulRootToolRun> {
        self.latest_successful_tool_runs.get(tool_name)
    }

    pub fn set_latest_successful_tool_run(&mut self, result: Option<SuccessfulRootToolRun>) {
        if let Some(result) = result {
            self.latest_successful_tool_runs
                .insert(result.tool_name.clone(), result);
        }
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
            for entry in compact_transcript_entries(&self.transcript_entries) {
                let indent = "    ".repeat(entry.depth);
                if let Some(agent_label) = entry.agent_label.as_deref() {
                    lines.push(format!("{indent}[agent] {agent_label}"));
                }
                let body_indent = match entry.kind {
                    TranscriptEntryKind::UserInput => format!("{indent}    "),
                    _ => indent.clone(),
                };
                for line in entry.content.lines() {
                    if line.is_empty() {
                        lines.push(match entry.kind {
                            TranscriptEntryKind::UserInput => body_indent.clone(),
                            _ => String::new(),
                        });
                    } else {
                        lines.push(format!("{body_indent}{line}"));
                    }
                }
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

    pub fn render_output_text(&self) -> Text<'static> {
        let mut lines = Vec::new();
        if !self.transcript_entries.is_empty() || self.active_tool_entry.is_some() {
            lines.push(Line::from("Tool Transcript:"));
            lines.push(Line::from(""));
            for entry in compact_transcript_entries(&self.transcript_entries) {
                match entry.kind {
                    TranscriptEntryKind::UserInput => {
                        lines.extend(render_padded_panel_entry(
                            None,
                            entry.depth,
                            &entry.content,
                            user_input_panel_style(),
                            4,
                        ));
                    }
                    TranscriptEntryKind::ToolCall => {
                        let agent_label = entry.agent_label.as_deref();
                        lines.extend(render_panel_entry(
                            agent_label,
                            entry.depth,
                            &entry.content,
                            tool_panel_style(),
                        ));
                    }
                    TranscriptEntryKind::AgentEvent => {
                        let agent_label = entry.agent_label.as_deref();
                        lines.extend(render_panel_entry(
                            agent_label,
                            entry.depth,
                            &entry.content,
                            agent_panel_style(),
                        ));
                    }
                    TranscriptEntryKind::Notice => {
                        for line in entry.content.lines() {
                            lines.push(Line::from(line.to_string()));
                        }
                    }
                }
                if matches!(entry.kind, TranscriptEntryKind::UserInput) {
                    lines.push(Line::from(vec![Span::styled(
                        String::new(),
                        user_input_panel_style(),
                    )]));
                } else if matches!(entry.kind, TranscriptEntryKind::ToolCall) {
                    lines.push(Line::from(vec![Span::styled(
                        String::new(),
                        tool_panel_style(),
                    )]));
                } else if matches!(entry.kind, TranscriptEntryKind::AgentEvent) {
                    lines.push(Line::from(vec![Span::styled(
                        String::new(),
                        agent_panel_style(),
                    )]));
                } else {
                    lines.push(Line::from(""));
                }
            }
            if let Some(active_entry) = self.active_tool_entry.as_deref() {
                lines.extend(render_panel_entry(
                    None,
                    0,
                    active_entry,
                    tool_panel_style(),
                ));
                lines.push(Line::from(vec![Span::styled(
                    String::new(),
                    tool_panel_style(),
                )]));
            }
        }

        if let Some(response) = self.final_response() {
            if !self.transcript_entries.is_empty() || self.active_tool_entry.is_some() {
                lines.push(Line::from("Final Answer:"));
                lines.push(Line::from(""));
            }
            if response.trim().is_empty() {
                lines.push(Line::from("<empty response>"));
            } else {
                for line in response.lines() {
                    lines.push(Line::from(line.to_string()));
                }
            }
        }

        if lines.is_empty() {
            lines.push(Line::from("Waiting for evil_gemma..."));
        }

        Text::from(lines)
    }
}

fn render_panel_entry(
    agent_label: Option<&str>,
    depth: usize,
    content: &str,
    style: Style,
) -> Vec<Line<'static>> {
    render_padded_panel_entry(agent_label, depth, content, style, 0)
}

fn render_padded_panel_entry(
    agent_label: Option<&str>,
    depth: usize,
    content: &str,
    style: Style,
    leading_padding: usize,
) -> Vec<Line<'static>> {
    let indent = "    ".repeat(depth);
    let padding = " ".repeat(leading_padding);
    let inner_indent = format!("{indent}    {padding}");
    let body_indent = format!("{indent}{padding}");
    let mut lines = Vec::new();
    if let Some(agent_label) = agent_label {
        lines.push(Line::from(vec![Span::styled(
            format!("{indent}[agent] {agent_label}"),
            style,
        )]));
    }
    for line in content.lines() {
        let text = if line.is_empty() {
            if agent_label.is_some() {
                inner_indent.clone()
            } else {
                body_indent.clone()
            }
        } else {
            let line_indent = if agent_label.is_some() {
                inner_indent.as_str()
            } else {
                body_indent.as_str()
            };
            format!("{line_indent}{line}")
        };
        lines.push(Line::from(vec![Span::styled(text, style)]));
    }
    lines
}

fn tool_panel_style() -> Style {
    Style::default().bg(TOOL_PANEL_BG).fg(TOOL_PANEL_FG)
}

fn user_input_panel_style() -> Style {
    Style::default()
        .bg(Color::Rgb(220, 220, 220))
        .fg(Color::Rgb(16, 16, 16))
}

fn agent_panel_style() -> Style {
    Style::default().bg(AGENT_PANEL_BG).fg(AGENT_PANEL_FG)
}

pub(crate) fn compact_transcript_entries(
    entries: &[TranscriptEntry],
) -> Vec<Cow<'_, TranscriptEntry>> {
    let mut compacted: Vec<Cow<'_, TranscriptEntry>> = Vec::with_capacity(entries.len());
    for entry in entries {
        if let Some(previous) = compacted.last_mut()
            && should_replace_agent_entry(previous.as_ref(), entry)
        {
            *previous = Cow::Borrowed(entry);
            continue;
        }
        compacted.push(Cow::Borrowed(entry));
    }
    compacted
}

fn should_replace_agent_entry(previous: &TranscriptEntry, current: &TranscriptEntry) -> bool {
    previous.kind == TranscriptEntryKind::AgentEvent
        && current.kind == TranscriptEntryKind::AgentEvent
        && previous.depth == current.depth
        && previous.agent_label == current.agent_label
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compacts_consecutive_agent_updates_to_latest_status() {
        let entries = vec![
            TranscriptEntry {
                kind: TranscriptEntryKind::AgentEvent,
                content: "status: running".to_string(),
                agent_label: Some("hydrate_actor_scope: did:plc:test".to_string()),
                depth: 1,
            },
            TranscriptEntry {
                kind: TranscriptEntryKind::AgentEvent,
                content: "status: completed".to_string(),
                agent_label: Some("hydrate_actor_scope: did:plc:test".to_string()),
                depth: 1,
            },
        ];

        let compacted = compact_transcript_entries(&entries);
        assert_eq!(compacted.len(), 1);
        assert_eq!(compacted[0].content, "status: completed");
    }

    #[test]
    fn does_not_compact_non_consecutive_agent_updates() {
        let entries = vec![
            TranscriptEntry {
                kind: TranscriptEntryKind::AgentEvent,
                content: "status: running".to_string(),
                agent_label: Some("hydrate_actor_scope: did:plc:test".to_string()),
                depth: 1,
            },
            TranscriptEntry {
                kind: TranscriptEntryKind::Notice,
                content: "separator".to_string(),
                agent_label: None,
                depth: 0,
            },
            TranscriptEntry {
                kind: TranscriptEntryKind::AgentEvent,
                content: "status: completed".to_string(),
                agent_label: Some("hydrate_actor_scope: did:plc:test".to_string()),
                depth: 1,
            },
        ];

        let compacted = compact_transcript_entries(&entries);
        assert_eq!(compacted.len(), 3);
    }

    #[test]
    fn does_not_compact_consecutive_user_input_entries() {
        let entries = vec![
            TranscriptEntry {
                kind: TranscriptEntryKind::UserInput,
                content: "first prompt".to_string(),
                agent_label: None,
                depth: 0,
            },
            TranscriptEntry {
                kind: TranscriptEntryKind::UserInput,
                content: "second prompt".to_string(),
                agent_label: None,
                depth: 0,
            },
        ];

        let compacted = compact_transcript_entries(&entries);
        assert_eq!(compacted.len(), 2);
        assert_eq!(compacted[0].content, "first prompt");
        assert_eq!(compacted[1].content, "second prompt");
    }
}
