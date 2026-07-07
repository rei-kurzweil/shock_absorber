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
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: nonbinary.computer Profile Summary
summary: The main themes associated with nonbinary.computer revolve around a multifaceted identity and professional focus, as indicated by their profile information. The individual describes themselves as a "Person who does electrical, computer, and music things," suggesting a broad technical and creative scope. A key thematic element is the self-identification, explicitly stated as "they/she/it," which aligns with the profile's designation as being for a nonbinary individual. Furthermore, the profile includes a creative or familial theme with the phrase "Mother of machines." The evidence is quite narrow, being drawn from a single profile record, but it strongly emphasizes technical aptitude, creative pursuits, and nonbinary identity. Secondary supporting details include their handle, "nonbinary.computer," and their support links to "github.com/sponsors/orual" and "https://ko-fi.com/orual" [at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self].
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 643
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 157 / estimated 157
- Search Prompt [local_task]: used 21 / estimated 21

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
collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
label: Profile for nonbinary.computer
collection_kind: actor_profile
item_count: 1
last_refreshed_at: 1783459087
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
What are the main themes in nonbinary.computer's recent posts?
```
