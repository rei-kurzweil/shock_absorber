# Task: Refactor Internal `llm_search` Planner Loop Around Explicit Tool And Review Phases

## Summary

The current internal `llm_search` runtime has most of the right pieces:

- a separate internal planner loop
- internal tools for actor resolution, hydration, collection search, and global post search
- a separate collection-review LLM call with its own context window

But those pieces are too tightly coupled inside one harness loop, and the message flow is not clean enough.

In particular:

- planner protocol failures can contaminate later planner rounds
- trailing planner prose can invent fake collection IDs and hypothetical results
- tool execution and review execution are not modeled as explicit phases
- the planner consumes rendered transcript text instead of clean structured step results

This task is to refactor internal `llm_search` so the runtime more clearly matches the intended model:

1. root agent calls public `llm_search(query)`
2. internal `llm_search` planner gets a narrow planner-only context
3. planner emits exactly one internal tool call or a final synthesis
4. harness executes that tool in its own worker context
5. if the tool is `collection_search`, the harness runs search and then review as explicit child phases
6. the reviewed result is returned to the planner as structured tool output
7. the planner decides the next tool or final synthesis until the round limit is reached

## Problem

### Planner loop ownership exists, but phase boundaries are blurry

Today the internal `llm_search` planner already owns a bounded internal tool loop.

However, the main runtime path in `src/harness/tools.rs` mixes together:

- planner generation
- strict tool-call parsing
- tool argument validation
- tool execution
- collection review
- retry handling
- transcript/debug rendering

That makes the control flow harder to reason about and easier to poison with malformed model output.

### Invalid or overlong planner output can poison later rounds

A planner response can currently:

- emit a valid tool call
- continue with trailing prose
- invent hypothetical tool results
- invent fake collection IDs such as `coll:def456ghi`

Then the retry path can append that bad assistant output back into the planner conversation.

That means later rounds may anchor on fabricated planner text instead of real tool results.

### Review already exists, but is not modeled as an explicit returned phase result

Today `collection_search` and `collection_review` already use separate contexts.

That part is good.

But review is still wired as an implementation detail of the search execution path rather than an explicit harness phase with a clearly shaped result returned to the planner.

The planner should conceptually receive:

- the collection-search result
- the review verdict
- any repair note or fallback diagnostic
- a clear usable/failed status

instead of reconstructing meaning from rendered transcript text.

## Desired Runtime Model

### Phase 1: Planner round

The planner receives only:

- the original user query
- accepted prior tool results
- concise diagnostics from prior rounds
- round-limit guidance

It does not receive:

- raw invalid planner output
- discarded speculative prose
- full collection payloads from prior worker contexts

The planner must emit either:

- exactly one internal `TOOL_CALL`
- or a final grounded synthesis

### Phase 2: Tool execution

Once a tool call is accepted, the harness executes it outside the planner context.

Tool execution should return a typed result object first, with transcript text rendered separately.

Examples:

- `resolve_actor_refs`
  - resolved handle/DID pairs
- `hydrate_actor_scope`
  - typed collection references with real collection IDs and labels
- `collection_search`
  - typed search result plus review result
- `search_global_posts`
  - typed hydrated global-search collection reference

### Phase 3: Review for collection search

If the accepted tool is `collection_search`, the harness should explicitly run:

1. collection-search worker
2. collection-review step
3. optional repair/fallback path
4. final reviewed result packaging

The planner does not need to explicitly call review.

But the planner should receive a single structured tool result whose semantics make the review outcome obvious.

### Phase 4: Planner resumes

The planner gets back:

- accepted tool name
- validated tool args
- typed tool result
- concise rendered summary for transcript/debug

It then chooses:

- another tool
- or a final synthesis

This continues until:

- final synthesis is produced
- round limit is reached
- or fallback collection resolution takes over

## Proposed Refactor

### 1. Extract the internal `llm_search` loop into explicit functions

Split the current monolithic loop into smaller units such as:

- `run_llm_search_planner_round(...)`
- `parse_or_finalize_planner_response(...)`
- `validate_internal_tool_call(...)`
- `execute_internal_llm_search_tool_call(...)`
- `run_reviewed_collection_search(...)`
- `append_planner_visible_tool_result(...)`
- `finalize_llm_search_from_outcomes(...)`

The goal is to make the runtime phases and state transitions obvious in code.

### 2. Accept the first valid tool block and discard trailing prose

If the planner emits one valid leading internal `TOOL_CALL`, the harness should:

- accept it
- discard any trailing prose
- log that trailing content was discarded
- avoid asking the planner to regenerate just because it kept talking

If multiple tool blocks or ambiguous content appear, the response can still be rejected.

But one valid first tool call should not be lost.

### 3. Never feed discarded or invalid planner prose back into planner history

The planner message history should contain only:

- accepted planner tool calls
- accepted final syntheses
- harness-generated validation feedback
- real tool results

It should not contain:

- discarded prose after an accepted tool block
- invented hypothetical tool outputs
- invalid assistant messages that failed structural validation

This is necessary to prevent self-poisoning across rounds.

### 4. Introduce typed internal tool-result structs

