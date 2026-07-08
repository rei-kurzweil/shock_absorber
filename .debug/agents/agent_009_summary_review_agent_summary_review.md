# Agent Debug

- agent_id: 9
- agent_type: SummaryReviewAgent
- agent_kind: SummaryReview
- label: summary review
- status: failed
- parent_agent_id: 8
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
- used_input_tokens: 569
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 35 / estimated 35
- Harness Scope Assessment [local_task]: used 67 / estimated 67
- Collection Evidence [review_evidence]: used 72 / estimated 72
- Proposed Summary [parent_search_results]: used 12 / estimated 12

## Rendered Context Window

```text
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `summary` result before it is used by parent `llm_search` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
Summarize the 50 most recent replies received by 2gd4.me, focusing on the general tone and common themes of the replies.

## Harness Scope Assessment
requested_scope: count 50
required_total_items: 3
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 3
current_window_offset: 3
current_window_size: 0

## Collection Evidence
collection_id: recent_replies_received:did:plc:edzlnzvoztauuygch4z5fvl3
collection_label: Recent replies received by did:plc:edzlnzvoztauuygch4z5fvl3 (items 0-3 of 3)
collection_kind: recent_replies_received
search_window_offset: 3
search_window_total_items: 0

## Proposed Summary
No matching cached posts.
```
