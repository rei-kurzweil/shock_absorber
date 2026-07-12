# Agent Debug

- agent_id: 3
- agent_type: SearchReviewAgent
- agent_kind: SearchReview
- label: search review
- lifecycle_status: failed
- result_status: fail
- parent_agent_id: 2
- child_agent_ids: <none>

## Result Summary

status: fail
grounded: false
sufficient: false
reason: No usable `summary:` paragraph exists.
repair_needed: false
additional_pages_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 370
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 12 / estimated 12
- Collection Evidence [review_evidence]: used 49 / estimated 49
- Proposed Summary [upstream_results]: used 12 / estimated 12

## Rendered Context Window

```text
Instructions:
You are the internal `search_review` agent.

Your job is to review one selective-evidence `collection_search` result before it is used by parent `search` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- `repair_needed: true` or `repair_needed: false`
- optional `repair_instructions: ...`

Rules:

- Review the summary against the actual selected evidence provided.
- Fail if the summary is missing, mostly metadata, mostly identifiers, unsupported by the selected records, or too thin to support parent synthesis.
- Pass only when the summary is one grounded paragraph and uses real phrases, quotes, list names, descriptions, or post/reply text from the matched records.
- When the prompt asks for sentiment, reputation, contrast, or list interpretation, expect the summary to preserve that distinction with grounded evidence.
- If the summary can likely be fixed by rewriting it from the existing selected records, set `repair_needed: true` and provide short repair instructions.
- Do not rewrite the summary yourself in this step.

## Search Prompt
what has CAT_DEBUG_RAYCAST=1

## Collection Evidence
collection_id: global_search_posts:b3019adb7ccc206
collection_label: Global Bluesky search results for "what has CAT_DEBUG_RAYCAST=1"
collection_kind: global_search_posts

## Proposed Summary
No matching cached posts.
```
