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

Ports should not be treated as single fixed edges only.

For some nodes, especially failure/recovery nodes, one port should be allowed to expose multiple candidate next nodes with harness-owned runtime selection.

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

That is the minimal shape.

The next practical shape should be slightly richer:

```text
LoopEdge
- from_node
- port_name
- to_node
- priority
- uses_allowed
- consumes_budget
- guard_key
```

This allows a single port such as `failure` to have several possible recovery routes.

## Runtime State

Declarative graph shape alone is not enough.

The harness also needs mutable runtime state that records:

- which recovery routes have already been used
- which pagination advances have occurred
- which inner loops updated shared state
- which paths are still legal

## Proposed Runtime Structs

### `LoopRuntimeState`

Owns mutable state for one active loop run.

Suggested responsibilities:

- shared state visible to the loop
- node/edge budget tracking
- runtime events emitted by inner harness steps

Possible shape:

```text
LoopRuntimeState
- shared
- node_state_by_id
- transition_log
```

### `NodeRuntimeState`

Owns mutable budget/use tracking for one node.

Possible shape:

```text
NodeRuntimeState
- port_state_by_name
```

### `PortRuntimeState`

Tracks remaining available uses for a named port or its candidate edges.

Possible shape:

```text
PortRuntimeState
- uses_remaining
- exhausted
- last_used_step
```

### `SharedLoopState`

Owns loop-wide state that should not come from LLM text.

This is where pagination truth should live.

Possible shape:

```text
SharedLoopState
- pagination_by_collection_id
- recovery_events
- loop_flags
```

### `PaginationState`

This is the key new structure for `collection_summary`.

The inner harness loop should own and update this directly.

Possible shape:

```text
PaginationState
- current_offset
- current_page
- pages_attempted
- pages_completed
- next_offset
- next_page
- covered_ranges
- available_total_items
- cursor_advanced
```

Important rule:

- pagination state must be harness-authored
- it must not depend on LLM prose being the source of truth

## Failure Ports With Multiple Recovery Options

The failure port should be able to describe several available follow-up nodes.

Example:

```text
summarize_page
  failure ->
    repair_summary_output   uses_allowed=1
    retry_compaction        uses_allowed=1
    fail                    terminal
```

When one of those routes is used:

- its `uses_remaining` is decremented
- the harness decides whether it is still available later

This is much better than scattered local booleans such as:

- `already_repaired`
- `already_retried`
- `saw_parse_failure_once`

Those are really runtime edge-budget facts and should be modeled that way.

## Upward State Propagation

Inner harness loops should be able to publish state changes upward without routing that truth through the LLM.

Example:

- `collection_summary` updates per-collection pagination state
- `llm_search` receives or observes that update
- `llm_search` uses the updated state as harness truth

That means:

- `collection_summary` owns per-collection pagination
- `llm_search` owns collection-level orchestration
- the parent planner does not need to infer paging from text

## Suggested Runtime Events

Near-term, upward state propagation can be modeled as explicit harness events.

Examples:

```text
PaginationAdvanced
- collection_id
- from_offset
- to_offset
- from_page
- to_page
```

```text
CoverageUpdated
- collection_id
- covered_ranges
- available_total_items
```

```text
RecoveryRouteUsed
- node_id
- port_name
- target_node_id
- uses_remaining
```

These do not need to be user-visible transcript messages first.

They can simply update `LoopRuntimeState`.

## Example: `collection_summary`

`collection_summary` is the clearest first target for this runtime model.

It should:

- read `PaginationState` from harness-owned loop state
- run `summarize_page`
- run `review_page`
- decide between:
  - `complete`
  - `continue`
  - `failure`
- update `PaginationState` directly on continuation

The key point is:

- `collection_summary` should update `llm_search` about cursor/coverage progress as harness state
- not by asking the LLM to describe where the cursor is

## Example: `llm_search`

`llm_search` should be able to observe:

- this collection has already paged through offsets `0` and `25`
- the next valid offset is `50`
- a recovery path has already been consumed
- a collection summary loop completed or failed

That is orchestration state.

It belongs in harness runtime structures, not in LLM-generated summaries.

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
  complete -> collection_summary_planner
  continue -> advance_cursor
  failure -> fail

collection_summary_planner
  success -> collection_summary_planner_review
  failure -> collection_summary_planner_repair

collection_summary_planner_review
  continue -> advance_cursor
  complete -> collection_summary_notes
  failure -> collection_summary_planner_repair

collection_summary_planner_repair
  success -> collection_summary_planner_review
  failure -> fail

