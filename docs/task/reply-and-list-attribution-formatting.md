# Task: Preserve Reply And List Attribution Through `collection_search` And Root Synthesis

## Summary

The current `llm_search` and `collection_search` pipeline can surface useful reply and list evidence, but the final root-visible output still loses important attribution.

For reply evidence, the root answer should show:

- the handle of the user who wrote the reply
- the reply text on the same line

Target rendering shape:

- `@some-user.bsky.social: <reply text>`

For moderation-list evidence, the root answer should eventually show:

- the handle of the user who created the list
- the list name
- the list description when present

Target rendering shape for a future richer list block:

```text
user-who-created-the-list.bsky.social's list:
list_name: ...
list_description: ...

```

This task is about deciding the cleanest ownership model for that attribution and then plumbing it through the harness so root synthesis can reuse it directly.

## Problem

### Reply evidence already contains enough attribution to render clean examples

The cached `recent_replies_received` collection already stores:

- `author`
- `reply_text`
- the reply item `uri`
- the source/root linkage in `source_post_uri`

Example from the current `.debug` review context:

- `author: bot-tan.suibari.com`
- `reply_text: This 3D render looks so cool! Such precise lines.`

That means the system already knows the reply author's handle before the `collection_search` agent sees the collection window.

Today, that attribution is not consistently preserved into the final root-visible example formatting.

### Lists are less complete than replies

For moderation-list evidence, the current cached record shape prominently exposes:

- `list_name`
- `list_description`
- list item `uri`

The final output can already mention list names and descriptions, but it does not yet consistently expose the list creator handle in a compact human-readable way.

The list creator handle may need either:

- direct extraction from the current list URL / stored record body
- or a secondary hydration step when the chosen list URI is promoted into parent/root output

That makes list attribution a slightly different problem from reply attribution.

## Decision: Prefer Attribution Before `collection_search` For Replies

For replies, the cleanest approach is to preserve attribution before the collection agent runs, not after.

Reasons:

- the reply author handle is already known when the cached collection is built
- `collection_search` should search over the same human-meaningful fields that later appear in output
- review and deterministic repair both benefit from having the handle present in the selected evidence
- post-hoc hydration would duplicate work the cache layer already performed

Concretely, reply-like collection records should continue to carry explicit structured reply-author information into the serialized collection window and all later typed results.

### Why not post-hydrate replies later?

Post-hydrating reply authors after `collection_search` would be strictly worse unless the cache truly lacked the handle.

It would:

- add another recovery/hydration step for data we already have
- make `collection_search` summaries less grounded because the agent would not see the final attribution format
- complicate deterministic repair, which currently rebuilds summaries from selected records

So for replies, the preferred model is:

1. cache/build reply collections with author handle preserved
2. show that handle in collection evidence given to `collection_search`
3. keep that handle available in selected-result records
4. let deterministic repair and root synthesis render `@handle: reply text`

## Preferred Model For Lists

Lists should be handled in two phases.

### Phase 1: Preserve what we already know before `collection_search`

Before `collection_search`, list records should expose at least:

- list `uri`
- `list_name`
- `list_description`

If the list creator handle can be parsed cheaply and reliably from the existing stored record or list URL, it should also be preserved at this stage.

### Phase 2: Optional creator-handle hydration for promoted list results

If creator handle is not already present in the cached record, a follow-up promotion/hydration step is acceptable for selected list results only.

That means:

- do not hydrate every list creator for the whole search window by default
- only hydrate creator attribution for the selected `search_result_*_uri` entries that survive review and are likely to be shown to the root agent

This keeps list enrichment targeted while preserving a simpler reply path.

## Desired Output Semantics

### Reply examples in root output

When root synthesis chooses to show reply examples, every example line should start with the reply author's handle.

Desired shape:

- `@bot-tan.suibari.com: This 3D render looks so cool! Such precise lines.`
- `@jitspoe.bsky.social: Such a cool and well explained video! Thanks for making awesome stuff and sharing how to do it!`

Important constraints:

- one example per line
- handle and reply text on the same line
- no raw `source_post_uri:` metadata in the visible example line

### List examples in root output

For lists, the future richer rendering should prefer stable attribution plus the exact list fields.

Desired shape:

```text
creator-handle.bsky.social's list:
list_name: The Great AI - NFT - CRYPTO Cull
list_description: If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will.

```

If creator handle is not yet available, the short-term fallback can still render:

```text
list_name: The Great AI - NFT - CRYPTO Cull
list_description: If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will.

```

But the long-term goal is creator-attributed list blocks.

## Likely Code Areas

- reply/list collection construction in `src/net_backend.rs`
- collection serialization and field rendering in `src/harness/tools.rs`
- deterministic repair formatting for selected reply/list evidence
- root synthesis formatting paths that turn reviewed `collection_search` results into final answer examples

## Acceptance Criteria

- reply collections preserve reply-author handle through the collection-search and review pipeline
- deterministic repair can render selected replies as `@handle: reply text`
- root synthesis can reuse that same reply formatting without additional lookup
- moderation-list output can render exact `list_name` and `list_description`
- a follow-up path exists to include list creator handle for promoted list examples without forcing global list hydration up front

## Recommended Implementation Order

1. Make reply attribution first-class in the selected-result and repair pipeline.
2. Update root output formatting so reply examples render as `@handle: reply text`.
3. Preserve or parse list creator handle where cheap.
4. Add targeted list creator hydration only for selected promoted list results if needed.
