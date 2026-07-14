use crate::app::App;
use crate::harness::runtime::{
    RootRunState, TranscriptEntry, TranscriptEntryKind, compact_transcript_entries,
};
use crate::harness::units::UnitInstanceState;
use crate::ui::units_renderer::unit_detail_lines;
use crossterm::Command;
use crossterm::cursor::{MoveTo, Show};
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use std::fmt;
use std::io::{Result as IoResult, Write};

const FOOTER_HEIGHT: u16 = 5;
const INPUT_ROWS: usize = 3;
const MIN_TERMINAL_WIDTH: u16 = 20;
const INPUT_BOX_LEFT_PADDING: usize = 4;
const INPUT_BOX_FG: Color = Color::Rgb {
    r: 16,
    g: 16,
    b: 16,
};
const INPUT_BOX_BG: Color = Color::Rgb {
    r: 220,
    g: 220,
    b: 220,
};

pub struct StdoutChatRenderer {
    active_run_id: Option<u64>,
    rendered_entry_count: usize,
    rendered_active_tool_entry: Option<String>,
    rendered_final_response: Option<String>,
    rendered_footer_view: Option<FooterRenderView>,
    terminal_layout: Option<TerminalLayout>,
}

impl StdoutChatRenderer {
    pub fn new() -> Self {
        Self {
            active_run_id: None,
            rendered_entry_count: 0,
            rendered_active_tool_entry: None,
            rendered_final_response: None,
            rendered_footer_view: None,
            terminal_layout: None,
        }
    }

    pub fn enter<W: Write>(&mut self, writer: &mut W, app: &App) -> IoResult<()> {
        queue!(writer, Show)?;
        self.render(writer, app, true)
    }

    pub fn enter_unit_detail<W: Write>(
        &mut self,
        writer: &mut W,
        app: &App,
        run: &RootRunState,
        unit: &UnitInstanceState,
    ) -> IoResult<()> {
        let layout = TerminalLayout::capture();
        let footer_view = render_footer(app, layout.width as usize, layout.fallback);
        queue!(writer, Show)?;
        self.apply_layout_change(writer, layout)?;
        self.move_to_transcript_cursor(writer, layout)?;
        queue!(
            writer,
            Print(format!(
                "\r\n=== Unit Detail · {} ===\r\n",
                unit.instance_label
            )),
            Print("Ask about this unit below, or press Escape to return to /units.\r\n\r\n")
        )?;
        for line in unit_detail_lines(run, unit) {
            let text = line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>();
            queue!(writer, Print(text), Print("\r\n"))?;
        }
        write_footer_view(writer, layout, &footer_view)?;
        writer.flush()?;
        self.rendered_footer_view = Some(footer_view);
        self.terminal_layout = Some(layout);
        Ok(())
    }

    pub fn sync_unit_detail<W: Write>(
        &mut self,
        writer: &mut W,
        app: &App,
        run: &RootRunState,
        unit: &UnitInstanceState,
    ) -> IoResult<()> {
        let layout = TerminalLayout::capture();
        if self.terminal_layout != Some(layout) {
            return self.enter_unit_detail(writer, app, run, unit);
        }
        let footer_view = render_footer(app, layout.width as usize, layout.fallback);
        if self.rendered_footer_view.as_ref() != Some(&footer_view) {
            write_footer_view(writer, layout, &footer_view)?;
            writer.flush()?;
            self.rendered_footer_view = Some(footer_view);
        }
        Ok(())
    }

    pub fn leave<W: Write>(&mut self, writer: &mut W) -> IoResult<()> {
        if let Some(layout) = self.terminal_layout {
            if !layout.fallback {
                queue!(writer, ResetScrollRegion)?;
            }
            clear_footer_area(writer, layout)?;
            queue!(
                writer,
                ResetColor,
                MoveTo(0, layout.height.saturating_sub(1)),
                Clear(ClearType::CurrentLine),
                Show
            )?;
            writer.flush()?;
        }

        self.active_run_id = None;
        self.rendered_entry_count = 0;
        self.rendered_active_tool_entry = None;
        self.rendered_final_response = None;
        self.rendered_footer_view = None;
        self.terminal_layout = None;
        Ok(())
    }

