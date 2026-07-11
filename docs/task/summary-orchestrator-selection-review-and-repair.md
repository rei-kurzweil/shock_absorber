# Task: Add Summary-Orchestrator Selection Review And Deterministic Repair

## Summary

The public `summary` orchestrator currently does the right downstream work once a collection has been selected:

- hydrate actor-backed collections
- choose one collection
- run the `collection_summary` loop
- review page summaries
- synthesize planner and notes output

The latest failure shows that this is not enough.

For the query:

- `summarize the last 300 posts by schizanon.bsky.social`

the orchestrator selected:

- `actor_profile`

instead of:

- `recent_posts`

That means the summary loop, review, and notes synthesis all succeeded over the wrong source collection.

This task adds a new orchestrator-level validation layer between collection selection and summary-loop execution:

- review whether the chosen collection matches the user’s requested summary scope
- deterministically repair obviously wrong selections
- only fall back to ambiguous/LLM-guided selection behavior when the query genuinely leaves room for interpretation

## Problem

### The failure is above page-level summary review

The recent debug run did not fail because:

- page summaries were malformed
- coverage pagination stopped early
- planner/notes synthesis broke

It failed because the public `summary` orchestration layer routed the request to the wrong collection kind.

That makes the current review stack incomplete:

- page-level `summary_review` can validate whether a page summary is grounded and sufficient for the selected collection
- it cannot validate whether the selected collection should have been summarized at all

### The current default-to-`recent_posts` rule is not enforced strongly enough

The root prompt for the public `summary` orchestrator already says:

- default to `recent_posts` for requests like "last 50 posts" unless the query explicitly asks for replies, moderation lists, or another collection target

That rule is currently advisory.

The runtime still allowed an incompatible selection:

- explicit count request for posts
- hydrated `recent_posts` collection available
- `actor_profile` selected anyway

For this query shape, the harness should treat that selection as invalid.

### A successful summary over the wrong collection is worse than a hard failure

When wrong collection selection passes through:

- the answer can look fluent and grounded
- debug output can say `status: ok`
- downstream synthesis can reinforce the mistake

That is more misleading than a clean failure because the pipeline appears healthy while answering the wrong question.

## Desired Model

### Add an orchestrator-level selection review step

After actor hydration and before `run_collection_summary_loop(...)`, the public `summary` flow should explicitly validate:

- the normalized user scope
- the selected collection kind
- whether a more appropriate hydrated collection already exists

This review should be harness-owned, not planner-owned.

### Review should validate request shape against collection kind

Examples:

- "last 300 posts" should strongly require `recent_posts`
- "recent replies" should strongly require a replies collection
- "summarize this profile" can allow `actor_profile`
- moderation-list wording can allow `clearsky_lists`

The review step should answer:

- is this selection compatible with the request?
- if not, is there an unambiguous deterministic repair?

### Deterministic repair should handle obvious mismatches

For explicit post-count requests such as:

- `last 50 posts`
- `last 300 posts`
- `summarize recent posts`

if `recent_posts` exists in the hydrated scope, the harness should deterministically reselect it.

This should not require another LLM call.

### Ambiguous cases can still keep current flexible behavior

Some queries are genuinely underspecified:

- `summarize this actor`
- `what are they about lately`
- `summarize their activity`

In those cases, the orchestrator may still choose among profile, posts, replies, or lists depending on the existing routing model.

The new review step should be strict only when the request is explicit enough to make the collection target obvious.

## Proposed Runtime Shape

### Phase 1: Hydrate actor-backed scope

Keep the existing hydration path unchanged.

The orchestrator still gathers collections such as:

- `actor_profile`
- `recent_posts`
- `recent_replies_sent`
- `recent_replies_received`
- `clearsky_lists`

### Phase 2: Initial selection

Keep the existing selection logic as the first pass.

This preserves current behavior for ambiguous cases and minimizes disruption.

### Phase 3: Selection review

Add a new harness-owned validation function after the first-pass selection.

Suggested responsibility:

- inspect the original summary query
- inspect the selected collection kind
- inspect the hydrated collection inventory
- return either:
  - accepted selection
  - deterministically repaired selection
  - hard failure when the request is explicit but no compatible collection exists

### Phase 4: Selection repair

If the selection review finds a deterministic better match, rewrite the selected collection before invoking the summary loop.

Examples:

- selected `actor_profile` + explicit `last 300 posts` request + hydrated `recent_posts` exists
  - repair to `recent_posts`
