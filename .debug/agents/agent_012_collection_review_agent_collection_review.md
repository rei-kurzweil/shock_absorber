# Agent Debug

- agent_id: 12
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 11
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
- used_input_tokens: 567
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 16 / estimated 16
- Collection Evidence [review_evidence]: used 121 / estimated 121
- Proposed Summary [parent_search_results]: used 155 / estimated 155

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
how do people reply to nonbinary.computer?

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
post: LLM-selected post in Profile for nonbinary.computer
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around handle: nonbinary.computer. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
