use crate::app::ConversationTurn;
use crate::harness::agents::AgentGraph;
use crate::harness::context_window::{
    BuiltContextSection, BuiltContextWindow, ContextSectionKind, LLMContext, ProviderContextLimits,
    approximate_tokens, build_context_window_report,
};
use crate::harness::llm_api::LlmApiClient;
use crate::harness::runtime::{ContextMessage, ContextMessageKind};
use crate::harness::tools::BlueskyTools;
use crate::visualization::context::{
    ContextCategory, ContextSegment, ContextVisualizationData, PromptContextSnapshot,
    snapshot_from_agent_node,
};

pub fn build_live_context_visualization(
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

pub fn build_root_context_snapshot(
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
            category: root_category_for_section(section),
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
            ContextCategory::LocalTask,
            false,
        ),
        ContextMessageKind::RepairPrompt => (
            "Repair Prompt".to_string(),
            ContextCategory::LocalTask,
            false,
        ),
    }
}

fn root_category_for_section(section: &BuiltContextSection) -> ContextCategory {
    match section.kind {
        ContextSectionKind::ToolDefinitions => ContextCategory::ToolDefinitions,
        ContextSectionKind::UiContext => ContextCategory::UiContext,
        ContextSectionKind::CurrentTask => ContextCategory::LocalTask,
        ContextSectionKind::UserAiChat => ContextCategory::UserAiChat,
        ContextSectionKind::CollectionEvidence => ContextCategory::CollectionEvidence,
        ContextSectionKind::ReviewEvidence => ContextCategory::ReviewEvidence,
        ContextSectionKind::ParentSearchResults => ContextCategory::ParentSearchResults,
        ContextSectionKind::Generic => match section.title.as_str() {
            "Tools" => ContextCategory::ToolDefinitions,
            "Search Hints" | "Current UI Context" => ContextCategory::UiContext,
            "Current Task" => ContextCategory::LocalTask,
            "Recent Chat" => ContextCategory::UserAiChat,
            _ => ContextCategory::UiContext,
        },
    }
}

pub fn build_tool_aware_query_context_window(
    selected_actor_did: Option<&bsky_sdk::api::types::string::Did>,
    selected_actor_summary: Option<String>,
    tools: &BlueskyTools,
    root_conversation: &[ConversationTurn],
    query: &str,
    llm_client: &LlmApiClient,
) -> BuiltContextWindow {
    let mut context =
        LLMContext::new("Use the available tools only when they materially improve the answer.");
    context.push_section_with_kind(
        "Tools",
        ContextSectionKind::ToolDefinitions,
        tools.render_tool_inventory(),
    );
    context.push_section_with_kind(
        "Search Hints",
        ContextSectionKind::UiContext,
        selected_actor_did
            .map(search_hints_for_actor_did)
            .unwrap_or_else(|| {
                "Use `search` with a natural-language `query` when you need selective Bluesky-grounded evidence about a handle/user or broader topic questions. Use `summary` for coverage-oriented requests like summarizing the last 50 posts by an actor.".to_string()
            }),
    );

    context.push_section_with_kind(
        "Current UI Context",
        ContextSectionKind::UiContext,
        selected_actor_summary
            .unwrap_or_else(|| "No actor is currently selected in the UI.".to_string()),
    );
    context.push_section_with_kind("Current Task", ContextSectionKind::CurrentTask, query);

    if !root_conversation.is_empty() {
        context.push_section_with_kind(
            "Recent Chat",
            ContextSectionKind::UserAiChat,
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
        "The selected actor is {did}. Use `search` with a natural-language `query` when you need selective grounded evidence about this actor or related topics. Use `summary` when you need broad coverage such as summarizing the actor's recent posts or replies."
    )
}

#[cfg(test)]
mod tests {
    use super::build_root_context_snapshot;
    use crate::harness::context_window::{
        BuiltContextSection, BuiltContextWindow, ContextSectionKind, ProviderContextLimits,
    };
    use crate::harness::llm_api::ChatMessage;
    use crate::harness::runtime::{ContextMessage, ContextMessageKind};
    use crate::visualization::context::ContextCategory;

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
                kind: ContextSectionKind::CurrentTask,
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
