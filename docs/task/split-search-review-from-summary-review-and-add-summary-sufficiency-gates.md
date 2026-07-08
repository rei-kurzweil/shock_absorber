# Task: Split `search_review` From `summary_review` And Add Summary Sufficiency Gates

## Summary

The current harness review step is trying to cover two different contracts:

- selective evidence review for `search`
- coverage and sufficiency review for `summary`

Those are not the same job.

Right now, `collection_review` has already started to branch internally depending on whether the preceding leaf was selective or coverage-oriented. That is a sign the abstraction is wrong.

We should split the review path into:

- `search_review`
- `summary_review`

Both should remain harness-owned post-processing steps rather than planner-callable tools.

## Problem

The `summary` leaf now exists specifically for whole-window coverage requests like:

- summarize the last 25 posts
- analyze the last 50 posts
- summarize page 1 of this collection

But the current shared review step still treats success mostly as:

- grounded paragraph exists
- claimed coverage metadata is internally consistent

That is not enough.

A run can currently pass review even when:

- the source collection is stale
- the source collection is much smaller than the requested scope
- the summary covers only one small page when the user asked for multiple pages
- the collection semantics are wrong for the request

Example failure mode:

- user asks for the last 50 posts
- harness supplies only a 7-item actor-backed collection window
- `summary` returns a grounded paragraph over those 7 items
- current review passes because the 7-item accounting is internally consistent

That is grounded, but not sufficient.

## Why The Split Matters

### `search_review`

This step should answer:

- is the selective evidence summary grounded?
- are the chosen records real?
- is the paragraph useful enough for parent synthesis?

`search_review` does not need to prove exhaustive coverage.

### `summary_review`

This step should answer:

- does the summary actually account for the whole requested window?
- is the requested window itself large enough for the user's stated scope?
- should the harness require more pages before parent synthesis is allowed?

This is a stronger contract than `search_review`.

## Desired Model

### Leaf tools

- `search`
- `summary`

### Harness review steps

- `search_review`
- `summary_review`

These should be visible in debug/runtime output as review steps, not planner agents or planner-callable tools.

## Behavior Goals

### After `search`

Run `search_review`.

It should keep the current selective-evidence behavior:

- grounded one-paragraph summary
- real cited records
- repair allowed when the summary can be rewritten deterministically from selected evidence

### After `summary`

Run `summary_review`.

It should be stricter:

- require grounded whole-window summary behavior
- require complete and valid per-window accounting
- reject windows that do not satisfy the requested scope
- decide whether additional `summary` pages are required before parent synthesis

## New Sufficiency Concept

This task should introduce a harness-side distinction between:

- grounded
- sufficient

A `summary` result may be grounded but still insufficient for the parent task.

Examples:

- a 7-item summary for a "last 50 posts" request is grounded but insufficient
- page 0 summary for a "summarize page 0" request may be sufficient
- page 0 summary for a "last 50 posts" request is not sufficient by itself

## Proposed `summary_review` Outputs

The exact shape can vary, but it should express at least:

- `status: pass` or `status: fail`
- `grounded: true/false`
- `sufficient: true/false`
- `reason: ...`
- optional `additional_pages_needed: true/false`
- optional `next_page` or `next_offset`
- optional `required_total_items`

The key point is that `summary_review` must be able to say:

- this page summary is grounded
- but parent synthesis should not stop here

## Harness Responsibilities

The harness should own the logic that decides whether the requested scope has actually been satisfied.

That means:

- parse scope hints from the original query when possible
- compare requested size against available actor-backed collection size
- compare requested size against completed summary pages
- refuse to treat one too-small page as sufficient for multi-page requests

This should not be delegated entirely to the model.

## Planner Interaction

The planner can still decide when to call `summary`, but the harness should decide whether the resulting work is sufficient to stop.

For explicit count/window requests, the harness should be able to require follow-up `summary` pages or fail the result as insufficient.

Examples:

- "last 50 posts" should require two 25-item `summary` windows unless the collection truly has fewer than 50 actor-authored items available
- "summarize page 1" should require only that one page
- "analyze the last 25 posts" should require exactly one sufficient page

## Suggested Implementation Slices

### Slice 1: Rename the existing review step

- rename generic `collection_review` runtime/debug/prompt naming to `search_review`
- keep current `search` repair behavior unchanged

### Slice 2: Add `summary_review`

- new prompt file
- new runtime/debug node naming
- new verdict/result struct if needed
- separate review function for `summary`

### Slice 3: Add sufficiency gates

- detect requested post/window count from the parent query
- track which windows have been summarized
- decide whether the completed `summary` work is sufficient
- fail or continue when the request still lacks enough coverage

## Checklist

- [ ] Rename the current shared review step to `search_review`.
- [ ] Update debug file naming, graph labels, and runtime node kinds to stop calling the generic review path `collection_review`.
- [ ] Add a new `summary_review` prompt.
- [ ] Add a separate `summary_review` runtime path after `summary`.
- [ ] Define a verdict/result shape that can express grounded-but-insufficient summaries.
- [ ] Parse explicit scope signals such as "last 25 posts", "last 50 posts", and "page 1".
- [ ] Add harness-side sufficiency checks for explicit window/count requests.
- [ ] Prevent a single too-small collection window from being treated as sufficient for a larger request.
- [ ] Ensure `search_review` retains current deterministic repair behavior where appropriate.
- [ ] Decide whether `summary_review` should allow any repair, or only fail/continue.
- [ ] Add tests for grounded-but-insufficient `summary` cases.
- [ ] Add tests for multi-page summary sufficiency.

## Acceptance Criteria

- `search` runs through `search_review`
- `summary` runs through `summary_review`
- runtime/debug output distinguishes the two review steps
- a grounded but undersized `summary` result is not treated as sufficient for explicit larger requests
- explicit requests like "last 50 posts" require enough summarized coverage before parent synthesis is accepted
- selective `search` behavior remains unchanged

## Non-Goals

- redesigning the public root-visible tool surface
- changing `summary` into a planner-callable review tool
- fixing every actor-backed collection modeling bug in this same task

Those are related, but this task is specifically about splitting review semantics and adding sufficiency enforcement for coverage-oriented summaries.
