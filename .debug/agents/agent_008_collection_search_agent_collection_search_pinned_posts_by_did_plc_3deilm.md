# Agent Debug

- agent_id: 8
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Pinned posts by did:plc:3deilm3cxnqundoo227xudg2
- status: completed
- parent_agent_id: 1
- child_agent_ids: 9
- collection_id: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2

## Result Summary

review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Pinned posts by did:plc:3deilm3cxnqundoo227xudg2
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around I moved all my blog articles from medium to my own site! I didn't like that Medium doesn't have any options for letting AI get access to / train on my articles, so Monday and I vibecoded a place for them.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3lqks73aop227
search_result_1_source_collection_id: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 599
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 202 / estimated 202
- Search Prompt [local_task]: used 49 / estimated 49

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return a grounded result block with `title:`, `summary:`, and up to four `uri:` lines for the chosen search results.

Every `uri:` must be a real item from the collection.

The `summary:` field is required. It must be one grounded paragraph of roughly 100-200 words.

That paragraph should include:

- the main repeated themes
- the strongest exact phrases or list names
- any meaningful split, ambiguity, or contrast inside the collection
- a short note about how broad or narrow the matched evidence seems when that matters

Use the `uri:` lines to point at the strongest supporting records.

If fewer than four real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not add higher-level interpretation beyond grouping repeated evidence and short contrasts that are directly supported by the collection text.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2
label: Pinned posts by did:plc:3deilm3cxnqundoo227xudg2
collection_kind: pinned_posts
item_count: 1
last_refreshed_at: 0
actor_did: did:plc:3deilm3cxnqundoo227xudg2
refresh_ttl_seconds: 900
metadata.actor_did: did:plc:3deilm3cxnqundoo227xudg2
metadata.collection_kind: pinned_posts

item[0]
uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3lqks73aop227
author: jcorvinus.bsky.social
body: I moved all my blog articles from medium to my own site! I didn't like that Medium doesn't have any options for letting AI get access to / train on my articles, so Monday and I vibecoded a place for them.

jcorvinus.black

The text is all copyright free. It's entirely a static site, no scripts
link: https://jcorvinus.black/


## Search Prompt
Analyze the sentiment of the pinned posts. Are they generally positive (optimistic/excited), negative (critical/concerned), or neutral (informational)? Provide a brief summary.
```
