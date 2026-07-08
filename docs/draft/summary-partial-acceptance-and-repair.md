# Draft: Partial Acceptance And Repair For Coverage-Oriented `summary`

## Summary

The current `summary` leaf is coverage-oriented, but its parsing boundary is too brittle.

Today a response is effectively treated as either:

- a fully valid structured summary result
- or a total failure

That is the wrong split.

We already know a common failure mode is:

- `SUMMARY_RESULT_START` is present
- `summary:` is present
- one or more `covered_item_uri:` lines are present
- some page/window metadata is present
- but `SUMMARY_RESULT_END` is missing

That should not be treated the same as unrelated prose or a structurally empty response.

## Goal

Keep the coverage-oriented `summary` contract:

- summarize everything within the paginated range the harness is willing to traverse
- preserve page/window identity
- preserve enough accounting to support summary sufficiency checks

But stop requiring perfect first-pass formatting before the harness can continue.

## Proposed Parsing States

Replace the current all-or-nothing boundary with three parser outcomes:

### 1. `complete`

Use this when:

- required structure is present
- required fields are present
- scalar fields parse cleanly
- the result can be accepted immediately as typed `LlmSummaryResult`

Next action:

- continue to `summary_review`

### 2. `repairable_partial`

Use this when:

- the response is clearly attempting the expected summary schema
- the response contains the minimum fields needed to identify the page/window and grounded summary attempt
- strict parsing still fails because of missing markers, one missing scalar, or other recoverable structural defects

Next action:

- run one harness-owned structured-output repair step

### 3. `invalid`

Use this when:

- the response does not clearly attempt the expected schema
- or the response lacks the minimum fields needed to trust it as a real coverage-summary attempt

Next action:

- fail the worker result

## Minimum Repair Eligibility

For a coverage-oriented `summary`, the harness should consider a malformed response repairable when it contains at least:

- `SUMMARY_RESULT_START` or otherwise unmistakable tagged summary fields
- `summary:`
- at least one `covered_item_uri:`
- `window_offset` or `page_index`
- `window_size`
- `collection_total_items` or `has_more`

Important:

- `SUMMARY_RESULT_END` should not be required for repair eligibility
- `omitted_item_uri` should not be required for repair eligibility

The reason is simple:

- a missing end marker is the known real-world failure
- requiring full omitted-item accounting up front increases output burden without helping decide whether repair is worthwhile

## Strict Success vs Repair Eligibility

The minimum repairable set is not the same as the strict success contract.

That means:

- a response may be good enough to repair
- without being good enough to accept immediately

This is intentional.

The harness should distinguish:

- "good enough to keep"
- from
- "good enough to trust without repair"

## Why `omitted_item_uri` Should Not Gate Continuation

`omitted_item_uri` is expensive.

It increases:

- token count
- repetition
- failure surface
- chances that Gemma truncates or drifts before finishing the block

If we keep coverage semantics, the stronger signals are:

- grounded summary paragraph
- correct page/window metadata
- explicit covered URIs
- review and sufficiency checks over the typed result

That is a better place to enforce coverage than making the model enumerate every omitted item before the harness is allowed to continue.

This does not necessarily mean `omitted_item_uri` must be deleted immediately.

It means:

- it should not be part of the minimum field set for repair eligibility
- and it may not need to remain a strict required field long-term

## Relationship To Repair Steps

This draft assumes two separate harness-owned repair concepts:

### Tool-call repair

Used when the planner emits an invalid internal tool call.

Example:

- wrong `collection_id`
- malformed paging args
- semantically close but unexecutable `summary(...)`

### Structured-output repair

Used when the `summary` worker emits a near-valid structured result that fails strict parsing.

Example:

- missing `SUMMARY_RESULT_END`
- one malformed scalar
- one missing required scalar with the rest of the block intact

These should remain separate.

## Recommended First Implementation Slice

1. Add parser outcomes:
   - `complete`
   - `repairable_partial`
   - `invalid`
2. Define the minimum repairable field set described above.
3. Route `repairable_partial` summary outputs into one structured-output repair step before `summary_review`.
4. Leave the current strict schema in place for immediate acceptance until the partial path is working.

This is the smallest change that fixes the known failure mode without changing the higher-level summary contract.

## Non-Goal

This draft does not change `summary` into a cherry-pick or best-items-only tool.

The contract remains:

- summarize everything within the requested paginated range

The change here is only:

- how tolerant the harness should be before deciding whether to continue, repair, or fail
