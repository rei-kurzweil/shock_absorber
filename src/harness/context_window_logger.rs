use crate::harness::agents::AgentGraph;
use crate::visualization::context::{ContextCategory, PromptContextSnapshot};
use std::fs;
use std::path::{Path, PathBuf};

pub fn reset_debug_dir(base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let debug_dir = debug_dir(base_dir);
    if debug_dir.exists() {
        fs::remove_dir_all(&debug_dir)?;
    }
    fs::create_dir_all(debug_dir.join("agents"))?;
    Ok(())
}

pub fn log_agent_graph(
    base_dir: &Path,
    graph: &AgentGraph,
) -> Result<(), Box<dyn std::error::Error>> {
    let agents_dir = debug_dir(base_dir).join("agents");
    fs::create_dir_all(&agents_dir)?;

    if let Some(root) = graph.node(graph.root_agent_id()) {
        fs::write(
            agents_dir.join(format!("agent_{:03}_root_agent.md", root.agent_id.0)),
            render_agent_node(root),
        )?;
    }

    for (_, node_id) in graph.descendant_ids_depth_first() {
        let Some(node) = graph.node(node_id) else {
            continue;
        };
        fs::write(agents_dir.join(agent_filename(node)), render_agent_node(node))?;
    }

    Ok(())
}

pub fn log_chat_transcript(
    base_dir: &Path,
    lines: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = String::from("# Chat Transcript\n\n");
    if lines.is_empty() {
        content.push_str("<empty>\n");
    } else {
        content.push_str("```text\n");
        content.push_str(&lines.join("\n"));
        content.push_str("\n```\n");
    }
    fs::write(debug_dir(base_dir).join("chat_transcript.md"), content)?;
    Ok(())
}

pub fn log_current_task(
    base_dir: &Path,
    graph: &AgentGraph,
    current_task: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lines = Vec::new();
    if let Some(root) = graph.node(graph.root_agent_id()) {
        lines.push(format!("root_agent_id: {}", root.agent_id.0));
        lines.push(format!("status: {}", root.status.as_str()));
        if let Some(summary) = root.result_summary.as_deref() {
            lines.push(format!("result_summary: {}", summary));
        }
    }
    lines.push(format!(
        "current_task: {}",
        current_task.unwrap_or("<none>")
    ));
    fs::write(debug_dir(base_dir).join("current_task.txt"), lines.join("\n"))?;
    Ok(())
}

pub fn log_root_prompt_snapshot(
    base_dir: &Path,
    snapshot: &PromptContextSnapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        debug_dir(base_dir).join("root_prompt_snapshot.md"),
        render_prompt_context_snapshot(snapshot),
    )?;
    Ok(())
}

fn debug_dir(base_dir: &Path) -> PathBuf {
    base_dir.join(".debug")
}

fn agent_filename(node: &crate::harness::agents::AgentNode) -> String {
    let kind = match node.agent_type {
        crate::harness::agents::AgentNodeKind::RootAgent => "root_agent",
        crate::harness::agents::AgentNodeKind::ToolAgent => "tool_agent",
        crate::harness::agents::AgentNodeKind::CollectionSearchAgent => "collection_search_agent",
    };
    let slug = slugify(&node.label);
    format!("agent_{:03}_{}_{}.md", node.agent_id.0, kind, slug)
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_was_sep = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    out.trim_matches('_').chars().take(48).collect()
}

