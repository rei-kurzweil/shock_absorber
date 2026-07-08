# Agent Debug

- agent_id: 8
- agent_type: CollectionSummaryTool
- agent_kind: CollectionSummary
- label: collection summary: Recent replies received by did:plc:edzlnzvoztauuygch4z5fvl3 (items 0-3 of 3)
- status: failed
- parent_agent_id: 1
- child_agent_ids: 9
- collection_id: recent_replies_received:did:plc:edzlnzvoztauuygch4z5fvl3

## Result Summary

review_status: fail
review_grounded: false
review_sufficient: false
review_reason: No usable `summary:` paragraph exists.
review_repair_needed: false
review_additional_pages_needed: false
No matching cached posts.

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 201
- truncated: false

## Included Sections


## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully. Return a compact result block with `uri:`, `title:`, and `analysis:` fields. Always choose one anchor item with a real `uri:` from the collection. The `analysis:` field is evidence-only: quote exact short snippets, list names, list descriptions, or other text taken from the collection, and note repeated themes across multiple items when relevant. For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context; do not invent a separate label field unless it appears explicitly in the collection text. Do not add higher-level interpretation beyond brief grouping of repeated evidence. Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.
```
