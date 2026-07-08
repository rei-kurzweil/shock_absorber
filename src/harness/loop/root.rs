use crate::harness::r#loop::{
    LoopDefinition, LoopExecutor, LoopKind, LoopNodeDefinition, LoopNodeKind, LoopPort,
    LoopPortTarget,
};

const ROOT_NODES: &[LoopNodeDefinition] = &[
    LoopNodeDefinition {
        id: "root_prompt",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Llm,
        handler_key: "root_prompt_round",
        ports: &[
            LoopPort {
                name: "tool_call",
                target: LoopPortTarget::Node("tool_execution"),
            },
            LoopPort {
                name: "final_answer",
                target: LoopPortTarget::Return,
            },
            LoopPort {
                name: "invalid",
                target: LoopPortTarget::Node("protocol_repair"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "protocol_repair",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "root_protocol_repair",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("root_prompt"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "tool_execution",
        kind: LoopNodeKind::Tool,
        executor: LoopExecutor::Harness,
        handler_key: "execute_root_tool_call",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("root_prompt"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
];

pub const DEFINITION: LoopDefinition = LoopDefinition {
    kind: LoopKind::Root,
    entry_node: "root_prompt",
    nodes: ROOT_NODES,
};
