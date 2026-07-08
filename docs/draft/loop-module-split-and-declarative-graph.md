# Draft: `src/harness/loop/` Split And Declarative Loop Graphs

## Summary

`src/harness/tools.rs` is carrying too many responsibilities at once.

Today it contains:

- prompt-facing tool definitions
- internal planner protocol rules
- collection worker execution
- summary paging behavior
- review execution
- parsing
- rendering
- loop control flow

That makes it hard to reason about the actual runtime shape.

We should split loop ownership into `src/harness/loop/` and move toward declarative loop definitions.

## Main Point

Not every step is a loop.

Current likely loop owners:

- `root`
- `llm_search`
- `collection_summary`

Current likely non-loop steps:

- `collection_search`
- `search_review`
- `summary_review`
- one-shot parsing / repair / rendering helpers

So `collection_search` is not really a loop today.

It is better modeled as a one-shot worker step inside another loop.

## Proposed Module Split

Create:

```text
src/harness/loop/
  mod.rs
  root.rs
  llm_search.rs
  collection_summary.rs
```

Near-term intent:

- `root.rs`
  - root tool loop / root run orchestration
- `llm_search.rs`
  - internal planner loop
  - internal tool-call validation / repair handoff
- `collection_summary.rs`
  - harness-owned per-collection paging loop
  - page continuation state
  - sufficiency-driven termination

Keep non-loop code elsewhere:

- parsing stays in `tools.rs` at first, then later split further if useful
- review helpers can remain where they are initially
- tool registry can remain outside `loop/`

This is a structural cleanup, not a full runtime rewrite.

## Why This Split Helps

It makes ownership explicit.

Instead of one giant file hiding all control flow, we get:

- one module per actual loop owner
- one place to inspect loop state transitions
- clearer separation between:
  - "what tool exists"
  - "what step parses output"
  - "what loop advances state"

That is the minimum cleanup that prepares the codebase for a more declarative runtime model.

## Declarative Direction

The deeper improvement is to stop encoding each loop directly as imperative Rust branching.

Instead, each loop should have:

- a declarative stage/step definition
- named transitions
- explicit terminal states
- imperative callbacks only where needed

This should look more like a graph than a tree.

## Why A Graph, Not A Tree

Many transitions are shared.

Examples:

- multiple steps can fail into the same repair step
- multiple steps can complete into the same synthesis step
- success and failure may both route into a harness-owned verifier

So the right abstraction is:

- nodes
- ports / outcomes
- edges

not just nested child execution.

## Proposed Concepts

### Loop Definition

A loop definition declares:

- loop id
- entry node
- node table
- optional max rounds / max pages / safety limits

### Loop Node

A node is one step in the loop graph.

Each node should have:

- `id`
- `kind`
- `executor`
- `ports`

### Port

A port is a named outcome edge.

Examples:

- `success`
- `failure`
- `retry`
- `continue`
- `complete`
- `return`

### Executor

Execution may still be imperative.

That is fine.

The goal is not to remove code-driven behavior.

The goal is to make control flow declarative while allowing callbacks or handlers to perform the real work.

## Sketch

Possible shape:

```text
LoopDefinition
- loop_id
- entry_node
- nodes[]
- limits
```

```text
LoopNode
- id
- kind: tool | review | repair | synthesize | return | fail | branch
- executor: harness | llm
- handler_key
- ports
```

```text
PortMap
- success -> next_node
- failure -> next_node
- continue -> next_node
- complete -> return
```

## Example: `collection_summary`

The current harness-owned summary flow is a good fit for this model.

Possible graph:

```text
init_window
  success -> summarize_page

summarize_page
  success -> review_page
  failure -> parse_or_output_repair

parse_or_output_repair
  success -> review_page
  failure -> fail

review_page
  complete -> return_summary
  continue -> advance_cursor
  failure -> fail

advance_cursor
  success -> summarize_page
  failure -> fail

return_summary
  return

fail
  return
```

That is much easier to read than a long function full of nested branches.

## Example: `llm_search`

Likewise:

```text
planner_decide
  tool_call -> execute_internal_tool
  final_answer -> return
  invalid -> planner_protocol_repair

planner_protocol_repair
  success -> planner_decide
  failure -> fallback_or_fail

execute_internal_tool
  success -> planner_decide
  invalid_call -> tool_call_repair
  failure -> fallback_or_fail

tool_call_repair
  success -> execute_internal_tool
  failure -> fallback_or_fail

fallback_or_fail
  return
```

Again, the important thing is not eliminating imperative code.

The important thing is making the graph itself explicit.

## Practical Near-Term Design

Do not attempt a fully generic runtime first.

Instead:

1. move real loop owners into `src/harness/loop/`
2. define one small per-loop declarative node table near each loop
3. let handlers remain ordinary Rust functions
4. wire node ports explicitly

That gets most of the clarity benefit without a big framework rewrite.

## Suggested First Slice

### Slice 1

- add `src/harness/loop/mod.rs`
- move loop-heavy functions out of `tools.rs` into:
  - `loop/root.rs`
  - `loop/llm_search.rs`
  - `loop/collection_summary.rs`
- keep behavior unchanged

### Slice 2

- add a small `LoopNode` / `LoopPort` representation
- use it first for `collection_summary`
- keep handlers imperative

### Slice 3

- migrate `llm_search` planner loop to the same representation
- make repair / review / continue edges explicit

### Slice 4

- evaluate whether `root` should use the same runtime primitive or remain a thinner orchestrator

## Good Boundaries

### `collection_summary` should own

- current page offset
- pages already covered
- next-page continuation
- sufficiency termination

### `llm_search` should own

- collection choice
- internal tool choice
- planner protocol validation
- handoff to collection workers

### `collection_search` should not own

- page loop state
- run-level summary scope
- multi-page continuation policy

It should remain a one-shot worker unless requirements change.

## Open Questions

- Should the declarative node graph be per-loop Rust data, or a shared generic runtime type?
- Should ports be fixed names like `success` / `failure` / `continue`, or arbitrary labels?
- Should review and repair be first-class node kinds, or simply ordinary nodes with handler keys?
- Should loop node execution always produce a typed result object, or can some nodes remain text-only during migration?

## Recommendation

Yes, this direction is viable.

The most defensible next move is:

- split actual loop owners into `src/harness/loop/`
- treat `collection_search` as a step, not a loop
- make `collection_summary` the first declarative loop graph

That gives immediate codebase relief and also sets up the broader graph-based runtime model cleanly.
