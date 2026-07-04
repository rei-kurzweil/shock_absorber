use crate::harness::context_window::{ProviderContextLimits, approximate_tokens, render_header};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
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
}

#[derive(Clone, Debug)]
pub struct ContextVisualizationData {
    pub title: String,
    pub provider_name: String,
    pub model_name: String,
    pub max_context_tokens: usize,
    pub reserved_output_tokens: usize,
    pub input_budget_tokens: usize,
    pub used_input_tokens: usize,
    pub segments: Vec<ContextSegment>,
}

impl ContextVisualizationData {
    pub fn from_root_context(
        system_prompt: &str,
        tool_protocol: &str,
        tools_inventory: &str,
        ui_header: &str,
        search_hints: &str,
        current_ui_context: &str,
        current_task: Option<&str>,
        recent_chat: Option<&str>,
        limits: &ProviderContextLimits,
    ) -> Self {
        let mut segments = Vec::new();

        let system_prompt_text = system_prompt.trim();
        let system_tokens = approximate_tokens(system_prompt_text);
        if system_tokens > 0 {
            segments.push(ContextSegment {
                label: "System Prompt".to_string(),
                category: ContextCategory::SystemPrompt,
                tokens: system_tokens,
            });
        }

        let tool_text = format!(
            "{}\n\n## Tools\n{}\n",
            tool_protocol.trim(),
            tools_inventory.trim()
        );
        let tool_tokens = approximate_tokens(&tool_text);
        if tool_tokens > 0 {
            segments.push(ContextSegment {
                label: "Tool Definitions".to_string(),
                category: ContextCategory::ToolDefinitions,
                tokens: tool_tokens,
            });
        }

        let ui_text = format!(
            "{}\n\n## Search Hints\n{}\n\n## Current UI Context\n{}\n",
            render_header(ui_header).trim_end(),
            search_hints.trim(),
            current_ui_context.trim()
        );
        let ui_tokens = approximate_tokens(&ui_text);
        if ui_tokens > 0 {
            segments.push(ContextSegment {
                label: "UI Context".to_string(),
                category: ContextCategory::UiContext,
                tokens: ui_tokens,
            });
        }

        let task_tokens = current_task
            .map(str::trim)
            .filter(|task| !task.is_empty())
            .map(approximate_tokens)
            .unwrap_or(0);
        segments.push(ContextSegment {
            label: "Current Task".to_string(),
            category: ContextCategory::CurrentTask,
            tokens: task_tokens,
        });

        let chat_tokens = recent_chat
            .map(str::trim)
            .filter(|chat| !chat.is_empty())
            .map(approximate_tokens)
            .unwrap_or(0);
        segments.push(ContextSegment {
            label: "User / AI Chat".to_string(),
            category: ContextCategory::UserAiChat,
            tokens: chat_tokens,
        });

        segments.push(ContextSegment {
            label: "Tool Results".to_string(),
            category: ContextCategory::ToolResults,
            tokens: 0,
        });

        let used_input_tokens = segments.iter().map(|segment| segment.tokens).sum();

        Self {
            title: "/context".to_string(),
            provider_name: limits.provider_name.clone(),
            model_name: limits.model_name.clone(),
            max_context_tokens: limits.max_context_tokens,
            reserved_output_tokens: limits.reserved_output_tokens,
            input_budget_tokens: limits.available_input_tokens(),
            used_input_tokens,
            segments,
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, data: &ContextVisualizationData) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let summary = Paragraph::new(Text::from(vec![
        Line::from(data.title.as_str()),
        Line::from(format!(
            "{} / {} | used {} of {} input tokens | total {} | reserved output {}",
            data.provider_name,
            data.model_name,
            data.used_input_tokens,
            data.input_budget_tokens,
            data.max_context_tokens,
            data.reserved_output_tokens
        )),
    ]));
    frame.render_widget(summary, layout[0]);

    let bar_text = Text::from(vec![
        context_bar_line(data, layout[1].width),
        context_bar_line(data, layout[1].width),
        context_bar_line(data, layout[1].width),
    ]);
    frame.render_widget(Paragraph::new(bar_text), layout[1]);

    let footer = Paragraph::new(Text::from(vec![legend_line()]));
    frame.render_widget(footer, layout[2]);

    let details = Paragraph::new(section_details(data));
    frame.render_widget(details, layout[3]);
}

fn context_bar_line(data: &ContextVisualizationData, width: u16) -> Line<'static> {
    let width = width as usize;
    if width == 0 || data.input_budget_tokens == 0 {
        return Line::from("");
    }

    let danger_start = (width * 3) / 4;
    let mut spans = Vec::new();
    let mut current_style = None;
    let mut current_width = 0usize;

    for cell in 0..width {
        let style = style_for_cell(data, cell, width, danger_start);
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
    data: &ContextVisualizationData,
    cell: usize,
    width: usize,
    danger_start: usize,
) -> Style {
    if cell >= danger_start {
        return Style::default().bg(Color::Red);
    }

    let position_token = cell * data.input_budget_tokens / width.max(1);
    if position_token >= data.used_input_tokens {
        return Style::default().bg(Color::DarkGray);
    }

    let mut running = 0usize;
    for segment in &data.segments {
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

fn section_details(data: &ContextVisualizationData) -> Text<'static> {
    let mut lines = vec![
        Line::from("Sections"),
        Line::from(""),
    ];

    for segment in &data.segments {
        let pct = if data.input_budget_tokens == 0 {
            0.0
        } else {
            (segment.tokens as f64 / data.input_budget_tokens as f64) * 100.0
        };
        lines.push(Line::from(format!(
            "{}: {} tokens ({pct:.1}%)",
            segment.label, segment.tokens
        )));
    }

    if data.used_input_tokens > data.input_budget_tokens {
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            "Warning: estimated input exceeds budget by {} tokens.",
            data.used_input_tokens - data.input_budget_tokens
        )));
    }

    Text::from(lines)
}
