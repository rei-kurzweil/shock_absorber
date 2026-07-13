use crate::harness::units::{UnitInstanceState, UnitInstanceStatus};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

const BOX_HEIGHT: u16 = 6;
const BOX_WIDTH: u16 = 28;

pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    root: Option<&UnitInstanceState>,
    selected: usize,
    vertical_scroll: u16,
    horizontal_scroll: u16,
) {
    let outer = Block::default()
        .title("Execution Units  Tab: chat  arrows: select  PgUp/PgDn: scroll  Shift+←/→: pan")
        .borders(Borders::ALL);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    let Some(root) = root else {
        frame.render_widget(
            Paragraph::new("No unit run available yet. Run a query to inspect its live graph."),
            inner,
        );
        return;
    };
    let mut flat = Vec::new();
    flatten(root, 0, &mut flat);
    let total = flat.len();
    for (index, (unit, depth)) in flat.into_iter().enumerate() {
        let row = index as u16;
        let x = inner
            .x
            .saturating_add(depth as u16 * 4)
            .saturating_sub(horizontal_scroll);
        let y = inner
            .y
            .saturating_add(row * (BOX_HEIGHT + 1))
            .saturating_sub(vertical_scroll);
        if y + BOX_HEIGHT <= inner.y || y >= inner.y + inner.height || x >= inner.x + inner.width {
            continue;
        }
        let width = BOX_WIDTH.min(inner.x.saturating_add(inner.width).saturating_sub(x));
        if width < 8 {
            continue;
        }
        let color = status_color(unit.status);
        let borders = if index == selected {
            Borders::ALL
        } else {
            Borders::TOP | Borders::BOTTOM | Borders::LEFT | Borders::RIGHT
        };
        let block = Block::default()
            .borders(borders)
            .border_style(if index == selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            });
        let input = "● input";
        let outputs = unit
            .definition
            .graph
            .as_ref()
            .map(|g| {
                g.ports
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join("  ")
            })
            .unwrap_or_else(|| "success  failure".into());
        let text = vec![
            Line::styled(
                format!("{} [{}]", unit.instance_label, unit.kind.as_str()),
                Style::default().fg(color).add_modifier(
                    if unit.status == UnitInstanceStatus::Running {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                ),
            ),
            Line::from(format!("{input}  {}", unit.status.as_str())),
            Line::from(format!(
                "node: {}",
                unit.active_node.as_deref().unwrap_or("—")
            )),
            Line::from(format!("○ {outputs}")),
        ];
        frame.render_widget(
            Paragraph::new(text).block(block),
            Rect::new(x, y, width, BOX_HEIGHT),
        );
        if index + 1 < total && y + BOX_HEIGHT < inner.y + inner.height {
            frame.render_widget(
                Paragraph::new("│\n▼"),
                Rect::new(x + 2, y + BOX_HEIGHT, 2, 2),
            );
        }
    }
}

fn flatten<'a>(
    unit: &'a UnitInstanceState,
    depth: usize,
    out: &mut Vec<(&'a UnitInstanceState, usize)>,
) {
    out.push((unit, depth));
    for child in &unit.children {
        flatten(child, depth + 1, out);
    }
}

fn status_color(status: UnitInstanceStatus) -> Color {
    match status {
        UnitInstanceStatus::Running | UnitInstanceStatus::BlockedOnChild => Color::LightCyan,
        UnitInstanceStatus::Completed => Color::Green,
        UnitInstanceStatus::CompletedWithWarnings => Color::Yellow,
        UnitInstanceStatus::Failed => Color::Red,
        UnitInstanceStatus::Ready => Color::DarkGray,
    }
}
