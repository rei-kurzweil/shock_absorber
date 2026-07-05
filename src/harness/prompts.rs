#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    CollectionSearch,
    LlmSearch,
}

impl AgentKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::CollectionSearch => "collection_search",
            Self::LlmSearch => "llm_search",
        }
    }

    pub fn system_prompt(self) -> &'static str {
        match self {
            Self::CollectionSearch => {
                include_str!("prompts/agents/collection_search.md").trim()
            }
            Self::LlmSearch => include_str!("prompts/agents/llm_search.md").trim(),
        }
    }
}

pub fn tool_prompt() -> &'static str {
    include_str!("prompts/tool_prompt.md").trim()
}
