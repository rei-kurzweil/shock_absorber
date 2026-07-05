use crate::harness::agents::{AgentNode, AgentNodeKind};
use crate::harness::context_window::BuiltContextWindow;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextCategory {
    SystemPrompt,
    ToolInstructions,
    RootInstructions,
    ToolDefinitions,
    UiContext,
    CurrentTask,
    UserAiChat,
    ToolResults,
}

#[derive(Clone, Debug)]
pub struct ContextSegment {
    pub label: String,
    pub category: ContextCategory,
    pub tokens: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug)]
pub struct PromptContextSnapshot {
    pub title: String,
    pub provider_name: String,
    pub model_name: String,
    pub max_context_tokens: usize,
    pub reserved_output_tokens: usize,
    pub input_budget_tokens: usize,
    pub used_input_tokens: usize,
    pub truncated: bool,
    pub segments: Vec<ContextSegment>,
}

#[derive(Clone, Debug)]
pub struct ContextVisualizationData {
    pub title: String,
    pub windows: Vec<PromptContextSnapshot>,
}

impl ContextVisualizationData {
    pub fn from_windows(
        title: impl Into<String>,
        windows: Vec<PromptContextSnapshot>,
    ) -> Self {
        Self {
            title: title.into(),
            windows,
        }
    }

    pub fn from_root_window(title: impl Into<String>, window: &BuiltContextWindow) -> Self {
        Self::from_windows(title, vec![snapshot_from_root_window(window)])
    }
}

pub fn snapshot_from_root_window(window: &BuiltContextWindow) -> PromptContextSnapshot {
    let mut segments = vec![ContextSegment {
        label: "System Prompt".to_string(),
        category: ContextCategory::SystemPrompt,
        tokens: window.header_tokens,
        truncated: false,
    }];

    for section in &window.sections {
        segments.push(ContextSegment {
            label: section.title.clone(),
            category: category_for_root_section(&section.title),
            tokens: section.used_tokens,
            truncated: section.truncated,
        });
    }

    snapshot_from_window("Root Agent", window, segments)
}

pub fn snapshot_from_llm_search_window(
    title: impl Into<String>,
    window: &BuiltContextWindow,
) -> PromptContextSnapshot {
    let mut segments = vec![ContextSegment {
        label: "Search Instructions".to_string(),
        category: ContextCategory::SystemPrompt,
        tokens: window.header_tokens,
        truncated: false,
    }];

    for section in &window.sections {
        let category = match section.title.as_str() {
            "Collection" => ContextCategory::ToolResults,
            "Search Prompt" => ContextCategory::CurrentTask,
            _ => ContextCategory::UiContext,
        };
        segments.push(ContextSegment {
            label: section.title.clone(),
            category,
            tokens: section.used_tokens,
            truncated: section.truncated,
        });
    }

    snapshot_from_window(title, window, segments)
}

pub fn snapshot_from_agent_node(
    node: &AgentNode,
    depth: usize,
) -> Option<PromptContextSnapshot> {
    let title = format!(
        "{}{} [{}]",
        "  ".repeat(depth),
        node.label,
        node.status.as_str()
    );
    match node.agent_type {
        AgentNodeKind::ToolAgent => node
            .context_window_report
            .as_ref()
            .map(|window| snapshot_from_tool_agent_window(title, window)),
        AgentNodeKind::CollectionSearchAgent => node
            .context_window_report
            .as_ref()
            .map(|window| snapshot_from_llm_search_window(title, window)),
        AgentNodeKind::RootAgent => None,
    }
}

fn snapshot_from_tool_agent_window(
    title: impl Into<String>,
    window: &BuiltContextWindow,
) -> PromptContextSnapshot {
    let mut segments = vec![ContextSegment {
        label: "Tool Instructions".to_string(),
        category: ContextCategory::SystemPrompt,
        tokens: window.header_tokens,
        truncated: false,
    }];

    for section in &window.sections {
        let category = match section.title.as_str() {
            "Original Search Prompt" => ContextCategory::CurrentTask,
            "Per-Collection Results" => ContextCategory::ToolResults,
            _ => ContextCategory::UiContext,
        };
        segments.push(ContextSegment {
            label: section.title.clone(),
            category,
            tokens: section.used_tokens,
            truncated: section.truncated,
        });
    }

    snapshot_from_window(title, window, segments)
}