fn render_agent_node(node: &crate::harness::agents::AgentNode) -> String {
    let mut out = String::from("# Agent Debug\n\n");
    out.push_str(&format!("- agent_id: {}\n", node.agent_id.0));
    out.push_str(&format!("- agent_type: {:?}\n", node.agent_type));
    if let Some(agent_kind) = node.agent_kind {
        out.push_str(&format!("- agent_kind: {:?}\n", agent_kind));
    }
    out.push_str(&format!("- label: {}\n", node.label));
    out.push_str(&format!("- status: {}\n", node.status.as_str()));
    out.push_str(&format!(
        "- parent_agent_id: {}\n",
        node.parent_agent_id
            .map(|id| id.0.to_string())
            .unwrap_or_else(|| "<none>".to_string())
    ));
    out.push_str(&format!(
        "- child_agent_ids: {}\n",
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
        out.push_str(&format!("- tool_name: {}\n", tool_name));
    }
    if let Some(collection_id) = node.collection_id.as_deref() {
        out.push_str(&format!("- collection_id: {}\n", collection_id));
    }
    out.push_str("\n## Result Summary\n\n");
    out.push_str(node.result_summary.as_deref().unwrap_or("<none>"));
    out.push_str("\n\n");

    if let Some(window) = node.context_window_report.as_ref() {
        out.push_str("## Context Window Stats\n\n");
        out.push_str(&format!("- provider: {}\n", window.limits.provider_name));
        out.push_str(&format!("- model: {}\n", window.limits.model_name));
        out.push_str(&format!(
            "- max_context_tokens: {}\n",
            window.limits.max_context_tokens
        ));
        out.push_str(&format!(
            "- reserved_output_tokens: {}\n",
            window.limits.reserved_output_tokens
        ));
        out.push_str(&format!("- used_input_tokens: {}\n", window.used_input_tokens));
        out.push_str(&format!("- truncated: {}\n\n", window.truncated));
        out.push_str("## Rendered Context Window\n\n```text\n");
        out.push_str(&window.rendered);
        if !window.rendered.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```\n");
    }

    out
}

fn render_prompt_context_snapshot(snapshot: &PromptContextSnapshot) -> String {
    let mut out = String::from("# Root Prompt Snapshot\n\n");
    out.push_str(&format!("- title: {}\n", snapshot.title));
    out.push_str(&format!("- provider: {}\n", snapshot.provider_name));
    out.push_str(&format!("- model: {}\n", snapshot.model_name));
    out.push_str(&format!(
        "- max_context_tokens: {}\n",
        snapshot.max_context_tokens
    ));
    out.push_str(&format!(
        "- reserved_output_tokens: {}\n",
        snapshot.reserved_output_tokens
    ));
    out.push_str(&format!(
        "- input_budget_tokens: {}\n",
        snapshot.input_budget_tokens
    ));
    out.push_str(&format!(
        "- used_input_tokens: {}\n",
        snapshot.used_input_tokens
    ));
    out.push_str(&format!("- truncated: {}\n\n", snapshot.truncated));

    out.push_str("## Category Totals\n\n");
    let totals = category_totals(snapshot);
    out.push_str(&format!("- system_prompt: {}\n", totals[0]));
    out.push_str(&format!("- tools: {}\n", totals[1]));
    out.push_str(&format!("- ui: {}\n", totals[2]));
    out.push_str(&format!("- task: {}\n", totals[3]));
    out.push_str(&format!("- chat: {}\n", totals[4]));
    out.push_str(&format!("- tool_output: {}\n\n", totals[5]));

    out.push_str("## Segments\n\n");
    for segment in &snapshot.segments {
        out.push_str(&format!(
            "- {}: {} tokens{}\n",
            segment.label,
            segment.tokens,
            if segment.truncated { " (trimmed)" } else { "" }
        ));
    }

    out
}

fn category_totals(snapshot: &PromptContextSnapshot) -> [usize; 6] {
    let mut totals = [0usize; 6];
    for segment in &snapshot.segments {
        totals[match segment.category {
            ContextCategory::SystemPrompt => 0,
            ContextCategory::ToolInstructions
            | ContextCategory::RootInstructions
            | ContextCategory::ToolDefinitions => 1,
            ContextCategory::UiContext => 2,
            ContextCategory::CurrentTask => 3,
            ContextCategory::UserAiChat => 4,
            ContextCategory::ToolResults => 5,
        }] += segment.tokens;
    }
    totals
}
