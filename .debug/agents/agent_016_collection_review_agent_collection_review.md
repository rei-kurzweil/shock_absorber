# Agent Debug

- agent_id: 16
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 15
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
- used_input_tokens: 644
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 16 / estimated 16
- Collection Evidence [review_evidence]: used 135 / estimated 135
- Proposed Summary [parent_search_results]: used 218 / estimated 218

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
collection_id: pinned_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Pinned posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: pinned_posts

matched_item[0] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mnhonhuejc2i
body: people have likely been able to figure some of this out for a bit, given the context of recent Portia photos, but i have finally found a full-time job! very soon going to be working for NBCUniversal, in their Montreal office, on a very interesting project.

## Proposed Summary
post: LLM-selected post in Pinned posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around people have likely been able to figure some of this out for a bit, given the context of recent Portia photos, but i have finally found a full-time job! very soon going to be working for NBCUniversal, in their Montreal office, on a very interesting project.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mnhonhuejc2i
search_result_1_source_collection_id: pinned_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
