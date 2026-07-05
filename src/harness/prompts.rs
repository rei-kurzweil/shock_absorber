#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    LlmSearch,
    LlmSearchParent,
}

impl AgentKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::LlmSearch => "llm_search",
            Self::LlmSearchParent => "llm_search_parent",
        }
    }

    pub fn system_prompt(self) -> &'static str {
        match self {
            Self::LlmSearch => include_str!("prompts/agents/llm_search.md").trim(),
            Self::LlmSearchParent => {
                include_str!("prompts/agents/llm_search_parent.md").trim()
            }
        }
    }
}

pub fn tool_prompt() -> &'static str {
    include_str!("prompts/tool_prompt.md").trim()
}
