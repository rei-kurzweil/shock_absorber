use crate::app::EvilGemmaConfig;
use crate::harness::agents::AgentNodeStatus;
use crate::harness::context_window_logger::{
    log_agent_graph, log_chat_transcript, log_current_task, log_root_prompt_snapshot,
};
use crate::harness::root_context::build_live_context_visualization;
use crate::harness::runtime::{
    ContextMessage, ContextMessageKind, RootRunState, RootRunStatus, SuccessfulRootLlmSearch,
    TranscriptEntryKind,
};
use crate::harness::tools::{BlueskyTools, PromptToolCall, ToolProgressEvent, parse_prompt_tool_call,
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
                    "Runtime Notice\nblocked root `llm_search` rerun and preserved the earlier grounded result",
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
        "llm_search" => compact_llm_search_result_for_root_context(tool_output),
        _ => truncate_for_root_context(tool_output, 24),
    }
}

fn compact_llm_search_result_for_root_context(tool_output: &str) -> String {
    let mut kept = Vec::new();
    let mut failed_collections = Vec::new();
    let mut current_collection_label: Option<String> = None;

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
    tool_call: &PromptToolCall,
    last_failed_read_collection_id: Option<&str>,
) -> Option<String> {
    let _ = tool_call;
    let _ = last_failed_read_collection_id;
    None
}

fn blocked_root_llm_search_rerun(
    tool_call: &PromptToolCall,
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
    tool_call: &PromptToolCall,
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

fn planned_tool_call_refresh_targets(
    store: &NotificationStore,
    selected_actor_did: Option<bsky_sdk::api::types::string::Did>,
    tool_call: &PromptToolCall,
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

#[cfg(test)]
mod tests {
    use super::{
        blocked_root_llm_search_rerun, classify_root_llm_search_intent,
        compact_llm_search_result_for_root_context, extract_successful_root_llm_search_record,
        fallback_or_failure_answer,
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

    #[test]
    fn compact_llm_search_keeps_summary_and_selected_result_fields() {
        let tool_output = "llm_search searched collections independently and combined the grounded results below.\nsummary: grounded answer\nselected_result_uri: at://one\nselected_result_source_collection_id: clearsky:test\nselected_result_collection_id: clearsky:test\nselected_result_collection_label: Clearsky test\n\ncollection_id: clearsky:test\ncollection_label: Clearsky test\nstatus: ok\npost: picked\nsummary: child details";

        let compact = compact_llm_search_result_for_root_context(tool_output);

        assert!(compact.contains("summary: grounded answer"));
        assert!(compact.contains("selected_result_uri: at://one"));
        assert!(compact.contains("selected_result_collection_id: clearsky:test"));
        assert!(!compact.contains("\ncollection_id: clearsky:test"));
        assert!(!compact.contains("post: picked"));
    }

    #[test]
    fn compact_llm_search_summarizes_failed_collections() {
        let tool_output = "llm_search searched collections independently and combined the grounded results below.\nsummary: grounded answer\nselected_result_uri: at://one\nselected_result_source_collection_id: clearsky:test\nselected_result_collection_id: clearsky:test\nselected_result_collection_label: Clearsky test\n\ncollection_id: replies:test\ncollection_label: Recent replies\nstatus: failed\nreview_reason: No usable summary.\n\ncollection_id: profile:test\ncollection_label: Profile\nstatus: ok";

        let compact = compact_llm_search_result_for_root_context(tool_output);

        assert!(compact.contains("collection_failures: Recent replies"));
    }
}
