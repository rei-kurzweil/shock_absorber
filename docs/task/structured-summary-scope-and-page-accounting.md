# Task: Add Structured Summary Scope Parsing And Page Accounting

## Summary

The current `summary_review` sufficiency gates are still partly heuristic.

They can reject an undersized summary window, but they do not yet rest on a fully structured contract for:

- what the user requested
- what window the leaf tool actually processed
- how much more coverage is still required

That is the wrong layer boundary.

If we want reliable handling for requests like:

- "last 50 posts"
- "summarize page 1"
- "summarize pages 1-2"
- "analyze the first 75 items"

then the harness needs explicit scope modeling and the leaf summary tool needs explicit processed-window metadata.

## Problem

Right now, the harness mostly infers scope from nearby tokens like:

- `page`
- `post` / `posts`
- `item` / `items`

That is not enough.

It leaves several gaps:

- no explicit handling for `pages`
- no explicit handling for `count`
- no structured representation of a page range
- no reliable distinction between page size and total collection size
- no explicit leaf result field saying which page was actually summarized

As a result, the harness cannot answer with confidence:

- how many more pages are required
- which next page or offset should be requested
- whether the planner summarized the requested page versus some other page

## Why This Needs To Be Split By Responsibility

This is not one job.

### Planner or harness scope parsing

Someone above the leaf tool must turn natural-language scope into a structured request.

Examples:

- `last 50 posts` -> count scope
- `page 1` -> single-page scope
- `pages 1-2` -> page-range scope

The leaf tool should not be asked to interpret vague scope text after the fact.

This scope interpretation belongs inside the internal `llm_search` workflow, not in the public root-visible `llm_search` tool definition.

The public root call should stay high level:

- `llm_search(query: "...")`

Then the harness and planner can establish an internal requested summary scope for that run.

### Leaf tool execution accounting

The `summary` tool must report the exact window it processed.

It should not just return a summary paragraph plus loose coverage URIs.

It should also say:

- which offset it used
- which page index that corresponds to
- how many items were in the processed window
- how many items exist in the full collection

### Harness sufficiency logic

The harness must compare:

- requested scope
- processed windows so far
- available collection size

That is the only layer that can reliably decide:

- sufficient vs insufficient
- additional pages needed
- exact next page or offset

## Desired Model

### Structured requested scope

Introduce an explicit harness-side scope model for summary requests.

Possible shape:

```rust
enum RequestedSummaryScope {
    CurrentWindow,
    Count { requested_items: usize },
    Page { page_index: usize },
    PageRange { start_page: usize, end_page: usize },
}
```

The exact Rust names can vary, but the harness should stop relying on ad hoc token-neighbor checks as the primary contract.

It is acceptable for the harness to begin a run with a default scope such as `CurrentWindow` or `Unknown`, then allow the internal `llm_search` planner one structured chance to refine that scope before the first `summary` leaf call.

That refinement should set requested result size or requested page scope, not "how many pages to fetch."

Example internal state transition:

- default: `CurrentWindow`
- planner refinement: `Count { requested_items: 50 }`
- frozen execution target for the rest of the run

If the planner is allowed to refine scope, that refinement should be one-shot and then frozen so the run has one authoritative requested scope.

Possible future shape:

```rust
enum RequestedSummaryScope {
    Unknown,
    CurrentWindow,
    Count { requested_items: usize },
    Page { page_index: usize },
    PageRange { start_page: usize, end_page: usize },
}
```

The exact choice between `Unknown` and `CurrentWindow` can vary, but the important constraint is that there should be one authoritative internal scope object once summary execution begins.

### Structured summary tool request

The planner-visible `summary` tool should continue to accept structured paging arguments like:

- `page`
- `offset`

Possible later addition:

- `limit`

The key point is that natural-language parsing should happen before the leaf tool runs.

### Structured summary tool result

The summary leaf should return exact processed-window metadata.

Recommended fields:

- `window_offset`
- `window_size`
- `page_index`
- `page_size`
- `collection_total_items`
- `has_more`

Plus the existing summary-oriented fields:

- `title`
- `summary`
- `covered_item_uris`
- `omitted_item_uris`

If some field names differ, the result still needs to express both:

- processed window facts
- full collection facts

### Structured sufficiency output

The review/harness layer should be able to produce exact follow-up guidance, for example:

- `additional_pages_needed: true`
- `next_page: 2`
- `next_offset: 50`
- `remaining_items_needed: 25`

That should come from structured accounting, not from prompt text guessing alone.

## Scope Parsing Expectations

Deterministic parsing is useful for obvious supported forms, but this task should not assume that plain string parsing can reliably interpret arbitrary natural-language scope.

The harness should only deterministically recognize a bounded supported sub-language, then either:

- keep a safe default scope, or
- allow the internal `llm_search` planner to refine scope once through a structured internal action

The system should not repeatedly recover scope by re-parsing loosely rewritten prompt text after each summary step.

This task should explicitly support at least:

- `last 25 posts`
- `last 50 posts`
- `first 25 posts`
- `page 0`
- `page 1`
- `pages 1-2`
- `count 50` when used in a summary-window context

It should also decide and document:

- whether page numbers are zero-based or one-based in user-facing interpretation
- whether `page 1` means the first page or the second zero-based page

That choice must be made explicit and applied consistently across planner prompting, parsing, and review.

If deterministic parsing is used, it should be treated as authoritative only for the supported patterns above.

