use crate::harness::context_window::BuiltContextWindow;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextCategory {
    SystemPrompt,
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

pub fn render(frame: &mut Frame, area: Rect, data: &ContextVisualizationData) {
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

        let bar_height = if index == 0 { 3 } else { 1 };
        for _ in 0..bar_height {
            lines.push(context_bar_line(window, bar_width));
        }

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

    frame.render_widget(Paragraph::new(Text::from(lines)), area);
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
        ContextCategory::ToolDefinitions => Color::Blue,
        ContextCategory::UiContext => Color::Green,
        ContextCategory::CurrentTask => Color::Cyan,
        ContextCategory::UserAiChat => Color::Yellow,
        ContextCategory::ToolResults => Color::Rgb(255, 165, 0),
    }
}

fn legend_line() -> Line<'static> {
    let items = [
        ("  ", Color::Magenta, " system "),
        ("  ", Color::Blue, " tools "),
        ("  ", Color::Green, " ui "),
        ("  ", Color::Cyan, " task "),
        ("  ", Color::Yellow, " chat "),
        ("  ", Color::Rgb(255, 165, 0), " tool results "),
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
