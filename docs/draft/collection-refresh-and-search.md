# Collection Refresh And Search Draft

Goal: simplify the harness around durable, refreshable actor collections that the LLM can enumerate, search, and selectively read into context.

This draft is intentionally pre-implementation.
It describes the target behavior before the next round of code changes.

## Why Change The Current Design

The current harness can search some cached collections, but the design is still too ad hoc:

- some collections are implicit cache side effects
- collection freshness is not explicit
- search source boundaries are inconsistent
- the model does not yet have a clean way to enumerate collections
- item-level read paths are too narrow

We want a model where:

- collections are first-class cached objects
- each collection knows when it was last refreshed
- the LLM can enumerate all collections
- the LLM can enumerate collections for one actor
- the LLM can refresh stale collections through a clear backend path
- the LLM can search collections broadly
- the LLM can load a specific item plus its richer context into `LLMContext`

## Core Collection Model

Collections should become the primary unit of cached inspectable data.

Each collection should have at least:

- `id`
- `label`
- `collection_kind`
- `actor_did` when actor-scoped
- `items`
- `related_collection_ids`
- `last_refreshed_at`
- `refresh_ttl_seconds` or an equivalent freshness policy
- freeform metadata

Important change:

- `last_refreshed_at` should become a real field, not just hidden inside metadata

That allows freshness checks without parsing ad hoc strings.

## Collection Kinds

Near-term actor-scoped collection kinds should be:

- `recent_posts`
  - recent top-level posts by the actor
  - intent: things the actor chose to say publicly

- `recent_replies_sent`
  - recent replies authored by the actor
  - intent: things the actor said directly to someone else

- `recent_posts_unaddressed`
  - recent posts by the actor that are not replies
  - intent: split “their broadcast posts” from “their conversational replies”

- `replies_to_actor`
  - recent replies sent to the actor
  - target behavior
  - note: current backend does not yet support this generally
  - current best-effort pinned-post reply cache is not enough for the final design

- `clearsky_lists`
  - Clearsky lists the actor appears on

Optional later kinds:

- `pinned_posts`
- `pinned_post_replies`
- `mentions_of_actor`
- `notifications_for_actor`

## Clarifying The Two Reply/Post Splits

The user request implies two distinct views that should both exist:

### 1. Recent replies sent by an actor

This means:

- posts authored by the actor
- where the post is a reply to another post

This is not the same as replies sent to the actor.

### 2. Recent posts they have not addressed to anyone

This means:

- posts authored by the actor
- where the post is not a reply

This is essentially a “recent top-level posts” or “recent unaddressed posts” collection.

So the clean split is:

- `recent_replies_sent`
- `recent_posts_unaddressed`

And separately, if supported:

- `replies_to_actor`

## Refresh Model

Collections should stay cached once loaded, but they should also be refreshable.

Target behavior:

- once a collection is fetched, it stays in memory
- it is also persisted to disk
- future queries reuse the cached collection by default
- if `last_refreshed_at` is older than a threshold, the system may refresh it

Suggested default refresh threshold:

- 15 minutes for fast-changing actor content

That threshold should be configurable by collection kind later, but we do not need to over-design it yet.

Example rough policy:

- `recent_posts_unaddressed`: 15 minutes
- `recent_replies_sent`: 15 minutes
- `replies_to_actor`: 15 minutes
- `clearsky_lists`: 15 minutes or longer if Clearsky updates less frequently

## Refresh Semantics

We should distinguish:

- cache hit
- stale cache hit
- forced refresh

Target backend behavior:

- if collection is missing: fetch and cache it
- if collection exists and is fresh: return cached version
- if collection exists but is stale: refresh it before use
- later: allow explicit forced refresh regardless of freshness

This suggests a backend API shape like:

- `ensure_collection_cached(actor_did, collection_kind, freshness_policy)`

or:

- `get_or_refresh_collection(...)`

The important point is that freshness should be checked inside a dedicated collection cache path, not scattered across tool code.

## LLM Tooling Model

The LLM should not need to know backend wiring.
It should know a small set of read/search/enumeration tools.

### 1. Enumerate collections

The model should be able to ask:

- what collections exist right now
- what actor-scoped collections exist for actor X

This can be exposed as:

- `list_collections`
- `list_actor_collections`

Returned fields should include:

- `id`
- `label`
- `collection_kind`
- `actor_did`
- `item_count`
- `last_refreshed_at`
- `related_collection_ids`

### 2. Search collections

The model should know it has one `llm_search` tool.

That tool should be able to:

- search one collection
- search many collections
- search all collections
- search all collections related to one actor

It should not need multiple scope systems that confuse the prompt.

A better shape is:

- `llm_search`
  - optional `actor_did`
  - optional `collection_ids`
  - optional `collection_kinds`
  - `prompt`

Behavior:

- if no actor or collection filter is given, it can search all cached collections
- if `actor_did` is given, it can search all collections related to that actor
- if `collection_ids` are given, it searches those exact collections
- if `collection_kinds` are given, it filters by kind

The tool implementation can synthesize a temporary merged search space internally.

### 3. Read a specific item into context

The model should know it has a read tool that loads one item plus richer surrounding detail into `LLMContext`.

This should be the main “zoom in” path after a search result is chosen.

Possible tool:

- `read_collection_item`

Inputs:

- `collection_id`
- `item_uri` or `item_id`

Target returned detail:

- full text/body
- facets
- quoted text
- reply parent/root references when relevant
- if item is a post:
  - optionally nearby reply context
  - optionally replies to that item

This is the tool that should load the richer context window slice, not `llm_search`.

## Relationship To `LLMContext`

The collection system should work with the bounded context design, not against it.

Desired flow:

1. main context contains:
   - current task
   - compact selected notification facts
   - compact collection inventory

2. model chooses a search

3. `llm_search` runs in a child/narrower context over collections only

4. search returns compact ranked results

5. model chooses one result to inspect

6. `read_collection_item` loads only that item and relevant details into parent context

This keeps the main prompt small while still allowing deeper inspection.

## Persistence Expectations

Collections should exist both:

- in memory for fast reuse during the session
- on disk so the app can resume with warm caches

This implies the database should persist at least:

- collection identity
- collection kind
- actor DID
- serialized items
- related collection IDs
- `last_refreshed_at`
- refresh policy metadata

The in-memory and on-disk representations should stay close enough that restoring them is straightforward.

## Required Backend Additions

To support the target model, `net_backend` likely needs explicit fetch paths for:

- recent top-level posts authored by actor
- recent replies authored by actor
- recent replies directed to actor
- Clearsky lists for actor

Important note:

The current backend already supports recent feed fetches and pinned-post reply caches, but not yet a general fetch for “recent replies sent to actor.”

That should be treated as a real backend feature gap, not papered over in tool logic.

## Recommended Near-Term Sequence

1. promote `last_refreshed_at` into first-class collection state
2. split actor-authored recent content into:
   - `recent_replies_sent`
   - `recent_posts_unaddressed`
3. add collection enumeration helpers:
   - all collections
   - actor-related collections
4. define a cleaner `llm_search` input shape around actor and collection filters
5. add `read_collection_item`
6. then revisit whether the current item type should remain `PostRecord` or become a broader collection-item type

## Non-Goals For This Draft

Not part of this immediate change:

- writing back to Bluesky
- user-curated collections yet
- multi-agent planning frameworks
- exact final schema names
- exact REST/UI command surface

The goal here is only to settle the collection-and-refresh model before code changes begin.
