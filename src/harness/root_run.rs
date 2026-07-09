use crate::app::EvilGemmaConfig;
use crate::harness::agents::AgentNodeStatus;
use crate::harness::context_window_logger::{
    log_agent_graph, log_chat_transcript, log_current_task, log_root_prompt_snapshot,
};
use crate::harness::root_context::build_live_context_visualization;
use crate::harness::runtime::{
    ContextMessage, ContextMessageKind, RootRunState, RootRunStatus, SuccessfulRootToolRun,
    TranscriptEntryKind,
};
use crate::harness::tools::{
    BlueskyTools, PreparedPromptToolInput, PromptToolCall, ToolPrepFailure, ToolProgressEvent,
    parse_prompt_tool_call,
};
use crate::net_backend::{
    NotificationStore, ensure_clearsky_lists_cached, ensure_pinned_posts_cached,
    ensure_recent_posts_cached,
};
use bsky_sdk::BskyAgent;
use bsky_sdk::api::app::bsky::notification::list_notifications::Notification;
use std::env;
use std::error::Error;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::time::timeout;

const MAX_TOOL_CALL_ROUNDS: usize = 3;
const MAX_TOOL_PREP_ATTEMPTS: usize = 2;
const INITIAL_COLLECTION_REFRESH_TIMEOUT: StdDuration = StdDuration::from_secs(30);