    pub fn sync<W: Write>(&mut self, writer: &mut W, app: &App) -> IoResult<()> {
        self.render(writer, app, false)
    }

    fn render<W: Write>(&mut self, writer: &mut W, app: &App, force_header: bool) -> IoResult<()> {
        let run = app.root_run();
        let run_id = run.map(RootRunState::run_id);
        let is_new_run = run_id != self.active_run_id;
        let layout = TerminalLayout::capture();
        let footer_view = render_footer(app, layout.width as usize, layout.fallback);

        let (compacted_entry_count, active_tool_entry, final_response) = if let Some(run) = run {
            let compacted_entries = compact_transcript_entries(run.transcript_entries());
            (
                compacted_entries.len(),
                run.active_tool_entry().map(str::to_owned),
                run.final_response().map(str::to_owned),
            )
        } else {
            (0, None, None)
        };

        let layout_changed = self.terminal_layout != Some(layout);
        let transcript_changed = force_header
            || is_new_run
            || compacted_entry_count != self.rendered_entry_count
            || active_tool_entry != self.rendered_active_tool_entry
            || final_response != self.rendered_final_response;
        let footer_changed = force_header
            || layout_changed
            || transcript_changed
            || self.rendered_footer_view.as_ref() != Some(&footer_view);

        if !layout_changed && !transcript_changed && !footer_changed {
            self.active_run_id = run_id;
            return Ok(());
        }

        if layout_changed {
            self.apply_layout_change(writer, layout)?;
        }

        if force_header || is_new_run {
            self.rendered_entry_count = 0;
            self.rendered_active_tool_entry = None;
            self.rendered_final_response = None;
        }

        if transcript_changed {
            self.write_transcript_updates(writer, app, run, layout, force_header || is_new_run)?;
            self.rendered_entry_count = compacted_entry_count;
            self.rendered_active_tool_entry = active_tool_entry;
            self.rendered_final_response = final_response;
        }

        if footer_changed {
            write_footer_view(writer, layout, &footer_view)?;
        }

        writer.flush()?;

        self.active_run_id = run_id;
        self.rendered_footer_view = Some(footer_view);
        self.terminal_layout = Some(layout);
        Ok(())
    }

    fn apply_layout_change<W: Write>(
        &mut self,
        writer: &mut W,
        layout: TerminalLayout,
    ) -> IoResult<()> {
        if let Some(previous) = self.terminal_layout {
            if !previous.fallback {
                queue!(writer, ResetScrollRegion)?;
            }
            clear_footer_area(writer, previous)?;
        }

        if !layout.fallback {
            queue!(
                writer,
                SetScrollRegion::new(1, layout.scroll_bottom),
                MoveTo(0, layout.scroll_bottom.saturating_sub(1))
            )?;
        }

        clear_footer_area(writer, layout)?;
        Ok(())
    }

