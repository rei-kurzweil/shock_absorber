# Agent Debug

- agent_id: 7
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 6
- child_agent_ids: <none>

## Result Summary

status: pass
reason: The summary is grounded in the selected records and contains substantive evidence.
repair_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 719
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 21 / estimated 21
- Collection Evidence [review_evidence]: used 121 / estimated 121
- Proposed Summary [parent_search_results]: used 302 / estimated 302

## Rendered Context Window

```text
Instructions:
You are the internal collection-summary review agent.

Your job is to review one `collection_search` summary before it is used by parent `llm_search` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `reason: ...`
- `repair_needed: true` or `repair_needed: false`
- optional `repair_instructions: ...`

Rules:

- Review the summary against the actual collection evidence provided.
- Fail if the summary is missing, mostly metadata, mostly identifiers, unsupported by the selected records, or too thin to support parent synthesis.
- Pass only when the summary is one grounded paragraph and uses real phrases, quotes, list names, descriptions, or post/reply text from the matched records.
- When the prompt asks for sentiment, reputation, contrast, or list interpretation, expect the summary to preserve that distinction with grounded evidence.
- If the summary can likely be fixed by rewriting it from the existing selected records, set `repair_needed: true` and provide short repair instructions.
- Do not rewrite the summary yourself in this step.

## Search Prompt
What are the main themes in nonbinary.computer's recent posts?

## Collection Evidence
collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Profile for nonbinary.computer
collection_kind: actor_profile

matched_item[0] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
body: handle: nonbinary.computer
did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
bio:
Person who does electrical, computer, and music things. 

Mother of machines.

they/she/it

support: github.com/sponsors/orual https://ko-fi.com/orual

## Proposed Summary
post: nonbinary.computer Profile Summary
summary: The main themes associated with nonbinary.computer revolve around a multifaceted identity and professional focus, as indicated by their profile information. The individual describes themselves as a "Person who does electrical, computer, and music things," suggesting a broad technical and creative scope. A key thematic element is the self-identification, explicitly stated as "they/she/it," which aligns with the profile's designation as being for a nonbinary individual. Furthermore, the profile includes a creative or familial theme with the phrase "Mother of machines." The evidence is quite narrow, being drawn from a single profile record, but it strongly emphasizes technical aptitude, creative pursuits, and nonbinary identity. Secondary supporting details include their handle, "nonbinary.computer," and their support links to "github.com/sponsors/orual" and "https://ko-fi.com/orual" [at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self].
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
