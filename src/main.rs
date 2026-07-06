use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::text::Text;
use ratatui::widgets::ListItem;
use ratatui::{Frame, Terminal};
use std::env;
use std::error::Error;
use std::fs;
use std::future::Future;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;
use tokio::time::timeout;

mod clearsky_v1;
mod db;
#[allow(dead_code)]
mod harness;
mod model;
mod net_backend;
mod post;
mod ui;
mod visualization;

use crate::db::AppDb;
use crate::harness::agents::{AgentGraph, AgentNodeStatus};
use crate::harness::context_window::{
    BuiltContextWindow, LLMContext, ProviderContextLimits, approximate_tokens,
    build_context_window_report,
};
use crate::harness::context_window_logger::{
    log_agent_graph, log_chat_transcript, log_current_task, log_root_prompt_snapshot,
    reset_debug_dir,
};
use crate::harness::llm_api::{ChatMessage, LlmApiClient, OpenAiRestConfig};
use crate::harness::runtime::{
    ContextMessage, ContextMessageKind, RootRunState, RootRunStatus, SuccessfulRootLlmSearch,
    TranscriptEntryKind,
};
use crate::harness::tools::{
    BlueskyTools, ToolProgressEvent, parse_prompt_tool_call, prompt_tool_protocol_instructions,
};
use crate::net_backend::{
    ActorProfile, CachedThreadReply, NotificationStore, ensure_actor_profile_cached,
    ensure_clearsky_lists_cached, ensure_pinned_posts_cached, ensure_recent_posts_cached,
    extract_reply_node, poll_notifications,
};
use crate::post::{PostNode, render_post_nodes};
use crate::visualization::context::{
    ContextCategory, ContextSegment, ContextVisualizationData, PromptContextSnapshot,
    snapshot_from_agent_node,
};

const POLL_INTERVAL: StdDuration = StdDuration::from_secs(30);
const UI_TICK: StdDuration = StdDuration::from_millis(200);
const DEFAULT_EVIL_GEMMA_BASE_URL: &str = "http://127.0.0.1:5000";
const DEFAULT_EVIL_GEMMA_MODEL: &str = "gemma-4-local";
const DEFAULT_SYSTEM_PROMPT_PATH: &str = "system_prompt.md";
const DEFAULT_DB_PATH: &str = "shock_absorber.sqlite3";
const MAX_TOOL_CALL_ROUNDS: usize = 3;
const INITIAL_COLLECTION_REFRESH_TIMEOUT: StdDuration = StdDuration::from_secs(30);

#[derive(Clone)]
enum DetailView {
    Notification,
    Command { title: String, lines: Vec<String> },
    AiChat { title: String, text: Text<'static> },
    ContextVisualization(ContextVisualizationData),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppActivity {
    NotificationDetail,
    CommandOverlay,
    ContextVisualization,
    AiChat,
}

struct App {
    store: NotificationStore,
    input: String,
    selected: usize,
    opened_notification: Option<usize>,
    selected_actor: Option<ActorProfile>,
    detail_scroll: u16,
    detail: DetailView,
    root_conversation: Vec<ConversationTurn>,
    root_run: Option<RootRunState>,
    active_root_run: Option<ActiveRootRunTask>,
    deferred_detail: Option<DetailView>,
    ai_chat_detail: Option<DetailView>,
    status: String,
    should_quit: bool,
}

struct ActiveRootRunTask {
    query: String,
    keep_context_overlay: bool,
    receiver: UnboundedReceiver<RootRunEvent>,
    handle: JoinHandle<()>,
}

enum RootRunEvent {
    Progress(RootRunState),
    ToolProgress(ToolProgressEvent),
    Completed {
        root_run: RootRunState,
        store: NotificationStore,
        response: String,
    },
    Failed {
        root_run: Option<RootRunState>,
        error: String,
    },
}

#[derive(Clone)]
struct ConversationTurn {
    user: String,
    assistant: String,
}

struct EvilGemmaConfig {
    client: LlmApiClient,
    system_prompt: String,
}

impl EvilGemmaConfig {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let base_url = env::var("EVIL_GEMMA_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_EVIL_GEMMA_BASE_URL.to_string());
        let model_name =
            env::var("EVIL_GEMMA_MODEL").unwrap_or_else(|_| DEFAULT_EVIL_GEMMA_MODEL.to_string());
        let system_prompt_path = env::var("SYSTEM_PROMPT_PATH")
            .unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT_PATH.to_string());

        let config = OpenAiRestConfig::llama_cpp(base_url, model_name);
        let system_prompt = fs::read_to_string(resolve_system_prompt_path(system_prompt_path))?;

        Ok(Self {
            client: LlmApiClient::new(config),
            system_prompt,
        })
    }
}

impl App {
    fn new(handle: String) -> Self {
        Self {
            store: NotificationStore::new(),
            input: String::new(),
            selected: 0,
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
            ai_chat_detail: None,
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
            DetailView::Command { title, .. } => title.clone(),
            DetailView::AiChat { title, .. } => title.clone(),
            DetailView::ContextVisualization(data) => data.title.clone(),
        }
    }

    fn current_activity(&self) -> AppActivity {
        match self.detail {
            DetailView::Notification => AppActivity::NotificationDetail,
            DetailView::Command { .. } => AppActivity::CommandOverlay,
            DetailView::AiChat { .. } => AppActivity::AiChat,
            DetailView::ContextVisualization(_) => AppActivity::ContextVisualization,
        }
    }

    fn is_fullscreen_overlay(&self) -> bool {
        matches!(
            self.detail,
            DetailView::Command { .. }
                | DetailView::AiChat { .. }
                | DetailView::ContextVisualization(_)
        )
    }

    fn detail_text(&self) -> Text<'static> {
        match &self.detail {
            DetailView::Notification => self.notification_detail_text(),
            DetailView::Command { lines, .. } => Text::from(lines.join("\n")),
            DetailView::AiChat { text, .. } => text.clone(),
            DetailView::ContextVisualization(_) => Text::from(""),
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

    fn notification_detail_text(&self) -> Text<'static> {
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

        Text::from(lines.join("\n"))
    }

