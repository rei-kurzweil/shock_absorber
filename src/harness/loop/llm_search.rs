use crate::harness::r#loop::{
    LoopDefinition, LoopExecutor, LoopKind, LoopNodeDefinition, LoopNodeKind, LoopPort,
    LoopPortTarget,
};

const LLM_SEARCH_NODES: &[LoopNodeDefinition] = &[
    LoopNodeDefinition {
        id: "planner_decide",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Llm,
        handler_key: "llm_search_planner_round",
        ports: &[
            LoopPort {
                name: "tool_call",
                target: LoopPortTarget::Node("execute_internal_tool"),
            },
            LoopPort {
                name: "final_answer",
                target: LoopPortTarget::Return,
            },
            LoopPort {
                name: "invalid",
                target: LoopPortTarget::Node("planner_protocol_repair"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "planner_protocol_repair",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "repair_planner_protocol",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("planner_decide"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "execute_internal_tool",
        kind: LoopNodeKind::Tool,
        executor: LoopExecutor::Harness,
        handler_key: "execute_llm_search_internal_tool",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("planner_decide"),
            },
            LoopPort {
                name: "invalid_tool_call",
                target: LoopPortTarget::Node("tool_call_repair"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "tool_call_repair",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "repair_llm_search_tool_call",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("execute_internal_tool"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
];

pub const DEFINITION: LoopDefinition = LoopDefinition {
    kind: LoopKind::LlmSearch,
    entry_node: "planner_decide",
    nodes: LLM_SEARCH_NODES,
};
