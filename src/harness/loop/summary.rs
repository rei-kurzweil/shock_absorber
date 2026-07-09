use crate::harness::r#loop::{
    LoopDefinition, LoopExecutor, LoopKind, LoopNodeDefinition, LoopNodeKind, LoopPort,
    LoopPortTarget,
};

const SUMMARY_NODES: &[LoopNodeDefinition] = &[
    LoopNodeDefinition {
        id: "resolve_scope",
        kind: LoopNodeKind::Tool,
        executor: LoopExecutor::Harness,
        handler_key: "resolve_summary_scope",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("run_collection_summary"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "run_collection_summary",
        kind: LoopNodeKind::Tool,
        executor: LoopExecutor::Harness,
        handler_key: "run_collection_summary",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Return,
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
];

pub const DEFINITION: LoopDefinition = LoopDefinition {
    kind: LoopKind::Summary,
    entry_node: "resolve_scope",
    nodes: SUMMARY_NODES,
};
