# Task: Preserve Collection-Summary Loop State And Fall Back To Raw Page Windows

## Summary

The `collection_summary` loop now routes explicit post-summary requests to the correct collection, but it still fails badly when a later page summary is truncated, ungrounded, or otherwise unusable.

The latest debug run showed:

- page 1 of `recent_posts` was accepted as grounded but insufficient
- the loop advanced correctly to page 2
- page 2 produced a truncated summary response
- page 2 failed review as ungrounded
- the loop stopped
- the final tool result synthesized from only the first accepted page
- the root agent then used that partial result as if it were a valid answer for the full 400-post request

That behavior is wrong.

If page summarization fails for one window, the harness should preserve loop state at the `collection_summary` level and degrade that page to a grounded raw-window fallback, not collapse the whole run into partial synthesis from earlier pages.

## Problem

### The continuation path exists, but later-page failures collapse the run

The current trace proves the loop can continue:

- page 1: grounded, insufficient, `next_offset: 50`
- page 2: started at `offset: 50`

So the bug is not "pagination does not exist."

The bug is what happens after a later page fails review.

### Failed page summaries are dropped instead of preserved

For coverage-oriented summary pages, the current runtime can do all of the following:

- parse a page summary
- review it
- decide it is unusable
- clear `execution.result`

Once that happens, the loop loses a usable representation of that page even though the page window itself is still grounded evidence.

### The loop then synthesizes from stale prior pages

After a later page fails:

- earlier accepted page summaries remain in the accumulator
- the failing page contributes no usable page payload
- the loop can terminate and still emit a final outcome derived only from earlier accepted pages

That is misleading for broad-scope requests like:

- `summarize the most recent 400 posts by this actor`

### Missing or failed review output should not be treated as fatal collection failure by default

The current design is too brittle around page-local failures such as:

- summary model output truncated mid-sentence
- summary parse failure
- review response missing
- review response unparsable
- review marks a page unusable but without a recoverable page payload

These are page-level failures and should remain page-level unless the loop exhausts repair and fallback options.

## Desired Model

### `collection_summary` owns page failure state

The nearest ancestor loop, `collection_summary`, should retain state for:

- current offset
- accepted page summaries so far
- covered post count
- next offset
- whether the current page has already had a repair attempt
- whether the current page fell back to a raw page window
- failure reasons and diagnostics for the current page

This state should not be lost just because one page summary was unusable.

### Raw page windows are the final grounded fallback for a page

If a page summary:

- fails parsing
- fails review
- fails repair
- or the review step fails to produce a usable verdict

then the harness should fall back to the exact 25-record page window and pass that structured fallback result back into the `collection_summary` loop.

This means the loop always retains a grounded representation of the page, even if the model could not summarize it cleanly.

### Partial final synthesis must not be emitted after later-page failure

If the request still requires additional coverage and a later page failed, the loop should not emit a final synthesized summary based only on earlier accepted pages unless one of these is true:

- source exhaustion was reached
- the requested scope was fully covered
- the runtime explicitly enters a partial-coverage fallback mode and says so

For normal full-scope summary requests, later-page failure should remain visible to the parent tool result.

## Proposed Runtime Shape

### Phase 1: Summarize page

Keep the current page summary attempt.

### Phase 2: Review page

Keep the current review step, but treat unusable review outcomes as page-local loop state, not immediate collapse.

Cases:

- grounded and sufficient/insufficient
- ungrounded but repairable
- malformed/missing review
- repair failed

### Phase 3: Raw-window fallback

If page summary plus repair still does not yield a usable page summary, create a structured raw-window fallback for the current page.

Suggested content:

- `window_offset`
- `window_size`
- `collection_total_items`
- `has_more`
- `page_index`
- `page_size`
- `failure_reason`
- `raw_summary_response` if present
- all 25 page records

This fallback should be grounded by definition because it is the exact page evidence.

### Phase 4: Loop-level decision

After page summary/review/repair/fallback, the `collection_summary` loop decides:

- continue to next page
- retry/repair same page once
- stop with explicit failure
- stop with explicit partial coverage

This decision should be made from persisted loop state, not just from whether the latest page summary was accepted.

### Phase 5: Final synthesis only from coherent state

Planner and notes synthesis should only run when the loop has a coherent representation of every processed page:

- accepted summary page
- or raw-window fallback page

If the loop does not have coherent page state, it should fail explicitly instead of synthesizing a misleading partial final answer.

## Result Contract Changes

The exact Rust types can vary, but the summary-page result needs to represent more than:

- accepted summary
- dropped summary

It should express at least:

- accepted summary page
- repairable page failure
- raw-window fallback page
- fatal page failure

One possible direction:

- extend `CollectionLeafResult::Summary(...)`
- or add a sibling fallback variant that carries raw records

The important requirement is that the page remains visible and structured for the parent `collection_summary` loop.

## Failure Policy

### Page-local failures should not silently erase progress

If page 2 fails after page 1 succeeded:

- page 1 should stay preserved
- page 2 should remain visible as failed or fallback evidence
- the loop should not quietly revert to "summary of page 1 only"

### Missing review output should trigger fallback or retry

If the review step fails to return a usable verdict:

- prefer harness heuristics when available
- otherwise mark the page as review-failed and continue through retry or raw-window fallback

### Full-scope requests should not degrade into fluent partial answers without an explicit partial marker

For requests like:

- `summarize the most recent 400 posts by this actor`

the final result must not read like a complete answer if only 50 posts were successfully summarized.

## Debugging Requirements

The debug trace should make page-local fallback visible.

Suggested trace entries:

- `[summary_leaf_repair]`
- `[summary_leaf_raw_window_fallback]`
- `[collection_summary_loop_page_state]`

Useful fields:

- page offset
- page index
- page status
- repair attempted
- fallback applied
- next offset
- accumulated covered count
- failure reason

## Acceptance Criteria

- A later-page summary failure does not cause `collection_summary` to synthesize from only earlier accepted pages without an explicit partial marker.
- If page summary and repair fail, the exact 25-record page window is preserved as structured fallback state for the `collection_summary` loop.
- Missing or unparsable review output does not immediately collapse the whole collection summary run.
- The `collection_summary` loop persists page-level failure state and decides explicitly whether to continue, retry, fallback, or fail.
- Full-scope requests like `summarize the most recent 400 posts by this actor` do not return a fluent answer derived from only the first page unless the result is explicitly marked partial.
- Debug traces expose the fallback path clearly enough to diagnose which page failed and how the loop responded.

## Implementation Checklist

- [ ] Add a page-level fallback representation for raw 25-record windows after summary/review/repair failure.
- [ ] Preserve current page failure state inside the `collection_summary` loop accumulator.
- [ ] Prevent failed later pages from disappearing when earlier pages were accepted.
- [ ] Prevent final notes/planner synthesis from using only stale accepted pages after a later page failure.
- [ ] Add explicit handling for missing/unparsable review output in coverage-oriented summary pages.
- [ ] Decide whether failed pages should retry once before raw-window fallback is emitted.
- [ ] Expose raw-window fallback state in `.debug/summary_trace.md`.
- [ ] Ensure parent tool rendering distinguishes complete coverage, partial coverage, and page-failure fallback.
- [ ] Add tests where page 1 succeeds, page 2 fails summary/review, and the loop does not emit a misleading full-scope answer.
- [ ] Add tests where review output is missing and the page falls back to structured raw-window evidence.
