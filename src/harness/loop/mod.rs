pub mod collection_summary;
pub mod llm_search;
pub mod root;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopKind {
    Root,
    LlmSearch,
    CollectionSummary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopNodeKind {
    Tool,
    Review,
    Repair,
    Synthesize,
    Branch,
    Return,
    Fail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopExecutor {
    Harness,
    Llm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopPortTarget {
    Node(&'static str),
    Return,
    Fail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopPort {
    pub name: &'static str,
    pub target: LoopPortTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopNodeDefinition {
    pub id: &'static str,
    pub kind: LoopNodeKind,
    pub executor: LoopExecutor,
    pub handler_key: &'static str,
    pub ports: &'static [LoopPort],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopDefinition {
    pub kind: LoopKind,
    pub entry_node: &'static str,
    pub nodes: &'static [LoopNodeDefinition],
}
