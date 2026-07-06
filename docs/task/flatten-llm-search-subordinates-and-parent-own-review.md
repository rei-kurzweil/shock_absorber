# Task: Flatten `llm_search` Subordinates And Make Collection Review Explicitly Parent-Owned

## Summary

The current `llm_search` subtree runs collection review in the right execution order, but the visible/typed agent graph nests each `collection review` node under its corresponding `collection search` node.

That likely mismatches the intended orchestration model.

The parent `llm_search` tool agent should be the obvious coordinator of all subordinate work for that run. That suggests a flat child list where both:

- collection searches
- collection reviews

are immediate children of the parent `llm_search` agent.

## Problem

Today the visible structure implies:

- parent `llm_search`
  - collection search A
    - collection review A
  - collection search B
    - collection review B

But the desired orchestration model is closer to:

- parent `llm_search`
  - collection search A
  - collection review A
  - collection search B
  - collection review B

or another parent-owned flat ordering with explicit coordination metadata.

## Why Flattening Matters

### 1. Ownership clarity

The parent `llm_search` agent is the true decision-maker for:

- which collections to search
- whether to review a result
- whether to attempt one repair pass
- whether a reviewed result is admitted into synthesis

The tree should reflect that ownership directly.

### 2. Better runtime control

Flat parent-owned children make it easier to reason about:

- cancellation
- scheduling
- transcript ordering
- parent-level retry or replay policies
- future batch/parallel execution

### 3. Better transcript semantics

If all subordinate work is parent-owned, the transcript can more naturally show:

1. parent launches search A
2. parent reviews search A
3. parent launches search B
4. parent reviews search B
5. parent synthesizes from the reviewed set

instead of making review look like a hidden internal behavior of the collection-search child.

## Non-Goals

- changing the public `llm_search(query)` interface
- removing blocking review semantics
- changing the one-repair-pass policy by itself

## Proposed Changes

### Agent graph

Change the subtree shape so:

- `collection search` nodes remain immediate children of the parent `llm_search` tool agent
- `collection review` nodes also become immediate children of the parent `llm_search` tool agent
- explicit metadata links each review node to the reviewed collection-search result

Possible metadata:

- `review_of_collection_id`
- `review_of_agent_id`
- `review_sequence_index`

### Execution state

Keep per-collection search/review data linked in execution state, but do not require the graph hierarchy to mirror that storage shape.

In other words:

- storage can remain per-collection
- graph ownership should become parent-oriented

### Transcript/debug output

Add explicit parent-level event ordering so it is clear that the parent agent is coordinating:

- search start/completion
- review start/pass/fail
- repair attempt
- final inclusion/exclusion from synthesis

## Acceptance Criteria

- `llm_search` immediate children are a flat list of subordinate work units
- `collection review` is no longer nested under `collection search`
- parent synthesis still consumes only reviewed/finalized collection results
- transcript/debug output makes the parent’s coordination role clearer than the current nested structure

## Test Plan

- one `llm_search` over multiple collections:
  - agent graph shows flat children under the parent `llm_search` node
- one review pass and one repair pass:
  - transcript/debug output shows parent-owned ordering clearly
- one hard review failure:
  - failed reviewed collection is still excluded from synthesis
  - graph ownership remains flat

## Open Questions

- Should search and review nodes be interleaved in actual execution order, or grouped by kind in the rendered tree?
- Do we want an explicit parent-owned synthetic node for “collection result finalization” in addition to search/review nodes?
- Is flat ownership sufficient, or should the graph support parent-owned sibling nodes plus cross-links for “reviews result of X”?