    fn write_transcript_updates<W: Write>(
        &mut self,
        writer: &mut W,
        app: &App,
        run: Option<&RootRunState>,
        layout: TerminalLayout,
        write_header: bool,
    ) -> IoResult<()> {
        self.move_to_transcript_cursor(writer, layout)?;

        if write_header {
            if let Some(title) = app.chat_title() {
                queue!(writer, Print(format!("\r\n=== {title} ===\r\n\r\n")))?;
            } else {
                queue!(writer, Print("\r\n=== AI Chat ===\r\n\r\n"))?;
            }
        }

        if let Some(run) = run {
            let compacted_entries = compact_transcript_entries(run.transcript_entries());
            for entry in compacted_entries.iter().skip(self.rendered_entry_count) {
                write_transcript_entry(writer, entry, layout.width as usize)?;
                queue!(writer, Print("\r\n"))?;
            }

            let active_tool_entry = run.active_tool_entry().map(str::to_owned);
            if active_tool_entry != self.rendered_active_tool_entry {
                if let Some(active_tool_entry) = active_tool_entry.as_deref() {
                    write_active_tool_entry(writer, active_tool_entry)?;
                }
            }

            let final_response = run.final_response().map(str::to_owned);
            if final_response != self.rendered_final_response {
                if let Some(final_response) = final_response.as_deref() {
                    queue!(writer, Print("Final Answer:\r\n\r\n"))?;
                    if final_response.trim().is_empty() {
                        queue!(writer, Print("<empty response>\r\n\r\n"))?;
                    } else {
                        for line in final_response.lines() {
                            queue!(writer, Print(line), Print("\r\n"))?;
                        }
                        queue!(writer, Print("\r\n"))?;
                    }
                } else if run.transcript_entries().is_empty() {
                    queue!(writer, Print("Waiting for evil_gemma...\r\n\r\n"))?;
                }
            } else if self.rendered_entry_count == 0
                && run.transcript_entries().is_empty()
                && run.final_response().is_none()
                && run.active_tool_entry().is_none()
            {
                queue!(writer, Print("Waiting for evil_gemma...\r\n\r\n"))?;
            }
        } else {
            queue!(writer, Print("No active chat transcript.\r\n\r\n"))?;
        }

        Ok(())
    }

