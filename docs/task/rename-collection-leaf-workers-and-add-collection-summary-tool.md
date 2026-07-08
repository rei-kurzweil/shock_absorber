# Task: Rename Collection Leaf Workers And Add `collection_summary`

## Summary

The current naming around per-collection LLM work is misleading.

Today, the harness refers to the one-shot per-collection worker as a `collection search agent`, but that leaf does not:

- own a private tool inventory
- run its own tool loop
- make multi-step plans

It is a one-shot LLM-powered collection tool invoked by the harness.

We should fix the naming first, then add a second collection leaf worker for exhaustive window summaries.

## Goal

Make the per-collection leaf terminology accurately reflect runtime behavior, and introduce a second one-shot collection tool for coverage-oriented summarization.

Target model:

- `llm_search` remains the actual internal planner agent
- `collection_search` becomes a one-shot internal collection tool
- `collection_summary` becomes a one-shot internal collection tool
- graph/runtime node kinds should stop calling those leaves `...Agent`

## Why This Needs To Change

The current term `CollectionSearchAgent` conflates two different ideas:

- it is a visible leaf node in the agent graph
- it behaves like an agent

Only the first part is true.

That creates several problems:

- code and docs imply a tool loop that does not exist
- future design discussions get muddied because "agent" suggests private autonomy
- adding `collection_summary` would extend the same misleading terminology

We want cleaner conceptual boundaries:

- planner agents
- one-shot LLM-backed tools
- review steps/tools

## Current Behavioral Reality

The current collection-search leaf is:

- one prompt build
- one model call
- one parsed structured result
- one review pass owned by the harness

That is tool-like behavior, not agent-like behavior.

The fact that it appears as a node in the runtime graph does not require the code to call it an agent.

## Desired Naming Model

### Planner-visible internal tool names

These should remain lowercase tool-call names:

- `collection_search`
- `collection_summary`

These names are good because they match the internal tool protocol already used by `llm_search`.

### Rust/runtime type names

Use explicit tool terminology:

- `CollectionSearchTool`
- `CollectionSummaryTool`

If shared execution structs remain generic, that is fine, but the leaf-specific public names should stop using `Agent`.

### Graph node kinds

The graph should also use tool terminology for these leaves.

Recommended node kinds:

- `CollectionSearchTool`
- `CollectionSummaryTool`

This keeps the graph honest without removing leaf visibility.

### Review naming

This task does not require a full review rename, but it should note the same issue exists for `CollectionReviewAgent`.

Follow-up options:

- `CollectionReviewTool`
- `CollectionReviewStep`

## New `collection_summary` Tool

This task should add a second one-shot internal collection tool with a different contract from `collection_search`.

### `collection_search`

Purpose:

- selective, relevance-oriented search over one collection window
- choose the strongest supporting records
- return a grounded paragraph plus selected result URIs

This is the existing behavior and should stay intact.

### `collection_summary`

Purpose:

- coverage-oriented summary over one collection window
- explicitly account for the whole requested window
- support requests like "analyze the last 25 posts" or "summarize page 1 of this collection"

This should not silently behave like a search/ranking worker.

## Contract Difference

The key distinction should be explicit:

- `collection_search` may return a small subset of strongest records
- `collection_summary` should be judged by coverage, not just by anchor quality

That difference should be visible in:

- prompt text
- parsed result shape
- review/verifier rules
- tool inventory shown to the `llm_search` planner

## Proposed `collection_summary` Shape

Arguments:

- `collection_id`
- `prompt`
- `page`, optional
- `offset`, optional

Possible later addition:

- `limit`, optional

Result should include enough structure to verify coverage, for example:

- `title`
- `summary`
- `covered_item_uris`
- `omitted_item_uris`
- `window_start`
- `window_total_items`

The exact JSON shape can be finalized during implementation, but the verifier needs coverage-aware fields.

## Implementation Strategy

Do this in two slices rather than mixing everything together at once.

### Slice 1: Naming cleanup

- rename leaf node kinds from `...Agent` to `...Tool`
- rename related labels, logger names, and debug file naming where appropriate
- keep existing `collection_search` behavior unchanged

This reduces conceptual confusion before adding the new worker.

### Slice 2: Add `collection_summary`

- add a new internal tool definition
- add a new prompt file for the one-shot summary worker
- add parsing and rendering for its structured output
- add a review/verifier pass specialized for coverage
- let `llm_search` call it explicitly

## Checklist

- [ ] Audit all current uses of `CollectionSearchAgent` in runtime types, graph nodes, logging, prompt selection, and docs.
- [ ] Rename the runtime leaf type/kind to `CollectionSearchTool`.
- [ ] Rename any user-visible debug or graph labels that incorrectly imply autonomous agents when they refer to one-shot collection workers.
- [ ] Decide whether `CollectionReviewAgent` is in scope now or explicitly deferred.
- [ ] Add a new prompt file for `collection_summary`.
- [ ] Add `CollectionSummaryTool` runtime/node naming.
- [ ] Add `collection_summary` to the internal `llm_search` tool inventory.
- [ ] Define the parsed structured result shape for `collection_summary`.
- [ ] Implement a coverage-aware verifier for `collection_summary`.
- [ ] Ensure `collection_search` keeps its current selective-search behavior.
- [ ] Ensure `collection_summary` uses existing paging primitives like `page` and `offset`.
- [ ] Update debug output and agent graph rendering so `collection_search` and `collection_summary` leaves are distinguishable.
- [ ] Add tests for parsing and verifying `collection_summary` output.
- [ ] Add tests that preserve current `collection_search` behavior.

## Acceptance Criteria

- the runtime no longer uses `CollectionSearchAgent` for the one-shot collection leaf
- `llm_search` still exposes `collection_search` as an internal tool
- `llm_search` additionally exposes `collection_summary` as an internal tool
- `collection_search` remains relevance-oriented
- `collection_summary` is clearly coverage-oriented
- graph/debug output distinguishes planner agents from one-shot collection tools

## Non-Goals

- fixing actor-authored post collection modeling in this task
- redesigning the public root-visible tool surface
- adding multi-tool autonomy to collection leaves

Those are related topics, but this task is specifically about naming clarity and the new summary tool.