If the planner is allowed to refine scope, that refinement must emit one structured scope choice rather than free-form prose.

## Required Leaf Metadata Clarification

The current `window_total_items` field is too ambiguous.

It can be misread as either:

- total items in the processed page
- total items in the full collection

That ambiguity should be removed.

Preferred split:

- `window_size`: items in this processed window
- `collection_total_items`: items available in the full collection

If backward compatibility is needed during migration, keep the old field temporarily but stop relying on it as the main contract.

## Harness Decision Rules

The harness should be able to answer these cases deterministically:

### Count requests

Example:

- request: `last 50 posts`
- page size: 25
- collection total: 73

Then the harness should require:

- page 0
- page 1

and know that:

- `next_page = 2` is not required

If collection total is only 32, then the harness should require enough pages to cover 32 available actor-authored items, not an impossible 50.

### Page requests

Example:

- request: `summarize page 1`

Then the harness should verify that the processed result actually corresponds to that page.

It should not accept page 0 or some arbitrary offset as sufficient.

### Page range requests

Example:

- request: `summarize pages 1-2`

Then the harness should track both required pages and avoid treating only one of them as sufficient.

## Planner Interaction

The planner can still choose to call `summary`, but it should do so against structured page arguments rather than vague natural-language scope recovery later.

That means:

- planner prompt should describe page semantics explicitly
- planner should be encouraged to emit exact `page` or `offset`
- harness review should validate that the processed result matches those arguments

The important boundary is:

- the public `llm_search` tool input stays natural-language
- requested summary scope is internal to the `llm_search` run
- the planner may get one structured chance to refine the requested scope before the first summary leaf call
- after that, the harness owns page accounting and sufficiency decisions

One reasonable shape is an internal one-shot scope-setting action such as:

- `set_summary_scope(kind="count", requested_items=50)`
- `set_summary_scope(kind="page", page_index=0)`
- `set_summary_scope(kind="page_range", start_page=0, end_page=1)`

The planner should not set "number of pages to fetch" directly.

It should set requested scope in terms of:

- requested result size
- requested single page
- requested page range

Then the harness can derive:

- how many pages are actually needed
- whether the last page is partial
- what `next_page` or `next_offset` should be

If this one-shot scope-setting step exists, it should be callable at most once per `llm_search` run and should no longer be mutable after the first `summary` call.

## Suggested Implementation Slices

### Slice 1: Introduce structured scope modeling

- add a `RequestedSummaryScope` model
- establish a default internal requested scope for each `llm_search` run
- parse count/page/page-range requests from the root query when the supported forms are obvious
- optionally allow a one-shot planner refinement step that sets requested result size or page scope before the first summary call
- decide and document page-numbering semantics

### Slice 2: Expand summary result metadata

- update `collection_summary` output contract
- distinguish processed-window size from full collection size
- include page index and/or exact offset explicitly

### Slice 3: Replace heuristic sufficiency math with structured accounting

- compute sufficiency from requested scope plus processed-window metadata
- compute exact next page/offset
- compute remaining required coverage

### Slice 4: Tighten planner and validation prompts

- update planner instructions for `summary`
- update `summary_review` prompt to expect explicit processed-window metadata
- reject outputs that omit or contradict required window facts

## Checklist

- [ ] Define a structured `RequestedSummaryScope` model.
- [ ] Decide whether the internal default scope is `CurrentWindow`, `Unknown`, or equivalent.
- [ ] Parse explicit summary scopes including count, single-page, and page-range requests.
- [ ] Decide whether `llm_search` gets a one-shot internal scope-setting action before the first `summary` call.
- [ ] If that action exists, make it set requested result size or page scope rather than number of pages.
- [ ] Freeze requested summary scope after it has been set for the run.
- [ ] Decide and document page-number interpretation for user-facing phrases like `page 1`.
- [ ] Update planner guidance so `summary` requests are emitted with exact `page` or `offset`.
- [ ] Update `collection_summary` result shape to include explicit processed-window metadata.
- [ ] Distinguish processed-window size from full collection size.
- [ ] Add explicit `collection_total_items` or equivalent field.
- [ ] Add explicit `page_index` and/or `window_offset` validation.
- [ ] Update `summary_review` and harness sufficiency logic to consume the structured fields.
- [ ] Compute exact `next_page` and/or `next_offset` for insufficient count/page-range requests.
- [ ] Prevent accepting the wrong page as sufficient for a page-specific request.
- [ ] Prevent accepting one page of a multi-page request as sufficient.
- [ ] Add tests for `pages 1-2`.
- [ ] Add tests for `count 50`.
- [ ] Add tests for insufficient but grounded multi-page summary runs.
- [ ] Add tests for exact next-page follow-up recommendations.

## Acceptance Criteria

- the harness no longer relies primarily on loose token-neighbor heuristics for summary scope
- `summary` results report the exact processed window in a non-ambiguous structured form
- the harness can deterministically compute whether more pages are required
- the harness can deterministically compute which next page or offset should be requested
- page-specific requests are validated against the actual processed page
- multi-page requests are not treated as sufficient after only one page

## Non-Goals

- redesigning the public root-visible tool surface
- changing `summary_review` into a planner-callable tool
- teaching the harness to fully interpret arbitrary natural-language scope without a bounded supported sub-language or structured planner refinement
- solving every collection-modeling bug in the same task

Those may interact with this work, but this task is specifically about structured scope parsing and explicit page/window accounting for summary sufficiency.
