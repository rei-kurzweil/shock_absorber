use crate::harness::context_window::BuiltContextWindow;
use crate::harness::prompts::AgentKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AgentNodeId(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentNodeKind {
    RootAgent,
    ToolAgent,
    CollectionSearchTool,
    CollectionSummaryTool,
    SearchReviewAgent,
    SummaryReviewAgent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentNodeStatus {
    Ready,
    Running,
    Completed,
    CompletedWithWarnings,
    Failed,
}

impl AgentNodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::CompletedWithWarnings => "warning",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentNode {
    pub agent_id: AgentNodeId,
    pub agent_type: AgentNodeKind,
    pub agent_kind: Option<AgentKind>,
    pub parent_agent_id: Option<AgentNodeId>,
    pub child_agent_ids: Vec<AgentNodeId>,
    pub label: String,
    pub status: AgentNodeStatus,
    pub tool_name: Option<String>,
    pub collection_id: Option<String>,
    pub context_window_report: Option<BuiltContextWindow>,
    pub result_summary: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AgentNodeTemplate {
    pub agent_type: AgentNodeKind,
    pub agent_kind: Option<AgentKind>,
    pub label: String,
    pub status: AgentNodeStatus,
    pub tool_name: Option<String>,
    pub collection_id: Option<String>,
    pub context_window_report: Option<BuiltContextWindow>,
    pub result_summary: Option<String>,
    pub children: Vec<AgentNodeTemplate>,
}

#[derive(Clone, Debug)]
pub struct AgentGraph {
    root_agent_id: AgentNodeId,
    nodes: Vec<AgentNode>,
}

impl AgentGraph {
    pub fn new_root(label: impl Into<String>) -> Self {
        let root_agent_id = AgentNodeId(0);
        Self {
            root_agent_id,
            nodes: vec![AgentNode {
                agent_id: root_agent_id,
                agent_type: AgentNodeKind::RootAgent,
                agent_kind: None,
                parent_agent_id: None,
                child_agent_ids: Vec::new(),
                label: label.into(),
                status: AgentNodeStatus::Running,
                tool_name: None,
                collection_id: None,
                context_window_report: None,
                result_summary: None,
            }],
        }
    }

    pub fn root_agent_id(&self) -> AgentNodeId {
        self.root_agent_id
    }

    pub fn node(&self, id: AgentNodeId) -> Option<&AgentNode> {
        self.nodes.get(id.0)
    }

    pub fn nodes(&self) -> &[AgentNode] {
        &self.nodes
    }

    pub fn node_mut(&mut self, id: AgentNodeId) -> Option<&mut AgentNode> {
        self.nodes.get_mut(id.0)
    }

    pub fn set_context_window(&mut self, id: AgentNodeId, window: BuiltContextWindow) {
        if let Some(node) = self.node_mut(id) {
            node.context_window_report = Some(window);
        }
    }

    pub fn set_result_summary(&mut self, id: AgentNodeId, result_summary: impl Into<String>) {
        if let Some(node) = self.node_mut(id) {
            node.result_summary = Some(result_summary.into());
        }
    }

    pub fn set_status(&mut self, id: AgentNodeId, status: AgentNodeStatus) {
        if let Some(node) = self.node_mut(id) {
            node.status = status;
        }
    }

    pub fn attach_template(
        &mut self,
        parent_id: AgentNodeId,
        template: AgentNodeTemplate,
    ) -> AgentNodeId {
        let node_id = self.push_node(parent_id, &template);
        for child in template.children {
            self.attach_template(node_id, child);
        }
        node_id
    }

    fn push_node(&mut self, parent_id: AgentNodeId, template: &AgentNodeTemplate) -> AgentNodeId {
        let agent_id = AgentNodeId(self.nodes.len());
        self.nodes.push(AgentNode {
            agent_id,
            agent_type: template.agent_type.clone(),
            agent_kind: template.agent_kind,
            parent_agent_id: Some(parent_id),
            child_agent_ids: Vec::new(),
            label: template.label.clone(),
            status: template.status.clone(),
            tool_name: template.tool_name.clone(),
            collection_id: template.collection_id.clone(),
            context_window_report: template.context_window_report.clone(),
            result_summary: template.result_summary.clone(),
        });
        if let Some(parent) = self.node_mut(parent_id) {
            parent.child_agent_ids.push(agent_id);
        }
        agent_id
    }

    pub fn descendant_ids_depth_first(&self) -> Vec<(usize, AgentNodeId)> {
        let mut ordered = Vec::new();
        self.collect_children(self.root_agent_id, 0, &mut ordered);
        ordered
    }

    pub fn ids_with_depth_in_display_order(&self) -> Vec<(usize, AgentNodeId)> {
        let mut ordered = vec![(0, self.root_agent_id)];
        ordered.extend(
            self.descendant_ids_depth_first()
                .into_iter()
                .map(|(depth, id)| (depth + 1, id)),
        );
        ordered
    }

    fn collect_children(
        &self,
        parent_id: AgentNodeId,
        depth: usize,
        ordered: &mut Vec<(usize, AgentNodeId)>,
    ) {
        let Some(parent) = self.node(parent_id) else {
            return;
        };
        for child_id in &parent.child_agent_ids {
            ordered.push((depth, *child_id));
            self.collect_children(*child_id, depth + 1, ordered);
        }
    }
}
