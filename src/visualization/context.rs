use crate::harness::agents::{AgentNode, AgentNodeKind};
use crate::harness::context_window::{BuiltContextSection, BuiltContextWindow, ContextSectionKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextCategory {
    SystemPrompt,
    ToolInstructions,
    RootInstructions,
    ToolDefinitions,
    UiContext,
    LocalTask,
    UserAiChat,
    ToolResults,
    CollectionEvidence,
    ReviewEvidence,
    ParentSearchResults,
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
    pub fn from_windows(title: impl Into<String>, windows: Vec<PromptContextSnapshot>) -> Self {
        Self {
            title: title.into(),
            windows,
        }
    }
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
        segments.push(ContextSegment {
            label: section.title.clone(),
            category: category_for_built_section(section),
            tokens: section.used_tokens,
            truncated: section.truncated,
        });
    }

    snapshot_from_window(title, window, segments)
}

pub fn snapshot_from_agent_node(node: &AgentNode, depth: usize) -> Option<PromptContextSnapshot> {
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
        AgentNodeKind::CollectionSearchTool | AgentNodeKind::CollectionSummaryTool => node
            .context_window_report
            .as_ref()
            .map(|window| snapshot_from_llm_search_window(title, window)),
        AgentNodeKind::SearchReviewAgent | AgentNodeKind::SummaryReviewAgent => node
            .context_window_report
            .as_ref()
            .map(|window| snapshot_from_tool_agent_window(title, window)),
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
        segments.push(ContextSegment {
            label: section.title.clone(),
            category: category_for_built_section(section),
            tokens: section.used_tokens,
            truncated: section.truncated,
        });
    }

    snapshot_from_window(title, window, segments)
}

fn category_for_built_section(section: &BuiltContextSection) -> ContextCategory {
    match section.kind {
        ContextSectionKind::Generic => match section.title.as_str() {
            "Collection" => ContextCategory::CollectionEvidence,
            "Search Prompt" | "Original Search Query" => ContextCategory::LocalTask,
            "Collection Evidence" => ContextCategory::ReviewEvidence,
            "Proposed Summary" | "Per-Collection Results" => ContextCategory::ParentSearchResults,
            "Tools" => ContextCategory::ToolDefinitions,
            "Search Hints" | "Current UI Context" => ContextCategory::UiContext,
            "Current Task" => ContextCategory::LocalTask,
            "Recent Chat" => ContextCategory::UserAiChat,
            _ => ContextCategory::UiContext,
        },
        ContextSectionKind::ToolDefinitions => ContextCategory::ToolDefinitions,
        ContextSectionKind::UiContext => ContextCategory::UiContext,
        ContextSectionKind::CurrentTask => ContextCategory::LocalTask,
        ContextSectionKind::UserAiChat => ContextCategory::UserAiChat,
        ContextSectionKind::CollectionEvidence => ContextCategory::CollectionEvidence,
        ContextSectionKind::ReviewEvidence => ContextCategory::ReviewEvidence,
        ContextSectionKind::ParentSearchResults => ContextCategory::ParentSearchResults,
    }
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
    let mut lines = vec![Line::from(data.title.as_str()), Line::from("")];

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
            lines.push(legend_line());
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

    frame.render_widget(Paragraph::new(Text::from(lines)).scroll((scroll, 0)), area);
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
    let mut totals = [0usize; 10];
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
        1 => "Tool Protocol",
        2 => "Tool Definitions",
        3 => "UI",
        4 => "Task",
        5 => "Chat",
        6 => "Tool Output",
        7 => "Collection Evidence",
        8 => "Review Evidence",
        9 => "Parent Search Results",
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
        return Style::default().bg(Color::Rgb(80, 80, 80));
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
        ContextCategory::ToolInstructions | ContextCategory::RootInstructions => Color::Blue,
        ContextCategory::ToolDefinitions => Color::LightBlue,
        ContextCategory::UiContext => Color::Green,
        ContextCategory::LocalTask => Color::Cyan,
        ContextCategory::UserAiChat => Color::Rgb(245, 215, 95),
        ContextCategory::ToolResults => Color::Rgb(242, 186, 92),
        ContextCategory::CollectionEvidence => Color::Rgb(226, 128, 168),
        ContextCategory::ReviewEvidence => Color::Rgb(244, 176, 205),
        ContextCategory::ParentSearchResults => Color::Rgb(215, 170, 120),
    }
}

fn display_category_index(category: &ContextCategory) -> usize {
    match category {
        ContextCategory::SystemPrompt => 0,
        ContextCategory::ToolInstructions | ContextCategory::RootInstructions => 1,
        ContextCategory::ToolDefinitions => 2,
        ContextCategory::UiContext => 3,
        ContextCategory::LocalTask => 4,
        ContextCategory::UserAiChat => 5,
        ContextCategory::ToolResults => 6,
        ContextCategory::CollectionEvidence => 7,
        ContextCategory::ReviewEvidence => 8,
        ContextCategory::ParentSearchResults => 9,
    }
}

fn legend_line() -> Line<'static> {
    let items = [
        ("  ", Color::Magenta, " system "),
        ("  ", Color::Blue, " tool protocol "),
        ("  ", Color::LightBlue, " tool defs "),
        ("  ", Color::Green, " ui "),
        ("  ", Color::Cyan, " task "),
        ("  ", Color::Rgb(245, 215, 95), " chat "),
        ("  ", Color::Rgb(242, 186, 92), " tool output "),
        ("  ", Color::Rgb(226, 128, 168), " collected data "),
        ("  ", Color::Rgb(244, 176, 205), " review "),
        ("  ", Color::Rgb(215, 170, 120), " parent results "),
        ("  ", Color::Rgb(80, 80, 80), " final 25% "),
        ("  ", Color::DarkGray, " unused "),
    ];

    let mut spans = Vec::new();
    for (block, color, label) in items {
        spans.push(Span::styled(block, Style::default().bg(color)));
        spans.push(Span::raw(label));
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::{ContextCategory, ContextSegment, PromptContextSnapshot, category_totals_summary};

    #[test]
    fn category_totals_separate_tool_protocol_from_tool_definitions() {
        let window = PromptContextSnapshot {
            title: "Root Agent".to_string(),
            provider_name: "test".to_string(),
            model_name: "test".to_string(),
            max_context_tokens: 1000,
            reserved_output_tokens: 100,
            input_budget_tokens: 900,
            used_input_tokens: 54,
            truncated: false,
            segments: vec![
                ContextSegment {
                    label: "Tool Instructions".to_string(),
                    category: ContextCategory::ToolInstructions,
                    tokens: 20,
                    truncated: false,
                },
                ContextSegment {
                    label: "Tools".to_string(),
                    category: ContextCategory::ToolDefinitions,
                    tokens: 12,
                    truncated: false,
                },
                ContextSegment {
                    label: "Tool Result #1".to_string(),
                    category: ContextCategory::ToolResults,
                    tokens: 22,
                    truncated: false,
                },
            ],
        };

        assert_eq!(
            category_totals_summary(&window),
            "Tool Protocol 20 | Tool Definitions 12 | Tool Output 22"
        );
    }

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
