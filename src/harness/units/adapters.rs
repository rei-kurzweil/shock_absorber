use crate::harness::agents::{AgentGraph, AgentNodeStatus, AgentNodeTemplate};
use crate::harness::r#loop::{LoopDefinition, LoopPortTarget};
use crate::harness::units::definition::{
    UnitDefinition, UnitEdgeDefinition, UnitEdgeTarget, UnitGraphDefinition, UnitLocalStateSchema,
    UnitPortDefinition,
};
use crate::harness::units::kinds::UnitKind;
use crate::harness::units::state::{UnitInstanceState, UnitInstanceStatus};
use std::collections::BTreeSet;

pub fn unit_definition_from_loop(
    id: impl Into<String>,
    label: impl Into<String>,
    kind: UnitKind,
    loop_definition: &LoopDefinition,
    local_state_schema: UnitLocalStateSchema,
) -> UnitDefinition {
    let mut ports = BTreeSet::new();
    let mut edges = Vec::new();
    let mut nodes = Vec::new();
    for node in loop_definition.nodes {
        nodes.push(node.id.to_string());
        for port in node.ports {
            ports.insert(port.name.to_string());
            edges.push(UnitEdgeDefinition {
                from_node: node.id.to_string(),
                port: port.name.to_string(),
                target: match port.target {
                    LoopPortTarget::Node(target) => UnitEdgeTarget::Node(target.to_string()),
                    LoopPortTarget::Return => UnitEdgeTarget::Return,
                    LoopPortTarget::Fail => UnitEdgeTarget::Fail,
                },
                child_unit_id: None,
            });
        }
    }

    UnitDefinition {
        id: id.into(),
        label: label.into(),
        kind,
        graph: Some(UnitGraphDefinition {
            entry_node: loop_definition.entry_node.to_string(),
            nodes,
            ports: ports
                .into_iter()
                .map(|name| UnitPortDefinition {
                    name,
                    description: None,
                })
                .collect(),
            edges,
        }),
        local_state_schema,
    }
}

pub fn agent_template_from_unit(unit: &UnitInstanceState) -> AgentNodeTemplate {
    AgentNodeTemplate {
        agent_type: unit.kind.compatibility_agent_node_kind(),
        agent_kind: unit.kind.compatibility_agent_kind(),
        label: unit.instance_label.clone(),
        status: match unit.status {
            UnitInstanceStatus::Ready => AgentNodeStatus::Ready,
            UnitInstanceStatus::Running | UnitInstanceStatus::BlockedOnChild => {
                AgentNodeStatus::Running
            }
            UnitInstanceStatus::Completed => AgentNodeStatus::Completed,
            UnitInstanceStatus::CompletedWithWarnings => AgentNodeStatus::CompletedWithWarnings,
            UnitInstanceStatus::Failed => AgentNodeStatus::Failed,
        },
        tool_name: unit.tool_name.clone(),
        collection_id: unit.collection_id.clone(),
        context_window_report: unit.context_window.clone(),
        result_summary: unit.result_summary.clone(),
        children: unit.children.iter().map(agent_template_from_unit).collect(),
    }
}

pub fn agent_graph_from_root_unit(root: &UnitInstanceState) -> AgentGraph {
    let mut graph = AgentGraph::new_root(root.instance_label.clone());
    if let Some(context_window) = root.context_window.clone() {
        graph.set_context_window(graph.root_agent_id(), context_window);
    }
    if let Some(result_summary) = root.result_summary.clone() {
        graph.set_result_summary(graph.root_agent_id(), result_summary);
    }
    graph.set_status(
        graph.root_agent_id(),
        match unit_status_to_agent_status(root.status) {
            Some(status) => status,
            None => AgentNodeStatus::Running,
        },
    );
    for child in &root.children {
        graph.attach_template(graph.root_agent_id(), agent_template_from_unit(child));
    }
    graph
}

fn unit_status_to_agent_status(status: UnitInstanceStatus) -> Option<AgentNodeStatus> {
    Some(match status {
        UnitInstanceStatus::Ready => AgentNodeStatus::Ready,
        UnitInstanceStatus::Running | UnitInstanceStatus::BlockedOnChild => AgentNodeStatus::Running,
        UnitInstanceStatus::Completed => AgentNodeStatus::Completed,
        UnitInstanceStatus::CompletedWithWarnings => AgentNodeStatus::CompletedWithWarnings,
        UnitInstanceStatus::Failed => AgentNodeStatus::Failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::r#loop::summary;

    #[test]
    fn loop_adapter_preserves_cycle_capable_graph_metadata() {
        let definition = unit_definition_from_loop(
            "summary",
            "summary",
            UnitKind::Loop,
            &summary::DEFINITION,
            UnitLocalStateSchema::None,
        );

        let graph = definition.graph.expect("graph");
        assert_eq!(graph.entry_node, "resolve_scope");
        assert!(graph.nodes.iter().any(|node| node == "run_collection_summary"));
        assert!(graph.ports.iter().any(|port| port.name == "success"));
    }
}
