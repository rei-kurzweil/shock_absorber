use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use bsky_sdk::BskyAgent;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::env;
use std::io::{self, Stdout};
use std::time::{Duration as StdDuration, Instant};

mod net_backend;
mod post;
mod clearsky_v1;

use crate::net_backend::{
    ActorProfile, CachedThreadReply, NotificationStore, ensure_actor_profile_cached, ensure_clearsky_lists_cached,
    ensure_pinned_posts_cached, extract_post_text, extract_reply_node, poll_notifications,
};
use crate::post::{PostNode, render_post_nodes};

const POLL_INTERVAL: StdDuration = StdDuration::from_secs(30);
const UI_TICK: StdDuration = StdDuration::from_millis(200);

enum DetailView {
    Notification,
    Command { title: String, lines: Vec<String> },
}

struct App {
    store: NotificationStore,
    input: String,
    selected: usize,
    opened_notification: Option<usize>,
    detail_scroll: u16,
    detail: DetailView,
    status: String,
    should_quit: bool,
}

impl App {
    fn new(handle: String) -> Self {
        Self {
            store: NotificationStore::new(),
            input: String::new(),
            selected: 0,
            opened_notification: None,
            detail_scroll: 0,
            detail: DetailView::Command {
                title: "Welcome".to_string(),
                lines: help_lines(),
            },
            status: format!("logged in as {handle}"),
            should_quit: false,
        }
    }

    fn clamp_selection(&mut self) {
        if self.store.notifications.is_empty() {
            self.selected = 0;
            self.opened_notification = None;
            self.detail_scroll = 0;
        } else if self.selected >= self.store.notifications.len() {
            self.selected = self.store.notifications.len() - 1;
        }

        if let Some(opened) = self.opened_notification {
            if opened >= self.store.notifications.len() {
                self.opened_notification = Some(self.store.notifications.len() - 1);
            }
        }
    }

    fn opened_notification(&self) -> Option<&Notification> {
        self.opened_notification
            .and_then(|index| self.store.notifications.get(index))
    }

