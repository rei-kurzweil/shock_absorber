use crate::db::AppDb;
use crate::harness::agents::{AgentGraph, AgentNode, AgentNodeId, AgentNodeKind};
use crate::harness::llm_api::{ChatMessage, LlmApiClient, OpenAiRestConfig};
use crate::harness::root_context::{
    build_live_context_visualization, build_root_context_snapshot,
    build_tool_aware_query_context_window,
};
use crate::harness::root_run::{RootRunEvent, run_root_query_task};
use crate::harness::runtime::{
    ContextMessage, ContextMessageKind, RootRunState, RootRunStatus, TranscriptEntryKind,
};
use crate::harness::tools::{BlueskyTools, ToolProgressEvent, prompt_tool_protocol_instructions};
use crate::net_backend::{
    ActorProfile, CachedThreadReply, NotificationStore, ensure_actor_profile_cached,
    ensure_clearsky_lists_cached, ensure_pinned_posts_cached, ensure_recent_posts_cached,
    extract_reply_node, poll_notifications,
};
use crate::post::{PostNode, render_post_nodes};
use crate::ui;
use crate::ui::chat_editor::ChatEditor;
use crate::ui::stdout_chat::StdoutChatRenderer;
use crate::visualization::context::ContextVisualizationData;
use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListItem;
use ratatui::{Frame, Terminal};
use std::env;
use std::error::Error;
use std::fs;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio::task::JoinHandle;

const POLL_INTERVAL: StdDuration = StdDuration::from_secs(30);
const UI_TICK: StdDuration = StdDuration::from_millis(200);
const DEFAULT_EVIL_GEMMA_BASE_URL: &str = "http://127.0.0.1:5000";
const DEFAULT_EVIL_GEMMA_MODEL: &str = "qwen-3.5-local";
const DEFAULT_SYSTEM_PROMPT_PATH: &str = "system_prompt.md";

