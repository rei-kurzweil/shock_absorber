use crate::app::App;
use crate::harness::runtime::{RootRunState, TranscriptEntry, TranscriptEntryKind};
use crossterm::cursor::{MoveDown, MoveToColumn, MoveUp, Show};
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, Clear, ClearType};
use std::io::{Result as IoResult, Write};

pub struct StdoutChatRenderer {
    active_run_id: Option<u64>,
    rendered_entry_count: usize,
    rendered_active_tool_entry: Option<String>,
    rendered_final_response: Option<String>,
    editor_line_count: u16,
    editor_cursor_row: u16,
}

impl StdoutChatRenderer {
    pub fn new() -> Self {
        Self {
            active_run_id: None,
            rendered_entry_count: 0,
            rendered_active_tool_entry: None,
            rendered_final_response: None,
            editor_line_count: 0,
            editor_cursor_row: 0,
        }
    }

    pub fn enter<W: Write>(&mut self, writer: &mut W, app: &App) -> IoResult<()> {
        queue!(writer, Show)?;
        self.render(writer, app, true)
    }

    pub fn leave<W: Write>(&mut self, writer: &mut W) -> IoResult<()> {
        if self.editor_line_count > 0 {
            self.move_to_editor_top(writer)?;
            queue!(writer, Clear(ClearType::FromCursorDown))?;
            writer.flush()?;
        }
        self.editor_line_count = 0;
        self.editor_cursor_row = 0;
        Ok(())
    }

    pub fn sync<W: Write>(&mut self, writer: &mut W, app: &App) -> IoResult<()> {
        self.render(writer, app, false)
    }

    fn render<W: Write>(&mut self, writer: &mut W, app: &App, force_header: bool) -> IoResult<()> {
        let run = app.root_run();
        let run_id = run.map(RootRunState::run_id);
        let is_new_run = run_id != self.active_run_id;

        if self.editor_line_count > 0 {
            self.move_to_editor_top(writer)?;
            queue!(writer, Clear(ClearType::FromCursorDown))?;
        }

        if force_header || is_new_run {
            if let Some(title) = app.chat_title() {
                queue!(writer, Print(format!("\r\n=== {title} ===\r\n\r\n")))?;
            } else {
                queue!(writer, Print("\r\n=== AI Chat ===\r\n\r\n"))?;
            }
            self.rendered_entry_count = 0;
            self.rendered_active_tool_entry = None;
            self.rendered_final_response = None;
        }

        if let Some(run) = run {
            for entry in run
                .transcript_entries()
                .iter()
                .skip(self.rendered_entry_count)
            {
                write_transcript_entry(writer, entry)?;
                queue!(writer, Print("\r\n"))?;
            }

            let active_tool_entry = run.active_tool_entry().map(str::to_owned);
            if active_tool_entry != self.rendered_active_tool_entry {
                if let Some(active_tool_entry) = active_tool_entry.as_deref() {
                    write_active_tool_entry(writer, active_tool_entry)?;
                }
                self.rendered_active_tool_entry = active_tool_entry;
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
                self.rendered_final_response = final_response;
            } else if self.rendered_entry_count == 0
                && run.transcript_entries().is_empty()
                && run.final_response().is_none()
                && run.active_tool_entry().is_none()
            {
                queue!(writer, Print("Waiting for evil_gemma...\r\n\r\n"))?;
            }

            self.rendered_entry_count = run.transcript_entries().len();
        } else {
            queue!(writer, Print("No active chat transcript.\r\n\r\n"))?;
            self.rendered_entry_count = 0;
            self.rendered_active_tool_entry = None;
            self.rendered_final_response = None;
        }

        let editor_view = render_editor(app);
        for line in &editor_view.lines {
            queue!(writer, Print(line), Print("\r\n"))?;
        }
        let move_up = editor_view
            .lines
            .len()
            .saturating_sub(editor_view.cursor_row as usize) as u16;
        queue!(
            writer,
            MoveUp(move_up),
            MoveToColumn(editor_view.cursor_column)
        )?;
        writer.flush()?;

        self.active_run_id = run_id;
        self.editor_line_count = editor_view.lines.len() as u16;
        self.editor_cursor_row = editor_view.cursor_row;
        Ok(())
    }

    fn move_to_editor_top<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        let lines_below_cursor = self
            .editor_line_count
            .saturating_sub(self.editor_cursor_row + 1);
        if lines_below_cursor > 0 {
            queue!(writer, MoveDown(lines_below_cursor))?;
        }
        queue!(writer, MoveToColumn(0))?;
        if self.editor_cursor_row > 0 {
            queue!(writer, MoveUp(self.editor_cursor_row))?;
        }
        Ok(())
    }
}

struct EditorRenderView {
    lines: Vec<String>,
    cursor_row: u16,
    cursor_column: u16,
}

fn render_editor(app: &App) -> EditorRenderView {
    let width = terminal::size()
        .map(|(width, _)| width.max(20) as usize)
        .unwrap_or(80);
    let mut lines = Vec::new();
    lines.push(
        "Prompt: Enter submits | Shift+Enter or Ctrl+J inserts newline | Tab toggles views"
            .to_string(),
    );

    let mut cursor_row = 1_u16;
    let mut cursor_column = 2_u16;
    let prompt_width = width.saturating_sub(2).max(1);
    let editor_lines = app.chat_editor().lines();
    let (cursor_line, cursor_column_in_line) = app.chat_editor().cursor_line_and_column();

    for (line_index, line) in editor_lines.iter().enumerate() {
        let wrapped = wrap_line(line, prompt_width);
        if wrapped.is_empty() {
            lines.push("> ".to_string());
            if line_index == cursor_line {
                cursor_row = (lines.len() - 1) as u16;
                cursor_column = 2;
            }
            continue;
        }
        let mut consumed = 0;
        for (segment_index, segment) in wrapped.iter().enumerate() {
            lines.push(format!("> {segment}"));
            if line_index == cursor_line
                && cursor_column_in_line >= consumed
                && cursor_column_in_line <= consumed + segment.chars().count()
            {
                cursor_row = (lines.len() - 1) as u16;
                cursor_column = 2 + (cursor_column_in_line - consumed) as u16;
            }
            consumed += segment.chars().count();
            if segment_index + 1 == wrapped.len()
                && line_index == cursor_line
                && cursor_column_in_line == consumed
            {
                cursor_row = (lines.len() - 1) as u16;
                cursor_column = 2 + segment.chars().count() as u16;
            }
        }
    }

    if app.chat_editor().text().is_empty() {
        cursor_row = 1;
        cursor_column = 2;
    }

    lines.push(String::new());
    lines.push(format!("Status: {}", app.status()));

    EditorRenderView {
        lines,
        cursor_row,
        cursor_column,
    }
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

fn write_transcript_entry<W: Write>(writer: &mut W, entry: &TranscriptEntry) -> IoResult<()> {
    let indent = "    ".repeat(entry.depth);
    match entry.kind {
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
