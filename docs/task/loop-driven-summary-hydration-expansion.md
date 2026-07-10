# Task: Add Loop-Driven Hydration Expansion For Actor Summary Coverage

## Summary

The current actor-summary path hydrates a fixed recent-post snapshot up front, then pages only within that cached collection.

That works for requests comfortably inside the initial fetch budget, but it breaks down conceptually for larger coverage requests:

- `summarize the last 400 posts`
- `summarize the last 800 posts`
- `summarize pages 1-10`

Right now the only remedy is to raise the initial hydration cap again.

That is the wrong abstraction.

The summary loop itself should be able to request more actor-backed source material when it reaches the end of the current cache window and still needs more grounded coverage.

## Problem

### Hydration is front-loaded and static

The public `summary` flow currently does this:

1. resolve actor
2. choose hydration args from requested scope
3. hydrate actor scope once
4. select a collection
5. run `collection_summary` over the cached collection

The important limitation is step 3.

Once hydration finishes, `collection_summary` only sees `collection.posts.len()` from the cached collection and cannot expand it.

### Cached collection size is acting like source size

For recent posts, the collection length is currently used as the available summary scope for:

- summary sufficiency
- `has_more`
- `collection_total_items`
- coverage accounting

That is only valid when the cache already contains the full reachable source scope.

When the cache is just a prefix of the actor feed, these concepts are being conflated:

- cached items
- currently hydrated items
- actually exhausted source items

That makes partial runs look like hard source limits.

### Raising static caps does not solve the real problem

Increasing hydration from `200` to `400` is useful, but it is still only a larger fixed snapshot.

Eventually the same failure mode reappears for a bigger request.

The runtime needs an on-demand expansion path instead of repeatedly increasing a constant.

## Desired Behavior

If a summary run reaches the end of the currently cached `recent_posts` collection and still needs more grounded coverage, the harness should be able to:

1. detect that the current cache window is exhausted
2. request more actor feed hydration
3. rebuild or refresh the affected collection
4. continue the summary loop from the prior offset

This should happen only when needed and only for collection types where expansion makes sense, especially:

- `recent_posts`
- possibly `recent_posts_unaddressed`

It should not be assumed for fixed-size collections such as:

- `pinned_posts`
- `actor_profile`
- fixed moderation-list snapshots

## Required Model Changes

### 1. Distinguish cached coverage from source exhaustion

The runtime needs separate concepts for:

- `cached_total_items`
- `source_exhausted`
- `required_total_items`

Today `collection_total_items` often behaves like both "items currently cached" and "items known to exist in source scope."

That is not sufficient for expandable actor feeds.

At minimum, the summary machinery should stop interpreting "current cached count" as "definitive source ceiling" unless hydration explicitly reports exhaustion.

### 2. Preserve enough hydration provenance to resume

Loop-driven expansion needs metadata describing how the collection was hydrated.

Examples:

- actor DID
- collection kind
- fetch budget already attempted
- whether the underlying source already returned no further cursor

Without that, `collection_summary` has no safe way to trigger incremental expansion.

### 3. Add an expansion hook from `collection_summary`

The collection-summary loop should be able to detect a state like:

- requested scope still insufficient
- current offset has reached cached collection end
- source is not yet known exhausted

When that happens, the loop should hand control back to a harness-side expansion step instead of terminating or synthesizing a misleading partial final summary.

Possible shape:

```text
summarize_page
  -> review_page
  -> planner
  -> expand_collection_if_needed
  -> advance_cursor
```

The exact graph can differ, but expansion should be a harness-owned transition, not an LLM decision.

### 4. Re-resolve the collection after expansion

After additional hydration, the loop should not continue using a stale in-memory collection snapshot.

It should reload the collection from the store so:

- `collection.posts.len()` reflects newly hydrated items
- later `paged_search_collection(...)` windows use the expanded backing collection
- coverage accounting remains aligned with the actual store state

## Non-Goals

This task should not:

- make every collection type dynamically expandable
- require the LLM planner to decide fetch budgets mid-loop
- silently fabricate unavailable source size
- convert summary paging into arbitrary unbounded fetching without explicit guards

## Constraints

### Expansion should be bounded

The loop must not fetch forever.

It should keep explicit limits such as:

- maximum expansion rounds per summary run
- maximum total hydrated posts for one run
- termination once source exhaustion is confirmed

### Expansion should be actor-collection specific

Only collections backed by refreshable actor feed hydration should use this path.

Fixed collections should keep their current terminal behavior.

### Review semantics must stay grounded

If expansion cannot obtain more posts, the final summary may still be partial or source-exhausted.

But that distinction must be real and explicit:

- partial because runtime stopped early
- partial because expansion guard fired
- complete because source was exhausted
- complete because requested coverage was reached

## Proposed Implementation Direction

### Phase 1: Represent hydration state explicitly

Extend stored collection metadata or add a separate typed structure capturing:

- actor DID
- collection kind
- hydrated item count
- fetch budget used
- source exhaustion status

This should be harness-owned state, not inferred back out of debug text.

### Phase 2: Add incremental hydration API shape

Introduce a harness-side path that can say, in effect:

- hydrate this actor's recent posts to at least `N` cached items

rather than only:

- hydrate recent posts with one fixed initial limit chosen before the loop starts

The existing `ensure_recent_posts_cached(...)` logic is a natural starting point, but it needs to support repeated growth instead of single-shot assumptions.

### Phase 3: Teach `collection_summary` when to expand

When coverage is insufficient, the loop should inspect:

- current covered offset
- current cached total
- requested scope
- source exhaustion flag

If the loop is blocked only by cache size and expansion is still possible, it should expand and continue rather than finalize.

### Phase 4: Fix final accounting semantics

Final summary results should clearly distinguish:

- requested coverage target
- grounded covered post count
- cached total at finish
- whether source was truly exhausted

The current final `required_total_items` reporting should not continue to reflect an impossible original request when the available source ceiling is smaller and known.

## Acceptance Criteria

- A request larger than the initial hydration budget can trigger additional hydration without restarting the whole summary run.
- `collection_summary` continues paging after expansion rather than finalizing at the old cache boundary.
- Final review output distinguishes cache boundary from actual source exhaustion.
- Coverage accounting remains contiguous across pre-expansion and post-expansion windows.
- Requests that still exceed the true available source end with explicit source exhaustion, not a misleading early partial summary.

## Suggested Tests

- summary request for `400` recent posts where initial cache has `200`, expansion reaches `400`, and the loop completes
- summary request for `800` posts where source exhausts at fewer than `800`, and final review reports true exhaustion
- summary request over a fixed collection like `pinned_posts` does not attempt expansion
- expansion preserves contiguous `next_offset` accounting across cache growth
- final result distinguishes `cached_total_items` from `source_exhausted`
