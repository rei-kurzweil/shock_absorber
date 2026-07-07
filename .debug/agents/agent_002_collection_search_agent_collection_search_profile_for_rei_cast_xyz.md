# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Profile for rei-cast.xyz
- status: completed
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: actor_profile:did:plc:frudpt5kpurby7s7qdaz7zyw

## Result Summary

review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Profile for rei-cast.xyz
summary: The provided evidence is narrow, consisting of a single actor profile for rei-cast.xyz, which details the creator's identity and primary focus. The main theme revolves around the development of a "cat engine powered by Vulkan," which is the core project mentioned in the bio. The profile explicitly states the author's handle is "rei-cast.xyz" and provides a link to their GitHub repository: "https://github.com/rei-kurzweil." Further biographical details include the creator being from Canada ("🇨🇦"), identifying as transgender ("🏳️‍⚧️"), and using she/her pronouns. The profile also includes a playful status update: "5/9 lives left." Since this is a self-profile, the evidence shows what rei-cast.xyz posts about, but it does not show direct replies or external commentary, though the bio itself functions as a statement about the creator's identity and work.
search_result_1_uri: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:frudpt5kpurby7s7qdaz7zyw

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 643
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 146 / estimated 146
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
collection_id: actor_profile:did:plc:frudpt5kpurby7s7qdaz7zyw
label: Profile for rei-cast.xyz
collection_kind: actor_profile
item_count: 1
last_refreshed_at: 1783449864
actor_did: did:plc:frudpt5kpurby7s7qdaz7zyw
refresh_ttl_seconds: 900
metadata.source: actor_profile

item[0]
uri: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.actor.profile/self
author: rei-cast.xyz
body: handle: rei-cast.xyz
did: did:plc:frudpt5kpurby7s7qdaz7zyw
bio:
#!/usr/bin/rei
making a cat engine powered by Vulkan.

# https://github.com/rei-kurzweil
# 5/9 lives left, 🇨🇦, 🏳️‍⚧️ (she/her)


## Search Prompt
what are people saying about rei-cast.xyz? how do people reply to rei-cast.xyz? and what do they post about?
```
