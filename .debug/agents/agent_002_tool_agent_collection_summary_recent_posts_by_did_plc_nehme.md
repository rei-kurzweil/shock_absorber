# Agent Debug

- agent_id: 2
- agent_type: ToolAgent
- label: collection summary: Recent posts by did:plc:nehmem7iy5fyifkqxjpcnj6e
- lifecycle_status: completed
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: recent_posts:did:plc:nehmem7iy5fyifkqxjpcnj6e

## Result Summary

diagnostic: collection_summary_planner accepted 0 page summaries and 5 raw-window fallbacks; collection_summary_notes produced final scope summary
covered_window_offsets: 0, 50, 100, 150, 200
covered_post_count: 229
planner_updates: 0
coherent_pages: 5
review_status: pass
review_grounded: true
review_sufficient: true
review_reason: collection_summary_notes produced a partial scope summary after considering 229 posts before exhaustion.
review_repair_needed: false
review_additional_pages_needed: false
review_required_total_items: 300
post: Summary of Recent posts by did:plc:nehmem7iy5fyifkqxjpcnj6e
summary: Grounded raw-window fallback preserved 5 page(s), but no final synthesis was produced.
window_offset: 0
window_size: 229
page_index: 0
page_size: 50
collection_total_items: 229
has_more: false
source_exhausted: true
concatenated_window_summaries:


## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 88
- truncated: false

## Included Sections

- Task [generic]: used 20 / estimated 20
- Collection [generic]: used 57 / estimated 57

## Rendered Context Window

```text
Instructions:
Raw window fallback context

## Task
summarize the most recent 300 posts by this actor into 6 paragraphs

## Collection
collection_id: recent_posts:did:plc:nehmem7iy5fyifkqxjpcnj6e
collection_label: Recent posts by did:plc:nehmem7iy5fyifkqxjpcnj6e (items 201-229 of 229)
item_count: 29
actor_did: did:plc:nehmem7iy5fyifkqxjpcnj6e
```