collection_summary_notes
  success -> collection_summary_notes_review
  failure -> fail

advance_cursor
  success -> summarize_page
  failure -> fail

collection_summary_notes_review
  success -> return_summary
  failure -> collection_summary_notes_repair

collection_summary_notes_repair
  success -> collection_summary_notes_review
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

For coverage-oriented loops, the important runtime truth is:

- each page/query-result window produces one accepted summary
- the harness concatenates those summaries as shared loop state
- `collection_summary_planner` is not a loop owner; it is an LLM node inside a harness-managed inner review/repair subloop that runs after each accepted page
- `collection_summary_notes` is not a loop owner; it is a terminal LLM node inside a harness-managed inner review/repair subloop
- `collection_summary_notes` runs only once the source is exhausted or the requested items have been considered

## Practical Near-Term Design

Do not attempt a fully generic runtime first.

Instead:

1. move real loop owners into `src/harness/loop/`
2. define one small per-loop declarative node table near each loop
3. let handlers remain ordinary Rust functions
4. wire node ports explicitly

That gets most of the clarity benefit without a big framework rewrite.

## Suggested Slices

### Slice 1

Landed shape:

- add `src/harness/loop/mod.rs`
- move loop-heavy functions out of `tools.rs` into:
  - `loop/root.rs`
  - `loop/llm_search.rs`
  - `loop/collection_summary.rs`
- keep handlers imperative

This gives us actual loop ownership boundaries without forcing a generic runtime yet.

### Slice 2

Landed shape:

- add a small `LoopDefinition` / `LoopNodeDefinition` / `LoopPort` representation
- define static node tables in:
  - `loop/root.rs`
  - `loop/llm_search.rs`
  - `loop/collection_summary.rs`
- use explicit port wiring for the current happy-path transitions

This is enough to make the control-flow graph visible in code, but it is still mostly descriptive metadata.

### Slice 3

Next step:

- add a runtime traversal helper that walks a `LoopDefinition`
- remove ad hoc per-loop `next_node(...)` lookup logic
- make terminal outcomes explicit:
  - `return`
  - `fail`
- keep node handlers as ordinary Rust functions returning typed step outcomes

The goal of this slice is to stop duplicating graph traversal logic inside each loop implementation.

### Slice 4

Next step:

- introduce `LoopRuntimeState`
- introduce per-node or per-port mutable runtime tracking
- record transition history as harness-authored state
- make room for recovery-edge budgets such as:
  - `uses_remaining`
  - `last_used_step`
  - `exhausted`

This is the slice where the declarative graph stops being static documentation and starts carrying real runtime truth.

### Slice 5

Next step:

- move `collection_summary` pagination state into harness-owned runtime structs
- add explicit runtime events or shared-state updates for:
  - pagination advance
  - coverage updates
  - recovery-route consumption
- stop relying on LLM-authored prose as the source of cursor truth

This should be the first slice that materially improves runtime correctness rather than just code organization.

### Slice 6

Next step:

- migrate `llm_search` to consume child-loop state directly
- let `llm_search` observe collection progress as harness truth
- make repair / review / continue routes explicit runtime transitions
- use runtime edge availability instead of scattered local booleans

At that point `llm_search` becomes a real orchestrator over child loop state, not just a caller that infers progress from text.

### Slice 7

Later evaluation:

- decide whether `root` should share the same traversal/runtime primitive
- or keep `root` as a thinner orchestrator with a lighter-weight graph

`root` is probably the right place to be conservative because its orchestration needs are simpler than `llm_search` and `collection_summary`.

## Immediate Next Slice Recommendation

The best next implementation slice is Slice 3.

Reason:

- Slice 1 and Slice 2 are already represented in the codebase
- `collection_summary` and `llm_search` already have static node tables
- the biggest remaining duplication is graph traversal and terminal-state handling
- adding runtime state before a shared traversal shape would likely produce more one-off loop logic

So the order should be:

1. shared traversal helper
2. runtime state and edge budgets
3. harness-owned pagination and upward state propagation

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
- Should `uses_remaining` live on ports, edges, or both?
- Should upward loop-state propagation use explicit event structs, direct shared-state mutation, or both?

## Recommendation

Yes, this direction is viable.

The most defensible next move is:

- split actual loop owners into `src/harness/loop/`
- treat `collection_search` as a step, not a loop
- make `collection_summary` the first declarative loop graph

That gives immediate codebase relief and also sets up the broader graph-based runtime model cleanly.
