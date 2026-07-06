# Agent Debug

- agent_id: 6
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Profile for nonbinary.computer
- status: completed
- parent_agent_id: 1
- child_agent_ids: 7
- collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Result Summary

review_status: pass
review_reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary omits meaningful text that was available in the matched records.
review_repair_needed: false
repair_diagnostic: Initial review failed. Original summary: The profile for 'nonbinary.computer' presents a multifaceted identity, primarily identifying as a 'Person who does electrical, computer, and music things.' A key theme is the blend of technical and creative pursuits, further emphasized b...
post: LLM-selected post in Profile for nonbinary.computer
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around handle: nonbinary.computer. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 526
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 157 / estimated 157
- Search Prompt [local_task]: used 21 / estimated 21

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
collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
label: Profile for nonbinary.computer
collection_kind: actor_profile
item_count: 1
last_refreshed_at: 1783369765
actor_did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
refresh_ttl_seconds: 900
metadata.source: actor_profile

item[0]
uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
author: nonbinary.computer
body: handle: nonbinary.computer
did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
bio:
Person who does electrical, computer, and music things. 

Mother of machines.

they/she/it

support: github.com/sponsors/orual https://ko-fi.com/orual


## Search Prompt
General sentiment and description of nonbinary.computer's posts?
```
