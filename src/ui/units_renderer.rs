use crate::harness::runtime::RootRunState;
use crate::harness::units::{UnitInstanceState, UnitInstanceStatus};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

const BOX_WIDTH: u16 = 38;
const CARD_GAP: u16 = 1;
const LOOP_LANE_GAP: u16 = 1;
const DEPTH_INDENT: u16 = 3;

struct PositionedUnit<'a> {
    run: &'a RootRunState,
    unit: &'a UnitInstanceState,
    index: usize,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    runs: &[&RootRunState],
    selected: usize,
    _vertical_scroll: u16,
    horizontal_scroll: u16,
) {
    let outer = Block::default()
        .title("Execution Units  ↑/↓: select  PgUp/PgDn: scroll  Shift+←/→: pan")
        .borders(Borders::ALL);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    if runs.is_empty() {
        frame.render_widget(
            Paragraph::new("No unit run available yet. Run a query to inspect its live graph."),
            inner,
        );
        return;
    }

    let mut positioned = Vec::new();
    let mut next_index = 0usize;
    let mut next_run_y = 0u16;
    for run in runs {
        next_run_y = layout_unit(
            run,
            run.root_unit(),
            0,
            next_run_y,
            &mut next_index,
            &mut positioned,
        )
        .saturating_add(CARD_GAP + 1);
    }
    let selected_position = positioned
        .iter()
        .find(|entry| entry.index == selected)
        .or_else(|| positioned.last());
    let vertical_offset = selected_position
        .map(|entry| {
            entry
                .y
                .saturating_sub(inner.height.saturating_sub(entry.height))
        })
        .unwrap_or(0);
    let automatic_horizontal_offset = selected_position
        .map(|entry| {
            entry
                .x
                .saturating_sub(inner.width.saturating_sub(entry.width))
        })
        .unwrap_or(0);
    let horizontal_offset = horizontal_scroll.max(automatic_horizontal_offset);

    for entry in &positioned {
        let x = inner
            .x
            .saturating_add(entry.x)
            .saturating_sub(horizontal_offset);
        let y = inner
            .y
            .saturating_add(entry.y)
            .saturating_sub(vertical_offset);
        if y + entry.height <= inner.y || y >= inner.y + inner.height || x >= inner.x + inner.width
        {
            continue;
        }
        let width = entry
            .width
            .min(inner.x.saturating_add(inner.width).saturating_sub(x));
        if width < 8 {
            continue;
        }
        let color = status_color(entry.unit.status);
        let block =
            Block::default()
                .borders(Borders::ALL)
                .border_style(if entry.index == selected {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                });
        let label = if entry.unit.kind == crate::harness::units::UnitKind::Root {
            format!("run {} · {}", entry.run.run_id(), entry.unit.instance_label)
        } else {
            entry.unit.instance_label.clone()
        };
        let mut text = wrap_label(&label, width.saturating_sub(2) as usize)
            .into_iter()
            .map(|line| Line::styled(line, Style::default().fg(color)))
            .collect::<Vec<_>>();
        text.push(Line::from(format!("● {}", entry.unit.status.as_str())));
        frame.render_widget(
            Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
            Rect::new(x, y, width, entry.height),
        );
    }
}

pub(crate) fn render_detail(
    frame: &mut Frame,
    area: Rect,
    run: &RootRunState,
    unit: &UnitInstanceState,
    vertical_scroll: u16,
) {
    let title = format!("Unit Detail · {} · Esc: back", unit.instance_label);
    frame.render_widget(
        Paragraph::new(unit_detail_lines(run, unit))
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .scroll((vertical_scroll, 0)),
        area,
    );
}

fn wrap_label(label: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let characters = label.chars().collect::<Vec<_>>();
    characters
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

fn layout_unit<'a>(
    run: &'a RootRunState,
    unit: &'a UnitInstanceState,
    x: u16,
    y: u16,
    next_index: &mut usize,
    out: &mut Vec<PositionedUnit<'a>>,
) -> u16 {
    let index = *next_index;
    *next_index += 1;
    let width = BOX_WIDTH;
    let label = if unit.kind == crate::harness::units::UnitKind::Root {
        format!("run {} · {}", run.run_id(), unit.instance_label)
    } else {
        unit.instance_label.clone()
    };
    let height = wrap_label(&label, BOX_WIDTH.saturating_sub(2) as usize)
        .len()
        .saturating_add(3)
        .try_into()
        .unwrap_or(u16::MAX);
    out.push(PositionedUnit {
        run,
        unit,
        index,
        x,
        y,
        width,
        height,
    });

    let mut bottom = y.saturating_add(height);
    if unit.definition.graph.is_some() {
        let child_x = x.saturating_add(width).saturating_add(LOOP_LANE_GAP);
        let mut child_y = y;
        for child in &unit.children {
            let child_bottom = layout_unit(run, child, child_x, child_y, next_index, out);
            bottom = bottom.max(child_bottom);
            child_y = child_bottom.saturating_add(CARD_GAP);
        }
    } else {
        let child_x = if unit.kind == crate::harness::units::UnitKind::Root {
            x
        } else {
            x.saturating_add(DEPTH_INDENT)
        };
        let mut child_y = bottom.saturating_add(CARD_GAP);
        for child in &unit.children {
            let child_bottom = layout_unit(run, child, child_x, child_y, next_index, out);
            bottom = bottom.max(child_bottom);
            child_y = child_bottom.saturating_add(CARD_GAP);
        }
    }
    bottom
}

