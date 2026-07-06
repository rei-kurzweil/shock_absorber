# Agent Debug

- agent_id: 18
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 17
- child_agent_ids: <none>

## Result Summary

status: pass
reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary omits meaningful text that was available in the matched records.
repair_needed: false
repair_diagnostic: Initial review failed. Original summary: The collection features a specific post from `nonbinary.computer` detailing a domestic incident: a 'girl who pooed in front of the bedroom door last night.' The core themes are domestic chaos, trust issues (as the door was locked because...

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 696
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 16 / estimated 16
- Collection Evidence [review_evidence]: used 142 / estimated 142
- Proposed Summary [parent_search_results]: used 263 / estimated 263

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
collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: recent_posts_unaddressed

matched_item[0] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpt6ubc2ac2w
body: girl who pooed in front of the bedroom door last night. door was locked because she cannot be trusted with air mattresses.

hopefully now that there's no more big changes other than all our furniture showing up, she'll be able to de-stress properly.

## Proposed Summary
post: LLM-selected post in Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The collection features a specific post from `nonbinary.computer` detailing a domestic incident: a 'girl who pooed in front of the bedroom door last night.' The core themes are domestic chaos, trust issues (as the door was locked because 'she cannot be trusted with air mattresses'), and the hope for emotional relief, as the author hopes this situation will allow her to 'de-stress properly.' Although the collection does not explicitly show replies, the evidence is narrow, focusing solely on the original statement. This post is highlighted as a 'Recent top-level post' by the actor, suggesting it is a key piece of content. The evidence is very narrow, capturing only the initial declaration of the event rather than the ensuing conversation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpt6ubc2ac2w
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
