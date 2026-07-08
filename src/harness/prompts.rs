#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    CollectionSearch,
    CollectionSummary,
    SearchReview,
    SummaryReview,
    LlmSearch,
}

impl AgentKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::CollectionSearch => "collection_search",
            Self::CollectionSummary => "collection_summary",
            Self::SearchReview => "search_review",
            Self::SummaryReview => "summary_review",
            Self::LlmSearch => "llm_search",
        }
    }

    pub fn system_prompt(self) -> &'static str {
        match self {
            Self::CollectionSearch => include_str!("prompts/agents/collection_search.md").trim(),
            Self::CollectionSummary => include_str!("prompts/agents/collection_summary.md").trim(),
            Self::SearchReview => include_str!("prompts/agents/search_review.md").trim(),
            Self::SummaryReview => include_str!("prompts/agents/summary_review.md").trim(),
            Self::LlmSearch => include_str!("prompts/agents/llm_search.md").trim(),
        }
    }
}

pub fn tool_prompt() -> &'static str {
    include_str!("prompts/tool_prompt.md").trim()
}