    fn move_to_transcript_cursor<W: Write>(
        &self,
        writer: &mut W,
        layout: TerminalLayout,
    ) -> IoResult<()> {
        if layout.fallback {
            queue!(
                writer,
                MoveTo(0, layout.height.saturating_sub(1)),
                Clear(ClearType::CurrentLine)
            )?;
        } else {
            queue!(writer, MoveTo(0, layout.scroll_bottom.saturating_sub(1)))?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TerminalLayout {
    width: u16,
    height: u16,
    scroll_bottom: u16,
    footer_top: u16,
    fallback: bool,
}

impl TerminalLayout {
    fn capture() -> Self {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        let width = width.max(MIN_TERMINAL_WIDTH);
        let fallback = height <= FOOTER_HEIGHT + 1;
        let scroll_bottom = if fallback {
            height.max(1)
        } else {
            height.saturating_sub(FOOTER_HEIGHT)
        };
        let footer_top = if fallback {
            height.saturating_sub(1)
        } else {
            height.saturating_sub(FOOTER_HEIGHT)
        };

        Self {
            width,
            height: height.max(1),
            scroll_bottom,
            footer_top,
            fallback,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FooterRenderLine {
    text: String,
    style: FooterLineStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FooterLineStyle {
    Blank,
    InputBox,
    Plain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FooterRenderView {
    lines: Vec<FooterRenderLine>,
    cursor_row: u16,
    cursor_column: u16,
    terminal_width: u16,
}

fn render_footer(app: &App, width: usize, fallback: bool) -> FooterRenderView {
    if fallback {
        return render_fallback_footer(app, width);
    }

    let (input_rows, cursor_row, cursor_column) = render_input_box(app, width);
    let mut lines = Vec::with_capacity(FOOTER_HEIGHT as usize);
    lines.push(FooterRenderLine {
        text: String::new(),
        style: FooterLineStyle::Blank,
    });

    for line in input_rows {
        lines.push(FooterRenderLine {
            text: line,
            style: FooterLineStyle::InputBox,
        });
    }

    lines.push(FooterRenderLine {
        text: build_status_line(app, width),
        style: FooterLineStyle::Plain,
    });

    FooterRenderView {
        lines,
        cursor_row: cursor_row + 1,
        cursor_column,
        terminal_width: width as u16,
    }
}

fn render_fallback_footer(app: &App, width: usize) -> FooterRenderView {
    let prompt_width = width.saturating_sub(2).max(1);
    let text = app
        .chat_editor()
        .lines()
        .last()
        .copied()
        .unwrap_or_default();
    let wrapped = wrap_line(text, prompt_width);
    let segment = wrapped.last().cloned().unwrap_or_default();
    let display = format!("⟩ {segment}");
    let cursor_column = 2 + segment.chars().count() as u16;

    FooterRenderView {
        lines: vec![FooterRenderLine {
            text: truncate_to_width(&display, width),
            style: FooterLineStyle::Plain,
        }],
        cursor_row: 0,
        cursor_column,
        terminal_width: width as u16,
    }
}

fn render_input_box(app: &App, width: usize) -> (Vec<String>, u16, u16) {
    let prompt_width = input_box_content_width(width);
    let editor_lines = app.chat_editor().lines();
    let (cursor_line, cursor_column_in_line) = app.chat_editor().cursor_line_and_column();

    let mut wrapped_rows = Vec::new();
    let mut cursor_row_index = 0usize;
    let mut cursor_column = INPUT_BOX_LEFT_PADDING as u16;

    for (line_index, line) in editor_lines.iter().enumerate() {
        let wrapped = wrap_line(line, prompt_width);
        let mut consumed = 0usize;
        for (segment_index, segment) in wrapped.iter().enumerate() {
            let segment_len = segment.chars().count();
            let row_index = wrapped_rows.len();
            wrapped_rows.push(format_input_box_row(segment));

            if line_index == cursor_line
                && cursor_column_in_line >= consumed
                && cursor_column_in_line <= consumed + segment_len
            {
                cursor_row_index = row_index;
                cursor_column =
                    INPUT_BOX_LEFT_PADDING as u16 + (cursor_column_in_line - consumed) as u16;
            }

            consumed += segment_len;
            if segment_index + 1 == wrapped.len()
                && line_index == cursor_line
                && cursor_column_in_line == consumed
            {
                cursor_row_index = row_index;
                cursor_column = INPUT_BOX_LEFT_PADDING as u16 + segment_len as u16;
            }
        }
    }

    if wrapped_rows.is_empty() {
        wrapped_rows.push("  ⮚ ".to_string());
        cursor_row_index = 0;
        cursor_column = INPUT_BOX_LEFT_PADDING as u16;
    }

    let visible_start = wrapped_rows.len().saturating_sub(INPUT_ROWS);
    let visible_rows = wrapped_rows[visible_start..].to_vec();
    let overflowed = wrapped_rows.len() > INPUT_ROWS;
    let mut visible_cursor_row = cursor_row_index.saturating_sub(visible_start);

    if overflowed {
        visible_cursor_row = visible_rows.len().saturating_sub(1);
        cursor_column = visible_rows
            .last()
            .map(|line| line.chars().count() as u16)
            .unwrap_or(4)
            .min(width.saturating_sub(1) as u16);
    }

    let blank_rows = INPUT_ROWS.saturating_sub(visible_rows.len());
    let top_blank_rows = blank_rows.min(1);
    let bottom_blank_rows = blank_rows - top_blank_rows;
    let mut final_rows = vec![String::new(); top_blank_rows];
    final_rows.extend(visible_rows);
    final_rows.extend(vec![String::new(); bottom_blank_rows]);

    (
        final_rows,
        (top_blank_rows + visible_cursor_row) as u16,
        cursor_column.min(width.saturating_sub(1) as u16),
    )
}

fn build_status_line(app: &App, width: usize) -> String {
    let right = "press tab to toggle modes";
    let left = if let Some(run) = app.root_run() {
        let window = run.root_context_window();
        let max_context_tokens = window.limits.max_context_tokens.max(1);
        let used_percent = ((window.used_input_tokens * 100) / max_context_tokens).min(999);
        format!(
            "{} · context {}% used · {} window",
            window.limits.model_name,
            used_percent,
            format_window_size(window.limits.max_context_tokens)
        )
    } else {
        app.status().to_string()
    };

    if width <= 2 {
        return String::new();
    }

    let inner_width = width - 2;
    let left_width = left.chars().count();
    let right_width = right.chars().count();
    let inner = if inner_width <= right_width + 1 || left_width + right_width + 1 > inner_width {
        truncate_to_width(&left, inner_width)
    } else {
        format!(
            "{left}{}{}",
            " ".repeat(inner_width - left_width - right_width),
            right
        )
    };

    format!(" {inner} ")
}

fn format_window_size(tokens: usize) -> String {
    let thousands = tokens / 1000;
    if thousands == 0 {
        format!("{tokens}")
    } else {
        format!("{thousands}K")
    }
}

fn truncate_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn clear_footer_area<W: Write>(writer: &mut W, layout: TerminalLayout) -> IoResult<()> {
    let start_row = if layout.fallback {
        layout.height.saturating_sub(1)
    } else {
        layout.footer_top
    };

    for row in start_row..layout.height {
        queue!(
            writer,
            ResetColor,
            MoveTo(0, row),
            Clear(ClearType::CurrentLine)
        )?;
    }
    Ok(())
}

fn write_footer_view<W: Write>(
    writer: &mut W,
    layout: TerminalLayout,
    footer_view: &FooterRenderView,
) -> IoResult<()> {
    let start_row = if layout.fallback {
        layout.height.saturating_sub(1)
    } else {
        layout.footer_top
    };

    for (index, line) in footer_view.lines.iter().enumerate() {
        let row = start_row + index as u16;
        queue!(writer, MoveTo(0, row), Clear(ClearType::CurrentLine))?;
        match line.style {
            FooterLineStyle::Blank => {}
            FooterLineStyle::InputBox => {
                write_input_box_styled_line(
                    writer,
                    &line.text,
                    footer_view.terminal_width as usize,
                )?;
            }
            FooterLineStyle::Plain => {
                queue!(
                    writer,
                    ResetColor,
                    Print(truncate_to_width(
                        &line.text,
                        footer_view.terminal_width as usize
                    ))
                )?;
            }
        }
    }

    let cursor_row = start_row + footer_view.cursor_row;
    let cursor_column = footer_view
        .cursor_column
        .min(footer_view.terminal_width.saturating_sub(1));
    queue!(writer, MoveTo(cursor_column, cursor_row), Show)?;
    Ok(())
}

fn pad_line_to_width(text: &str, width: usize) -> String {
    let mut out = text.chars().take(width).collect::<String>();
    let visible_width = out.chars().count();
    if visible_width < width {
        out.push_str(&" ".repeat(width - visible_width));
    }
    out
}

fn wrap_line(line: &str, width: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = String::new();
    for ch in line.chars() {
        if current.chars().count() >= width {
            wrapped.push(current);
            current = String::new();
        }
        current.push(ch);
    }
    wrapped.push(current);
    wrapped
}

fn input_box_content_width(width: usize) -> usize {
    width.saturating_sub(INPUT_BOX_LEFT_PADDING).max(1)
}

fn format_input_box_row(segment: &str) -> String {
    format!("{}{}", " ".repeat(INPUT_BOX_LEFT_PADDING), segment)
}

fn render_user_input_echo_rows(content: &str, width: usize) -> Vec<String> {
    let mut rows = Vec::new();
    for line in content.lines() {
        for segment in wrap_line(line, input_box_content_width(width)) {
            rows.push(format_input_box_row(&segment));
        }
    }
    if rows.is_empty() {
        rows.push(format_input_box_row(""));
    }
    rows
}

fn write_input_box_styled_line<W: Write>(writer: &mut W, text: &str, width: usize) -> IoResult<()> {
    queue!(
        writer,
        SetForegroundColor(INPUT_BOX_FG),
        SetBackgroundColor(INPUT_BOX_BG),
        Print(pad_line_to_width(text, width)),
        ResetColor
    )
}

fn write_transcript_entry<W: Write>(
    writer: &mut W,
    entry: &TranscriptEntry,
    terminal_width: usize,
) -> IoResult<()> {
    let indent = "    ".repeat(entry.depth);
    match entry.kind {
        TranscriptEntryKind::UserInput => {
            for line in render_user_input_echo_rows(&entry.content, terminal_width) {
                write_input_box_styled_line(writer, &line, terminal_width)?;
                queue!(writer, Print("\r\n"))?;
            }
        }
        TranscriptEntryKind::ToolCall => {
            if let Some(agent_label) = entry.agent_label.as_deref() {
                queue!(writer, Print(format!("{indent}[agent] {agent_label}\r\n")))?;
            }
            let body_indent = if entry.agent_label.is_some() {
                format!("{indent}    ")
            } else {
                indent.clone()
            };
            for line in entry.content.lines() {
                if line.is_empty() {
                    queue!(writer, Print("\r\n"))?;
                } else {
                    queue!(writer, Print(format!("{body_indent}{line}\r\n")))?;
                }
            }
        }
        TranscriptEntryKind::AgentEvent => {
            queue!(
                writer,
                SetForegroundColor(Color::Rgb {
                    r: 210,
                    g: 210,
                    b: 210
                }),
                SetBackgroundColor(Color::Rgb {
                    r: 70,
                    g: 70,
                    b: 70
                })
            )?;
            if let Some(agent_label) = entry.agent_label.as_deref() {
                queue!(writer, Print(format!("{indent}[agent] {agent_label}\r\n")))?;
            }
            let body_indent = if entry.agent_label.is_some() {
                format!("{indent}    ")
            } else {
                indent.clone()
            };
            for line in entry.content.lines() {
                if line.is_empty() {
                    queue!(writer, Print("\r\n"))?;
                } else {
                    queue!(writer, Print(format!("{body_indent}{line}\r\n")))?;
                }
            }
            queue!(writer, ResetColor)?;
        }
        TranscriptEntryKind::Notice => {
            for line in entry.content.lines() {
                queue!(writer, Print(line), Print("\r\n"))?;
            }
        }
    }
    Ok(())
}

fn write_active_tool_entry<W: Write>(writer: &mut W, entry: &str) -> IoResult<()> {
    for line in entry.lines() {
        queue!(writer, Print(line), Print("\r\n"))?;
    }
    queue!(writer, Print("\r\n"))?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SetScrollRegion {
    top: u16,
    bottom: u16,
}

impl SetScrollRegion {
    fn new(top: u16, bottom: u16) -> Self {
        Self { top, bottom }
    }
}

impl Command for SetScrollRegion {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(f, "\u{1b}[{};{}r", self.top, self.bottom)
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResetScrollRegion;

impl Command for ResetScrollRegion {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        f.write_str("\u{1b}[r")
    }

    #[cfg(windows)]
    fn execute_winapi(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{INPUT_ROWS, TerminalLayout, render_user_input_echo_rows, wrap_line};

    #[test]
    fn wrap_line_preserves_empty_line() {
        assert_eq!(wrap_line("", 8), vec![String::new()]);
    }

    #[test]
    fn wrap_line_wraps_at_exact_width() {
        assert_eq!(
            wrap_line("abcdef", 3),
            vec!["abc".to_string(), "def".to_string()]
        );
    }

    #[test]
    fn short_terminal_uses_fallback_layout() {
        let layout = TerminalLayout {
            width: 80,
            height: 5,
            scroll_bottom: 5,
            footer_top: 4,
            fallback: true,
        };
        assert!(layout.fallback);
        assert_eq!(layout.footer_top, 4);
    }

    #[test]
    fn fixed_input_row_count_stays_at_three() {
        assert_eq!(INPUT_ROWS, 3);
    }

    #[test]
    fn multiline_user_input_echo_preserves_line_breaks_and_padding() {
        let rendered = render_user_input_echo_rows("alpha\nbeta gamma", 12);
        assert_eq!(
            rendered,
            vec![
                "    alpha".to_string(),
                "    beta gam".to_string(),
                "    ma".to_string(),
            ]
        );
    }
}
