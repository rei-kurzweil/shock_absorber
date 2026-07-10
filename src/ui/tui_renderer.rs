use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, List, ListItem, ListState, Padding, Paragraph, Wrap};

use crate::visualization::context::ContextVisualizationData;

pub(crate) fn layout(frame: &mut Frame) -> (Vec<Rect>, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());
    let input_area = chunks[1];
    let status_area = chunks[2];
    (chunks.to_vec(), input_area, status_area)
}

pub(crate) fn render_fullscreen_text(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    text: Text<'static>,
    scroll: u16,
) {
    let detail = Paragraph::new(text)
        .block(Block::default().title(title))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, area);
}

pub(crate) fn render_context_visualization(
    frame: &mut Frame,
    area: Rect,
    data: &ContextVisualizationData,
    scroll: u16,
) {
    crate::visualization::context::render(frame, area, data, scroll);
}

pub(crate) fn render_list_detail_split(
    frame: &mut Frame,
    area: Rect,
    list_title: &str,
    items: Vec<ListItem<'_>>,
    selected: Option<usize>,
    detail_title: &str,
    detail_text: Text<'static>,
    detail_scroll: u16,
) {
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let notifications = List::new(items)
        .block(Block::default().title(list_title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(selected);
    frame.render_stateful_widget(notifications, body[0], &mut list_state);

    let detail = Paragraph::new(detail_text)
        .block(Block::default().title(detail_title))
        .scroll((detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, body[1]);
}

pub(crate) fn render_input(frame: &mut Frame, area: Rect, input: &str) {
    let input = Paragraph::new(input)
        .block(
            Block::default()
                .title("Command")
                .padding(Padding::symmetric(1, 0))
                .style(
                    Style::default()
                        .bg(Color::Rgb(220, 220, 220))
                        .fg(Color::Black),
                ),
        )
        .style(
            Style::default()
                .bg(Color::Rgb(220, 220, 220))
                .fg(Color::Black),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(input, area);
}

pub(crate) fn render_status(frame: &mut Frame, area: Rect, status: &str) {
    let status = Paragraph::new(status.to_string()).style(Style::default());
    frame.render_widget(status, area);
}