    async fn execute_command(
        &mut self,
        agent: &Arc<BskyAgent>,
        evil_gemma: &Arc<EvilGemmaConfig>,
    ) -> Result<(), Box<dyn Error>> {
        let command = self.input.trim().to_owned();
        self.input.clear();

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
                ensure_recent_posts_cached(agent, &mut self.store, &profile.did, 20).await?;
                ensure_pinned_posts_cached(agent, &mut self.store, &profile.did).await?;
                ensure_clearsky_lists_cached(&mut self.store, &profile.did).await?;
                let lines = format_pins_output(&self.store, &profile);
                self.set_command_output(format!("/pins {}", profile.handle), lines);
                self.status = format!("pins loaded for {}", profile.handle);
            }
            "clear" | "/clear" => {
                self.clear_root_conversation();
                self.root_run = None;
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
                self.detail =
                    DetailView::ContextVisualization(self.build_context_visualization(evil_gemma));
                self.status = "context visualization loaded".to_string();
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

    #[allow(dead_code)]
    async fn run_evil_gemma_query(
        &mut self,
        agent: &BskyAgent,
        evil_gemma: &EvilGemmaConfig,
        query: String,
    ) -> Result<(), Box<dyn Error>> {
        let keep_context_overlay = matches!(self.detail, DetailView::ContextVisualization(_));
        self.deferred_detail = None;
        let root_context_window = {
            let tools = BlueskyTools::new();
            build_tool_aware_query_context_window(
                self.selected_actor_did(),
                self.selected_actor_summary(),
                &tools,
                &self.root_conversation,
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
        let mut response = String::new();
        let mut hit_tool_round_limit = false;
        let mut last_failed_read_collection_id: Option<String> = None;
        let mut agent_graph = AgentGraph::new_root("Root Agent");
        agent_graph.set_context_window(agent_graph.root_agent_id(), root_context_window.clone());
        agent_graph.set_result_summary(agent_graph.root_agent_id(), query.clone());
        let mut root_run = RootRunState::new(
            query.clone(),
            root_context_window.clone(),
            messages,
            agent_graph,
        );
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

        for round in 0..MAX_TOOL_CALL_ROUNDS {
            response = evil_gemma
                .client
                .complete_chat(root_run.llm_messages(), 1024)
                .await?;

            let Some(tool_call) = parse_prompt_tool_call(&response) else {
                root_run.record_round(round + 1, response.clone(), None, false, None);
                break;
            };
            root_run.record_tool_call(round + 1, &tool_call, true)?;

            let tool_name = tool_call.name.clone();
            let tool_args = serde_json::to_string(&tool_call.args)?;
            let duplicate_search_after_failed_read = repeated_llm_search_after_failed_read(
                &tool_call,
                last_failed_read_collection_id.as_deref(),
            );
            let selected_actor_did = self.selected_actor_did().cloned();
            let mut prep_log = vec![format!(
                "[tool_prep] inspecting tool `{}` for possible initial collection refresh",
                tool_call.name
            )];
            if let Some(collection_id) = duplicate_search_after_failed_read.as_deref() {
                prep_log.push(format!(
                    "[tool_prep] prevented immediate re-search of `{collection_id}` after a failed `read_collection_item`"
                ));
            }
            let actor_dids = if duplicate_search_after_failed_read.is_some() {
                Vec::new()
            } else {
                planned_tool_call_refresh_targets(&self.store, selected_actor_did, &tool_call)
            };
            if keep_context_overlay {
                root_run.set_active_tool_entry(Some(build_tool_entry(
                    &tool_name, &tool_args, &prep_log, None,
                )));
                self.root_run = Some(root_run.clone());
                self.set_ai_chat_output(
                    format!("evil_gemma: {query}"),
                    root_run.render_output_text(),
                    false,
                );
            } else {
                root_run.set_active_tool_entry(Some(build_tool_entry(
                    &tool_name, &tool_args, &prep_log, None,
                )));
                self.set_evil_gemma_progress(&query, &root_run);
            }

            let mut prep_warnings = Vec::new();
            if actor_dids.is_empty() {
                prep_log.push("[tool_prep] no initial refresh needed".to_string());
                if !keep_context_overlay {
                    root_run.set_active_tool_entry(Some(build_tool_entry(
                        &tool_name, &tool_args, &prep_log, None,
                    )));
                    self.set_evil_gemma_progress(&query, &root_run);
                }
            } else {
                for did in actor_dids {
                    prep_log.push(format!(
                        "[tool_prep] initial refresh needed for actor {} before tool `{}`",
                        did.as_str(),
                        tool_call.name
                    ));
                    if !keep_context_overlay {
                        root_run.set_active_tool_entry(Some(build_tool_entry(
                            &tool_name, &tool_args, &prep_log, None,
                        )));
                        self.set_evil_gemma_progress(&query, &root_run);
                    }

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_recent_posts_cached {}",
                        did.as_str()
                    ));
                    if !keep_context_overlay {
                        root_run.set_active_tool_entry(Some(build_tool_entry(
                            &tool_name, &tool_args, &prep_log, None,
                        )));
                        self.set_evil_gemma_progress(&query, &root_run);
                    }
                    if let Err(message) = run_tool_prep_step(
                        INITIAL_COLLECTION_REFRESH_TIMEOUT,
                        ensure_recent_posts_cached(agent, &mut self.store, &did, 20),
                        format!("ensure_recent_posts_cached {}", did.as_str()),
                    )
                    .await
                    {
                        prep_log.push(format!(
                            "[tool_prep] initial refresh failed for {}: {message}",
                            did.as_str()
                        ));
                        prep_log.push(
                            "[tool_prep] continuing with already cached collections".to_string(),
                        );
                        if !keep_context_overlay {
                            root_run.set_active_tool_entry(Some(build_tool_entry(
                                &tool_name, &tool_args, &prep_log, None,
                            )));
                            self.set_evil_gemma_progress(&query, &root_run);
                        }
                        prep_warnings.push(format!(
                            "initial refresh for {} failed during recent-post fetch: {}",
                            did.as_str(),
                            message
                        ));
                        break;
                    }

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_pinned_posts_cached {}",
                        did.as_str()
                    ));
                    if !keep_context_overlay {
                        root_run.set_active_tool_entry(Some(build_tool_entry(
                            &tool_name, &tool_args, &prep_log, None,
                        )));
                        self.set_evil_gemma_progress(&query, &root_run);
                    }
                    if let Err(message) = run_tool_prep_step(
                        INITIAL_COLLECTION_REFRESH_TIMEOUT,
                        ensure_pinned_posts_cached(agent, &mut self.store, &did),
                        format!("ensure_pinned_posts_cached {}", did.as_str()),
                    )
                    .await
                    {
                        prep_log.push(format!(
                            "[tool_prep] initial refresh failed for {}: {message}",
                            did.as_str()
                        ));
                        prep_log.push(
                            "[tool_prep] continuing with already cached collections".to_string(),
                        );
                        if !keep_context_overlay {
                            root_run.set_active_tool_entry(Some(build_tool_entry(
                                &tool_name, &tool_args, &prep_log, None,
                            )));
                            self.set_evil_gemma_progress(&query, &root_run);
                        }
                        prep_warnings.push(format!(
                            "initial refresh for {} failed during pinned-post fetch: {}",
                            did.as_str(),
                            message
                        ));
                        break;
                    }

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_clearsky_lists_cached {}",
                        did.as_str()
                    ));
                    if !keep_context_overlay {
                        root_run.set_active_tool_entry(Some(build_tool_entry(
                            &tool_name, &tool_args, &prep_log, None,
                        )));
                        self.set_evil_gemma_progress(&query, &root_run);
                    }
                    if let Err(message) = run_tool_prep_step(
                        INITIAL_COLLECTION_REFRESH_TIMEOUT,
                        ensure_clearsky_lists_cached(&mut self.store, &did),
                        format!("ensure_clearsky_lists_cached {}", did.as_str()),
                    )
                    .await
                    {
                        prep_log.push(format!(
                            "[tool_prep] initial refresh failed for {}: {message}",
                            did.as_str()
                        ));
                        prep_log.push(
                            "[tool_prep] continuing with already cached collections".to_string(),
                        );
                        if !keep_context_overlay {
                            root_run.set_active_tool_entry(Some(build_tool_entry(
                                &tool_name, &tool_args, &prep_log, None,
                            )));
                            self.set_evil_gemma_progress(&query, &root_run);
                        }
                        prep_warnings.push(format!(
                            "initial refresh for {} failed during Clearsky list fetch: {}",
                            did.as_str(),
                            message
                        ));
                        break;
                    }

                    prep_log.push(format!(
                        "[tool_prep] initial refresh complete for {}",
                        did.as_str()
                    ));
                    if !keep_context_overlay {
                        root_run.set_active_tool_entry(Some(build_tool_entry(
                            &tool_name, &tool_args, &prep_log, None,
                        )));
                        self.set_evil_gemma_progress(&query, &root_run);
                    }
                }
            }

            if !keep_context_overlay {
                root_run.set_active_tool_entry(Some(build_tool_entry(
                    &tool_name,
                    &tool_args,
                    &prep_log,
                    Some("<running tool...>"),
                )));
                self.set_evil_gemma_progress(&query, &root_run);
            }
            let tools = BlueskyTools::new();
            let opened_notification = self.opened_notification().cloned();
            let tool_output = if let Some(collection_id) =
                duplicate_search_after_failed_read.as_deref()
            {
                crate::harness::tools::ToolExecutionOutput {
                    rendered: format!(
                        "Tool execution prevented: the previous `read_collection_item` failed for `{collection_id}` because the requested item URI was not one of the returned search results.\n\nReuse one of the existing `search_result_*_uri` values from the prior `llm_search` result, or answer directly from that grounded summary. Do not rerun the same collection search unchanged."
                    ),
                    context_windows: Vec::new(),
                    agent_node: None,
                }
            } else {
                match tools
                    .execute_prompt_tool_call(
                        &tool_call,
                        opened_notification.as_ref(),
                        agent,
                        &mut self.store,
                        &evil_gemma.client,
                        None,
                    )
                    .await
                {
                    Ok(output) => output,
                    Err(err) => crate::harness::tools::ToolExecutionOutput {
                        rendered: format!("Tool execution failed: {err}"),
                        context_windows: Vec::new(),
                        agent_node: None,
                    },
                }
            };
            if let Some(agent_node) = tool_output.agent_node.clone() {
                let root_agent_id = root_run.agent_graph().root_agent_id();
                root_run
                    .agent_graph_mut()
                    .attach_template(root_agent_id, agent_node);
            }
            let tool_output = if prep_warnings.is_empty() {
                tool_output.rendered
            } else {
                format!(
                    "Tool preparation warning:\n{}\n\n{}",
                    prep_warnings.join("\n"),
                    tool_output.rendered
                )
            };
            let hard_tool_failure_answer =
                deterministic_tool_failure_answer(&tool_name, &tool_output);

