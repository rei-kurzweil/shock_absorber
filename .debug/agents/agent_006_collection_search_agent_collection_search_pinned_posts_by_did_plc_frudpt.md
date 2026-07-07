# Agent Debug

- agent_id: 6
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Pinned posts by did:plc:frudpt5kpurby7s7qdaz7zyw
- status: warning
- parent_agent_id: 1
- child_agent_ids: 7
- collection_id: pinned_posts:did:plc:frudpt5kpurby7s7qdaz7zyw

## Result Summary

review_status: pass
review_reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary is dominated by identifiers or metadata placeholders.
review_repair_needed: false
repair_diagnostic: Initial review failed. Applied deterministic cited repair when possible. Original summary: The evidence provided focuses on a single post from rei-cast.xyz, which details updates and features related to a graphics or animation library. The main themes revolve around adding new drawing functions and improving existing APIs. Spe...
post: Rei-cast.xyz Posts
summary: Selected evidence is drawn from 1 cited record(s). [0] @rei-cast.xyz: "Added R.star(points: u32, inner_radius: f32, outer_bevel:u32, inner_bevel: u32) R.heart(detail: u32) R.partial_annulus_2d(inner_radius, outer_radius, start_a...".
search_result_1_uri: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.feed.post/3mpkcq5nqck23
search_result_1_source_collection_id: pinned_posts:did:plc:frudpt5kpurby7s7qdaz7zyw

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 676
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 179 / estimated 179
- Search Prompt [local_task]: used 32 / estimated 32

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return exactly one JSON object with this shape:

```json
{
  "title": "short title",
  "summary": "one grounded paragraph",
  "uris": ["real item uri 1", "real item uri 2"]
}
```

- `title` is optional but preferred.
- `summary` is required.
- `uris` is required and may contain up to ten real item URIs from the collection.
- Do not return markdown, code fences, or any text outside the JSON object.

Every `uris` entry must be a real item from the collection.

The `summary` field must be one grounded paragraph of roughly 100-200 words.

That paragraph should include:

- the main repeated themes
- the strongest exact phrases or list names
- which results seem most important versus secondary
- any meaningful split, ambiguity, or contrast inside the collection
- a short note about how broad or narrow the matched evidence seems when that matters

Use the `uris` array to point at the strongest supporting records.

If fewer than ten real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not mention `item[...]`, `matched_item[...]`, or raw metadata labels inside the `summary`. Keep citations in `uris`, not in the paragraph.

Do not add higher-level interpretation beyond grouping repeated evidence and short contrasts that are directly supported by the collection text.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: pinned_posts:did:plc:frudpt5kpurby7s7qdaz7zyw
label: Pinned posts by did:plc:frudpt5kpurby7s7qdaz7zyw
collection_kind: pinned_posts
item_count: 1
last_refreshed_at: 1783043105
actor_did: did:plc:frudpt5kpurby7s7qdaz7zyw
refresh_ttl_seconds: 900

item[0]
uri: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.feed.post/3mpkcq5nqck23
author: rei-cast.xyz
body: Added

R.star(points: u32, inner_radius: f32, outer_bevel:u32, inner_bevel: u32)

R.heart(detail: u32)

R.partial_annulus_2d(inner_radius, outer_radius, start_angle_radians, sweep_angle_radians, segments).

Realizing the KeyFrame{} api is a bit clunky. Simplifying that + adding more tweening now
link: https://R.star(points


## Search Prompt
what are people saying about rei-cast.xyz? how do people reply to rei-cast.xyz? and what do they post about?
```