Add typed result shapes for internal planner-visible tool outcomes.

Suggested categories:

- `ResolvedActorRefsResult`
- `HydratedActorScopeResult`
- `CollectionSearchToolResult`
- `GlobalSearchCollectionResult`

For collection search specifically, the typed result should include:

- `collection_id`
- `collection_label`
- `search_status`
- `review_status`
- `review_reason`
- `repair_diagnostic`
- selected result URIs
- summary text actually admitted for synthesis
- whether fallback/reduced retry was used

Transcript rendering should be derived from these structs, not used as the planner state source.

### 5. Make collection review a first-class harness phase

Keep the planner-facing tool surface simple:

- the planner still calls `collection_search`

But internally, model review as a named harness phase with explicit ownership and outputs.

That means:

- separate review result struct
- explicit review event ordering
- clear handoff from search worker to review step
- clear handoff from reviewed result back to planner

### 6. Narrow the planner context window

Planner context should be intentionally compact.

It should include:

- original query
- accepted internal tool results in concise structured form
- high-signal diagnostics

It should exclude:

- raw collection documents
- full review evidence payloads
- malformed planner raw output except possibly in debug logs

Collection payloads belong in worker/review contexts only.

### 7. Flatten visible subordinate ownership

The parent internal `llm_search` planner should remain the obvious loop owner.

Visible subordinate work should be rendered as parent-owned phases or siblings, not as if review owns an autonomous loop.

This task should align with:

- `flatten-llm-search-subordinates-and-parent-own-review`
- `reframe-review-steps-vs-agent-loops`

### 8. Improve failure typing and diagnostics

Split current broad failure cases into clearer buckets:

- planner protocol invalid
- planner tool-argument invalid
- unknown collection id
- accepted tool call with trailing prose discarded
- collection search parse failure
- collection review failure
- fallback reduced-search retry used
- provider call failure with no usable retry

Those failure types should be available both for transcript/debug rendering and planner-visible diagnostics where appropriate.

## Non-Goals

- redesigning the public root-visible `llm_search(query)` tool API
- changing the meaning of review pass/fail by itself
- rewriting all internal tooling around provider-native tool calling in one step
- parallelizing internal `llm_search` execution in this task

## Likely Code Touchpoints

- `src/harness/tools.rs`
  - internal `llm_search` loop
  - planner response validation
  - internal tool execution
  - collection review orchestration
  - transcript/debug rendering
- `src/harness/prompts/agents/llm_search.md`
  - planner prompt requirements
- `src/harness/llm_api.rs`
  - only if optional early-stop or streaming support is added later

## Recommended Implementation Order

1. accept one valid internal tool call and discard trailing prose
2. stop replaying invalid or discarded planner assistant content into message history
3. extract explicit planner/tool/review helper functions
4. introduce typed internal tool-result structs
5. return reviewed collection-search results to the planner from structured state rather than transcript strings
6. flatten visible ownership and event rendering
7. tighten diagnostics and planner-context composition

## Acceptance Criteria

- internal `llm_search` still behaves as a bounded planner loop under the public `llm_search(query)` tool
- planner-visible state is derived from accepted tool results, not malformed prior assistant output
- one valid leading internal `TOOL_CALL` is accepted even if trailing prose exists
- discarded planner prose is visible as a diagnostic but does not poison later rounds
- `collection_search` returns a planner-visible reviewed result with explicit review status
- planner, collection-search worker, and collection-review step each retain separate context windows
- transcript/debug output makes the phase ordering and ownership clear

## Test Plan

- planner emits one valid `TOOL_CALL` followed by self-correction prose
  - first tool call is accepted
  - trailing prose is discarded
  - later rounds do not reference fabricated text from the discarded prose
- planner emits fake hypothetical collection IDs after a valid tool call
  - fake IDs do not enter planner-visible state
  - later rounds use real collection IDs from tool results
- planner emits invalid tool args
  - harness returns validation feedback
  - planner history does not preserve bad speculative prose as accepted state
- `collection_search` succeeds and review passes
  - planner receives reviewed usable result
- `collection_search` succeeds but review fails
  - planner receives explicit failed reviewed result
  - synthesis excludes that collection
- primary full-collection search fails but reduced retry succeeds
  - planner-visible result records retry diagnostic without losing the usable reviewed result

## Relationship To Existing Task Docs

This task overlaps with, but is broader than:

- `stop-internal-tool-call-generation-early`
- `flatten-llm-search-subordinates-and-parent-own-review`
- `reframe-review-steps-vs-agent-loops`

Those tasks cover important slices of the problem.

This task is the runtime refactor that ties them together around:

- planner-loop ownership
- tool/result state boundaries
- review handoff semantics
- planner-context hygiene

## Open Questions

- Should accepted tool calls be persisted in planner history as the raw accepted tool block, a normalized tool-call record, or both?
- Should collection review remain planner-implicit permanently, or should future generic verification primitives be planner-callable?
- Do we want a single generic internal-tool result envelope type, or strongly typed per-tool variants only?
- Should parent-owned search/review phases be rendered strictly in execution order, or grouped by kind in some views?
