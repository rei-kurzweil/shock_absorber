# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Recent top-level posts by did:plc:uv76n3a4zrgxzo45cgpemkve
- status: completed
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve

## Result Summary

review_status: pass
review_reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary is dominated by identifiers or metadata placeholders.
review_repair_needed: false
repair_diagnostic: Initial review failed. Applied deterministic cited repair when possible. Original summary: The recent unaddressed posts cover a diverse range of topics, including media consumption, technical features, and general musings. A recurring theme involves visual and auditory experiences, such as capturing starlight and aligning imag...
post: Recent Unaddressed Posts
summary: Selected evidence is drawn from 6 cited record(s). [0] @p-s-v.bsky.social: "📸abdul : ''same image stacked differently. i capture about an hour of starlight and align them to get more detail out of the milky way. but if i don’t align ...". [1] @nerdyspinosaurid.bsky.social: "Separate sound channels with independently adjustable volume and ability to mute would be really convenient in meatspace.". [2] @snowanddrugs.bsky.social: "ok give me some idea what do with fable since it's supposed to run out tonorrow". [3] @mara.x0f.nl: "fallow_irl". [4] @faz.ms: "A good workman always praises his tools.". [5] @mara.x0f.nl: "TIL".
search_result_1_uri: at://did:plc:cjf5cyja7mbog73soxw7y5lt/app.bsky.feed.post/3mpxwi4j4mk2d
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_2_uri: at://did:plc:mo6di3tejom2usriqbnrkh2w/app.bsky.feed.post/3mpxnrgkhec2u
search_result_2_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_3_uri: at://did:plc:7cgn3czdxcpsas5a6kp5bt2x/app.bsky.feed.post/3mpxi3uh6gs2j
search_result_3_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_4_uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpw6ramgzc2p
search_result_4_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_5_uri: at://did:plc:huocg7zsbthk54vjai56wd27/app.bsky.feed.post/3mptlcs3ops2n
search_result_5_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_6_uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mptlvdrf5s2z
search_result_6_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 939
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 463 / estimated 463
- Search Prompt [local_task]: used 11 / estimated 11

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
collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
label: Recent top-level posts by did:plc:uv76n3a4zrgxzo45cgpemkve
collection_kind: recent_posts_unaddressed
item_count: 7
last_refreshed_at: 1783480587
actor_did: did:plc:uv76n3a4zrgxzo45cgpemkve
refresh_ttl_seconds: 900
related_collections: pinned_posts:did:plc:uv76n3a4zrgxzo45cgpemkve
metadata.source_feed: author_feed

item[0]
uri: at://did:plc:qn3cmxkfisazt25du36o3txj/app.bsky.feed.post/3mq25ij3i4224
author: lottievixen.xyz
body: ooo www.youtube.com/watch?v=GYdx...
link: https://www.youtube.com/watch?v=GYdxmZvYvMg

item[1]
uri: at://did:plc:cjf5cyja7mbog73soxw7y5lt/app.bsky.feed.post/3mpxwi4j4mk2d
author: p-s-v.bsky.social
body: 📸abdul :
''same image stacked differently. i capture about an hour of starlight and align them to get more detail out of the milky way. but if i don’t align them it demonstrates the rotation of the earth.''
x.com/Advil/status...
link: https://x.com/Advil/status/2073858316011286618

item[2]
uri: at://did:plc:mo6di3tejom2usriqbnrkh2w/app.bsky.feed.post/3mpxnrgkhec2u
author: nerdyspinosaurid.bsky.social
body: Separate sound channels with independently adjustable volume and ability to mute would be really convenient in meatspace.

item[3]
uri: at://did:plc:7cgn3czdxcpsas5a6kp5bt2x/app.bsky.feed.post/3mpxi3uh6gs2j
author: snowanddrugs.bsky.social
body: ok give me some idea what do with fable since it's supposed to run out tonorrow

item[4]
uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpw6ramgzc2p
author: mara.x0f.nl
body: fallow_irl

item[5]
uri: at://did:plc:huocg7zsbthk54vjai56wd27/app.bsky.feed.post/3mptlcs3ops2n
author: faz.ms
body: A good workman always praises his tools.

item[6]
uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mptlvdrf5s2z
author: mara.x0f.nl
body: TIL


## Search Prompt
analyze the last 50 posts
```