            if tool_name == "read_collection_item" {
                if tool_output.contains("Tool execution failed:") {
                    last_failed_read_collection_id = tool_call
                        .args
                        .get("collection_id")
                        .and_then(|value| value.as_str())
                        .map(str::to_string);
                } else {
                    last_failed_read_collection_id = None;
                }
            } else if duplicate_search_after_failed_read.is_none() {
                last_failed_read_collection_id = None;
            }

            root_run.push_transcript_entry(
                TranscriptEntryKind::ToolCall,
                build_tool_entry(&tool_name, &tool_args, &prep_log, Some(&tool_output)),
            );
            root_run.set_active_tool_entry(None);
            let tool_result_summary =
                compact_tool_result_for_root_context(&tool_name, &tool_output);
            root_run.push_message(
                ContextMessageKind::ToolRequest,
                "assistant",
                response.clone(),
            );
            root_run.push_message(
                ContextMessageKind::ToolResult,
                "user",
                format!(
                    "Tool Result\nname: {tool_name}\nargs: {tool_args}\n\n{tool_result_summary}\n\n{}",
                    if round + 1 == MAX_TOOL_CALL_ROUNDS {
                        "This was the final allowed tool round. Answer the original query directly now. Do not request another tool or emit a TOOL_CALL block."
                    } else {
                        "Use this result to answer the original query, or request exactly one more tool if needed."
                    }
                ),
            );
            root_run.record_round(
                round + 1,
                response.clone(),
                Some(tool_call.clone()),
                duplicate_search_after_failed_read.is_none(),
                Some(tool_output.clone()),
            );
            if let Some(failure_answer) = hard_tool_failure_answer {
                response = failure_answer;
                break;
            }
            hit_tool_round_limit = round + 1 == MAX_TOOL_CALL_ROUNDS;
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
            self.root_run = Some(root_run.clone());
            if keep_context_overlay {
                self.set_ai_chat_output(
                    format!("evil_gemma: {query}"),
                    root_run.render_output_text(),
                    false,
                );
            } else {
                self.set_evil_gemma_progress(&query, &root_run);
            }
        }

        if hit_tool_round_limit && parse_prompt_tool_call(&response).is_some() {
            root_run.push_message(
                ContextMessageKind::ToolRequest,
                "assistant",
                response.clone(),
            );
            root_run.push_message(
                ContextMessageKind::RoundLimitPrompt,
                "user",
                "You have already used the maximum number of tool rounds. Answer the original query directly using the tool results already provided. Do not emit TOOL_CALL.",
            );
            response = evil_gemma
                .client
                .complete_chat(root_run.llm_messages(), 1024)
                .await
                .unwrap_or_else(|_| {
                    "Tool loop stopped after the configured maximum number of tool rounds without a final answer.".to_string()
                });
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
        }

        if parse_prompt_tool_call(&response).is_none() && response_looks_incomplete(&response) {
            root_run.push_message(
                ContextMessageKind::AssistantReply,
                "assistant",
                response.clone(),
            );
            root_run.push_message(
                ContextMessageKind::RepairPrompt,
                "user",
                "Your previous answer appears cut off. Finish the answer directly in at most one short paragraph plus up to 3 bullets. Start with a bottom-line conclusion sentence. Do not emit TOOL_CALL.",
            );
            response = evil_gemma
                .client
                .complete_chat(root_run.llm_messages(), 320)
                .await
                .unwrap_or(response);
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
        }

        root_run.set_final_response(response.clone());
        let output_lines = root_run.render_output_lines();
        if keep_context_overlay {
            self.set_ai_chat_output(
                format!("evil_gemma: {query}"),
                root_run.render_output_text(),
                false,
            );
            self.status = "evil_gemma response ready; dismiss /context to view output".to_string();
        } else {
            self.set_ai_chat_output(
                format!("evil_gemma: {query}"),
                root_run.render_output_text(),
                true,
            );
            self.status = "evil_gemma response loaded".to_string();
        }
        self.root_conversation.push(ConversationTurn {
            user: query,
            assistant: response.clone(),
        });
        let root_agent_id = root_run.agent_graph().root_agent_id();
        root_run
            .agent_graph_mut()
            .set_status(root_agent_id, AgentNodeStatus::Completed);
        if let Some(last_turn) = self.root_conversation.last() {
            root_run
                .agent_graph_mut()
                .set_result_summary(root_agent_id, last_turn.assistant.clone());
        }
        root_run.set_status(RootRunStatus::Completed);
        let mut final_messages = root_run.messages().to_vec();
        final_messages.push(ContextMessage {
            kind: ContextMessageKind::AssistantReply,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            },
        });
        let final_context_visualization = build_live_context_visualization(
            "/context",
            &final_messages,
            evil_gemma.system_prompt.trim(),
            prompt_tool_protocol_instructions(),
            root_run.root_context_window(),
            root_run.agent_graph(),
            &evil_gemma.client.context_limits(),
        );
        root_run.set_context_visualization(final_context_visualization.clone());
        let debug_base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let _ = log_agent_graph(&debug_base_dir, root_run.agent_graph());
        let _ = log_chat_transcript(&debug_base_dir, &output_lines);
        let _ = log_current_task(
            &debug_base_dir,
            root_run.agent_graph(),
            Some(root_run.query()),
        );
        if let Some(root_snapshot) = final_context_visualization.windows.first() {
            let _ = log_root_prompt_snapshot(&debug_base_dir, root_snapshot);
        }
        self.root_run = Some(root_run);
        Ok(())
    }

    fn set_evil_gemma_progress(&mut self, query: &str, run: &RootRunState) {
        self.root_run = Some(run.clone());
        self.set_ai_chat_output(
            format!("evil_gemma: {query}"),
            run.render_output_text(),
            true,
        );
    }

    fn set_ai_chat_output<T: Into<String>>(
        &mut self,
        title: T,
        text: Text<'static>,
        visible: bool,
    ) {
        let detail = DetailView::AiChat {
            title: title.into(),
            text,
        };
        self.ai_chat_detail = Some(detail.clone());
        if visible {
            if !matches!(self.detail, DetailView::AiChat { .. }) {
                self.deferred_detail = Some(self.detail.clone());
            }
            self.detail_scroll = 0;
            self.detail = detail;
        }
    }

    fn set_command_output<T: Into<String>>(&mut self, title: T, lines: Vec<String>) {
        self.detail_scroll = 0;
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
    }

    fn toggle_ai_chat_view(&mut self) {
        match self.current_activity() {
            AppActivity::AiChat => {
                self.detail_scroll = 0;
                if let Some(detail) = self.deferred_detail.take() {
                    self.detail = detail;
                } else {
                    self.detail = DetailView::Notification;
                }
                self.status = "returned to previous view".to_string();
            }
            _ => {
                let Some(ai_chat_detail) = self.ai_chat_detail.clone() else {
                    self.status = "no ai chat view available".to_string();
                    return;
                };
                self.deferred_detail = Some(self.detail.clone());
                self.detail_scroll = 0;
                self.detail = ai_chat_detail;
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
                selected_actor_summary.clone(),
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
        if keep_context_overlay {
            self.set_ai_chat_output(
                format!("evil_gemma: {query}"),
                root_run.render_output_text(),
                false,
            );
            self.status = "evil_gemma run started; dismiss /context to view output".to_string();
        } else {
            self.set_evil_gemma_progress(&query, &root_run);
            self.status = "evil_gemma run started".to_string();
        }

        let (sender, receiver) = unbounded_channel();
        let task_query = query.clone();
        let runtime = tokio::runtime::Handle::current();
        let handle = tokio::task::spawn_blocking(move || {
            runtime.block_on(async move {
                run_root_query_task(
                    agent,
                    evil_gemma,
                    store,
                    selected_notification,
                    selected_actor_did,
                    selected_actor_summary,
                    root_conversation,
                    task_query,
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
        if let Some(root_run) = self.root_run.as_ref() {
            self.set_ai_chat_output(
                format!("evil_gemma: {}", active.query),
                root_run.render_output_text(),
                !active.keep_context_overlay,
            );
        }
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
                    self.set_ai_chat_output(
                        format!("evil_gemma: {query}"),
                        root_run.render_output_text(),
                        !keep_context_overlay,
                    );
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
                        let text = root_run.render_output_text();
                        self.set_ai_chat_output(
                            format!("evil_gemma: {query}"),
                            text,
                            !keep_context_overlay,
                        );
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
                    self.set_ai_chat_output(
                        format!("evil_gemma: {query}"),
                        root_run.render_output_text(),
                        !keep_context_overlay,
                    );
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
                        self.set_ai_chat_output(
                            format!("evil_gemma: {query}"),
                            root_run.render_output_text(),
                            !keep_context_overlay,
                        );
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let handle = env::var("BSKY_HANDLE")?;
    let password = env::var("BSKY_APP_PASSWORD")?;

    let agent = Arc::new(BskyAgent::builder().build().await?);
    agent.login(&handle, &password).await?;
    let evil_gemma = Arc::new(EvilGemmaConfig::from_env()?);
    let db_path =
        env::var("SHOCK_ABSORBER_DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());
    let db = AppDb::new(resolve_db_path(db_path))?;
    let mut app = App::new(handle);
    restore_store_from_db(&mut app.store, &db)?;
    app.status = format!("{} | db {}", app.status, db.path().display());
    let debug_base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = reset_debug_dir(&debug_base_dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, agent, evil_gemma, app, &db).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    agent: Arc<BskyAgent>,
    evil_gemma: Arc<EvilGemmaConfig>,
    mut app: App,
    db: &AppDb,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_poll = Instant::now() - POLL_INTERVAL;

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
                            app.open_selected_notification();
                        } else if let Err(err) = app.execute_command(&agent, &evil_gemma).await {
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

async fn run_root_query_task(
    agent: Arc<BskyAgent>,
    evil_gemma: Arc<EvilGemmaConfig>,
    mut store: NotificationStore,
    selected_notification: Option<Notification>,
    selected_actor_did: Option<bsky_sdk::api::types::string::Did>,
    _selected_actor_summary: Option<String>,
    _root_conversation: Vec<ConversationTurn>,
    _query: String,
    mut root_run: RootRunState,
    sender: UnboundedSender<RootRunEvent>,
) {
    let result: Result<(RootRunState, NotificationStore, String), Box<dyn Error>> = async {
        let mut response = String::new();
        let mut hit_tool_round_limit = false;
        let mut last_failed_read_collection_id: Option<String> = None;

        let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

        for round in 0..MAX_TOOL_CALL_ROUNDS {
            response = evil_gemma
                .client
                .complete_chat(root_run.llm_messages(), 1024)
                .await?;

            let Some(tool_call) = parse_prompt_tool_call(&response) else {
                root_run.record_round(round + 1, response.clone(), None, false, None);
                break;
            };

            let tool_name = tool_call.name.clone();
            let tool_args = serde_json::to_string(&tool_call.args)?;
            let duplicate_search_after_failed_read = repeated_llm_search_after_failed_read(
                &tool_call,
                last_failed_read_collection_id.as_deref(),
            );
            let blocked_root_rerun = blocked_root_llm_search_rerun(
                &tool_call,
                root_run.latest_successful_llm_search(),
            );
            let mut prep_log = vec![format!(
                "[tool_prep] inspecting tool `{}` for possible initial collection refresh",
                tool_call.name
            )];
            if let Some(collection_id) = duplicate_search_after_failed_read.as_deref() {
                prep_log.push(format!(
                    "[tool_prep] prevented immediate re-search of `{collection_id}` after a failed `read_collection_item`"
                ));
            }
            if let Some(reason) = blocked_root_rerun.as_deref() {
                prep_log.push(format!(
                    "[tool_prep] blocked root llm_search rerun because a prior grounded result already covers this scope: {reason}"
                ));
            }
            let actor_dids = if duplicate_search_after_failed_read.is_some()
                || blocked_root_rerun.is_some()
            {
                Vec::new()
            } else {
                planned_tool_call_refresh_targets(&store, selected_actor_did.clone(), &tool_call)
            };
            root_run.record_tool_call(
                round + 1,
                &tool_call,
                duplicate_search_after_failed_read.is_none() && blocked_root_rerun.is_none(),
            )?;

            root_run.set_active_tool_entry(Some(build_tool_entry(
                &tool_name,
                &tool_args,
                &prep_log,
                None,
            )));
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

            let mut prep_warnings = Vec::new();
            if actor_dids.is_empty() {
                prep_log.push("[tool_prep] no initial refresh needed".to_string());
                root_run.set_active_tool_entry(Some(build_tool_entry(
                    &tool_name,
                    &tool_args,
                    &prep_log,
                    None,
                )));
                let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
            } else {
                for did in actor_dids {
                    prep_log.push(format!(
                        "[tool_prep] initial refresh needed for actor {} before tool `{}`",
                        did.as_str(),
                        tool_call.name
                    ));
                    root_run.set_active_tool_entry(Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
                    let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_recent_posts_cached {}",
                        did.as_str()
                    ));
                    root_run.set_active_tool_entry(Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
                    let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
                    if let Err(message) = run_tool_prep_step(
                        INITIAL_COLLECTION_REFRESH_TIMEOUT,
                        ensure_recent_posts_cached(&agent, &mut store, &did, 20),
                        format!("ensure_recent_posts_cached {}", did.as_str()),
                    )
                    .await
                    {
                        prep_log.push(format!(
                            "[tool_prep] initial refresh failed for {}: {message}",
                            did.as_str()
                        ));
                        prep_log.push(
                            "[tool_prep] continuing with already cached collections".to_string(),
                        );
                        prep_warnings.push(format!(
                            "initial refresh for {} failed during recent-post fetch: {}",
                            did.as_str(),
                            message
                        ));
                        break;
                    }

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_pinned_posts_cached {}",
                        did.as_str()
                    ));
                    root_run.set_active_tool_entry(Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
                    let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
                    if let Err(message) = run_tool_prep_step(
                        INITIAL_COLLECTION_REFRESH_TIMEOUT,
                        ensure_pinned_posts_cached(&agent, &mut store, &did),
                        format!("ensure_pinned_posts_cached {}", did.as_str()),
                    )
                    .await
                    {
                        prep_log.push(format!(
                            "[tool_prep] initial refresh failed for {}: {message}",
                            did.as_str()
                        ));
                        prep_log.push(
                            "[tool_prep] continuing with already cached collections".to_string(),
                        );
                        prep_warnings.push(format!(
                            "initial refresh for {} failed during pinned-post fetch: {}",
                            did.as_str(),
                            message
                        ));
                        break;
                    }

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_clearsky_lists_cached {}",
                        did.as_str()
                    ));
                    root_run.set_active_tool_entry(Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
                    let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
                    if let Err(message) = run_tool_prep_step(
                        INITIAL_COLLECTION_REFRESH_TIMEOUT,
                        ensure_clearsky_lists_cached(&mut store, &did),
                        format!("ensure_clearsky_lists_cached {}", did.as_str()),
                    )
                    .await
                    {
                        prep_log.push(format!(
                            "[tool_prep] initial refresh failed for {}: {message}",
                            did.as_str()
                        ));
                        prep_log.push(
                            "[tool_prep] continuing with already cached collections".to_string(),
                        );
                        prep_warnings.push(format!(
                            "initial refresh for {} failed during Clearsky list fetch: {}",
                            did.as_str(),
                            message
                        ));
                        break;
                    }

                    prep_log.push(format!(
                        "[tool_prep] initial refresh complete for {}",
                        did.as_str()
                    ));
                    root_run.set_active_tool_entry(Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
                    let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
                }
            }

            root_run.set_active_tool_entry(Some(build_tool_entry(
                &tool_name,
                &tool_args,
                &prep_log,
                Some("<running tool...>"),
            )));
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

            let tools = BlueskyTools::new();
            let (tool_progress_sender, mut tool_progress_receiver) = unbounded_channel();
            let progress_sender = sender.clone();
            let forward_handle = tokio::spawn(async move {
                while let Some(event) = tool_progress_receiver.recv().await {
                    let _ = progress_sender.send(RootRunEvent::ToolProgress(event));
                }
            });
            let tool_output = if let Some(collection_id) = duplicate_search_after_failed_read.as_deref() {
                crate::harness::tools::ToolExecutionOutput {
                    rendered: format!(
                        "Tool execution prevented: the previous `read_collection_item` failed for `{collection_id}` because the requested item URI was not one of the returned search results.\n\nReuse one of the existing `search_result_*_uri` values from the prior `llm_search` result, or answer directly from that grounded summary. Do not rerun the same collection search unchanged."
                    ),
                    context_windows: Vec::new(),
                    agent_node: None,
                }
            } else if let Some(reason) = blocked_root_rerun.as_deref() {
                let prior = root_run
                    .latest_successful_llm_search()
                    .expect("blocked rerun requires prior grounded result");
                crate::harness::tools::ToolExecutionOutput {
                    rendered: format!(
                        "Tool execution prevented: a previous grounded `llm_search` result in this root run already covers this scope.\nreason: {reason}\n\nUse the existing grounded result unless you can name a materially new scope.\n\nprior_query: {}\nprior_summary: {}\nprior_collection_ids: {}",
                        prior.query,
                        prior.summary,
                        if prior.collection_ids.is_empty() {
                            "<none>".to_string()
                        } else {
                            prior.collection_ids.join(", ")
                        }
                    ),
                    context_windows: Vec::new(),
                    agent_node: None,
                }
            } else {
                match tools
                    .execute_prompt_tool_call(
                        &tool_call,
                        selected_notification.as_ref(),
                        &agent,
                        &mut store,
                        &evil_gemma.client,
                        Some(tool_progress_sender.clone()),
                    )
                    .await
                {
                    Ok(output) => output,
                    Err(err) => crate::harness::tools::ToolExecutionOutput {
                        rendered: format!("Tool execution failed: {err}"),
                        context_windows: Vec::new(),
                        agent_node: None,
                    },
                }
            };
            drop(tool_progress_sender);
            let _ = forward_handle.await;
            if let Some(agent_node) = tool_output.agent_node.clone() {
                let root_agent_id = root_run.agent_graph().root_agent_id();
                root_run.agent_graph_mut().attach_template(root_agent_id, agent_node);
            }
            let tool_output = if prep_warnings.is_empty() {
                tool_output.rendered
            } else {
                format!(
                    "Tool preparation warning:\n{}\n\n{}",
                    prep_warnings.join("\n"),
                    tool_output.rendered
                )
            };
            let hard_tool_failure_answer =
                deterministic_tool_failure_answer(&tool_name, &tool_output);

            if tool_name == "llm_search" && blocked_root_rerun.is_none() {
                if let Some(successful_result) =
                    extract_successful_root_llm_search_record(&tool_call, &tool_output)
                {
                    root_run.set_latest_successful_llm_search(Some(successful_result));
                }
            }

            if tool_name == "read_collection_item" {
                if tool_output.contains("Tool execution failed:") {
                    last_failed_read_collection_id = tool_call
                        .args
                        .get("collection_id")
                        .and_then(|value| value.as_str())
                        .map(str::to_string);
                } else {
                    last_failed_read_collection_id = None;
                }
            } else if duplicate_search_after_failed_read.is_none() && blocked_root_rerun.is_none() {
                last_failed_read_collection_id = None;
            }

            if blocked_root_rerun.is_some() {
                root_run.push_transcript_entry(
                    TranscriptEntryKind::Notice,
                    format!(
                        "Runtime Notice\nblocked root `llm_search` rerun and preserved the earlier grounded result"
                    ),
                );
            }
            root_run.push_transcript_entry(
                TranscriptEntryKind::ToolCall,
                build_tool_entry(&tool_name, &tool_args, &prep_log, Some(&tool_output)),
            );
            root_run.set_active_tool_entry(None);
            let tool_result_summary = compact_tool_result_for_root_context(&tool_name, &tool_output);
            root_run.push_message(
                ContextMessageKind::ToolRequest,
                "assistant",
                response.clone(),
            );
            root_run.push_message(
                ContextMessageKind::ToolResult,
                "user",
                format!(
                    "Tool Result\nname: {tool_name}\nargs: {tool_args}\n\n{tool_result_summary}\n\n{}",
                    if round + 1 == MAX_TOOL_CALL_ROUNDS {
                        "This was the final allowed tool round. Answer the original query directly now. Do not request another tool or emit a TOOL_CALL block."
                    } else {
                        "Use this result to answer the original query, or request exactly one more tool if needed."
                    }
                ),
            );
            root_run.record_round(
                round + 1,
                response.clone(),
                Some(tool_call.clone()),
                duplicate_search_after_failed_read.is_none() && blocked_root_rerun.is_none(),
                Some(tool_output.clone()),
            );
            if let Some(failure_answer) = hard_tool_failure_answer {
                response = fallback_or_failure_answer(
                    root_run.latest_successful_llm_search(),
                    &failure_answer,
                );
                break;
            }
            hit_tool_round_limit = round + 1 == MAX_TOOL_CALL_ROUNDS;
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
        }

        if hit_tool_round_limit && parse_prompt_tool_call(&response).is_some() {
            root_run.push_message(ContextMessageKind::ToolRequest, "assistant", response.clone());
            root_run.push_message(
                ContextMessageKind::RoundLimitPrompt,
                "user",
                "You have already used the maximum number of tool rounds. Answer the original query directly using the tool results already provided. Do not emit TOOL_CALL.",
            );
            response = evil_gemma
                .client
                .complete_chat(root_run.llm_messages(), 1024)
                .await
                .unwrap_or_else(|_| {
                    "Tool loop stopped after the configured maximum number of tool rounds without a final answer.".to_string()
                });
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
        }

        if parse_prompt_tool_call(&response).is_none() && response_looks_incomplete(&response) {
            root_run.push_message(ContextMessageKind::AssistantReply, "assistant", response.clone());
            root_run.push_message(
                ContextMessageKind::RepairPrompt,
                "user",
                "Your previous answer appears cut off. Finish the answer directly in at most one short paragraph plus up to 3 bullets. Start with a bottom-line conclusion sentence. Do not emit TOOL_CALL.",
            );
            response = evil_gemma
                .client
                .complete_chat(root_run.llm_messages(), 320)
                .await
                .unwrap_or(response);
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
        }

        root_run.set_final_response(response.clone());
        let root_agent_id = root_run.agent_graph().root_agent_id();
        root_run.agent_graph_mut().set_status(root_agent_id, AgentNodeStatus::Completed);
        root_run
            .agent_graph_mut()
            .set_result_summary(root_agent_id, response.clone());
        root_run.set_status(RootRunStatus::Completed);
        let mut final_messages = root_run.messages().to_vec();
        final_messages.push(ContextMessage {
            kind: ContextMessageKind::AssistantReply,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            },
        });
        let final_context_visualization = build_live_context_visualization(
            "/context",
            &final_messages,
            evil_gemma.system_prompt.trim(),
            prompt_tool_protocol_instructions(),
            root_run.root_context_window(),
            root_run.agent_graph(),
            &evil_gemma.client.context_limits(),
        );
        root_run.set_context_visualization(final_context_visualization.clone());
        let debug_base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let output_lines = root_run.render_output_lines();
        let _ = log_agent_graph(&debug_base_dir, root_run.agent_graph());
        let _ = log_chat_transcript(&debug_base_dir, &output_lines);
        let _ = log_current_task(&debug_base_dir, root_run.agent_graph(), Some(root_run.query()));
        if let Some(root_snapshot) = final_context_visualization.windows.first() {
            let _ = log_root_prompt_snapshot(&debug_base_dir, root_snapshot);
        }
        Ok((root_run, store, response))
    }
    .await;

    match result {
        Ok((root_run, store, response)) => {
            let _ = sender.send(RootRunEvent::Completed {
                root_run,
                store,
                response,
            });
        }
        Err(err) => {
            let _ = sender.send(RootRunEvent::Failed {
                root_run: None,
                error: err.to_string(),
            });
        }
    }
}

fn normalize_actor_ref(actor: &str) -> &str {
    actor.strip_prefix('@').unwrap_or(actor)
}

fn planned_tool_call_refresh_targets(
    store: &NotificationStore,
    selected_actor_did: Option<bsky_sdk::api::types::string::Did>,
    tool_call: &crate::harness::tools::PromptToolCall,
) -> Vec<bsky_sdk::api::types::string::Did> {
    let _ = selected_actor_did;
    let mut actor_dids = Vec::new();

    match tool_call.name.as_str() {
        "list_collections" => {
            if let Some(did) = tool_call
                .args
                .get("actor_did")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse().ok())
            {
                if actor_needs_initial_refresh(store, &did) {
                    actor_dids.push(did);
                }
            }
        }
        "llm_search" => {}
        "read_collection_item" => {
            if let Some(collection_id) = tool_call
                .args
                .get("collection_id")
                .and_then(|value| value.as_str())
            {
                if collection_needs_initial_refresh(store, collection_id) {
                    if let Some(did) = actor_did_from_collection_id(collection_id) {
                        actor_dids.push(did);
                    }
                }
            }
        }
        _ => {}
    }

    actor_dids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    actor_dids.dedup_by(|left, right| left.as_str() == right.as_str());
    actor_dids
}

fn build_live_context_visualization(
    title: &str,
    messages: &[ContextMessage],
    system_prompt: &str,
    tool_protocol: &str,
    root_context_window: &BuiltContextWindow,
    agent_graph: &AgentGraph,
    limits: &ProviderContextLimits,
) -> ContextVisualizationData {
    let root_snapshot = build_root_context_snapshot(
        messages,
        system_prompt,
        tool_protocol,
        root_context_window,
        limits,
    );
    let mut windows = vec![root_snapshot];
    windows.extend(
        agent_graph
            .descendant_ids_depth_first()
            .into_iter()
            .filter_map(|(depth, node_id)| agent_graph.node(node_id).map(|node| (depth, node)))
            .filter_map(|(depth, node)| snapshot_from_agent_node(node, depth)),
    );
    ContextVisualizationData::from_windows(title, windows)
}

fn build_root_context_snapshot(
    messages: &[ContextMessage],
    system_prompt: &str,
    tool_protocol: &str,
    root_context_window: &BuiltContextWindow,
    limits: &ProviderContextLimits,
) -> PromptContextSnapshot {
    let mut segments = Vec::new();

    let system_prompt_tokens = approximate_tokens(system_prompt.trim());
    if system_prompt_tokens > 0 {
        segments.push(ContextSegment {
            label: "System Prompt".to_string(),
            category: ContextCategory::SystemPrompt,
            tokens: system_prompt_tokens,
            truncated: false,
        });
    }

    let tool_instruction_tokens = approximate_tokens(tool_protocol.trim());
    if tool_instruction_tokens > 0 {
        segments.push(ContextSegment {
            label: "Tool Instructions".to_string(),
            category: ContextCategory::ToolInstructions,
            tokens: tool_instruction_tokens,
            truncated: false,
        });
    }

    if root_context_window.header_tokens > 0 {
        segments.push(ContextSegment {
            label: "Root Instructions".to_string(),
            category: ContextCategory::RootInstructions,
            tokens: root_context_window.header_tokens,
            truncated: false,
        });
    }

    for section in &root_context_window.sections {
        segments.push(ContextSegment {
            label: section.title.clone(),
            category: root_category_for_section(&section.title),
            tokens: section.used_tokens,
            truncated: section.truncated,
        });
    }

    let mut tool_round = 0usize;
    for entry in messages {
        if matches!(
            entry.kind,
            ContextMessageKind::InitialSystem | ContextMessageKind::InitialUserContext
        ) {
            continue;
        }
        let trimmed = entry.message.content.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (label, category, increment_round) = classify_context_message(entry, tool_round + 1);
        segments.push(ContextSegment {
            label,
            category,
            tokens: approximate_tokens(trimmed),
            truncated: false,
        });
        if increment_round {
            tool_round += 1;
        }
    }

    let used_input_tokens = segments.iter().map(|segment| segment.tokens).sum();
    PromptContextSnapshot {
        title: "Root Agent".to_string(),
        provider_name: limits.provider_name.clone(),
        model_name: limits.model_name.clone(),
        max_context_tokens: limits.max_context_tokens,
        reserved_output_tokens: limits.reserved_output_tokens,
        input_budget_tokens: limits.available_input_tokens(),
        used_input_tokens,
        truncated: root_context_window.truncated,
        segments,
    }
}

fn classify_context_message(
    message: &ContextMessage,
    next_tool_round: usize,
) -> (String, ContextCategory, bool) {
    match message.kind {
        ContextMessageKind::InitialSystem | ContextMessageKind::InitialUserContext => {
            (String::new(), ContextCategory::UiContext, false)
        }
        ContextMessageKind::ToolRequest => (
            format!("Tool Request #{next_tool_round}"),
            ContextCategory::ToolResults,
            false,
        ),
        ContextMessageKind::ToolResult => (
            format!("Tool Result #{next_tool_round}"),
            ContextCategory::ToolResults,
            true,
        ),
        ContextMessageKind::AssistantReply => (
            "Assistant Reply".to_string(),
            ContextCategory::UserAiChat,
            false,
        ),
        ContextMessageKind::UserFollowUp => (
            "User Follow-up".to_string(),
            ContextCategory::UserAiChat,
            false,
        ),
        ContextMessageKind::RoundLimitPrompt => (
            "Round Limit Prompt".to_string(),
            ContextCategory::CurrentTask,
            false,
        ),
        ContextMessageKind::RepairPrompt => (
            "Repair Prompt".to_string(),
            ContextCategory::CurrentTask,
            false,
        ),
    }
}

fn root_category_for_section(title: &str) -> ContextCategory {
    match title {
        "Tools" => ContextCategory::ToolDefinitions,
        "Search Hints" | "Current UI Context" => ContextCategory::UiContext,
        "Current Task" => ContextCategory::CurrentTask,
        "Recent Chat" => ContextCategory::UserAiChat,
        _ => ContextCategory::UiContext,
    }
}

fn compact_tool_result_for_root_context(tool_name: &str, tool_output: &str) -> String {
    match tool_name {
        "llm_search" => compact_llm_search_result_for_root_context(tool_output),
        _ => truncate_for_root_context(tool_output, 24),
    }
}

fn compact_llm_search_result_for_root_context(tool_output: &str) -> String {
    let mut kept = Vec::new();
    for line in tool_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if kept.is_empty() {
            kept.push(trimmed.to_string());
            continue;
        }
        if trimmed.starts_with("summary:")
            || trimmed.starts_with("selected_result_uri:")
            || trimmed.starts_with("selected_result_source_collection_id:")
            || trimmed.starts_with("selected_result_collection_id:")
            || trimmed.starts_with("selected_result_collection_label:")
            || trimmed.starts_with("collection_id:")
            || trimmed.starts_with("collection_label:")
            || trimmed.starts_with("status:")
            || trimmed.starts_with("error:")
        {
            kept.push(trimmed.to_string());
        }
    }
    if kept.is_empty() {
        return truncate_for_root_context(tool_output, 24);
    }
    kept.join("\n")
}

fn truncate_for_root_context(text: &str, max_lines: usize) -> String {
    let mut lines = text
        .lines()
        .take(max_lines)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if text.lines().count() > max_lines {
        lines.push("...".to_string());
    }
    lines.join("\n")
}

fn deterministic_tool_failure_answer(tool_name: &str, tool_output: &str) -> Option<String> {
    if tool_name != "llm_search" {
        return None;
    }

    if tool_output.contains("Tool execution failed:") {
        let failure_line = tool_output
            .lines()
            .find(|line| line.trim_start().starts_with("Tool execution failed:"))
            .map(str::trim)
            .unwrap_or("Tool execution failed.");
        let prep_warning = tool_output
            .lines()
            .find(|line| line.contains("Profile not found"))
            .map(str::trim);

        let mut lines = vec![
            "I couldn't inspect the requested cached collections, so I can't ground a sentiment answer."
                .to_string(),
            failure_line.to_string(),
        ];

        if let Some(prep_warning) = prep_warning {
            lines.push(prep_warning.to_string());
        }

        lines.push(
            "No list contents or reply evidence were successfully loaded for this search, so any sentiment summary would be speculative."
                .to_string(),
        );

        return Some(lines.join("\n\n"));
    }

    if tool_output.trim() == "No matching cached posts." {
        return Some(
            "The latest `llm_search` returned no grounded search results for that scope.\n\nI can't safely expand that into a sentiment or list-by-list analysis without inventing evidence."
                .to_string(),
        );
    }

    None
}

fn repeated_llm_search_after_failed_read(
    tool_call: &crate::harness::tools::PromptToolCall,
    last_failed_read_collection_id: Option<&str>,
) -> Option<String> {
    let _ = tool_call;
    let _ = last_failed_read_collection_id;
    None
}

fn blocked_root_llm_search_rerun(
    tool_call: &crate::harness::tools::PromptToolCall,
    prior: Option<&SuccessfulRootLlmSearch>,
) -> Option<String> {
    if tool_call.name != "llm_search" {
        return None;
    }
    let prior = prior?;
    let query = tool_call.args.get("query")?.as_str()?;
    let current_actor_refs = detect_actor_refs_for_guard(query);
    if current_actor_refs.is_empty() || current_actor_refs != prior.actor_refs {
        return None;
    }
    let current_intent = classify_root_llm_search_intent(query);
    if current_intent != prior.intent {
        return None;
    }
    let current_collection_targets = detect_collection_targets_in_query(query);
    if !current_collection_targets.is_empty()
        && current_collection_targets.iter().any(|target| {
            !prior
                .collection_ids
                .iter()
                .any(|existing| existing.starts_with(target))
        })
    {
        return None;
    }
    Some(
        "same actor and same reputation/list scope with no materially new collection target"
            .to_string(),
    )
}

fn extract_successful_root_llm_search_record(
    tool_call: &crate::harness::tools::PromptToolCall,
    tool_output: &str,
) -> Option<SuccessfulRootLlmSearch> {
    if tool_call.name != "llm_search" || !llm_search_output_is_grounded(tool_output) {
        return None;
    }
    let query = tool_call.args.get("query")?.as_str()?.to_string();
    let summary = tool_output
        .lines()
        .find_map(|line| line.trim().strip_prefix("summary:").map(str::trim))
        .filter(|summary| !summary.is_empty())?
        .to_string();
    Some(SuccessfulRootLlmSearch {
        query: query.clone(),
        rendered_result: tool_output.to_string(),
        summary,
        actor_refs: detect_actor_refs_for_guard(&query),
        collection_ids: extract_collection_ids_from_llm_output(tool_output),
        intent: classify_root_llm_search_intent(&query),
    })
}

fn llm_search_output_is_grounded(tool_output: &str) -> bool {
    let has_summary = tool_output.lines().any(|line| {
        line.trim()
            .strip_prefix("summary:")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    });
    let has_anchor = tool_output.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("search_result_") || trimmed.starts_with("selected_result_")
    });
    let success_blocks = tool_output
        .lines()
        .filter(|line| line.trim() == "status: ok")
        .count();
    has_summary && has_anchor && success_blocks >= 1
}

fn extract_collection_ids_from_llm_output(tool_output: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in tool_output.lines() {
        if let Some(value) = line.trim().strip_prefix("collection_id:") {
            let value = value.trim().to_string();
            if !ids.iter().any(|seen| seen == &value) {
                ids.push(value);
            }
        }
    }
    ids
}

fn classify_root_llm_search_intent(query: &str) -> String {
    let lower = query.to_ascii_lowercase();
    if [
        "reputation",
        "sentiment",
        "positive",
        "negative",
        "known for",
        "how are",
        "list",
        "lists",
        "accusation",
        "dispute",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        "reputation_lists".to_string()
    } else {
        "general".to_string()
    }
}

fn detect_actor_refs_for_guard(query: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for raw in query.split_whitespace() {
        let trimmed = raw.trim_matches(|ch: char| {
            matches!(
                ch,
                ',' | '.'
                    | '!'
                    | '?'
                    | ':'
                    | ';'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
                    | '"'
                    | '\''
            )
        });
        let candidate = trimmed.strip_prefix('@').unwrap_or(trimmed);
        let candidate = candidate.trim_end_matches("'s");
        if candidate.starts_with("did:")
            || (candidate.contains('.')
                && candidate
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_')))
        {
            if !refs.iter().any(|seen| seen == candidate) {
                refs.push(candidate.to_string());
            }
        }
    }
    refs
}

fn detect_collection_targets_in_query(query: &str) -> Vec<String> {
    let lower = query.to_ascii_lowercase();
    [
        "clearsky_lists",
        "recent_replies_received",
        "actor_profile",
        "recent_posts_unaddressed",
        "pinned_posts",
    ]
    .iter()
    .filter(|target| lower.contains(**target))
    .map(|target| target.to_string())
    .collect()
}

fn fallback_or_failure_answer(
    prior: Option<&SuccessfulRootLlmSearch>,
    failure_answer: &str,
) -> String {
    let Some(prior) = prior else {
        return failure_answer.to_string();
    };
    format!(
        "A later `llm_search` attempt failed, so I'm using the earlier grounded result from this run.\n\n{}\n\nDiagnostic from the later failed attempt:\n{}",
        prior.rendered_result, failure_answer
    )
}

fn build_tool_entry(
    tool_name: &str,
    tool_args: &str,
    prep_log: &[String],
    tool_result: Option<&str>,
) -> String {
    let mut entry = format!("Tool Call\nname: {tool_name}\nargs: {tool_args}\n\nTool Prep\n");
    if prep_log.is_empty() {
        entry.push_str("<no tool prep>");
    } else {
        entry.push_str(&prep_log.join("\n"));
    }
    if let Some(tool_result) = tool_result {
        entry.push_str("\n\nTool Result\n");
        entry.push_str(tool_result);
    }
    entry
}

fn response_looks_incomplete(response: &str) -> bool {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.ends_with(':') || trimmed.ends_with(',') || trimmed.ends_with(';') {
        return true;
    }

    if trimmed.ends_with("Here is the breakdown")
        || trimmed.ends_with("Here is the breakdown:")
        || trimmed.ends_with("Here is the breakdown of the sentiment")
        || trimmed.ends_with("Here is the breakdown of the sentiment:")
    {
        return true;
    }

    let last_line = trimmed.lines().last().unwrap_or(trimmed).trim();
    if last_line == "-" || last_line == "*" || last_line.chars().all(|ch| ch == '.') {
        return true;
    }

    false
}

async fn run_tool_prep_step<F, T>(
    step_timeout: StdDuration,
    future: F,
    label: String,
) -> Result<T, String>
where
    F: Future<Output = Result<T, Box<dyn Error>>>,
{
    timeout(step_timeout, future)
        .await
        .map_err(|_| format!("{label} timed out after {} seconds", step_timeout.as_secs()))?
        .map_err(|err| err.to_string())
}

fn actor_needs_initial_refresh(
    store: &NotificationStore,
    actor_did: &bsky_sdk::api::types::string::Did,
) -> bool {
    store.actor_post_collections(actor_did).is_empty()
}

fn collection_needs_initial_refresh(store: &NotificationStore, collection_id: &str) -> bool {
    match store.get_post_collection(collection_id) {
        Some(collection) => collection.posts.is_empty() && collection.last_refreshed_at == 0,
        None => actor_did_from_collection_id(collection_id).is_some(),
    }
}

fn actor_did_from_collection_id(collection_id: &str) -> Option<bsky_sdk::api::types::string::Did> {
    collection_id
        .split_once(':')
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.parse().ok())
}

fn build_tool_aware_query_context_window(
    selected_actor_did: Option<&bsky_sdk::api::types::string::Did>,
    selected_actor_summary: Option<String>,
    tools: &BlueskyTools,
    root_conversation: &[ConversationTurn],
    query: &str,
    llm_client: &LlmApiClient,
) -> crate::harness::context_window::BuiltContextWindow {
    let mut context =
        LLMContext::new("Use the available tools only when they materially improve the answer.");
    context.push_section("Tools", tools.render_tool_inventory());
    context.push_section(
        "Search Hints",
        selected_actor_did
            .map(search_hints_for_actor_did)
            .unwrap_or_else(|| {
                "Use `llm_search` with a natural-language `query` when you need Bluesky-grounded evidence about a handle/user or about a broader topic. The harness will decide whether to look up actors, hydrate actor collections, or search Bluesky posts globally.".to_string()
            }),
    );

    context.push_section(
        "Current UI Context",
        selected_actor_summary
            .unwrap_or_else(|| "No actor is currently selected in the UI.".to_string()),
    );
    context.push_section("Current Task", query);

    if !root_conversation.is_empty() {
        context.push_section(
            "Recent Chat",
            root_conversation
                .iter()
                .map(|turn| {
                    format!(
                        "user:\n{}\n\nassistant:\n{}",
                        turn.user.trim(),
                        turn.assistant.trim()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n"),
        );
    }

    build_context_window_report(&context, &llm_client.context_limits())
}

fn search_hints_for_actor_did(did: &bsky_sdk::api::types::string::Did) -> String {
    let did = did.as_str();
    format!(
        "The selected actor is {did}. Use `llm_search` with a natural-language `query` when you need grounded evidence about this actor or related topics. The harness may reuse cached actor collections, load more actor data, or search Bluesky posts globally as needed."
    )
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

#[cfg(test)]
mod root_guard_tests {
    use super::{
        blocked_root_llm_search_rerun, classify_root_llm_search_intent,
        extract_successful_root_llm_search_record, fallback_or_failure_answer,
    };
    use crate::harness::runtime::SuccessfulRootLlmSearch;
    use crate::harness::tools::PromptToolCall;
    use serde_json::json;

    #[test]
    fn blocks_same_scope_root_llm_search_rerun() {
        let prior = SuccessfulRootLlmSearch {
            query: "What is the sentiment about elsyluna.bsky.social?".to_string(),
            rendered_result: "summary: grounded\ncollection_id: clearsky_lists:did:plc:testactor\nstatus: ok\nsearch_result_1_uri: at://one".to_string(),
            summary: "grounded".to_string(),
            actor_refs: vec!["elsyluna.bsky.social".to_string()],
            collection_ids: vec!["clearsky_lists:did:plc:testactor".to_string()],
            intent: "reputation_lists".to_string(),
        };
        let tool_call = PromptToolCall {
            name: "llm_search".to_string(),
            args: json!({"query":"How is elsyluna.bsky.social known on lists?"}),
        };

        assert!(blocked_root_llm_search_rerun(&tool_call, Some(&prior)).is_some());
    }

    #[test]
    fn preserves_prior_grounded_result_on_failure() {
        let prior = SuccessfulRootLlmSearch {
            query: "What is the sentiment about elsyluna.bsky.social?".to_string(),
            rendered_result: "summary: grounded earlier result".to_string(),
            summary: "grounded earlier result".to_string(),
            actor_refs: vec!["elsyluna.bsky.social".to_string()],
            collection_ids: vec!["clearsky_lists:did:plc:testactor".to_string()],
            intent: "reputation_lists".to_string(),
        };
        let answer = fallback_or_failure_answer(Some(&prior), "Tool execution failed: boom");
        assert!(answer.contains("earlier grounded result"));
        assert!(answer.contains("boom"));
    }

    #[test]
    fn extracts_successful_root_llm_search_record_from_grounded_output() {
        let tool_call = PromptToolCall {
            name: "llm_search".to_string(),
            args: json!({"query":"What is the sentiment about elsyluna.bsky.social?"}),
        };
        let output = "summary: grounded summary\nselected_result_uri: at://one\ncollection_id: clearsky_lists:did:plc:testactor\nstatus: ok\nsearch_result_1_uri: at://one";
        let record = extract_successful_root_llm_search_record(&tool_call, output)
            .expect("expected successful record");
        assert_eq!(record.intent, "reputation_lists");
        assert_eq!(record.actor_refs, vec!["elsyluna.bsky.social"]);
    }

    #[test]
    fn classifies_reputation_queries_for_root_guard() {
        assert_eq!(
            classify_root_llm_search_intent("How is elsyluna.bsky.social known on Bluesky lists?"),
            "reputation_lists"
        );
    }
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
        "Navigation: Up/Down selects a notification; Enter opens it; PageUp/PageDown scroll detail; Tab toggles AI chat view.".to_string(),
    ]
}

fn is_local_command(verb: &str) -> bool {
    matches!(
        verb,
        "/bio"
            | "/replies_from"
            | "/pins"
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

fn resolve_db_path(path: String) -> PathBuf {
    let candidate = PathBuf::from(&path);
    if candidate.is_absolute() {
        candidate
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(candidate)
    }
}

fn restore_store_from_db(
    store: &mut NotificationStore,
    db: &AppDb,
) -> Result<(), Box<dyn std::error::Error>> {
    let persisted = db.load_state()?;

    for actor in persisted.actors {
        store.restore_actor_cache(&actor.did, &actor.handle, actor.bio)?;
    }

    for collection in persisted.post_collections {
        store.cache_post_collection(collection);
    }

    for entry in persisted.clearsky_lists {
        store.restore_clearsky_lists(&entry.actor_did, entry.lists)?;
    }

    Ok(())
}

fn line_to_string(line: ratatui::text::Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn draw_ui(frame: &mut Frame, app: &App) {
    let (chunks, input_area) = ui::tui_renderer::layout(frame);

    if app.is_fullscreen_overlay() {
        match &app.detail {
            DetailView::Command { .. } | DetailView::AiChat { .. } => match &app.detail {
                DetailView::AiChat { text, .. } => ui::chat_renderer::render_chat_detail(
                    frame,
                    chunks[0],
                    &app.detail_title(),
                    text.clone(),
                    app.detail_scroll,
                ),
                _ => ui::tui_renderer::render_fullscreen_text(
                    frame,
                    chunks[0],
                    &app.detail_title(),
                    app.detail_text(),
                    app.detail_scroll,
                ),
            },
            DetailView::ContextVisualization(data) => {
                ui::tui_renderer::render_context_visualization(
                    frame,
                    chunks[0],
                    data,
                    app.detail_scroll,
                );
            }
            DetailView::Notification => {}
        }
    } else {
        let selected = if app.store.notifications.is_empty() {
            None
        } else {
            Some(app.selected)
        };
        ui::tui_renderer::render_notification_split(
            frame,
            chunks[0],
            app.notification_items(),
            selected,
            &app.detail_title(),
            app.detail_text(),
            app.detail_scroll,
        );
    }

    ui::tui_renderer::render_input(frame, input_area, app.input.as_str(), &app.status);

    let cursor_x = input_area
        .x
        .saturating_add(app.input.chars().count() as u16 + 1)
        .min(input_area.x + input_area.width.saturating_sub(2));
    let cursor_y = input_area.y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}

#[cfg(test)]
mod tests {
    use super::{build_root_context_snapshot, compact_llm_search_result_for_root_context};
    use crate::harness::context_window::{
        BuiltContextSection, BuiltContextWindow, ProviderContextLimits,
    };
    use crate::harness::llm_api::ChatMessage;
    use crate::harness::runtime::{ContextMessage, ContextMessageKind};
    use crate::visualization::context::ContextCategory;

    #[test]
    fn compact_llm_search_keeps_summary_and_selected_result_fields() {
        let tool_output = "llm_search searched collections independently and combined the grounded results below.\nsummary: grounded answer\nselected_result_uri: at://one\nselected_result_source_collection_id: clearsky:test\nselected_result_collection_id: clearsky:test\nselected_result_collection_label: Clearsky test\n\ncollection_id: clearsky:test\ncollection_label: Clearsky test\nstatus: ok\npost: picked\nsummary: child details";

        let compact = compact_llm_search_result_for_root_context(tool_output);

        assert!(compact.contains("summary: grounded answer"));
        assert!(compact.contains("selected_result_uri: at://one"));
        assert!(compact.contains("selected_result_collection_id: clearsky:test"));
    }

    #[test]
    fn root_snapshot_splits_system_tool_and_root_instructions() {
        let window = BuiltContextWindow {
            rendered: String::new(),
            header_tokens: 30,
            used_input_tokens: 50,
            truncated: false,
            limits: ProviderContextLimits {
                provider_name: "test".to_string(),
                model_name: "test".to_string(),
                max_context_tokens: 1000,
                reserved_output_tokens: 100,
            },
            sections: vec![BuiltContextSection {
                title: "Current Task".to_string(),
                estimated_tokens: 20,
                used_tokens: 20,
                truncated: false,
            }],
        };

        let snapshot = build_root_context_snapshot(
            &[
                ContextMessage {
                    kind: ContextMessageKind::InitialSystem,
                    message: ChatMessage {
                        role: "system".to_string(),
                        content: "system".to_string(),
                    },
                },
                ContextMessage {
                    kind: ContextMessageKind::InitialUserContext,
                    message: ChatMessage {
                        role: "user".to_string(),
                        content: "context".to_string(),
                    },
                },
            ],
            "short system prompt",
            "tool protocol text",
            &window,
            &window.limits,
        );

        assert_eq!(snapshot.segments[0].label, "System Prompt");
        assert_eq!(snapshot.segments[0].category, ContextCategory::SystemPrompt);
        assert_eq!(snapshot.segments[1].label, "Tool Instructions");
        assert_eq!(
            snapshot.segments[1].category,
            ContextCategory::ToolInstructions
        );
        assert_eq!(snapshot.segments[2].label, "Root Instructions");
        assert_eq!(
            snapshot.segments[2].category,
            ContextCategory::RootInstructions
        );
    }
}
