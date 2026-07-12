use crate::harness::agents::AgentNodeKind;
use crate::harness::prompts::AgentKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnitKind {
    Root,
    PublicToolOrchestration,
    Loop,
    OneShotLlm,
    DeterministicOrchestration,
    CollectionSearch,
    CollectionSummary,
    SearchReview,
    SummaryReview,
}

impl UnitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::PublicToolOrchestration => "public_tool_orchestration",
            Self::Loop => "loop",
            Self::OneShotLlm => "one_shot_llm",
            Self::DeterministicOrchestration => "deterministic_orchestration",
            Self::CollectionSearch => "collection_search",
            Self::CollectionSummary => "collection_summary",
            Self::SearchReview => "search_review",
            Self::SummaryReview => "summary_review",
        }
    }

    pub fn compatibility_agent_node_kind(self) -> AgentNodeKind {
        match self {
            Self::Root => AgentNodeKind::RootAgent,
            Self::PublicToolOrchestration
            | Self::Loop
            | Self::OneShotLlm
            | Self::DeterministicOrchestration => AgentNodeKind::ToolAgent,
            Self::CollectionSearch => AgentNodeKind::CollectionSearchTool,
            Self::CollectionSummary => AgentNodeKind::CollectionSummaryTool,
            Self::SearchReview => AgentNodeKind::SearchReviewAgent,
            Self::SummaryReview => AgentNodeKind::SummaryReviewAgent,
        }
    }

    pub fn compatibility_agent_kind(self) -> Option<AgentKind> {
        match self {
            Self::PublicToolOrchestration => None,
            Self::CollectionSearch => Some(AgentKind::CollectionSearch),
            Self::CollectionSummary => Some(AgentKind::CollectionSummary),
            Self::SearchReview => Some(AgentKind::SearchReview),
            Self::SummaryReview => Some(AgentKind::SummaryReview),
            Self::OneShotLlm => None,
            Self::DeterministicOrchestration => None,
            Self::Loop => None,
            Self::Root => None,
        }
    }
}