    fn notification_items(&self) -> Vec<ListItem<'_>> {
        self.store
            .notifications
            .iter()
            .map(|notif| {
                let unread = if notif.data.is_read { " " } else { "*" };
                let time = notif.indexed_at.as_ref().format("%m-%d %H:%M");
                let line = format!(
                    "{unread} {:<7} @{:<20} {}",
                    notif.data.reason,
                    notif.author.data.handle.as_str(),
                    time
                );
                ListItem::new(line)
            })
            .collect()
    }

    fn detail_title(&self) -> String {
        match &self.detail {
            DetailView::Notification => self
                .opened_notification()
                .map(|notif| format!("Notification: @{}", notif.author.data.handle.as_str()))
                .unwrap_or_else(|| "Notification".to_string()),
            DetailView::Command { title, .. } => title.clone(),
        }
    }

    fn detail_text(&self) -> Text<'static> {
        match &self.detail {
            DetailView::Notification => self.notification_detail_text(),
            DetailView::Command { lines, .. } => Text::from(lines.join("\n")),
        }
    }

    fn notification_detail_lines(&self) -> Vec<String> {
        let Some(notif) = self.opened_notification() else {
            return vec![
                "No notification opened.".to_string(),
                "Use Up/Down to highlight a notification, then press Enter.".to_string(),
            ];
        };

        let did = &notif.author.data.did;
        let bio = self.store.get_bio(did).flatten().unwrap_or("<no bio cached>");
        let pinned_count = self
            .store
            .get_pinned_posts(did)
            .map(|posts| posts.len())
            .unwrap_or(0);

        let mut lines = vec![
            format!("handle: {}", notif.author.data.handle.as_str()),
            format!("did: {}", did.as_str()),
            format!("reason: {}", notif.data.reason),
            format!("indexed_at: {}", notif.indexed_at.as_ref()),
            format!("uri: {}", notif.data.uri),
        ];

        if let Some(reason_subject) = notif.data.reason_subject.as_deref() {
            lines.push(format!("reason_subject: {reason_subject}"));
        }

        lines.push(format!("pinned_posts_cached: {pinned_count}"));
        lines.push(String::new());
        lines.push("bio:".to_string());
        if bio.is_empty() {
            lines.push("<empty>".to_string());
        } else {
            lines.extend(bio.lines().map(str::to_owned));
        }

        lines
    }

    fn notification_posts_text(&self) -> Text<'static> {
        let Some(notif) = self.opened_notification() else {
            return Text::from("Open a notification to inspect pinned posts.");
        };

        let Some(posts) = self.store.get_pinned_posts(&notif.author.data.did) else {
            return Text::from("Pinned posts not loaded yet.");
        };

        let nodes = posts
            .iter()
            .map(|post| PostNode {
                header: format!("@{} pinned post", post.author.data.handle.as_str()),
                uri: post.uri.clone(),
                text: extract_post_text(&post.record).unwrap_or_default(),
                children: build_post_nodes(
                    self.store
                        .get_pinned_post_replies(&post.uri)
                        .map(|replies| replies.to_vec())
                        .unwrap_or_default(),
                ),
            })
            .collect::<Vec<_>>();

        render_post_nodes(&nodes)
    }

    fn notification_detail_text(&self) -> Text<'static> {
        let mut lines = vec![
            "Notification:".to_string(),
            String::new(),
        ];
        lines.extend(self.notification_detail_lines());

        if let Some(notif) = self.opened_notification() {
            if notif.data.reason == "reply" {
                let mut reply_lines = vec![
                    String::new(),
                    "Reply:".to_string(),
                    String::new(),
                ];
                let reply = extract_reply_node(&notif.data.record);
                if let Some(parent_uri) = reply.parent_uri.as_deref() {
                    reply_lines.push(format!("parent: {parent_uri}"));
                }
                if let Some(root_uri) = reply.root_uri.as_deref() {
                    reply_lines.push(format!("root: {root_uri}"));
                }
                reply_lines.push("text:".to_string());
                match reply.text {
                    Some(text_body) if !text_body.is_empty() => {
                        for line in text_body.lines() {
                            reply_lines.push(line.to_owned());
                        }
                    }
                    _ => reply_lines.push("<no text>".to_string()),
                }
                lines.extend(reply_lines);
            }
        }

        lines.push(String::new());
        lines.push("Pinned Posts:".to_string());
        lines.push(String::new());
        lines.extend(
            self.notification_posts_text()
                .lines
                .into_iter()
                .map(line_to_string),
        );

        if let Some(notif) = self.opened_notification() {
            let created_lists = if let Some(lists) = self.store.get_clearsky_lists(&notif.author.data.did)
            {
                if lists.is_empty() {
                    "This actor has no returned Clearsky moderation lists.".to_string()
                } else {
                    lists.iter()
                        .map(|list| {
                            let mut parts = vec![
                                format!("name: {}", list.name),
                                format!("url: {}", list.url),
                                format!("did: {}", list.did),
                                format!("date_added: {}", list.date_added),
                                format!("created_date: {}", list.created_date),
                            ];
                            if !list.description.is_empty() {
                                parts.push(format!("description: {}", list.description));
                            }
                            parts.join("\n")
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n")
                }
            } else {
                "Clearsky moderation lists not loaded yet.".to_string()
            };
            lines.push(String::new());
            lines.push("Clearsky Moderation Lists:".to_string());
            lines.push(String::new());
            lines.extend(created_lists.lines().map(str::to_owned));
        }

        Text::from(lines.join("\n"))
    }

    async fn execute_command(
        &mut self,
        agent: &BskyAgent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let command = self.input.trim().to_owned();
        self.input.clear();

        if command.is_empty() {
            return Ok(());
        }

        let mut parts = command.split_whitespace();
        let verb = parts.next().unwrap_or_default();

        match verb {
            "bio" => {
                let Some(actor) = parts.next() else {
                    self.set_command_output("bio", vec!["usage: bio handle.bsky.social".to_string()]);
                    return Ok(());
                };
                let profile =
                    ensure_actor_profile_cached(agent, &mut self.store, normalize_actor_ref(actor))
                        .await?;
                let lines = format_bio_output(&profile);
                self.set_command_output(format!("bio {}", profile.handle), lines);
                self.status = format!("bio loaded for {}", profile.handle);
            }
            "replies_from" => {
                let Some(actor) = parts.next() else {
                    self.set_command_output(
                        "replies_from",
                        vec!["usage: replies_from handle.bsky.social".to_string()],
                    );
                    return Ok(());
                };
                let profile =
                    ensure_actor_profile_cached(agent, &mut self.store, normalize_actor_ref(actor))
                        .await?;
                let lines = format_replies_output(&self.store, &profile);
                self.set_command_output(format!("replies_from {}", profile.handle), lines);
                self.status = format!("reply cache queried for {}", profile.handle);
            }
            "pins" => {
                let Some(actor) = parts.next() else {
                    self.set_command_output("pins", vec!["usage: pins handle.bsky.social".to_string()]);
                    return Ok(());
                };
                let profile =
                    ensure_actor_profile_cached(agent, &mut self.store, normalize_actor_ref(actor))
                        .await?;
                ensure_pinned_posts_cached(agent, &mut self.store, &profile.did).await?;
                ensure_clearsky_lists_cached(&mut self.store, &profile.did).await?;
                let lines = format_pins_output(&self.store, &profile);
                self.set_command_output(format!("pins {}", profile.handle), lines);
                self.status = format!("pins loaded for {}", profile.handle);
            }
            "help" => {
                self.set_command_output("help", help_lines());
            }
            "q" | "quit" | "exit" => {
                self.should_quit = true;
            }
            _ => {
                self.set_command_output(
                    "unknown command",
                    vec![
                        format!("unknown command: {verb}"),
                        "try: help".to_string(),
                    ],
                );
            }
        }

        Ok(())
    }

    fn set_command_output<T: Into<String>>(&mut self, title: T, lines: Vec<String>) {
        self.detail = DetailView::Command {
            title: title.into(),
            lines,
        };
    }

    fn open_selected_notification(&mut self) {
        if self.store.notifications.is_empty() {
            return;
        }
        self.opened_notification = Some(self.selected);
        self.detail_scroll = 0;
        self.detail = DetailView::Notification;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenvy::dotenv().is_ok() {
        eprintln!("loaded .env");
    }

    let handle = env::var("BSKY_HANDLE")?;
    let password = env::var("BSKY_APP_PASSWORD")?;

    let agent = BskyAgent::builder().build().await?;
    agent.login(&handle, &password).await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &agent, App::new(handle)).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    agent: &BskyAgent,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_poll = Instant::now() - POLL_INTERVAL;

    loop {
        if last_poll.elapsed() >= POLL_INTERVAL {
            match poll_notifications(agent, &mut app.store).await {
                Ok(count) => {
                    app.clamp_selection();
                    if count > 0 {
                        app.status = format!("loaded {count} new notifications");
                    } else {
                        app.status = "poll complete; no new notifications".to_string();
                    }
                }
                Err(err) => {
                    app.status = format!("poll failed: {err}");
                }
            }
            last_poll = Instant::now();
        }

        terminal.draw(|frame| draw_ui(frame, &app))?;

        if event::poll(UI_TICK)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('q') if app.input.is_empty() => {
                        app.should_quit = true;
                    }
                    KeyCode::Up => {
                        if app.selected > 0 {
                            app.selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if app.selected + 1 < app.store.notifications.len() {
                            app.selected += 1;
                        }
                    }
                    KeyCode::PageUp => {
                        app.detail_scroll = app.detail_scroll.saturating_sub(4);
                    }
                    KeyCode::PageDown => {
                        app.detail_scroll = app.detail_scroll.saturating_add(4);
                    }
                    KeyCode::Esc => {
                        app.input.clear();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if app.input.trim().is_empty() {
                            app.open_selected_notification();
                        } else if let Err(err) = app.execute_command(agent).await {
                            app.status = format!("command failed: {err}");
                            app.set_command_output(
                                "error",
                                vec![
                                    format!("command failed: {err}"),
                                    "try `help` for supported commands".to_string(),
                                ],
                            );
                        }
                    }
                    KeyCode::Char(ch) => {
                        app.input.push(ch);
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn normalize_actor_ref(actor: &str) -> &str {
    actor.strip_prefix('@').unwrap_or(actor)
}

fn format_bio_output(profile: &ActorProfile) -> Vec<String> {
    let mut lines = vec![
        format!("handle: {}", profile.handle),
        format!("did: {}", profile.did.as_str()),
        String::new(),
        "bio:".to_string(),
    ];

    match profile.bio.as_deref() {
        Some(bio) if !bio.is_empty() => lines.extend(bio.lines().map(str::to_owned)),
        Some(_) => lines.push("<empty>".to_string()),
        None => lines.push("<no bio>".to_string()),
    }

    lines
}

fn format_replies_output(store: &NotificationStore, profile: &ActorProfile) -> Vec<String> {
    let replies = store.replies_from(&profile.did);
    if replies.is_empty() {
        return vec![
            format!("No cached reply notifications for {}", profile.handle),
            "The reply cache is populated from notifications loaded into memory.".to_string(),
        ];
    }

    let mut lines = vec![format!("reply notifications for {}", profile.handle), String::new()];
    for notif in replies {
        let reply = extract_reply_node(&notif.data.record);
        lines.push(format!(
            "{} {}",
            notif.indexed_at.as_ref().format("%Y-%m-%d %H:%M"),
            notif.data.uri
        ));
        if let Some(reason_subject) = notif.data.reason_subject.as_deref() {
            lines.push(format!("reason_subject: {reason_subject}"));
        }
        if let Some(parent_uri) = reply.parent_uri.as_deref() {
            lines.push(format!("parent: {parent_uri}"));
        }
        if let Some(root_uri) = reply.root_uri.as_deref() {
            lines.push(format!("root: {root_uri}"));
        }
        lines.push("text:".to_string());
        match reply.text {
            Some(text) if !text.is_empty() => lines.extend(text.lines().map(str::to_owned)),
            _ => lines.push("<no text>".to_string()),
        }
        lines.push(String::new());
    }
    lines
}

fn format_pins_output(store: &NotificationStore, profile: &ActorProfile) -> Vec<String> {
    let Some(posts) = store.get_pinned_posts(&profile.did) else {
        return vec![format!("No pinned-post cache entry for {}", profile.handle)];
    };

    if posts.is_empty() {
        return vec![format!("{} has no pinned posts", profile.handle)];
    }

    let nodes = posts
        .iter()
        .map(|post| PostNode {
            header: format!("@{} pinned post", post.author.data.handle.as_str()),
            uri: post.uri.clone(),
            text: extract_post_text(&post.record).unwrap_or_default(),
            children: build_post_nodes(
                store
                    .get_pinned_post_replies(&post.uri)
                        .map(|replies| replies.to_vec())
                    .unwrap_or_default(),
            ),
        })
        .collect::<Vec<_>>();

    let mut lines = vec![format!("pinned posts for {}", profile.handle), String::new()];
    lines.extend(render_post_nodes(&nodes).lines.into_iter().map(line_to_string));
    lines
}

fn build_post_nodes(replies: Vec<CachedThreadReply>) -> Vec<PostNode> {
    replies
        .into_iter()
        .map(|reply| PostNode {
            header: format!("@{} replied", reply.author_handle),
            uri: reply.uri,
            text: reply.text,
            children: build_post_nodes(reply.children),
        })
        .collect()
}

fn help_lines() -> Vec<String> {
    vec![
        "Commands:".to_string(),
        "  bio handle.bsky.social".to_string(),
        "  replies_from handle.bsky.social".to_string(),
        "  pins handle.bsky.social".to_string(),
        "  help".to_string(),
        "  quit".to_string(),
        String::new(),
        "Navigation: Up/Down selects a notification; Enter opens it; PageUp/PageDown scroll detail.".to_string(),
    ]
}

fn line_to_string(line: ratatui::text::Line<'_>) -> String {
    line.spans.iter().map(|span| span.content.as_ref()).collect()
}

fn draw_ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    let notifications = List::new(app.notification_items())
        .block(Block::default().title("Notifications").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    if !app.store.notifications.is_empty() {
        list_state.select(Some(app.selected));
    }
    frame.render_stateful_widget(notifications, body[0], &mut list_state);

    let detail = Paragraph::new(app.detail_text())
        .block(Block::default().title(app.detail_title()).borders(Borders::ALL))
        .scroll((app.detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(detail, body[1]);

    let input = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .title(format!("Command | {} ", app.status))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(input, chunks[1]);

    let cursor_x = chunks[1]
        .x
        .saturating_add(app.input.chars().count() as u16 + 1)
        .min(chunks[1].x + chunks[1].width.saturating_sub(2));
    let cursor_y = chunks[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}
