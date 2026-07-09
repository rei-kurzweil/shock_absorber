#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    CollectionSearch,
    CollectionSummary,
    SearchReview,
    SummaryReview,
    Search,
    Summary,
}

impl AgentKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::CollectionSearch => "collection_search",
            Self::CollectionSummary => "collection_summary",
            Self::SearchReview => "search_review",
            Self::SummaryReview => "summary_review",
            Self::Search => "search",
            Self::Summary => "summary",
        }
    }

    pub fn system_prompt(self) -> &'static str {
        match self {
            Self::CollectionSearch => include_str!("prompts/agents/collection_search.md").trim(),
            Self::CollectionSummary => include_str!("prompts/agents/collection_summary.md").trim(),
            Self::SearchReview => include_str!("prompts/agents/search_review.md").trim(),
            Self::SummaryReview => include_str!("prompts/agents/summary_review.md").trim(),
            Self::Search => include_str!("prompts/agents/search.md").trim(),
            Self::Summary => include_str!("prompts/agents/summary.md").trim(),
        }
    }
}

pub fn tool_prompt() -> &'static str {
    include_str!("prompts/tool_prompt.md").trim()
}