fn snapshot_from_window(
    title: impl Into<String>,
    window: &BuiltContextWindow,
    segments: Vec<ContextSegment>,
) -> PromptContextSnapshot {
    PromptContextSnapshot {
        title: title.into(),
        provider_name: window.limits.provider_name.clone(),
        model_name: window.limits.model_name.clone(),
        max_context_tokens: window.limits.max_context_tokens,
        reserved_output_tokens: window.limits.reserved_output_tokens,
        input_budget_tokens: window.limits.available_input_tokens(),
        used_input_tokens: window.used_input_tokens,
        truncated: window.truncated,
        segments,
    }
}

pub fn render(frame: &mut Frame, area: Rect, data: &ContextVisualizationData, scroll: u16) {
    let bar_width = area.width.saturating_sub(2);
    let mut lines = vec![
        Line::from(data.title.as_str()),
        Line::from(""),
    ];

    for (index, window) in data.windows.iter().enumerate() {
        lines.push(Line::from(format!(
            "{} | {} / {} | used {} of {} input tokens | total {} | reserved output {}{}",
            window.title,
            window.provider_name,
            window.model_name,
            window.used_input_tokens,
            window.input_budget_tokens,
            window.max_context_tokens,
            window.reserved_output_tokens,
            if window.truncated { " | truncated" } else { "" }
        )));

        let bar_height = if index == 0 { 2 } else { 1 };
        for _ in 0..bar_height {
            lines.push(context_bar_line(window, bar_width));
        }

        lines.push(Line::from(category_totals_summary(window)));

        if index == 0 {
            for segment in &window.segments {
                let pct = if window.input_budget_tokens == 0 {
                    0.0
                } else {
                    (segment.tokens as f64 / window.input_budget_tokens as f64) * 100.0
                };
                let suffix = if segment.truncated { " (trimmed)" } else { "" };
                lines.push(Line::from(format!(
                    "{}: {} tokens ({pct:.1}%){}",
                    segment.label, segment.tokens, suffix
                )));
            }
        } else {
            lines.push(Line::from(compact_segment_summary(window)));
        }

        if window.used_input_tokens > window.input_budget_tokens {
            lines.push(Line::from(format!(
                "Warning: estimated input exceeds budget by {} tokens.",
                window.used_input_tokens - window.input_budget_tokens
            )));
        }

        if index + 1 < data.windows.len() {
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));
    lines.push(legend_line());

    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .scroll((scroll, 0)),
        area,
    );
}

