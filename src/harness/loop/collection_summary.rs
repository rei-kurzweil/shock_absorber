use crate::harness::r#loop::{
    LoopDefinition, LoopExecutor, LoopKind, LoopNodeDefinition, LoopNodeKind, LoopPort,
    LoopPortTarget,
};

const COLLECTION_SUMMARY_NODES: &[LoopNodeDefinition] = &[
    LoopNodeDefinition {
        id: "init_window",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Harness,
        handler_key: "initialize_summary_window",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("summarize_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "summarize_page",
        kind: LoopNodeKind::Tool,
        executor: LoopExecutor::Llm,
        handler_key: "run_summary_page_compaction",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("review_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Node("repair_summary_output"),
            },
        ],
    },
    LoopNodeDefinition {
        id: "repair_summary_output",
        kind: LoopNodeKind::Repair,
        executor: LoopExecutor::Harness,
        handler_key: "repair_summary_output",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("review_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "review_page",
        kind: LoopNodeKind::Review,
        executor: LoopExecutor::Harness,
        handler_key: "review_summary_page",
        ports: &[
            LoopPort {
                name: "complete",
                target: LoopPortTarget::Node("return_summary"),
            },
            LoopPort {
                name: "continue",
                target: LoopPortTarget::Node("advance_cursor"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "advance_cursor",
        kind: LoopNodeKind::Branch,
        executor: LoopExecutor::Harness,
        handler_key: "advance_summary_cursor",
        ports: &[
            LoopPort {
                name: "success",
                target: LoopPortTarget::Node("summarize_page"),
            },
            LoopPort {
                name: "failure",
                target: LoopPortTarget::Fail,
            },
        ],
    },
    LoopNodeDefinition {
        id: "return_summary",
        kind: LoopNodeKind::Return,
        executor: LoopExecutor::Harness,
        handler_key: "return_collection_summary",
        ports: &[],
    },
];

pub const DEFINITION: LoopDefinition = LoopDefinition {
    kind: LoopKind::CollectionSummary,
    entry_node: "init_window",
    nodes: COLLECTION_SUMMARY_NODES,
};