pub(crate) fn unit_detail_lines(
    run: &RootRunState,
    unit: &UnitInstanceState,
) -> Vec<Line<'static>> {
    let value = |value: Option<&str>| value.unwrap_or("<none>").to_string();
    let mut lines = vec![
        Line::from(format!("run: {}", run.run_id())),
        Line::from(format!("query: {}", run.query())),
        Line::from(""),
        Line::from(format!("instance_id: {}", unit.instance_id)),
        Line::from(format!("definition_id: {}", unit.definition.id)),
        Line::from(format!("label: {}", unit.instance_label)),
        Line::from(format!("kind: {}", unit.kind.as_str())),
        Line::from(format!("status: {}", unit.status.as_str())),
        Line::from(format!(
            "active_node: {}",
            value(unit.active_node.as_deref())
        )),
        Line::from(format!("visit_count: {}", unit.visit_count)),
        Line::from(format!(
            "visited_nodes: {}",
            if unit.visited_nodes.is_empty() {
                "<none>".to_string()
            } else {
                unit.visited_nodes.join(", ")
            }
        )),
        Line::from(format!(
            "last_output_port: {}",
            value(unit.last_output_port.as_deref())
        )),
        Line::from(format!(
            "blocked_on_child: {}",
            value(unit.blocked_on_child.as_deref())
        )),
        Line::from(format!("tool_name: {}", value(unit.tool_name.as_deref()))),
        Line::from(format!(
            "collection_id: {}",
            value(unit.collection_id.as_deref())
        )),
        heading("Local State"),
    ];
    push_multiline(
        &mut lines,
        unit.local_state
            .compact_summary()
            .as_deref()
            .unwrap_or("<none>"),
    );
    lines.push(heading("Context Window"));
    if let Some(window) = unit.context_window.as_ref() {
        lines.extend([
            Line::from(format!("model: {}", window.limits.model_name)),
            Line::from(format!("max_tokens: {}", window.limits.max_context_tokens)),
            Line::from(format!("used_tokens: {}", window.used_input_tokens)),
            Line::from(format!(
                "reserved_output: {}",
                window.limits.reserved_output_tokens
            )),
            Line::from(format!("truncated: {}", window.truncated)),
        ]);
        for section in &window.sections {
            lines.push(Line::from(format!(
                "{} [{:?}]: {}/{} tokens; truncated={}",
                section.title,
                section.kind,
                section.used_tokens,
                section.estimated_tokens,
                section.truncated
            )));
        }
    } else {
        lines.push(Line::from("<none>"));
    }
    lines.push(heading("Transitions"));
    if unit.transitions.is_empty() {
        lines.push(Line::from("<none>"));
    } else {
        for transition in &unit.transitions {
            lines.push(Line::from(format!(
                "#{} {} -> {} (traversals: {})",
                transition.sequence,
                transition.output_port,
                transition
                    .target_instance_id
                    .as_deref()
                    .or(transition.graph_output.as_deref())
                    .unwrap_or("<none>"),
                transition.traversal_count
            )));
        }
    }
    lines.push(heading("Output"));
    if let Some(output) = unit.output.as_ref() {
        lines.push(Line::from(format!(
            "port: {}",
            value(output.port.as_deref())
        )));
        lines.push(Line::from(format!(
            "payload: {}",
            output
                .payload
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "<none>".to_string())
        )));
    } else {
        lines.push(Line::from("<none>"));
    }
    lines.push(heading("Result Summary"));
    push_multiline(
        &mut lines,
        unit.result_summary.as_deref().unwrap_or("<none>"),
    );
    lines
}

fn heading(text: &'static str) -> Line<'static> {
    Line::styled(
        format!("\n{text}"),
        Style::default().add_modifier(Modifier::BOLD),
    )
}

fn push_multiline(lines: &mut Vec<Line<'static>>, text: &str) {
    lines.extend(text.lines().map(|line| Line::from(line.to_string())));
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
