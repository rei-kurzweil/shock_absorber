use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use std::env;
use std::error::Error;
use std::future::Future;
use std::fs;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::{Duration as StdDuration, Instant};
use tokio::time::timeout;

mod clearsky_v1;
mod db;
#[allow(dead_code)]
mod harness;
mod model;
mod net_backend;
mod post;
mod visualization;

use crate::db::AppDb;
use crate::harness::context_window::{LLMContext, build_context_window};
use crate::harness::llm_api::{ChatMessage, LlmApiClient, OpenAiRestConfig};
use crate::harness::tools::{
    BlueskyTools, parse_prompt_tool_call, prompt_tool_protocol_instructions,
};
use crate::net_backend::{
    ActorProfile, CachedThreadReply, NotificationStore, ensure_actor_profile_cached,
    ensure_clearsky_lists_cached, ensure_pinned_posts_cached, ensure_recent_posts_cached,
    extract_reply_node, poll_notifications,
};
use crate::post::{PostNode, render_post_nodes};
use crate::visualization::context::ContextVisualizationData;

const POLL_INTERVAL: StdDuration = StdDuration::from_secs(30);
const UI_TICK: StdDuration = StdDuration::from_millis(200);
const DEFAULT_EVIL_GEMMA_BASE_URL: &str = "http://127.0.0.1:5000";
const DEFAULT_EVIL_GEMMA_MODEL: &str = "gemma-4-local";
const DEFAULT_SYSTEM_PROMPT_PATH: &str = "system_prompt.md";
const DEFAULT_DB_PATH: &str = "shock_absorber.sqlite3";
const MAX_TOOL_CALL_ROUNDS: usize = 3;
const INITIAL_COLLECTION_REFRESH_TIMEOUT: StdDuration = StdDuration::from_secs(30);

