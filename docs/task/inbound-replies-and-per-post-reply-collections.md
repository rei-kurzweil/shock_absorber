# Task: Add Actor Inbound Reply Collections And On-Demand Per-Post Reply Collections

## Goal

Fill the current reply-data gap so actor-focused search can answer questions like:

- what sentiment do other people express toward this actor
- what kinds of replies does this actor receive
- what replies did a specific post receive

without incorrectly anchoring on only:

- the currently selected notification reply
- pinned-post reply caches

## Why This Needs To Exist

The current system can already search:

- actor profile
- recent posts by the actor
- recent replies sent by the actor
- Clearsky lists

But it cannot generally search:

- recent replies from other actors to this actor's posts
- replies to an arbitrary post the harness wants to inspect

That gap creates a bad failure mode for actor-sentiment questions.

Example failure:

- the root agent sees a selected reply notification in UI context
- it reads that selected reply with `read_selected_post`
- it rewrites the task into "what are people saying about this post?"
- `llm_search` then searches the wrong scope and may return no grounded results

The missing substrate is not just prompt wording.
The missing substrate is reply collections that match the user intent.

## Current State

Today:

- `recent_replies_sent:<did>` exists
  - replies authored by the actor
- `replies_to_actor:<did>` is only a synthesized best-effort view
  - built from replies to pinned posts only
- pinned-post thread replies can be cached for one known post
- there is no general persisted collection for:
  - recent inbound replies to an actor
  - replies to any arbitrary post URI we decide to inspect

Important limitation:

- `replies_to_actor` sounds broader than it really is today
- current behavior does not satisfy "what did other people reply back to this actor?"

## Desired Collections

### 1. Recent inbound replies to actor

Add a real collection kind:

- `recent_replies_received:<did>`

Semantics:

- up to 50 recent replies authored by other actors
- where the reply target is one of the actor's posts
- intended for actor-level sentiment and interaction analysis

This collection should be queryable, cacheable, refreshable, and searchable by `llm_search`.

### 2. Replies for one specific post

Add a real collection kind:

- `post_replies:<post_uri>`

Semantics:

- replies to one specific post URI
- cached on demand when the harness or agent decides a post needs deeper inspection
- intended for post-level sentiment or thread analysis

This collection should also be queryable, cacheable, refreshable, and searchable by `llm_search`.

## Behavioral Requirements

### Actor-focused questions

If the user asks about:

- sentiment toward an actor
- how people respond to an actor
- what kinds of replies an actor receives

then `llm_search` should be able to cause `recent_replies_received:<did>` to exist and search it as part of actor-centric analysis.

It should not need to reinterpret the task as a search about the currently selected notification post unless the user actually asked about that post.

### Post-focused questions

If the user asks about:

- replies to a specific post
- sentiment in a specific thread
- what people said back to one concrete URI

then the harness should be able to cause `post_replies:<post_uri>` to exist and search it directly.

## Data And Cache Model

Both new collection kinds should behave like first-class cached collections.

Minimum fields:

- collection ID
- label
- collection kind
- actor DID when relevant
- source post URI when relevant
- posts/items
- last refreshed timestamp
- refresh TTL
- metadata describing how the collection was derived

Suggested near-term limits:

- `recent_replies_received:<did>`: up to 50 replies
- `post_replies:<post_uri>`: enough replies to make one thread inspection useful without overloading context

## Required Backend Additions

We likely need explicit backend fetch paths for two different scopes.

### A. Actor inbound replies fetch

Add a backend capability that can gather recent replies directed at an actor's posts, not just replies found under pinned posts.

Expected output:

- one normalized collection for `recent_replies_received:<did>`

Important note:

- if Bluesky API limitations require approximation, document the approximation explicitly
- do not silently label pinned-post-only coverage as general actor inbound replies

### B. Arbitrary post reply fetch

Add a backend capability that can fetch and cache replies for any post URI the harness wants to inspect.

Expected output:

- one normalized collection for `post_replies:<post_uri>`

This should generalize the current pinned-post-reply cache path instead of keeping it as a special isolated behavior.

## `llm_search` Integration

`llm_search` should treat these as normal searchable collections.

For actor-centric queries, it should be able to include:

- `actor_profile`
- `clearsky_lists`
- `recent_posts_unaddressed`
- `recent_replies_sent`
- `recent_replies_received`

For post-centric queries, it should be able to:

1. identify the post URI in question
2. ensure `post_replies:<post_uri>` exists
3. run collection search over that reply collection
4. synthesize the result

## Scope Boundaries

This task is about reply collection availability and search scope.

It is not, by itself, the task for:

- internal `collection_search` tool definitions
- root duplicate-call prevention
- `/stop`
- provider-level tool-calling protocol work

Those tasks still matter, but this reply-data gap should be described separately.

## Recommended Breakdown

### Phase 1: Real actor inbound reply collection

Implement:

- `recent_replies_received:<did>`
- backend fetch path
- cache/persistence support
- `list_collections` visibility
- `llm_search` inclusion for actor-focused queries

### Phase 2: General per-post reply collection

Implement:

- `post_replies:<post_uri>`
- backend fetch path
- cache/persistence support
- reuse by `llm_search` for post-focused queries

### Phase 3: Prompt/routing cleanup

After the collection substrate exists:

- reduce accidental bias toward the selected notification reply
- prefer actor-wide inbound-reply scope for actor-sentiment questions
- only choose post-reply scope when the user asked about one post specifically

## Acceptance Criteria

This task is complete when:

- the system can cache and search up to 50 recent inbound replies to an actor
- the system can cache and search replies for an arbitrary post URI on demand
- actor-sentiment questions can use inbound replies from other actors as grounding
- post-specific reply questions can use a dedicated post-reply collection as grounding
- the implementation no longer relies on pinned-post reply caches as the only source for `replies_to_actor`-style analysis