fn compact_segment_summary(window: &PromptContextSnapshot) -> String {
    window
        .segments
        .iter()
        .map(|segment| {
            let suffix = if segment.truncated { "*" } else { "" };
            format!("{} {}{}", segment.label, segment.tokens, suffix)
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn category_totals_summary(window: &PromptContextSnapshot) -> String {
    let mut totals = [0usize; 6];
    let mut order = Vec::new();
    for segment in &window.segments {
        let index = display_category_index(&segment.category);
        totals[index] += segment.tokens;
        if !order.contains(&index) {
            order.push(index);
        }
    }

    order
        .into_iter()
        .map(|index| format!("{} {}", category_label(index), totals[index]))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn category_label(index: usize) -> &'static str {
    match index {
        0 => "System Prompt",
        1 => "Tools",
        2 => "UI",
        3 => "Task",
        4 => "Chat",
        5 => "Tool Output",
        _ => "Unknown",
    }
}

fn context_bar_line(window: &PromptContextSnapshot, width: u16) -> Line<'static> {
    let width = width as usize;
    if width == 0 || window.input_budget_tokens == 0 {
        return Line::from("");
    }

    let danger_start = (width * 3) / 4;
    let mut spans = Vec::new();
    let mut current_style = None;
    let mut current_width = 0usize;

    for cell in 0..width {
        let style = style_for_cell(window, cell, width, danger_start);
        if current_style == Some(style) {
            current_width += 1;
        } else {
            if let Some(style) = current_style {
                spans.push(Span::styled(" ".repeat(current_width), style));
            }
            current_style = Some(style);
            current_width = 1;
        }
    }

    if let Some(style) = current_style {
        spans.push(Span::styled(" ".repeat(current_width), style));
    }

    Line::from(spans)
}

fn style_for_cell(
    window: &PromptContextSnapshot,
    cell: usize,
    width: usize,
    danger_start: usize,
) -> Style {
    if cell >= danger_start {
        return Style::default().bg(Color::Red);
    }

    let position_token = cell * window.input_budget_tokens / width.max(1);
    if position_token >= window.used_input_tokens {
        return Style::default().bg(Color::DarkGray);
    }

    let mut running = 0usize;
    for segment in &window.segments {
        running += segment.tokens;
        if position_token < running {
            return Style::default().bg(color_for_category(&segment.category));
        }
    }

    Style::default().bg(Color::DarkGray)
}

fn color_for_category(category: &ContextCategory) -> Color {
    match category {
        ContextCategory::SystemPrompt => Color::Magenta,
        ContextCategory::ToolInstructions
        | ContextCategory::RootInstructions
        | ContextCategory::ToolDefinitions => Color::Blue,
        ContextCategory::UiContext => Color::Green,
        ContextCategory::CurrentTask => Color::Cyan,
        ContextCategory::UserAiChat => Color::Yellow,
        ContextCategory::ToolResults => Color::Rgb(240, 210, 170),
    }
}

fn display_category_index(category: &ContextCategory) -> usize {
    match category {
        ContextCategory::SystemPrompt => 0,
        ContextCategory::ToolInstructions
        | ContextCategory::RootInstructions
        | ContextCategory::ToolDefinitions => 1,
        ContextCategory::UiContext => 2,
        ContextCategory::CurrentTask => 3,
        ContextCategory::UserAiChat => 4,
        ContextCategory::ToolResults => 5,
    }
}

fn legend_line() -> Line<'static> {
    let items = [
        ("  ", Color::Magenta, " system "),
        ("  ", Color::Blue, " tools "),
        ("  ", Color::Green, " ui "),
        ("  ", Color::Cyan, " task "),
        ("  ", Color::Yellow, " chat "),
        ("  ", Color::Rgb(240, 210, 170), " tool output "),
        ("  ", Color::Red, " final 25% "),
        ("  ", Color::DarkGray, " unused "),
    ];

    let mut spans = Vec::new();
    for (block, color, label) in items {
        spans.push(Span::styled(block, Style::default().bg(color)));
        spans.push(Span::raw(label));
    }

    Line::from(spans)
}

fn category_for_root_section(title: &str) -> ContextCategory {
    match title {
        "Tools" => ContextCategory::ToolDefinitions,
        "Search Hints" | "Current UI Context" => ContextCategory::UiContext,
        "Current Task" => ContextCategory::CurrentTask,
        "Recent Chat" => ContextCategory::UserAiChat,
        title if title.starts_with("Tool Result") => ContextCategory::ToolResults,
        _ => {
            let lowered = title.to_ascii_lowercase();
            if lowered.contains("tool") {
                ContextCategory::ToolDefinitions
            } else if lowered.contains("chat") {
                ContextCategory::UserAiChat
            } else if lowered.contains("task") || lowered.contains("prompt") {
                ContextCategory::CurrentTask
            } else if lowered.contains("result") || lowered.contains("collection") {
                ContextCategory::ToolResults
            } else {
                ContextCategory::UiContext
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{category_totals_summary, ContextCategory, ContextSegment, PromptContextSnapshot};

    #[test]
    fn category_totals_follow_first_segment_order() {
        let window = PromptContextSnapshot {
            title: "Root Agent".to_string(),
            provider_name: "test".to_string(),
            model_name: "test".to_string(),
            max_context_tokens: 1000,
            reserved_output_tokens: 100,
            input_budget_tokens: 900,
            used_input_tokens: 60,
            truncated: false,
            segments: vec![
                ContextSegment {
                    label: "System Prompt".to_string(),
                    category: ContextCategory::SystemPrompt,
                    tokens: 10,
                    truncated: false,
                },
                ContextSegment {
                    label: "Tool Result #1".to_string(),
                    category: ContextCategory::ToolResults,
                    tokens: 20,
                    truncated: false,
                },
                ContextSegment {
                    label: "Assistant Reply".to_string(),
                    category: ContextCategory::UserAiChat,
                    tokens: 30,
                    truncated: false,
                },
            ],
        };

        assert_eq!(
            category_totals_summary(&window),
            "System Prompt 10 | Tool Output 20 | Chat 30"
        );
    }
}
