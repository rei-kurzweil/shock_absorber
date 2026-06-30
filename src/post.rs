use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

#[derive(Clone)]
pub struct PostNode {
    pub header: String,
    pub uri: String,
    pub text: String,
    pub children: Vec<PostNode>,
}

pub fn render_post_nodes(posts: &[PostNode]) -> Text<'static> {
    let mut lines = Vec::new();

    if posts.is_empty() {
        lines.push(Line::from("No pinned posts cached."));
        return Text::from(lines);
    }

    for (index, post) in posts.iter().enumerate() {
        push_post(&mut lines, post, 0);
        if index + 1 < posts.len() {
            lines.push(Line::default());
        }
    }

    Text::from(lines)
}

fn push_post(lines: &mut Vec<Line<'static>>, post: &PostNode, depth: usize) {
    let indent = "  ".repeat(depth);
    lines.push(Line::from(vec![
        Span::styled(
            format!("{indent}{}", post.header),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(vec![Span::styled(
        format!("{indent}{}", post.uri),
        Style::default().fg(Color::DarkGray),
    )]));

    if post.text.trim().is_empty() {
        lines.push(Line::from(format!("{indent}<no text>")));
    } else {
        for line in post.text.lines() {
            lines.push(Line::from(format!("{indent}{line}")));
        }
    }

    for child in &post.children {
        lines.push(Line::default());
        push_post(lines, child, depth + 1);
    }
}
