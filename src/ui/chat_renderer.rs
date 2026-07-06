use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

pub(crate) fn render_chat_detail(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    text: Text<'static>,
    scroll: u16,
) {
    let text = pad_background_lines(text, area.width.saturating_sub(2));
    let detail = Paragraph::new(text)
        .block(Block::default().title(title))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, area);
}

fn pad_background_lines(text: Text<'static>, inner_width: u16) -> Text<'static> {
    let width = inner_width as usize;
    let lines = text
        .lines
        .into_iter()
        .map(|mut line| {
            let bg_style = line.spans.iter().find_map(|span| span.style.bg.map(|_| span.style));
            if let Some(style) = bg_style {
                let used = plain_line_width(&line);
                if width > used {
                    line.spans
                        .push(Span::styled(" ".repeat(width - used), style));
                }
            }
            Line::from(line.spans)
        })
        .collect::<Vec<_>>();
    Text::from(lines)
}

fn plain_line_width(line: &Line<'_>) -> usize {
    line.spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum()
}