#[derive(Clone)]
pub enum DetailView {
    Notification,
    Agents,
    Command { title: String, lines: Vec<String> },
    ContextVisualization(ContextVisualizationData),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppActivity {
    NotificationDetail,
    Agents,
    CommandOverlay,
    ContextVisualization,
    AiChat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationMode {
    Tui,
    StdoutChat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SplitView {
    Notifications,
    Agents,
}

pub struct App {
    store: NotificationStore,
    input: String,
    chat_editor: ChatEditor,
    selected: usize,
    selected_agent: usize,
    opened_notification: Option<usize>,
    selected_actor: Option<ActorProfile>,
    detail_scroll: u16,
    detail: DetailView,
    root_conversation: Vec<ConversationTurn>,
    root_run: Option<RootRunState>,
    active_root_run: Option<ActiveRootRunTask>,
    deferred_detail: Option<DetailView>,
    last_non_chat_detail: Option<DetailView>,
    chat_title: Option<String>,
    presentation_mode: PresentationMode,
    status: String,
    should_quit: bool,
}

struct ActiveRootRunTask {
    query: String,
    keep_context_overlay: bool,
    receiver: UnboundedReceiver<RootRunEvent>,
    handle: JoinHandle<()>,
}

#[derive(Clone)]
pub struct ConversationTurn {
    pub user: String,
    pub assistant: String,
}

pub struct EvilGemmaConfig {
    pub client: LlmApiClient,
    pub system_prompt: String,
}

impl EvilGemmaConfig {
    pub async fn from_env() -> Result<Self, Box<dyn Error>> {
        let base_url = env::var("EVIL_GEMMA_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_EVIL_GEMMA_BASE_URL.to_string());
        let model_name =
            env::var("EVIL_GEMMA_MODEL").unwrap_or_else(|_| DEFAULT_EVIL_GEMMA_MODEL.to_string());
        let system_prompt_path = env::var("SYSTEM_PROMPT_PATH")
            .unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT_PATH.to_string());

        let mut config = OpenAiRestConfig::llama_cpp(base_url, model_name);
        match LlmApiClient::fetch_capabilities(&config.base_url).await {
            Ok(capabilities) => config.apply_capabilities(&capabilities),
            Err(err) => eprintln!(
                "warning: failed to fetch evil_gemma capabilities from {}: {err}; using local defaults",
                config.base_url
            ),
        }
        let system_prompt = fs::read_to_string(resolve_system_prompt_path(system_prompt_path))?;

        Ok(Self {
            client: LlmApiClient::new(config),
            system_prompt,
        })
    }
}

impl App {
    pub fn new(handle: String) -> Self {
        Self {
            store: NotificationStore::new(),
            input: String::new(),
            chat_editor: ChatEditor::new(),
            selected: 0,
            selected_agent: 0,
            opened_notification: None,
            selected_actor: None,
            detail_scroll: 0,
            detail: DetailView::Command {
                title: "Welcome".to_string(),
                lines: help_lines(),
            },
            root_conversation: Vec::new(),
            root_run: None,
            active_root_run: None,
            deferred_detail: None,
            last_non_chat_detail: None,
            chat_title: None,
            presentation_mode: PresentationMode::Tui,
            status: format!("logged in as {handle}"),
            should_quit: false,
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn status(&self) -> &str {
        &self.status
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

        let agent_count = self.agent_list_entries().len();
        if agent_count == 0 {
            self.selected_agent = 0;
        } else if self.selected_agent >= agent_count {
            self.selected_agent = agent_count - 1;
        }
    }

    fn opened_notification(&self) -> Option<&Notification> {
        self.opened_notification
            .and_then(|index| self.store.notifications.get(index))
    }

    fn selected_actor_did(&self) -> Option<&bsky_sdk::api::types::string::Did> {
        self.opened_notification()
            .map(|notification| &notification.author.data.did)
            .or_else(|| self.selected_actor.as_ref().map(|actor| &actor.did))
    }

    fn selected_actor_summary(&self) -> Option<String> {
        if let Some(notification) = self.opened_notification() {
            return Some(serialize_notification_summary_for_llm(notification));
        }

        self.selected_actor.as_ref().map(|actor| {
            let mut lines = vec![
                format!("selected_actor_handle: {}", actor.handle),
                format!("selected_actor_did: {}", actor.did.as_str()),
            ];
            match actor.bio.as_deref() {
                Some(bio) if !bio.is_empty() => {
                    lines.push("selected_actor_bio:".to_string());
                    lines.extend(bio.lines().map(str::to_owned));
                }
                _ => lines.push("selected_actor_bio: <none>".to_string()),
            }
            lines.join("\n")
        })
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
            DetailView::Agents => self
                .selected_agent_node()
                .map(|node| format!("Agent {}", node.agent_id.0))
                .unwrap_or_else(|| "Agent".to_string()),
            DetailView::Command { title, .. } => title.clone(),
            DetailView::ContextVisualization(data) => data.title.clone(),
        }
    }

    fn current_activity(&self) -> AppActivity {
        if self.presentation_mode == PresentationMode::StdoutChat {
            return AppActivity::AiChat;
        }
        match self.detail {
            DetailView::Notification => AppActivity::NotificationDetail,
            DetailView::Agents => AppActivity::Agents,
            DetailView::Command { .. } => AppActivity::CommandOverlay,
            DetailView::ContextVisualization(_) => AppActivity::ContextVisualization,
        }
    }

    fn current_split_view(&self) -> SplitView {
        match self.detail {
            DetailView::Agents => SplitView::Agents,
            _ => SplitView::Notifications,
        }
    }

    fn is_fullscreen_overlay(&self) -> bool {
        matches!(
            self.detail,
            DetailView::Command { .. } | DetailView::ContextVisualization(_)
        )
    }

    fn detail_text(&self) -> ratatui::text::Text<'static> {
        match &self.detail {
            DetailView::Notification => self.notification_detail_text(),
            DetailView::Agents => self.agent_detail_text(),
            DetailView::Command { lines, .. } => ratatui::text::Text::from(lines.join("\n")),
            DetailView::ContextVisualization(_) => ratatui::text::Text::from(""),
        }
    }

    fn agent_list_entries(&self) -> Vec<(usize, AgentNodeId)> {
        self.root_run
            .as_ref()
            .map(|run| run.agent_graph().ids_with_depth_in_display_order())
            .unwrap_or_default()
    }

    fn selected_agent_entry(&self) -> Option<(usize, AgentNodeId)> {
        self.agent_list_entries().get(self.selected_agent).copied()
    }

    fn selected_agent_node(&self) -> Option<&AgentNode> {
        let (_, node_id) = self.selected_agent_entry()?;
        self.root_run
            .as_ref()
            .and_then(|run| run.agent_graph().node(node_id))
    }

    fn agent_items(&self) -> Vec<ListItem<'_>> {
        let Some(run) = self.root_run.as_ref() else {
            return vec![ListItem::new("No agent run available yet.")];
        };

        self.agent_list_entries()
            .into_iter()
            .filter_map(|(depth, node_id)| {
                let node = run.agent_graph().node(node_id)?;
                let indent = "  ".repeat(depth.saturating_sub(1));
                let kind = match node.agent_type {
                    AgentNodeKind::RootAgent => "root",
                    AgentNodeKind::ToolAgent => "tool",
                    AgentNodeKind::CollectionSearchTool => "search",
                    AgentNodeKind::CollectionSummaryTool => "summary",
                    AgentNodeKind::SearchReviewAgent => "search_review",
                    AgentNodeKind::SummaryReviewAgent => "summary_review",
                };
                let line = format!(
                    "{}#{:<3} [{:<9}] {:<8} {}",
                    indent,
                    node.agent_id.0,
                    kind,
                    node.status.as_str(),
                    node.label
                );
                Some(ListItem::new(line))
            })
            .collect()
    }

    fn agent_detail_text(&self) -> ratatui::text::Text<'static> {
        let Some(node) = self.selected_agent_node() else {
            return ratatui::text::Text::from(
                "No agent selected.\nRun a query, then use `/agents` to inspect the agent graph.",
            );
        };

        let mut lines = vec![
            format!("agent_id: {}", node.agent_id.0),
            format!("label: {}", node.label),
            format!("status: {}", node.status.as_str()),
            format!(
                "node_type: {}",
                match node.agent_type {
                    AgentNodeKind::RootAgent => "root",
                    AgentNodeKind::ToolAgent => "tool",
                    AgentNodeKind::CollectionSearchTool => "collection_search",
                    AgentNodeKind::CollectionSummaryTool => "collection_summary",
                    AgentNodeKind::SearchReviewAgent => "search_review",
                    AgentNodeKind::SummaryReviewAgent => "summary_review",
                }
            ),
        ];

        if let Some(kind) = node.agent_kind {
            lines.push(format!("agent_kind: {}", kind.id()));
        }
        if let Some(parent_id) = node.parent_agent_id {
            lines.push(format!("parent_agent_id: {}", parent_id.0));
        } else {
            lines.push("parent_agent_id: <none>".to_string());
        }
        lines.push(format!(
            "child_agent_ids: {}",
            if node.child_agent_ids.is_empty() {
                "<none>".to_string()
            } else {
                node.child_agent_ids
                    .iter()
                    .map(|id| id.0.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
        if let Some(tool_name) = node.tool_name.as_deref() {
            lines.push(format!("tool_name: {tool_name}"));
        }
        if let Some(collection_id) = node.collection_id.as_deref() {
            lines.push(format!("collection_id: {collection_id}"));
        }

        if let Some(window) = node.context_window_report.as_ref() {
            lines.push(String::new());
            lines.push("context_window:".to_string());
            lines.push(format!(
                "provider: {} / {}",
                window.limits.provider_name, window.limits.model_name
            ));
            lines.push(format!(
                "input_tokens: {} / {}",
                window.used_input_tokens,
                window.limits.available_input_tokens()
            ));
            lines.push(format!("header_tokens: {}", window.header_tokens));
            lines.push(format!("truncated: {}", window.truncated));
            lines.push(format!("sections: {}", window.sections.len()));
            for section in &window.sections {
                lines.push(format!(
                    "- {} [{}] used={} estimated={} truncated={}",
                    section.title,
                    format!("{:?}", section.kind).to_ascii_lowercase(),
                    section.used_tokens,
                    section.estimated_tokens,
                    section.truncated
                ));
            }
            lines.push(String::new());
            lines.push("rendered_context:".to_string());
            lines.extend(window.rendered.lines().map(str::to_owned));
        }

        if let Some(summary) = node.result_summary.as_deref() {
            lines.push(String::new());
            lines.push("result_summary:".to_string());
            lines.extend(summary.lines().map(str::to_owned));
        }

        ratatui::text::Text::from(lines.join("\n"))
    }

    fn notification_detail_lines(&self) -> Vec<String> {
        let Some(notif) = self.opened_notification() else {
            return vec![
                "No notification opened.".to_string(),
                "Use Up/Down to highlight a notification, then press Enter.".to_string(),
            ];
        };

        let did = &notif.author.data.did;
        let bio = self
            .store
            .get_bio(did)
            .flatten()
            .unwrap_or("<no bio cached>");
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

    fn notification_posts_text(&self) -> ratatui::text::Text<'static> {
        let Some(notif) = self.opened_notification() else {
            return ratatui::text::Text::from("Open a notification to inspect pinned posts.");
        };

        let Some(posts) = self.store.get_pinned_posts(&notif.author.data.did) else {
            return ratatui::text::Text::from("Pinned posts not loaded yet.");
        };

        let nodes = posts
            .iter()
            .map(|post| PostNode {
                header: format!("@{} pinned post", post.author_handle),
                uri: post.uri.clone(),
                text: post.body.clone(),
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

    fn notification_detail_text(&self) -> ratatui::text::Text<'static> {
        let mut lines = vec!["Notification:".to_string(), String::new()];
        lines.extend(self.notification_detail_lines());

        if let Some(notif) = self.opened_notification() {
            if notif.data.reason == "reply" {
                let mut reply_lines = vec![String::new(), "Reply:".to_string(), String::new()];
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
            let created_lists =
                if let Some(lists) = self.store.get_clearsky_lists(&notif.author.data.did) {
                    if lists.is_empty() {
                        "This actor has no returned Clearsky moderation lists.".to_string()
                    } else {
                        lists
                            .iter()
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

        ratatui::text::Text::from(lines.join("\n"))
    }

    async fn execute_input(
        &mut self,
        agent: &Arc<BskyAgent>,
        evil_gemma: &Arc<EvilGemmaConfig>,
        command: String,
    ) -> Result<(), Box<dyn Error>> {
        if command.is_empty() {
            return Ok(());
        }

        let mut parts = command.split_whitespace();
        let verb = parts.next().unwrap_or_default();

        if !is_local_command(verb) {
            self.start_root_run(agent.clone(), evil_gemma.clone(), command)?;
            return Ok(());
        }

        match verb {
            "bio" | "/bio" => {
                let Some(actor) = parts.next() else {
                    self.set_command_output(
                        "/bio",
                        vec!["usage: /bio handle.bsky.social".to_string()],
                    );
                    return Ok(());
                };
                let profile =
                    ensure_actor_profile_cached(agent, &mut self.store, normalize_actor_ref(actor))
                        .await?;
                self.selected_actor = Some(profile.clone());
                let lines = format_bio_output(&profile);
                self.set_command_output(format!("/bio {}", profile.handle), lines);
                self.status = format!("bio loaded for {}", profile.handle);
            }
            "replies_from" | "/replies_from" => {
                let Some(actor) = parts.next() else {
                    self.set_command_output(
                        "/replies_from",
                        vec!["usage: /replies_from handle.bsky.social".to_string()],
                    );
                    return Ok(());
                };
                let profile =
                    ensure_actor_profile_cached(agent, &mut self.store, normalize_actor_ref(actor))
                        .await?;
                self.selected_actor = Some(profile.clone());
                let lines = format_replies_output(&self.store, &profile);
                self.set_command_output(format!("/replies_from {}", profile.handle), lines);
                self.status = format!("reply cache queried for {}", profile.handle);
            }
            "pins" | "/pins" => {
                let Some(actor) = parts.next() else {
                    self.set_command_output(
                        "/pins",
                        vec!["usage: /pins handle.bsky.social".to_string()],
                    );
                    return Ok(());
                };
                let profile =
                    ensure_actor_profile_cached(agent, &mut self.store, normalize_actor_ref(actor))
                        .await?;
                self.selected_actor = Some(profile.clone());
                ensure_recent_posts_cached(agent, &mut self.store, &profile.did, 20, 10).await?;
                ensure_pinned_posts_cached(agent, &mut self.store, &profile.did).await?;
                ensure_clearsky_lists_cached(&mut self.store, &profile.did).await?;
                let lines = format_pins_output(&self.store, &profile);
                self.set_command_output(format!("/pins {}", profile.handle), lines);
                self.status = format!("pins loaded for {}", profile.handle);
            }
            "clear" | "/clear" => {
                self.clear_root_conversation();
                self.root_run = None;
                self.chat_title = None;
                self.set_command_output(
                    "/clear",
                    vec![
                        "Root agent context cleared.".to_string(),
                        "The next query will start from system prompt, tool inventory, and current UI state.".to_string(),
                    ],
                );
                self.status = "root agent context cleared".to_string();
            }
            "context" | "/context" => {
                self.detail_scroll = 0;
                self.presentation_mode = PresentationMode::Tui;
                self.detail =
                    DetailView::ContextVisualization(self.build_context_visualization(evil_gemma));
                self.status = "context visualization loaded".to_string();
            }
            "agents" | "/agents" => {
                self.clamp_selection();
                self.detail_scroll = 0;
                self.presentation_mode = PresentationMode::Tui;
                self.detail = DetailView::Agents;
                self.status = "agents view loaded".to_string();
            }
            "notifications" | "/notifications" => {
                self.detail_scroll = 0;
                self.presentation_mode = PresentationMode::Tui;
                self.detail = DetailView::Notification;
                self.status = "notifications view loaded".to_string();
            }
            "stop" | "/stop" => {
                self.stop_active_root_run();
            }
            "task" | "/task" => {
                self.set_command_output("/task", self.task_lines());
                self.status = "task loaded".to_string();
            }
            "help" | "/help" => {
                self.set_command_output("/help", help_lines());
            }
            "q" | "/q" | "quit" | "/quit" | "exit" | "/exit" => {
                self.should_quit = true;
            }
            _ => unreachable!("command list is checked before dispatch"),
        }

        Ok(())
    }

    fn set_evil_gemma_progress(&mut self, query: &str, run: &RootRunState) {
        self.root_run = Some(run.clone());
        self.chat_title = Some(format!("evil_gemma: {query}"));
        self.show_chat_output(true);
    }

    fn show_chat_output(&mut self, visible: bool) {
        if visible {
            if self.presentation_mode != PresentationMode::StdoutChat {
                self.last_non_chat_detail = Some(self.detail.clone());
            }
            self.presentation_mode = PresentationMode::StdoutChat;
        }
    }

    fn set_command_output<T: Into<String>>(&mut self, title: T, lines: Vec<String>) {
        self.detail_scroll = 0;
        if self.presentation_mode == PresentationMode::StdoutChat {
            self.deferred_detail = Some(self.current_non_chat_detail());
        }
        self.presentation_mode = PresentationMode::Tui;
        self.detail = DetailView::Command {
            title: title.into(),
            lines,
        };
    }

    fn dismiss_command_output(&mut self) {
        self.detail_scroll = 0;
        if let Some(detail) = self.deferred_detail.take() {
            self.detail = detail;
        } else {
            self.detail = DetailView::Notification;
        }
        self.presentation_mode = PresentationMode::Tui;
    }

    fn toggle_ai_chat_view(&mut self) {
        match self.current_activity() {
            AppActivity::AiChat => {
                self.detail_scroll = 0;
                if let Some(detail) = self.last_non_chat_detail.clone() {
                    self.detail = detail;
                } else {
                    self.detail = DetailView::Notification;
                }
                self.presentation_mode = PresentationMode::Tui;
                self.status = "returned to previous view".to_string();
            }
            _ => {
                if self.chat_title.is_none() && self.root_run.is_none() {
                    self.status = "no ai chat view available".to_string();
                    return;
                }
                self.last_non_chat_detail = Some(self.detail.clone());
                self.detail_scroll = 0;
                self.presentation_mode = PresentationMode::StdoutChat;
                self.status = "ai chat view loaded".to_string();
            }
        }
    }

    fn clear_root_conversation(&mut self) {
        self.root_conversation.clear();
    }

    fn task_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(run) = self.root_run.as_ref() {
            lines.push(format!("run_id: {}", run.run_id()));
            lines.push(format!("run_status: {}", run.status().as_str()));
            lines.push(format!("cancel_requested: {}", run.cancel_requested()));
            lines.push(format!("rounds: {}", run.rounds().len()));
            lines.push(format!("tool_calls: {}", run.tool_call_history().len()));
            if let Some(root) = run.agent_graph().node(run.agent_graph().root_agent_id()) {
                lines.push(format!("root_agent_id: {}", root.agent_id.0));
                lines.push(format!("status: {}", root.status.as_str()));
                if let Some(summary) = root.result_summary.as_deref() {
                    lines.push(format!("result_summary: {}", summary));
                }
            }
        }
        lines.push(String::new());
        lines.push("current_task:".to_string());
        lines.push(
            self.root_run
                .as_ref()
                .map(|run| run.query().to_string())
                .unwrap_or_else(|| "<none>".to_string()),
        );
        lines
    }

    fn build_context_visualization(
        &self,
        evil_gemma: &EvilGemmaConfig,
    ) -> ContextVisualizationData {
        if let Some(data) = self
            .root_run
            .as_ref()
            .and_then(|run| run.context_visualization())
        {
            return data.clone();
        }

        let tools = BlueskyTools::new();
        let root_window = build_tool_aware_query_context_window(
            self.selected_actor_did(),
            self.selected_actor_summary(),
            &tools,
            &self.root_conversation,
            self.input.trim(),
            &evil_gemma.client,
        );
        let root_snapshot = build_root_context_snapshot(
            &[],
            evil_gemma.system_prompt.trim(),
            prompt_tool_protocol_instructions(),
            &root_window,
            &evil_gemma.client.context_limits(),
        );
        ContextVisualizationData::from_windows("/context", vec![root_snapshot])
    }

    fn start_root_run(
        &mut self,
        agent: Arc<BskyAgent>,
        evil_gemma: Arc<EvilGemmaConfig>,
        query: String,
    ) -> Result<(), Box<dyn Error>> {
        if self.active_root_run.is_some() {
            self.status = "a root run is already active; use /stop first".to_string();
            self.set_command_output(
                "/stop",
                vec![
                    "A root run is already active.".to_string(),
                    "Use `/stop` to cancel it before starting another query.".to_string(),
                ],
            );
            return Ok(());
        }

        let keep_context_overlay = matches!(self.detail, DetailView::ContextVisualization(_));
        self.deferred_detail = None;

        let selected_actor_did = self.selected_actor_did().cloned();
        let selected_actor_summary = self.selected_actor_summary();
        let root_conversation = self.root_conversation.clone();
        let selected_notification = self.opened_notification().cloned();
        let store = self.store.clone();

        let root_context_window = {
            let tools = BlueskyTools::new();
            build_tool_aware_query_context_window(
                selected_actor_did.as_ref(),
                selected_actor_summary,
                &tools,
                &root_conversation,
                &query,
                &evil_gemma.client,
            )
        };
        let initial_context = root_context_window.rendered.clone();
        let messages = vec![
            ContextMessage {
                kind: ContextMessageKind::InitialSystem,
                message: ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "{}\n\n{}",
                        evil_gemma.system_prompt.trim(),
                        prompt_tool_protocol_instructions()
                    ),
                },
            },
            ContextMessage {
                kind: ContextMessageKind::InitialUserContext,
                message: ChatMessage {
                    role: "user".to_string(),
                    content: initial_context,
                },
            },
        ];
        let mut agent_graph = AgentGraph::new_root("Root Agent");
        agent_graph.set_context_window(agent_graph.root_agent_id(), root_context_window.clone());
        agent_graph.set_result_summary(agent_graph.root_agent_id(), query.clone());
        let mut root_run =
            RootRunState::new(query.clone(), root_context_window, messages, agent_graph);
        let initial_visualization = build_live_context_visualization(
            "/context",
            root_run.messages(),
            evil_gemma.system_prompt.trim(),
            prompt_tool_protocol_instructions(),
            root_run.root_context_window(),
            root_run.agent_graph(),
            &evil_gemma.client.context_limits(),
        );
        root_run.set_context_visualization(initial_visualization);
        self.root_run = Some(root_run.clone());
        self.chat_title = Some(format!("evil_gemma: {query}"));
        if keep_context_overlay {
            self.show_chat_output(false);
            self.status = "evil_gemma run started; dismiss /context to view output".to_string();
        } else {
            self.set_evil_gemma_progress(&query, &root_run);
            self.status = "evil_gemma run started".to_string();
        }

        let (sender, receiver) = unbounded_channel();
        let runtime = tokio::runtime::Handle::current();
        let handle = tokio::task::spawn_blocking(move || {
            runtime.block_on(async move {
                run_root_query_task(
                    agent,
                    evil_gemma,
                    store,
                    selected_notification,
                    selected_actor_did,
                    root_run,
                    sender,
                )
                .await;
            });
        });

        self.active_root_run = Some(ActiveRootRunTask {
            query,
            keep_context_overlay,
            receiver,
            handle,
        });
        Ok(())
    }

    fn stop_active_root_run(&mut self) {
        let Some(active) = self.active_root_run.take() else {
            self.status = "no active root run to stop".to_string();
            self.set_command_output("/stop", vec!["No active root run.".to_string()]);
            return;
        };

        active.handle.abort();
        if let Some(root_run) = self.root_run.as_mut() {
            root_run.request_cancel();
            root_run.set_status(RootRunStatus::Cancelled);
            root_run.push_transcript_entry(
                TranscriptEntryKind::Notice,
                "Runtime Notice\nrun cancelled by user with `/stop`",
            );
        }
        self.chat_title = Some(format!("evil_gemma: {}", active.query));
        self.show_chat_output(!active.keep_context_overlay);
        self.status = "active root run cancelled".to_string();
    }

    fn process_background_events(&mut self, db: &AppDb) {
        let mut completed = false;
        let mut save_store = false;
        loop {
            let event = {
                let Some(active) = self.active_root_run.as_mut() else {
                    break;
                };
                match active.receiver.try_recv() {
                    Ok(event) => event,
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        completed = true;
                        break;
                    }
                }
            };

            let keep_context_overlay = self
                .active_root_run
                .as_ref()
                .map(|active| active.keep_context_overlay)
                .unwrap_or(false);
            match event {
                RootRunEvent::Progress(root_run) => {
                    let query = root_run.query().to_string();
                    self.root_run = Some(root_run.clone());
                    self.chat_title = Some(format!("evil_gemma: {query}"));
                    self.clamp_selection();
                    self.show_chat_output(!keep_context_overlay);
                    self.status = format!("evil_gemma running: {}", root_run.status().as_str());
                }
                RootRunEvent::ToolProgress(event) => {
                    if let Some(root_run) = self.root_run.as_mut() {
                        match event {
                            ToolProgressEvent::AgentUpdate {
                                label,
                                depth,
                                content,
                            } => root_run.push_agent_entry(
                                TranscriptEntryKind::AgentEvent,
                                label,
                                depth,
                                content,
                            ),
                        }
                        let query = root_run.query().to_string();
                        self.chat_title = Some(format!("evil_gemma: {query}"));
                        self.show_chat_output(!keep_context_overlay);
                        self.status = "evil_gemma running: subagent progress".to_string();
                    }
                }
                RootRunEvent::Completed {
                    root_run,
                    store,
                    response,
                } => {
                    let query = root_run.query().to_string();
                    self.store = store;
                    self.root_conversation.push(ConversationTurn {
                        user: query.clone(),
                        assistant: response,
                    });
                    self.root_run = Some(root_run.clone());
                    self.chat_title = Some(format!("evil_gemma: {query}"));
                    self.clamp_selection();
                    self.show_chat_output(!keep_context_overlay);
                    self.status = if keep_context_overlay {
                        "evil_gemma response ready; dismiss /context to view output".to_string()
                    } else {
                        "evil_gemma response loaded".to_string()
                    };
                    completed = true;
                    save_store = true;
                }
                RootRunEvent::Failed { root_run, error } => {
                    if let Some(root_run) = root_run {
                        let query = root_run.query().to_string();
                        self.root_run = Some(root_run.clone());
                        self.chat_title = Some(format!("evil_gemma: {query}"));
                        self.clamp_selection();
                        self.show_chat_output(!keep_context_overlay);
                    }
                    self.status = format!("root run failed: {error}");
                    self.set_command_output("error", vec![format!("root run failed: {error}")]);
                    completed = true;
                }
            }
        }

        if completed {
            self.active_root_run = None;
        }
        if save_store {
            let _ = db.save_store(&self.store);
        }
    }

    fn open_selected_notification(&mut self) {
        if self.store.notifications.is_empty() {
            return;
        }
        self.opened_notification = Some(self.selected);
        self.selected_actor = None;
        self.detail_scroll = 0;
        self.detail = DetailView::Notification;
    }

    fn move_selection_up(&mut self) {
        match self.current_split_view() {
            SplitView::Notifications => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            SplitView::Agents => {
                if self.selected_agent > 0 {
                    self.selected_agent -= 1;
                    self.detail_scroll = 0;
                }
            }
        }
    }

    fn move_selection_down(&mut self) {
        match self.current_split_view() {
            SplitView::Notifications => {
                if self.selected + 1 < self.store.notifications.len() {
                    self.selected += 1;
                }
            }
            SplitView::Agents => {
                let len = self.agent_list_entries().len();
                if self.selected_agent + 1 < len {
                    self.selected_agent += 1;
                    self.detail_scroll = 0;
                }
            }
        }
    }

    fn activate_selected_item(&mut self) {
        match self.current_split_view() {
            SplitView::Notifications => self.open_selected_notification(),
            SplitView::Agents => {}
        }
    }

    pub fn presentation_mode(&self) -> PresentationMode {
        self.presentation_mode
    }

    pub fn chat_editor(&self) -> &ChatEditor {
        &self.chat_editor
    }

    pub fn chat_editor_mut(&mut self) -> &mut ChatEditor {
        &mut self.chat_editor
    }

    pub fn chat_title(&self) -> Option<&str> {
        self.chat_title.as_deref()
    }

    pub fn root_run(&self) -> Option<&RootRunState> {
        self.root_run.as_ref()
    }

    fn current_non_chat_detail(&self) -> DetailView {
        self.last_non_chat_detail
            .clone()
            .unwrap_or_else(|| self.detail.clone())
    }
}

pub async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    agent: Arc<BskyAgent>,
    evil_gemma: Arc<EvilGemmaConfig>,
    mut app: App,
    db: &AppDb,
) -> Result<(), Box<dyn Error>> {
    let mut last_poll = Instant::now() - POLL_INTERVAL;
    let mut stdout_chat = StdoutChatRenderer::new();
    let mut terminal_mode = PresentationMode::Tui;

    loop {
        if app.active_root_run.is_none() && last_poll.elapsed() >= POLL_INTERVAL {
            match poll_notifications(&agent, &mut app.store).await {
                Ok(count) => {
                    if count > 0 {
                        db.save_store(&app.store)?;
                    }
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

        app.process_background_events(db);

        if app.presentation_mode() != terminal_mode {
            match app.presentation_mode() {
                PresentationMode::StdoutChat => {
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    stdout_chat.enter(terminal.backend_mut(), &app)?;
                }
                PresentationMode::Tui => {
                    stdout_chat.leave(terminal.backend_mut())?;
                    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                    terminal.clear()?;
                    terminal.draw(|frame| draw_ui(frame, &app))?;
                }
            }
            terminal_mode = app.presentation_mode();
        }

        match terminal_mode {
            PresentationMode::Tui => {
                terminal.draw(|frame| draw_ui(frame, &app))?;
            }
            PresentationMode::StdoutChat => {
                stdout_chat.sync(terminal.backend_mut(), &app)?;
            }
        }

        if event::poll(UI_TICK)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.presentation_mode() {
                    PresentationMode::Tui => match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('q') if app.input.is_empty() => {
                            app.should_quit = true;
                        }
                        KeyCode::Up => app.move_selection_up(),
                        KeyCode::Down => app.move_selection_down(),
                        KeyCode::PageUp => {
                            app.detail_scroll = app.detail_scroll.saturating_sub(4);
                        }
                        KeyCode::PageDown => {
                            app.detail_scroll = app.detail_scroll.saturating_add(4);
                        }
                        KeyCode::Tab => {
                            app.toggle_ai_chat_view();
                        }
                        KeyCode::Esc => {
                            if !app.input.is_empty() {
                                app.input.clear();
                            } else if app.is_fullscreen_overlay() {
                                app.dismiss_command_output();
                            }
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            if app.input.trim().is_empty() {
                                app.activate_selected_item();
                            } else {
                                let command = app.input.trim().to_owned();
                                app.input.clear();
                                if let Err(err) =
                                    app.execute_input(&agent, &evil_gemma, command).await
                                {
                                    app.status = format!("command failed: {err}");
                                    app.set_command_output(
                                        "error",
                                        vec![
                                            format!("command failed: {err}"),
                                            "try `/help` for supported commands".to_string(),
                                        ],
                                    );
                                } else {
                                    db.save_store(&app.store)?;
                                }
                            }
                        }
                        KeyCode::Char(ch)
                            if !key.modifiers.contains(KeyModifiers::CONTROL)
                                && !key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            app.input.push(ch);
                        }
                        _ => {}
                    },
                    PresentationMode::StdoutChat => match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.chat_editor_mut().insert_newline();
                        }
                        KeyCode::Left => app.chat_editor_mut().move_left(),
                        KeyCode::Right => app.chat_editor_mut().move_right(),
                        KeyCode::Up => app.chat_editor_mut().move_up(),
                        KeyCode::Down => app.chat_editor_mut().move_down(),
                        KeyCode::Backspace => app.chat_editor_mut().backspace(),
                        KeyCode::Tab => app.toggle_ai_chat_view(),
                        KeyCode::Esc => {
                            if !app.chat_editor().is_empty() {
                                app.chat_editor_mut().clear();
                            } else {
                                app.toggle_ai_chat_view();
                            }
                        }
                        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                            app.chat_editor_mut().insert_newline();
                        }
                        KeyCode::Enter => {
                            if !app.chat_editor().trimmed_text().is_empty() {
                                let command = app.chat_editor_mut().take_text();
                                if let Err(err) =
                                    app.execute_input(&agent, &evil_gemma, command).await
                                {
                                    app.status = format!("command failed: {err}");
                                    app.set_command_output(
                                        "error",
                                        vec![
                                            format!("command failed: {err}"),
                                            "try `/help` for supported commands".to_string(),
                                        ],
                                    );
                                } else {
                                    db.save_store(&app.store)?;
                                }
                            }
                        }
                        KeyCode::Char(ch)
                            if !key.modifiers.contains(KeyModifiers::CONTROL)
                                && !key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            app.chat_editor_mut().insert_char(ch);
                        }
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

pub fn restore_store_from_db(app: &mut App, db: &AppDb) -> Result<(), Box<dyn Error>> {
    let persisted = db.load_state()?;

    for actor in persisted.actors {
        app.store
            .restore_actor_cache(&actor.did, &actor.handle, actor.bio)?;
    }

    for collection in persisted.post_collections {
        app.store.cache_post_collection(collection);
    }

    for entry in persisted.clearsky_lists {
        app.store
            .restore_clearsky_lists(&entry.actor_did, entry.lists)?;
    }

    Ok(())
}

fn normalize_actor_ref(actor: &str) -> &str {
    actor.strip_prefix('@').unwrap_or(actor)
}

fn serialize_notification_summary_for_llm(notification: &Notification) -> String {
    let mut lines = vec![
        format!("reason: {}", notification.data.reason),
        format!(
            "author_handle: {}",
            notification.author.data.handle.as_str()
        ),
        format!("author_did: {}", notification.author.data.did.as_str()),
        format!("uri: {}", notification.data.uri),
        format!("indexed_at: {}", notification.indexed_at.as_ref()),
    ];

    if let Some(reason_subject) = notification.data.reason_subject.as_deref() {
        lines.push(format!("reason_subject: {reason_subject}"));
    }

    let reply = extract_reply_node(&notification.data.record);
    if let Some(parent_uri) = reply.parent_uri.as_deref() {
        lines.push(format!("reply_parent_uri: {parent_uri}"));
    }
    if let Some(root_uri) = reply.root_uri.as_deref() {
        lines.push(format!("reply_root_uri: {root_uri}"));
    }
    if let Some(text) = reply.text.as_deref() {
        let preview = text.lines().next().unwrap_or_default();
        if !preview.is_empty() {
            lines.push(format!("reply_text_preview: {preview}"));
        }
    }

    lines.join("\n")
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

    let mut lines = vec![
        format!("reply notifications for {}", profile.handle),
        String::new(),
    ];
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
            header: format!("@{} pinned post", post.author_handle),
            uri: post.uri.clone(),
            text: post.body.clone(),
            children: build_post_nodes(
                store
                    .get_pinned_post_replies(&post.uri)
                    .map(|replies| replies.to_vec())
                    .unwrap_or_default(),
            ),
        })
        .collect::<Vec<_>>();

    let mut lines = vec![
        format!("pinned posts for {}", profile.handle),
        String::new(),
    ];
    lines.extend(
        render_post_nodes(&nodes)
            .lines
            .into_iter()
            .map(line_to_string),
    );
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
        "  /bio handle.bsky.social".to_string(),
        "  /replies_from handle.bsky.social".to_string(),
        "  /pins handle.bsky.social".to_string(),
        "  /agents".to_string(),
        "  /notifications".to_string(),
        "  /context".to_string(),
        "  /stop".to_string(),
        "  /task".to_string(),
        "  /clear".to_string(),
        "  /help".to_string(),
        "  /quit".to_string(),
        String::new(),
        "Any other input is sent to evil_gemma over local OpenAI-compatible REST.".to_string(),
        "Default endpoint: http://127.0.0.1:5000/v1/chat/completions".to_string(),
        "Override with EVIL_GEMMA_BASE_URL and EVIL_GEMMA_MODEL.".to_string(),
        String::new(),
        "Navigation: Up/Down selects the active left-list item; Enter opens a notification; PageUp/PageDown scroll detail; Tab toggles AI chat view.".to_string(),
    ]
}

fn is_local_command(verb: &str) -> bool {
    matches!(
        verb,
        "/bio"
            | "/replies_from"
            | "/pins"
            | "/agents"
            | "/notifications"
            | "/context"
            | "/stop"
            | "/task"
            | "/clear"
            | "/help"
            | "/q"
            | "/quit"
            | "/exit"
            | "bio"
            | "replies_from"
            | "pins"
            | "agents"
            | "notifications"
            | "context"
            | "stop"
            | "task"
            | "clear"
            | "help"
            | "q"
            | "quit"
            | "exit"
    )
}

fn resolve_system_prompt_path(path: String) -> PathBuf {
    let candidate = PathBuf::from(&path);
    if candidate.is_absolute() {
        candidate
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(candidate)
    }
}

fn line_to_string(line: ratatui::text::Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn draw_ui(frame: &mut Frame, app: &App) {
    let (chunks, input_area, status_area) = ui::tui_renderer::layout(frame);

    if app.is_fullscreen_overlay() {
        match &app.detail {
            DetailView::Command { .. } => ui::tui_renderer::render_fullscreen_text(
                frame,
                chunks[0],
                &app.detail_title(),
                app.detail_text(),
                app.detail_scroll,
            ),
            DetailView::ContextVisualization(data) => {
                ui::tui_renderer::render_context_visualization(
                    frame,
                    chunks[0],
                    data,
                    app.detail_scroll,
                );
            }
            DetailView::Notification | DetailView::Agents => {}
        }
    } else {
        match app.current_split_view() {
            SplitView::Notifications => {
                let selected = if app.store.notifications.is_empty() {
                    None
                } else {
                    Some(app.selected)
                };
                ui::tui_renderer::render_list_detail_split(
                    frame,
                    chunks[0],
                    "Notifications",
                    app.notification_items(),
                    selected,
                    &app.detail_title(),
                    app.detail_text(),
                    app.detail_scroll,
                );
            }
            SplitView::Agents => {
                let items = app.agent_items();
                let selected = (!items.is_empty()).then_some(app.selected_agent);
                ui::tui_renderer::render_list_detail_split(
                    frame,
                    chunks[0],
                    "Agents",
                    items,
                    selected,
                    &app.detail_title(),
                    app.detail_text(),
                    app.detail_scroll,
                );
            }
        }
    }

    ui::tui_renderer::render_input(frame, input_area, app.input.as_str());
    ui::tui_renderer::render_status(frame, status_area, &app.status);

    let cursor_x = input_area
        .x
        .saturating_add(app.input.chars().count() as u16 + 1)
        .min(input_area.x + input_area.width.saturating_sub(2));
    let cursor_y = input_area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_toggle_restores_last_non_chat_detail() {
        let mut app = App::new("tester".to_string());
        app.detail = DetailView::Agents;
        app.chat_title = Some("evil_gemma: test".to_string());

        app.toggle_ai_chat_view();
        assert_eq!(app.presentation_mode(), PresentationMode::StdoutChat);

        app.toggle_ai_chat_view();
        assert_eq!(app.presentation_mode(), PresentationMode::Tui);
        assert!(matches!(app.detail, DetailView::Agents));
    }

    #[test]
    fn command_output_from_chat_returns_to_tui() {
        let mut app = App::new("tester".to_string());
        app.detail = DetailView::Notification;
        app.chat_title = Some("evil_gemma: test".to_string());
        app.show_chat_output(true);

        app.set_command_output("/help", vec!["hello".to_string()]);

        assert_eq!(app.presentation_mode(), PresentationMode::Tui);
        assert!(matches!(app.detail, DetailView::Command { .. }));
        assert!(matches!(
            app.deferred_detail,
            Some(DetailView::Notification)
        ));
    }
}
