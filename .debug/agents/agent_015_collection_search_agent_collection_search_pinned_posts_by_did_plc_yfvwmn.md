# Agent Debug

- agent_id: 15
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Pinned posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
- status: completed
- parent_agent_id: 10
- child_agent_ids: 16
- collection_id: pinned_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Result Summary

review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Pinned posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around people have likely been able to figure some of this out for a bit, given the context of recent Portia photos, but i have finally found a full-time job! very soon going to be working for NBCUniversal, in their Montreal office, on a very interesting project.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mnhonhuejc2i
search_result_1_source_collection_id: pinned_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 527
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 163 / estimated 163
- Search Prompt [local_task]: used 16 / estimated 16

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
collection_id: pinned_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g
label: Pinned posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: pinned_posts
item_count: 1
last_refreshed_at: 1783190620
actor_did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
refresh_ttl_seconds: 900

item[0]
uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mnhonhuejc2i
author: nonbinary.computer
body: people have likely been able to figure some of this out for a bit, given the context of recent Portia photos, but i have finally found a full-time job! very soon going to be working for NBCUniversal, in their Montreal office, on a very interesting project.


## Search Prompt
how do people reply to nonbinary.computer?
```