enum DetailView {
    Notification,
    Command { title: String, lines: Vec<String> },
    ContextVisualization(ContextVisualizationData),
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
    status: String,
    should_quit: bool,
}

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
            DetailView::ContextVisualization(data) => data.title.clone(),
        }
    }

    fn is_fullscreen_overlay(&self) -> bool {
        matches!(
            self.detail,
            DetailView::Command { .. } | DetailView::ContextVisualization(_)
        )
    }

    fn detail_text(&self) -> Text<'static> {
        match &self.detail {
            DetailView::Notification => self.notification_detail_text(),
            DetailView::Command { lines, .. } => Text::from(lines.join("\n")),
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
        agent: &BskyAgent,
        evil_gemma: &EvilGemmaConfig,
    ) -> Result<(), Box<dyn Error>> {
        let command = self.input.trim().to_owned();
        self.input.clear();

        if command.is_empty() {
            return Ok(());
        }

        let mut parts = command.split_whitespace();
        let verb = parts.next().unwrap_or_default();

        if !is_local_command(verb) {
            return self.run_evil_gemma_query(agent, evil_gemma, command).await;
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
                self.detail = DetailView::ContextVisualization(
                    self.build_context_visualization(evil_gemma),
                );
                self.status = "context visualization loaded".to_string();
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

    async fn run_evil_gemma_query(
        &mut self,
        agent: &BskyAgent,
        evil_gemma: &EvilGemmaConfig,
        query: String,
    ) -> Result<(), Box<dyn Error>> {
        let initial_context = {
            let tools = BlueskyTools::new(&self.store);
            build_tool_aware_query_context(
                self.selected_actor_did(),
                self.selected_actor_summary(),
                &tools,
                &self.root_conversation,
                &query,
                &evil_gemma.client,
            )
        };
        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "{}\n\n{}",
                    evil_gemma.system_prompt.trim(),
                    prompt_tool_protocol_instructions()
                ),
            },
            ChatMessage {
                role: "user".to_string(),
                content: initial_context,
            },
        ];
        let mut tool_transcript = Vec::new();
        let mut response = String::new();
        let mut hit_tool_round_limit = false;

        for round in 0..MAX_TOOL_CALL_ROUNDS {
            response = evil_gemma
                .client
                .complete_chat(messages.clone(), 1024)
                .await?;

            let Some(tool_call) = parse_prompt_tool_call(&response) else {
                break;
            };

            let tool_name = tool_call.name.clone();
            let tool_args = serde_json::to_string(&tool_call.args)?;
            let selected_actor_did = self.selected_actor_did().cloned();
            let mut prep_log = vec![format!(
                "[tool_prep] inspecting tool `{}` for possible initial collection refresh",
                tool_call.name
            )];
            let actor_dids =
                planned_tool_call_refresh_targets(&self.store, selected_actor_did, &tool_call);
            self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                &tool_name,
                &tool_args,
                &prep_log,
                None,
            )));

            let mut prep_warnings = Vec::new();
            if actor_dids.is_empty() {
                prep_log.push("[tool_prep] no initial refresh needed".to_string());
                self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                    &tool_name,
                    &tool_args,
                    &prep_log,
                    None,
                )));
            } else {
                for did in actor_dids {
                    prep_log.push(format!(
                        "[tool_prep] initial refresh needed for actor {} before tool `{}`",
                        did.as_str(),
                        tool_call.name
                    ));
                    self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));

                    prep_log.push(format!(
                        "[tool_prep] -> ensure_recent_posts_cached {}",
                        did.as_str()
                    ));
                    self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
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
                        self.set_evil_gemma_progress(
                            &query,
                            &tool_transcript,
                            Some(build_tool_entry(&tool_name, &tool_args, &prep_log, None)),
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
                    self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
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
                        self.set_evil_gemma_progress(
                            &query,
                            &tool_transcript,
                            Some(build_tool_entry(&tool_name, &tool_args, &prep_log, None)),
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
                    self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
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
                        self.set_evil_gemma_progress(
                            &query,
                            &tool_transcript,
                            Some(build_tool_entry(&tool_name, &tool_args, &prep_log, None)),
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
                    self.set_evil_gemma_progress(&query, &tool_transcript, Some(build_tool_entry(
                        &tool_name,
                        &tool_args,
                        &prep_log,
                        None,
                    )));
                }
            }

            self.set_evil_gemma_progress(
                &query,
                &tool_transcript,
                Some(build_tool_entry(
                    &tool_name,
                    &tool_args,
                    &prep_log,
                    Some("<running tool...>"),
                )),
            );
            let tools = BlueskyTools::new(&self.store);
            let tool_output = match tools
                .execute_prompt_tool_call(
                    &tool_call,
                    self.opened_notification(),
                    &evil_gemma.client,
                )
                .await
            {
                Ok(output) => output,
                Err(err) => format!("Tool execution failed: {err}"),
            };
            let tool_output = if prep_warnings.is_empty() {
                tool_output
            } else {
                format!(
                    "Tool preparation warning:\n{}\n\n{}",
                    prep_warnings.join("\n"),
                    tool_output
                )
            };

            tool_transcript.push(build_tool_entry(
                &tool_name,
                &tool_args,
                &prep_log,
                Some(&tool_output),
            ));
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            });
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Tool Result\nname: {tool_name}\nargs: {tool_args}\n\n{tool_output}\n\n{}",
                    if round + 1 == MAX_TOOL_CALL_ROUNDS {
                        "This was the final allowed tool round. Answer the original query directly now. Do not request another tool or emit a TOOL_CALL block."
                    } else {
                        "Use this result to answer the original query, or request exactly one more tool if needed."
                    }
                ),
            });
            hit_tool_round_limit = round + 1 == MAX_TOOL_CALL_ROUNDS;
            self.set_evil_gemma_progress(&query, &tool_transcript, None);
        }

        if hit_tool_round_limit && parse_prompt_tool_call(&response).is_some() {
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            });
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: "You have already used the maximum number of tool rounds. Answer the original query directly using the tool results already provided. Do not emit TOOL_CALL.".to_string(),
            });
            response = evil_gemma
                .client
                .complete_chat(messages.clone(), 1024)
                .await
                .unwrap_or_else(|_| {
                    "Tool loop stopped after the configured maximum number of tool rounds without a final answer.".to_string()
                });
        }

        if parse_prompt_tool_call(&response).is_none() && response_looks_incomplete(&response) {
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            });
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: "Your previous answer appears cut off. Finish the answer directly in at most one short paragraph plus up to 3 bullets. Start with a bottom-line conclusion sentence. Do not emit TOOL_CALL.".to_string(),
            });
            response = evil_gemma
                .client
                .complete_chat(messages, 320)
                .await
                .unwrap_or(response);
        }

        self.set_command_output(
            format!("evil_gemma: {query}"),
            build_evil_gemma_output_lines(&tool_transcript, None, Some(&response)),
        );
        self.root_conversation.push(ConversationTurn {
            user: query,
            assistant: response,
        });
        self.status = "evil_gemma response loaded".to_string();
        Ok(())
    }

    fn set_evil_gemma_progress(
        &mut self,
        query: &str,
        tool_transcript: &[String],
        active_entry: Option<String>,
    ) {
        self.set_command_output(
            format!("evil_gemma: {query}"),
            build_evil_gemma_output_lines(tool_transcript, active_entry.as_deref(), None),
        );
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
        self.detail = DetailView::Notification;
    }

    fn clear_root_conversation(&mut self) {
        self.root_conversation.clear();
    }

    fn build_context_visualization(
        &self,
        evil_gemma: &EvilGemmaConfig,
    ) -> ContextVisualizationData {
        let tools = BlueskyTools::new(&self.store);
        let tools_inventory = tools.render_tool_inventory();
        let recent_chat = recent_chat_text(&self.root_conversation);
        let current_task = self.input.trim();

        ContextVisualizationData::from_root_context(
            evil_gemma.system_prompt.trim(),
            prompt_tool_protocol_instructions(),
            &tools_inventory,
            "Use the available tools only when they materially improve the answer.",
            &self
                .selected_actor_did()
                .map(search_hints_for_actor_did)
                .unwrap_or_else(|| {
                    "If you need cached search, use `list_collections` to inspect available collections, then `llm_search` with a `prompt` plus either explicit `collection_ids` or an `actor_did`. Do not call `llm_search` without one of those scope selectors.".to_string()
            }),
            &self
                .selected_actor_summary()
                .unwrap_or_else(|| "No actor is currently selected in the UI.".to_string()),
            (!current_task.is_empty()).then_some(current_task),
            recent_chat.as_deref(),
            &evil_gemma.client.context_limits(),
        )
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

    let agent = BskyAgent::builder().build().await?;
    agent.login(&handle, &password).await?;
    let evil_gemma = EvilGemmaConfig::from_env()?;
    let db_path =
        env::var("SHOCK_ABSORBER_DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());
    let db = AppDb::new(resolve_db_path(db_path))?;
    let mut app = App::new(handle);
    restore_store_from_db(&mut app.store, &db)?;
    app.status = format!("{} | db {}", app.status, db.path().display());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &agent, &evil_gemma, app, &db).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    agent: &BskyAgent,
    evil_gemma: &EvilGemmaConfig,
    mut app: App,
    db: &AppDb,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_poll = Instant::now() - POLL_INTERVAL;

    loop {
        if last_poll.elapsed() >= POLL_INTERVAL {
            match poll_notifications(agent, &mut app.store).await {
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
                        } else if let Err(err) = app.execute_command(agent, evil_gemma).await {
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

fn normalize_actor_ref(actor: &str) -> &str {
    actor.strip_prefix('@').unwrap_or(actor)
}

fn planned_tool_call_refresh_targets(
    store: &NotificationStore,
    selected_actor_did: Option<bsky_sdk::api::types::string::Did>,
    tool_call: &crate::harness::tools::PromptToolCall,
) -> Vec<bsky_sdk::api::types::string::Did> {
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
        "llm_search" => {
            if let Some(collection_ids) = tool_call.args.get("collection_ids").and_then(|value| value.as_array()) {
                for collection_id in collection_ids.iter().filter_map(|value| value.as_str()) {
                    if collection_needs_initial_refresh(store, collection_id) {
                        if let Some(did) = actor_did_from_collection_id(collection_id) {
                            actor_dids.push(did);
                        }
                    }
                }
            } else if let Some(did) = tool_call
                .args
                .get("actor_did")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse().ok())
            {
                if actor_needs_initial_refresh(store, &did) {
                    actor_dids.push(did);
                }
            } else if let Some(did) = selected_actor_did {
                if actor_needs_initial_refresh(store, &did) {
                    actor_dids.push(did);
                }
            }
        }
        "read_collection_item" => {
            if let Some(collection_id) = tool_call.args.get("collection_id").and_then(|value| value.as_str()) {
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

fn build_evil_gemma_output_lines(
    tool_transcript: &[String],
    active_entry: Option<&str>,
    response: Option<&str>,
) -> Vec<String> {
    let mut lines = Vec::new();
    if !tool_transcript.is_empty() || active_entry.is_some() {
        lines.push("Tool Transcript:".to_string());
        lines.push(String::new());
        for entry in tool_transcript {
            lines.extend(entry.lines().map(str::to_owned));
            lines.push(String::new());
        }
        if let Some(active_entry) = active_entry {
            lines.extend(active_entry.lines().map(str::to_owned));
            lines.push(String::new());
        }
    }

    if let Some(response) = response {
        if !tool_transcript.is_empty() || active_entry.is_some() {
            lines.push("Final Answer:".to_string());
            lines.push(String::new());
        }
        if response.trim().is_empty() {
            lines.push("<empty response>".to_string());
        } else {
            lines.extend(response.lines().map(str::to_owned));
        }
    }

    if lines.is_empty() {
        lines.push("Waiting for evil_gemma...".to_string());
    }

    lines
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
        .map_err(|_| {
            format!(
                "{label} timed out after {} seconds",
                step_timeout.as_secs()
            )
        })?
        .map_err(|err| err.to_string())
}

fn actor_needs_initial_refresh(store: &NotificationStore, actor_did: &bsky_sdk::api::types::string::Did) -> bool {
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

fn build_tool_aware_query_context(
    selected_actor_did: Option<&bsky_sdk::api::types::string::Did>,
    selected_actor_summary: Option<String>,
    tools: &BlueskyTools<'_>,
    root_conversation: &[ConversationTurn],
    query: &str,
    llm_client: &LlmApiClient,
) -> String {
    let mut context =
        LLMContext::new("Use the available tools only when they materially improve the answer.");
    context.push_section("Tools", tools.render_tool_inventory());
    context.push_section(
        "Search Hints",
        selected_actor_did
            .map(search_hints_for_actor_did)
            .unwrap_or_else(|| {
                "If you need cached search, use `list_collections` to inspect available collections, then `llm_search` with a `prompt` plus either explicit `collection_ids` or an `actor_did`. For interaction or frequency questions, pass explicit conversational `collection_ids` like `recent_replies_sent`, `recent_posts_unaddressed`, `pinned_posts`, or `replies_to_actor` instead of broad actor-wide search. Likes are not currently available as a searchable collection.".to_string()
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

    build_context_window(&context, &llm_client.context_limits())
}

fn search_hints_for_actor_did(did: &bsky_sdk::api::types::string::Did) -> String {
    let did = did.as_str();
    format!(
        "The selected actor is {did}. Use `list_collections` to see cached collections for this actor, then `llm_search` with either `collection_ids` or an `actor_did`. For interaction/frequency questions, prefer explicit conversational `collection_ids` instead of broad actor-wide search. Likes are not currently available as a searchable collection. Provide another actor DID only if a mentioned actor becomes relevant."
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
        "  /clear".to_string(),
        "  /help".to_string(),
        "  /quit".to_string(),
        String::new(),
        "Any other input is sent to evil_gemma over local OpenAI-compatible REST.".to_string(),
        "Default endpoint: http://127.0.0.1:5000/v1/chat/completions".to_string(),
        "Override with EVIL_GEMMA_BASE_URL and EVIL_GEMMA_MODEL.".to_string(),
        String::new(),
        "Navigation: Up/Down selects a notification; Enter opens it; PageUp/PageDown scroll detail.".to_string(),
    ]
}

fn is_local_command(verb: &str) -> bool {
    matches!(
        verb,
        "/bio"
            | "/replies_from"
            | "/pins"
            | "/context"
            | "/clear"
            | "/help"
            | "/q"
            | "/quit"
            | "/exit"
            | "bio"
            | "replies_from"
            | "pins"
            | "context"
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

fn recent_chat_text(root_conversation: &[ConversationTurn]) -> Option<String> {
    if root_conversation.is_empty() {
        return None;
    }

    Some(
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
    )
}

fn line_to_string(line: ratatui::text::Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn draw_ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(frame.area());

    if app.is_fullscreen_overlay() {
        match &app.detail {
            DetailView::Command { .. } => {
                let detail = Paragraph::new(app.detail_text())
                    .block(Block::default().title(app.detail_title()))
                    .scroll((app.detail_scroll, 0))
                    .wrap(Wrap { trim: false });
                frame.render_widget(detail, chunks[0]);
            }
            DetailView::ContextVisualization(data) => {
                visualization::context::render(frame, chunks[0], data);
            }
            DetailView::Notification => {}
        }
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[0]);

        let notifications = List::new(app.notification_items())
            .block(Block::default().title("Notifications"))
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
            .block(Block::default().title(app.detail_title()))
            .scroll((app.detail_scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, body[1]);
    }

    let input = Paragraph::new(app.input.as_str())
        .block(
            Block::default()
                .title(format!("Command | {} ", app.status))
                .style(Style::default().bg(Color::Rgb(220, 220, 220)).fg(Color::Black)),
        )
        .style(Style::default().bg(Color::Rgb(220, 220, 220)).fg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(input, chunks[1]);

    let cursor_x = chunks[1]
        .x
        .saturating_add(app.input.chars().count() as u16 + 1)
        .min(chunks[1].x + chunks[1].width.saturating_sub(2));
    let cursor_y = chunks[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));
}