- selected `recent_posts` + explicit `recent replies sent` request + hydrated sent replies exists
  - repair to `recent_replies_sent`

### Phase 5: Summary loop runs on the reviewed selection

Only after selection review/repair should the existing `collection_summary` loop begin.

This keeps the layers clean:

- orchestrator-level review validates source collection routing
- page-level review validates grounded summary behavior within that collection

## Selection Review Contract

The exact Rust type can vary, but the review result should be able to express:

- original selected collection id/kind
- final selected collection id/kind
- `accepted`, `repaired`, or `rejected`
- reason text suitable for debug traces
- whether the repair was deterministic

Suggested fields:

- `status`
- `reason`
- `original_collection_id`
- `original_collection_kind`
- `final_collection_id`
- `final_collection_kind`
- `deterministic_repair_applied`

This result should be traceable in `.debug/summary_trace.md`.

## Query-Scope Rules

The new review step should use harness-side rules, not just prompt text.

### Strong `recent_posts` signals

Treat these as explicit post-summary requests:

- `last N posts`
- `recent posts`
- `most recent posts`
- `summarize the last N posts by ...`

If `recent_posts` is hydrated, prefer it over:

- `actor_profile`
- replies collections
- moderation-list collections

### Strong replies signals

Treat these as explicit replies requests:

- `recent replies`
- `last N replies`
- `replies sent`
- `replies received`

Prefer the corresponding replies collection when hydrated.

### Strong profile signals

Allow `actor_profile` only when the query explicitly asks for profile-style content, for example:

- `summarize this profile`
- `what does their profile say`
- `who is this actor`

`actor_profile` should not satisfy a "last N posts" request.

### Strong lists/moderation signals

Allow `clearsky_lists` or similar structured collections only when the query explicitly asks about:

- lists
- moderation
- blocks
- follows graph / follow lists

## Failure Policy

### Reject invalid final selection when explicit routing rules are violated

If the query explicitly asks for posts and:

- `recent_posts` exists
- but the final selected collection is still not `recent_posts`

the orchestrator should fail fast rather than summarizing the wrong collection.

### Exhaustion is acceptable only after correct selection

It is acceptable for `recent_posts` to contain fewer than the requested number of items.

It is not acceptable to switch to `actor_profile` just because it is smaller or easier to summarize.

Correct behavior is:

- select the correct collection kind
- summarize what is available
- let scope sufficiency / exhaustion logic explain the gap

## Debugging And Trace Requirements

Add trace entries around selection review similar to the page-level traces already added.

Suggested trace nodes:

- `[summary_collection_selection]`
- `[summary_collection_selection_review]`
- `[summary_collection_selection_repair]`

Useful trace fields:

- query
- requested scope
- hydrated candidate collections
- original selected collection id/kind
- review status
- repair action
- final selected collection id/kind
- reason

This should make future routing failures visible before the summary loop starts.

## Acceptance Criteria

- Explicit "last N posts" queries do not route to `actor_profile` when `recent_posts` is available.
- Explicit replies queries do not route to `recent_posts` or `actor_profile` when the correct replies collection is available.
- `actor_profile` is still allowed for explicitly profile-oriented queries.
- Orchestrator-level debug output makes it clear whether the initial selection was accepted or repaired.
- Page-level `summary_review` remains unchanged in purpose and still runs after collection routing is finalized.
- Wrong-collection fluent successes are replaced by correct routing or explicit failure.

## Implementation Checklist

- [ ] Add a harness-owned selection-review function in the public `summary` orchestration path, after initial collection selection and before `run_collection_summary_loop(...)`.
- [ ] Define a small result type for orchestrator selection review/repair status.
- [ ] Normalize explicit query-scope signals for posts, replies, profile, and lists.
- [ ] Add deterministic repair rules for explicit post-summary requests to force `recent_posts` when available.
- [ ] Add deterministic repair rules for explicit replies requests to force the matching replies collection when available.
- [ ] Prevent `actor_profile` from satisfying explicit "last N posts" and "recent posts" requests.
- [ ] Fail fast when explicit routing rules are violated and no compatible hydrated collection exists.
- [ ] Add selection-review traces to `.debug/summary_trace.md`.
- [ ] Add tests for `last N posts` routing to `recent_posts`.
- [ ] Add tests for explicit replies routing to replies collections.
- [ ] Add tests proving `actor_profile` remains valid for explicitly profile-oriented queries.
- [ ] Add a regression test for the observed failure shape: hydrated `recent_posts` exists, initial selection chooses `actor_profile`, review repairs to `recent_posts`.
