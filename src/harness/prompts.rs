#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    LlmSearch,
}

impl AgentKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::LlmSearch => "llm_search",
        }
    }

    pub fn system_prompt(self) -> &'static str {
        match self {
            Self::LlmSearch => include_str!("prompts/agents/llm_search.md").trim(),
        }
    }
}

pub fn tool_prompt() -> &'static str {
    include_str!("prompts/tool_prompt.md").trim()
}