pub enum RootRunEvent {
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

pub async fn run_root_query_task(
    agent: Arc<BskyAgent>,
    evil_gemma: Arc<EvilGemmaConfig>,
    mut store: NotificationStore,
    selected_notification: Option<Notification>,
    selected_actor_did: Option<bsky_sdk::api::types::string::Did>,
    mut root_run: RootRunState,
    sender: UnboundedSender<RootRunEvent>,
) {
    let result: Result<(RootRunState, NotificationStore, String), Box<dyn Error>> = async {
        let mut response = String::new();
        let mut hit_tool_round_limit = false;

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
            let blocked_root_rerun = blocked_root_tool_rerun(
                &tool_call,
                root_run.latest_successful_tool_run(&tool_name),
            );
            let mut prep_log = vec![format!(
                "[tool_prep] inspecting tool `{}` for harness-owned preparation",
                tool_call.name
            )];
            if let Some(reason) = blocked_root_rerun.as_deref() {
                prep_log.push(format!(
                    "[tool_prep] blocked root `{tool_name}` rerun because a prior grounded result already covers this scope: {reason}"
                ));
            }
            root_run.record_tool_call(
                round + 1,
                &tool_call,
                blocked_root_rerun.is_none(),
            )?;

            root_run.set_active_tool_entry(Some(build_tool_entry(
                &tool_name,
                &tool_args,
                &prep_log,
                None,
            )));
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

            let mut prep_warnings = Vec::new();
            let tools = BlueskyTools::new();
            let prepared_input = if blocked_root_rerun.is_some() {
                None
            } else {
                match prepare_root_tool_input_loop(
                    &tools,
                    &tool_call,
                    selected_actor_did.as_ref(),
                    &agent,
                    &mut store,
                    &mut prep_log,
                    &mut prep_warnings,
                    &mut root_run,
                    &sender,
                )
                .await
                {
                    Ok(prepared) => prepared,
                    Err(prep_failure) => {
                        let rendered = render_tool_prep_failure(&prep_failure);
                        root_run.push_transcript_entry(
                            TranscriptEntryKind::Notice,
                            format!(
                                "Runtime Notice\npreparation failed for root `{tool_name}` before public execution"
                            ),
                        );
                        root_run.push_transcript_entry(
                            TranscriptEntryKind::ToolCall,
                            build_tool_entry(&tool_name, &tool_args, &prep_log, Some(&rendered)),
                        );
                        root_run.set_active_tool_entry(None);
                        root_run.push_message(
                            ContextMessageKind::ToolRequest,
                            "assistant",
                            response.clone(),
                        );
                        root_run.push_message(
                            ContextMessageKind::ToolResult,
                            "user",
                            format!(
                                "Tool Result\nname: {tool_name}\nargs: {tool_args}\n\n{}\n\n{}",
                                compact_tool_result_for_root_context(&tool_name, &rendered),
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
                            true,
                            Some(rendered),
                        );
                        hit_tool_round_limit = round + 1 == MAX_TOOL_CALL_ROUNDS;
                        continue;
                    }
                }
            };

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
            let tool_output = if let Some(reason) = blocked_root_rerun.as_deref() {
                let prior = root_run
                    .latest_successful_tool_run(&tool_name)
                    .expect("blocked rerun requires prior grounded result");
                crate::harness::tools::ToolExecutionOutput {
                    rendered: format!(
                        "Tool execution prevented: a previous grounded `{tool_name}` result in this root run already covers this scope.\nreason: {reason}\n\nUse the existing grounded result unless you can name a materially new scope.\n\nprior_query: {}\nprior_summary: {}\nprior_collection_ids: {}",
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
                    .execute_prepared_prompt_tool_call(
                        &tool_call,
                        prepared_input.as_ref(),
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

            if blocked_root_rerun.is_none() {
                if let Some(successful_result) =
                    extract_successful_root_tool_record(&tool_call, &tool_output)
                {
                    root_run.set_latest_successful_tool_run(Some(successful_result));
                }
            }

            if blocked_root_rerun.is_some() {
                root_run.push_transcript_entry(
                    TranscriptEntryKind::Notice,
                    format!(
                        "Runtime Notice\nblocked root `{tool_name}` rerun and preserved the earlier grounded result"
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
                blocked_root_rerun.is_none(),
                Some(tool_output.clone()),
            );
            if let Some(failure_answer) = hard_tool_failure_answer {
                response = fallback_or_failure_answer(
                    root_run.latest_successful_tool_run(&tool_name),
                    &failure_answer,
                );
                break;
            }
            hit_tool_round_limit = round + 1 == MAX_TOOL_CALL_ROUNDS;
            let live_visualization = build_live_context_visualization(
                "/context",
                root_run.messages(),
                evil_gemma.system_prompt.trim(),
                crate::harness::tools::prompt_tool_protocol_instructions(),
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
                crate::harness::tools::prompt_tool_protocol_instructions(),
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
                crate::harness::tools::prompt_tool_protocol_instructions(),
                root_run.root_context_window(),
                root_run.agent_graph(),
                &evil_gemma.client.context_limits(),
            );
            root_run.set_context_visualization(live_visualization);
            let _ = sender.send(RootRunEvent::Progress(root_run.clone()));
        }

        root_run.set_final_response(response.clone());
        let root_agent_id = root_run.agent_graph().root_agent_id();
        root_run
            .agent_graph_mut()
            .set_status(root_agent_id, AgentNodeStatus::Completed);
        root_run
            .agent_graph_mut()
            .set_result_summary(root_agent_id, response.clone());
        root_run.set_status(RootRunStatus::Completed);
        let mut final_messages = root_run.messages().to_vec();
        final_messages.push(ContextMessage {
            kind: ContextMessageKind::AssistantReply,
            message: crate::harness::llm_api::ChatMessage {
                role: "assistant".to_string(),
                content: response.clone(),
            },
        });
        let final_context_visualization = build_live_context_visualization(
            "/context",
            &final_messages,
            evil_gemma.system_prompt.trim(),
            crate::harness::tools::prompt_tool_protocol_instructions(),
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

pub fn compact_tool_result_for_root_context(tool_name: &str, tool_output: &str) -> String {
    match tool_name {
        "search" | "summary" => compact_search_like_result_for_root_context(tool_output),
        _ => truncate_for_root_context(tool_output, 24),
    }
}

fn compact_search_like_result_for_root_context(tool_output: &str) -> String {
    let mut kept = Vec::new();
    let mut failed_collections = Vec::new();
    let mut current_collection_label: Option<String> = None;
    let mut diagnostic_count = 0usize;

    for line in tool_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if kept.is_empty() {
            kept.push(trimmed.to_string());
            continue;
        }
        if trimmed.starts_with("summary:") {
            kept.push(truncate_line_for_root_context(trimmed, 320));
            continue;
        }
        if trimmed.starts_with("diagnostic:") {
            diagnostic_count += 1;
            if diagnostic_count <= 2 {
                kept.push(truncate_line_for_root_context(trimmed, 220));
            }
            continue;
        }
        if trimmed.starts_with("selected_result_uri:")
            || trimmed.starts_with("selected_result_source_collection_id:")
            || trimmed.starts_with("selected_result_collection_id:")
            || trimmed.starts_with("selected_result_collection_label:")
            || trimmed.starts_with("error:")
        {
            kept.push(trimmed.to_string());
        }

        if let Some(label) = trimmed.strip_prefix("collection_label:") {
            current_collection_label = Some(label.trim().to_string());
            continue;
        }

        if trimmed == "status: failed" {
            failed_collections.push(
                current_collection_label
                    .clone()
                    .unwrap_or_else(|| "unknown collection".to_string()),
            );
        }
    }

    if !failed_collections.is_empty() {
        kept.push(format!(
            "collection_failures: {}",
            failed_collections.join(" | ")
        ));
    }
    if diagnostic_count > 2 {
        kept.push(format!("diagnostic_count: {diagnostic_count}"));
    }

    if kept.is_empty() {
        return truncate_for_root_context(tool_output, 24);
    }
    kept.join("\n")
}

fn truncate_line_for_root_context(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut truncated = text
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

async fn prepare_root_tool_input_loop(
    tools: &BlueskyTools,
    tool_call: &PromptToolCall,
    selected_actor_did: Option<&bsky_sdk::api::types::string::Did>,
    agent: &BskyAgent,
    store: &mut NotificationStore,
    prep_log: &mut Vec<String>,
    prep_warnings: &mut Vec<String>,
    root_run: &mut RootRunState,
    sender: &UnboundedSender<RootRunEvent>,
) -> Result<Option<PreparedPromptToolInput>, ToolPrepFailure> {
    if !matches!(tool_call.name.as_str(), "search" | "summary") {
        prep_log.push("[tool_prep] direct path: no harness preparation required".to_string());
        return Ok(None);
    }

    let mut last_failure: Option<ToolPrepFailure> = None;
    for attempt in 1..=MAX_TOOL_PREP_ATTEMPTS {
        prep_log.push(format!(
            "[tool_prep] attempt {attempt}/{MAX_TOOL_PREP_ATTEMPTS}\ntool: {}\nphase: resolve",
            tool_call.name
        ));
        root_run.set_active_tool_entry(Some(build_tool_entry(
            &tool_call.name,
            &serde_json::to_string(&tool_call.args).unwrap_or_default(),
            prep_log,
            None,
        )));
        let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

        match tools.prepare_root_tool_input(tool_call, selected_actor_did, store) {
            Ok(prepared_input) => {
                let actor_source = BlueskyTools::prepared_actor_anchor(&prepared_input)
                    .map(|anchor| anchor.source.as_str())
                    .unwrap_or("none");
                prep_log.push(format!(
                    "[tool_prep] attempt {attempt}/{MAX_TOOL_PREP_ATTEMPTS}\ntool: {}\nphase: verify\nverify_result: callable\nactor_anchor_source: {actor_source}",
                    tool_call.name
                ));
                return Ok(Some(prepared_input));
            }
            Err(mut failure) => {
                failure.attempt_count = attempt;
                prep_log.push(format!(
                    "[tool_prep] attempt {attempt}/{MAX_TOOL_PREP_ATTEMPTS}\ntool: {}\nphase: verify\nverify_result: missing_prerequisites\nmissing: {}\nactor_anchor_source: {}\ntried: {}",
                    tool_call.name,
                    failure
                        .missing
                        .iter()
                        .map(|item| item.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                    failure
                        .actor_anchor_source
                        .as_ref()
                        .map(|source| source.as_str())
                        .unwrap_or("none"),
                    failure.tried.join(", ")
                ));

                let should_refresh = attempt < MAX_TOOL_PREP_ATTEMPTS
                    && failure
                        .missing
                        .iter()
                        .any(|item| matches!(item, crate::harness::tools::ToolPrepMissingPrerequisite::CollectionTarget));
                if should_refresh {
                    if let Some(anchor_did) = planned_tool_call_refresh_targets(
                        store,
                        selected_actor_did.cloned(),
                        tool_call,
                    )
                    .into_iter()
                    .next()
                    {
                        run_initial_refresh_for_actor(
                            tool_call,
                            agent,
                            store,
                            &anchor_did,
                            prep_log,
                            prep_warnings,
                            root_run,
                            sender,
                        )
                        .await;
                    }
                }
                last_failure = Some(failure);
            }
        }
    }

    Err(last_failure.unwrap_or(ToolPrepFailure {
        tool_name: tool_call.name.clone(),
        attempt_count: MAX_TOOL_PREP_ATTEMPTS,
        missing: Vec::new(),
        actor_anchor_source: None,
        tried: vec!["no_preparation_outcome".to_string()],
    }))
}

async fn run_initial_refresh_for_actor(
    tool_call: &PromptToolCall,
    agent: &BskyAgent,
    store: &mut NotificationStore,
    did: &bsky_sdk::api::types::string::Did,
    prep_log: &mut Vec<String>,
    prep_warnings: &mut Vec<String>,
    root_run: &mut RootRunState,
    sender: &UnboundedSender<RootRunEvent>,
) {
    prep_log.push(format!(
        "[tool_prep] side_effect: initial_refresh\nactor_did: {}",
        did.as_str()
    ));
    let tool_args = serde_json::to_string(&tool_call.args).unwrap_or_default();
    root_run.set_active_tool_entry(Some(build_tool_entry(
        &tool_call.name,
        &tool_args,
        prep_log,
        None,
    )));
    let _ = sender.send(RootRunEvent::Progress(root_run.clone()));

    let (recent_post_fetch_limit, min_top_level_posts) =
        planned_recent_post_refresh_requirements(tool_call);
    if let Err(message) = run_tool_prep_step(
        INITIAL_COLLECTION_REFRESH_TIMEOUT,
        ensure_recent_posts_cached(
            agent,
            store,
            did,
            recent_post_fetch_limit,
            min_top_level_posts,
        ),
        format!("ensure_recent_posts_cached {}", did.as_str()),
    )
    .await
    {
        prep_warnings.push(format!(
            "initial refresh for {} failed during recent-post fetch: {}",
            did.as_str(),
            message
        ));
        return;
    }
    let _ = run_tool_prep_step(
        INITIAL_COLLECTION_REFRESH_TIMEOUT,
        ensure_pinned_posts_cached(agent, store, did),
        format!("ensure_pinned_posts_cached {}", did.as_str()),
    )
    .await;
    let _ = run_tool_prep_step(
        INITIAL_COLLECTION_REFRESH_TIMEOUT,
        ensure_clearsky_lists_cached(store, did),
        format!("ensure_clearsky_lists_cached {}", did.as_str()),
    )
    .await;
}

fn render_tool_prep_failure(failure: &ToolPrepFailure) -> String {
    format!(
        "Tool preparation failed: {}\nstatus: prep_failed\nattempts: {}\nmissing_prerequisites: {}\nactor_anchor_source: {}\ntried: {}",
        failure.tool_name,
        failure.attempt_count,
        if failure.missing.is_empty() {
            "<none>".to_string()
        } else {
            failure
                .missing
                .iter()
                .map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        },
        failure
            .actor_anchor_source
            .as_ref()
            .map(|source| source.as_str())
            .unwrap_or("none"),
        failure.tried.join(", ")
    )
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
    if tool_name != "search" && tool_name != "summary" {
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
            format!(
                "I couldn't inspect the requested cached collections, so I can't ground a `{tool_name}` answer."
            ),
            failure_line.to_string(),
        ];

        if let Some(prep_warning) = prep_warning {
            lines.push(prep_warning.to_string());
        }

        lines.push(
            "No grounded collection evidence was successfully loaded for this request, so any answer would be speculative."
                .to_string(),
        );

        return Some(lines.join("\n\n"));
    }

    if tool_output.trim() == "No matching cached posts." {
        return Some(
            format!(
                "The latest `{tool_name}` call returned no grounded results for that scope.\n\nI can't safely expand that without inventing evidence."
            )
                .to_string(),
        );
    }

    None
}

fn blocked_root_tool_rerun(
    tool_call: &PromptToolCall,
    prior: Option<&SuccessfulRootToolRun>,
) -> Option<String> {
    if tool_call.name != "search" && tool_call.name != "summary" {
        return None;
    }
    let prior = prior?;
    let query = tool_call.args.get("query")?.as_str()?;
    let current_actor_refs = detect_actor_refs_for_guard(query);
    if current_actor_refs.is_empty() || current_actor_refs != prior.actor_refs {
        return None;
    }
    let current_intent = classify_root_search_intent(query);
    if current_intent != prior.intent {
        return None;
    }
    if tool_call.name == "summary" {
        let current_requested_post_count = requested_post_window_size(query);
        if current_requested_post_count != prior.requested_post_count {
            return None;
        }
        return Some(
            "same actor and same summary scope with no materially new collection target"
                .to_string(),
        );
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

fn extract_successful_root_tool_record(
    tool_call: &PromptToolCall,
    tool_output: &str,
) -> Option<SuccessfulRootToolRun> {
    if (tool_call.name != "search" && tool_call.name != "summary")
        || !tool_output_is_grounded(&tool_call.name, tool_output)
    {
        return None;
    }
    let query = tool_call.args.get("query")?.as_str()?.to_string();
    let summary = tool_output
        .lines()
        .find_map(|line| line.trim().strip_prefix("summary:").map(str::trim))
        .filter(|summary| !summary.is_empty())?
        .to_string();
    Some(SuccessfulRootToolRun {
        tool_name: tool_call.name.clone(),
        query: query.clone(),
        rendered_result: tool_output.to_string(),
        summary,
        actor_refs: detect_actor_refs_for_guard(&query),
        collection_ids: extract_collection_ids_from_llm_output(tool_output),
        intent: classify_root_search_intent(&query),
        requested_post_count: requested_post_window_size(&query),
    })
}

fn tool_output_is_grounded(tool_name: &str, tool_output: &str) -> bool {
    let has_summary = tool_output.lines().any(|line| {
        line.trim()
            .strip_prefix("summary:")
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    });
    let has_search_anchor = tool_output.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("search_result_") || trimmed.starts_with("selected_result_")
    });
    let has_summary_anchor = tool_output
        .lines()
        .any(|line| line.trim().starts_with("covered_item_"));
    let success_blocks = tool_output
        .lines()
        .filter(|line| line.trim() == "status: ok")
        .count();
    let has_anchor = if tool_name == "summary" {
        has_summary_anchor
    } else {
        has_search_anchor
    };
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

fn classify_root_search_intent(query: &str) -> String {
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
        "recent_posts",
        "recent_posts_unaddressed",
        "pinned_posts",
    ]
    .iter()
    .filter(|target| lower.contains(**target))
    .map(|target| target.to_string())
    .collect()
}

fn fallback_or_failure_answer(
    prior: Option<&SuccessfulRootToolRun>,
    failure_answer: &str,
) -> String {
    let Some(prior) = prior else {
        return failure_answer.to_string();
    };
    format!(
        "A later `{}` attempt failed, so I'm using the earlier grounded result from this run.\n\n{}\n\nDiagnostic from the later failed attempt:\n{}",
        prior.tool_name, prior.rendered_result, failure_answer
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

fn planned_recent_post_refresh_requirements(tool_call: &PromptToolCall) -> (usize, usize) {
    if tool_call.name != "search" && tool_call.name != "summary" {
        return (100, 25);
    }

    let query = tool_call
        .args
        .get("query")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let requested_posts = requested_post_window_size(query).unwrap_or(25).max(25);
    let feed_fetch_limit = requested_posts.saturating_mul(2).max(100).min(200);
    (feed_fetch_limit, requested_posts)
}

fn planned_tool_call_refresh_targets(
    store: &NotificationStore,
    selected_actor_did: Option<bsky_sdk::api::types::string::Did>,
    tool_call: &PromptToolCall,
) -> Vec<bsky_sdk::api::types::string::Did> {
    let mut actor_dids: Vec<bsky_sdk::api::types::string::Did> = Vec::new();

    match tool_call.name.as_str() {
        "search" | "summary" => {
            let query = tool_call
                .args
                .get("query")
                .and_then(|value| value.as_str())
                .unwrap_or_default();

            for actor_ref in detect_actor_refs_for_guard(query) {
                if let Some(did) = store
                    .find_did(&actor_ref)
                    .or_else(|| actor_ref.parse().ok())
                {
                    actor_dids.push(did);
                }
            }

            if actor_dids.is_empty() {
                if let Some(selected_actor_did) = selected_actor_did {
                    actor_dids.push(selected_actor_did);
                }
            }
        }
        _ => {}
    }

    actor_dids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    actor_dids.dedup_by(|left, right| left.as_str() == right.as_str());
    actor_dids
}

fn requested_post_window_size(query: &str) -> Option<usize> {
    let lower = query.to_ascii_lowercase();
    let tokens = lower.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        let cleaned = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
        let Ok(value) = cleaned.parse::<usize>() else {
            continue;
        };
        let prev = index.checked_sub(1).and_then(|i| tokens.get(i)).copied();
        let next = tokens.get(index + 1).copied();
        let nearby = [prev, next];
        if nearby.into_iter().flatten().any(|neighbor| {
            let neighbor = neighbor.trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
            matches!(
                neighbor,
                "post" | "posts" | "thing" | "things" | "item" | "items"
            )
        }) {
            return Some(value);
        }
        if matches!(prev, Some("count" | "first" | "last")) {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        blocked_root_tool_rerun, classify_root_search_intent,
        compact_search_like_result_for_root_context, extract_successful_root_tool_record,
        fallback_or_failure_answer, planned_recent_post_refresh_requirements,
        planned_tool_call_refresh_targets, requested_post_window_size,
    };
    use crate::harness::runtime::SuccessfulRootToolRun;
    use crate::harness::tools::PromptToolCall;
    use crate::net_backend::NotificationStore;
    use bsky_sdk::api::types::string::Did;
    use serde_json::json;

    #[test]
    fn blocks_same_scope_root_search_rerun() {
        let prior = SuccessfulRootToolRun {
            tool_name: "search".to_string(),
            query: "What is the sentiment about elsyluna.bsky.social?".to_string(),
            rendered_result: "summary: grounded\ncollection_id: clearsky_lists:did:plc:testactor\nstatus: ok\nsearch_result_1_uri: at://one".to_string(),
            summary: "grounded".to_string(),
            actor_refs: vec!["elsyluna.bsky.social".to_string()],
            collection_ids: vec!["clearsky_lists:did:plc:testactor".to_string()],
            intent: "reputation_lists".to_string(),
            requested_post_count: None,
        };
        let tool_call = PromptToolCall {
            name: "search".to_string(),
            args: json!({"query":"How is elsyluna.bsky.social known on lists?"}),
        };

        assert!(blocked_root_tool_rerun(&tool_call, Some(&prior)).is_some());
    }

    #[test]
    fn preserves_prior_grounded_result_on_failure() {
        let prior = SuccessfulRootToolRun {
            tool_name: "search".to_string(),
            query: "What is the sentiment about elsyluna.bsky.social?".to_string(),
            rendered_result: "summary: grounded earlier result".to_string(),
            summary: "grounded earlier result".to_string(),
            actor_refs: vec!["elsyluna.bsky.social".to_string()],
            collection_ids: vec!["clearsky_lists:did:plc:testactor".to_string()],
            intent: "reputation_lists".to_string(),
            requested_post_count: None,
        };
        let answer = fallback_or_failure_answer(Some(&prior), "Tool execution failed: boom");
        assert!(answer.contains("earlier grounded result"));
        assert!(answer.contains("boom"));
    }

    #[test]
    fn extracts_successful_root_search_record_from_grounded_output() {
        let tool_call = PromptToolCall {
            name: "search".to_string(),
            args: json!({"query":"What is the sentiment about elsyluna.bsky.social?"}),
        };
        let output = "summary: grounded summary\nselected_result_uri: at://one\ncollection_id: clearsky_lists:did:plc:testactor\nstatus: ok\nsearch_result_1_uri: at://one";
        let record = extract_successful_root_tool_record(&tool_call, output)
            .expect("expected successful record");
        assert_eq!(record.intent, "reputation_lists");
        assert_eq!(record.actor_refs, vec!["elsyluna.bsky.social"]);
    }

    #[test]
    fn classifies_reputation_queries_for_root_guard() {
        assert_eq!(
            classify_root_search_intent("How is elsyluna.bsky.social known on Bluesky lists?"),
            "reputation_lists"
        );
    }

    #[test]
    fn requested_post_window_size_detects_last_50_posts() {
        assert_eq!(
            requested_post_window_size("analyze the last 50 posts by mara.x0f.nl"),
            Some(50)
        );
    }

    #[test]
    fn summary_refresh_requirements_scale_for_last_50_posts() {
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: json!({"query":"analyze the last 50 posts by mara.x0f.nl"}),
        };

        assert_eq!(
            planned_recent_post_refresh_requirements(&tool_call),
            (100, 50)
        );
    }

    #[test]
    fn planned_refresh_targets_detect_actor_ref_from_summary_query() {
        let mut store = NotificationStore::new();
        let did: Did = "did:plc:testactor".parse().expect("invalid did");
        store.cache_actor(&did, "mara.x0f.nl", None);
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: json!({"query":"analyze the last 50 posts by mara.x0f.nl"}),
        };

        let targets = planned_tool_call_refresh_targets(&store, None, &tool_call);
        assert_eq!(targets, vec![did]);
    }

    #[test]
    fn planned_refresh_targets_fall_back_to_selected_actor_did() {
        let store = NotificationStore::new();
        let did: Did = "did:plc:testactor".parse().expect("invalid did");
        let tool_call = PromptToolCall {
            name: "summary".to_string(),
            args: json!({"query":"analyze the last 50 posts by this actor"}),
        };

        let targets = planned_tool_call_refresh_targets(&store, Some(did.clone()), &tool_call);
        assert_eq!(targets, vec![did]);
    }

    #[test]
    fn compact_search_keeps_summary_and_selected_result_fields() {
        let tool_output = "search searched collections independently and combined the grounded results below.\nsummary: grounded answer\nselected_result_uri: at://one\nselected_result_source_collection_id: clearsky:test\nselected_result_collection_id: clearsky:test\nselected_result_collection_label: Clearsky test\n\ncollection_id: clearsky:test\ncollection_label: Clearsky test\nstatus: ok\npost: picked\nsummary: child details";

        let compact = compact_search_like_result_for_root_context(tool_output);

        assert!(compact.contains("summary: grounded answer"));
        assert!(compact.contains("selected_result_uri: at://one"));
        assert!(compact.contains("selected_result_collection_id: clearsky:test"));
        assert!(!compact.contains("\ncollection_id: clearsky:test"));
        assert!(!compact.contains("post: picked"));
    }

    #[test]
    fn compact_search_truncates_summary_and_caps_diagnostics() {
        let long_summary = format!("summary: {}", "x".repeat(800));
        let tool_output = format!(
            "search searched collections independently and combined the grounded results below.\n\
diagnostic: first repeated diagnostic that should be kept in compact root context\n\
diagnostic: second repeated diagnostic that should also be kept in compact root context\n\
diagnostic: third repeated diagnostic that should be counted but not kept verbatim\n\
{}\n\
selected_result_uri: at://one",
            long_summary
        );

        let compact = compact_search_like_result_for_root_context(&tool_output);

        assert!(compact.contains("selected_result_uri: at://one"));
        assert!(compact.contains("diagnostic_count: 3"));
        assert!(compact.contains("summary: "));
        assert!(compact.len() < tool_output.len());
    }

    #[test]
    fn compact_search_summarizes_failed_collections() {
        let tool_output = "search searched collections independently and combined the grounded results below.\nsummary: grounded answer\nselected_result_uri: at://one\nselected_result_source_collection_id: clearsky:test\nselected_result_collection_id: clearsky:test\nselected_result_collection_label: Clearsky test\n\ncollection_id: replies:test\ncollection_label: Recent replies\nstatus: failed\nreview_reason: No usable summary.\n\ncollection_id: profile:test\ncollection_label: Profile\nstatus: ok";

        let compact = compact_search_like_result_for_root_context(tool_output);

        assert!(compact.contains("collection_failures: Recent replies"));
    }
}
