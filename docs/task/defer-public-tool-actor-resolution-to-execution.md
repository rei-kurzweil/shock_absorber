# Task: Defer Public-Tool Actor Resolution To Execution

## Summary

The latest `summary` prep failure exposed an unnecessary asymmetry between the public `search` and `summary` tools.

For the query:

- `summarize 150 posts by segyges.bsky.social into 3 paragraphs`

tool preparation failed before execution with:

- `missing_prerequisites: actor_anchor`

The handle-shaped actor ref was present in the query, but the runtime only accepted it if that handle was already cached in `NotificationStore`.

That is too strict for public tool prep.

The harness should treat explicit actor refs in public tool queries as enough to proceed into execution, where the runtime can resolve and hydrate them against Bluesky if needed.

This task changes actor-resolution timing for public tools. It does not broadly move collection selection out of execution, because collection selection already belongs there for most cases.

## Problem

### `summary` currently requires a fully resolved actor anchor during prep

The public `summary` prep path currently:

1. reads the query
2. resolves summary scope and collection hint
3. calls `resolve_prepared_actor_anchor(...)`
4. fails if that function cannot produce a concrete DID

That means an explicit handle in the query is not enough on its own.

If the handle is uncached and no actor is selected in the UI, prep aborts before `summary` execution can hydrate anything.

### `search` already behaves more flexibly

The public `search` path can start with:

- a resolved prepared actor anchor when available
- or only the raw detected actor refs from the query

During execution it can call the actor-resolution path that uses `ensure_actor_profile_cached(...)` and resolve uncached handles through the networked Bluesky API.

So today:

- `search` tolerates uncached explicit handles
- `summary` does not

That mismatch is the bug.

### The current failure mode is worse than necessary

The runtime already has a safe execution-time resolver for actor refs.

Because `summary` fails earlier, the user sees a prep failure for a query the runtime is otherwise capable of satisfying.

This is not a grounding safeguard. It is a sequencing problem.

## Desired Behavior

### Public tool prep should accept unresolved explicit actor refs

If a public-tool query contains an explicit actor ref such as:

- `segyges.bsky.social`
- `@segyges.bsky.social`
- `did:plc:...`

prep should preserve that actor ref even if the store does not yet know its DID.

Prep should only fail for missing actor input when both are true:

- no explicit actor ref was detected in the query
- no selected actor fallback is available from the UI

### Execution should own actor resolution

At the start of public tool execution, the runtime should turn the prepared actor input into a concrete actor profile and DID by:

- using selected-actor fallback immediately when already concrete
- resolving explicit uncached refs through `ensure_actor_profile_cached(...)`
- caching the result before downstream hydration and collection work

This should apply consistently to public `search` and public `summary`.

### Collection selection should stay execution-time

This task should not relocate collection selection into tool prep.

That is already the wrong abstraction for:

- `search`, which is planner-driven and may touch multiple collections
- `summary`, which already selects a collection after actor hydration and scope normalization

The problem is not "collection resolution happens in the wrong phase."

The problem is "actor resolution is required too early for `summary`."

## Proposed Model

### 1. Represent prepared actor input separately from resolved actor identity

The public-tool prep layer should distinguish between:

- an explicit actor ref detected from the query
- a selected actor fallback from the UI
- a fully resolved actor DID and handle

The exact Rust type can vary, but prep should no longer require every `PreparedSummaryInput` actor field to already be a resolved DID-bearing anchor.

### 2. Add a shared execution-time actor resolver for public tools

Both public `search` and public `summary` should resolve prepared actor input through one harness-owned execution helper.

That helper should:

- accept the prepared actor input plus store and agent access
- return a concrete DID/handle pair
- use cache when available
- fall back to `ensure_actor_profile_cached(...)` for uncached explicit refs

This keeps actor resolution rules consistent across both public tools.

### 3. Keep summary scope parsing in prep

`summary` prep should still normalize:

- requested count/page scope
- collection target hint

That part is independent of whether the actor is already cached.

### 4. Keep collection selection after actor hydration

Once execution has a concrete actor DID, the existing `summary` path can continue to:

- choose hydration args from the summary hint and scope
- hydrate actor-backed collections
- select the best collection
- run selection review/repair if applicable
- run `collection_summary`

That sequencing remains sound.

## Scope Boundaries

### In scope

- stop requiring a cached DID for explicit-actor `summary` prep
- align public `search` and `summary` actor-resolution timing
- add execution-time resolution for uncached explicit actor refs
- preserve useful debug traces showing whether actor identity came from cache, explicit ref resolution, or selected actor fallback

### Out of scope

- moving collection selection into prep
- changing `collection_summary` paging behavior
- redesigning the internal `search` planner loop
- changing summary scope parsing semantics

## Failure Cases To Handle

### Explicit actor ref exists but is invalid

If prep detects something that looks like an actor ref, but execution cannot parse or resolve it as a valid Bluesky identifier, the public tool should fail during execution with a concrete actor-resolution error.

That is preferable to a misleading prep-time `missing actor_anchor` failure.

### No explicit actor ref and no selected actor

If the query is relative, such as:

- `summarize their last 50 posts`

and the UI has no selected actor, prep should still fail.

There is no actor input to defer.

### Explicit actor ref resolves, but no compatible collection exists

That should remain a later failure or review/repair outcome in the existing execution flow.

This task only changes whether execution can begin.

## Debug Expectations

The runtime should make the actor-resolution path visible in debug output.

Useful states include:

- detected explicit actor ref during prep
- selected actor fallback during prep
- resolved actor from cache during execution
- resolved actor through profile fetch during execution
- actor resolution failed during execution

This will make future failures much easier to classify.

## Acceptance Criteria

- `summary` prep no longer fails for an explicit handle-shaped actor ref solely because that handle is uncached.
- Public `summary` can begin execution for a query like `summarize 150 posts by segyges.bsky.social into 3 paragraphs` when no actor is selected in the UI.
- Public `summary` resolves the actor during execution and then proceeds into hydration and collection selection.
- Public `search` and public `summary` follow the same actor-resolution timing model.
- Relative actor queries without explicit refs and without a selected actor still fail during prep.
- Debug traces clearly show whether the actor was resolved from cache, selected fallback, or explicit ref lookup.

## Implementation Notes

Likely code areas:

- `src/harness/tools.rs`
- `src/net_backend.rs`
- public tool prep types for `search` and `summary`
- public tool execution entry points

Relevant current behavior:

- `detect_actor_refs(...)` already recognizes handle-shaped refs
- `resolve_prepared_actor_anchor(...)` currently requires store-backed DID resolution
- `ensure_actor_profile_cached(...)` already provides the execution-time resolution primitive needed for uncached explicit refs
