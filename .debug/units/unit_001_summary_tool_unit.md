# Unit Debug

- unit_id: summary.public_tool
- unit_kind: public_tool_orchestration
- label: summary tool unit
- lifecycle_status: completed
- parent_unit: Root Unit
- active_node: run_collection_summary
- blocked_on_child: <none>
- tool_name: summary

## Local State

<none>

## Result Summary

tool_name: collection_summary
collection_id: recent_posts:did:plc:nehmem7iy5fyifkqxjpcnj6e
collection_label: Recent posts by did:plc:nehmem7iy5fyifkqxjpcnj6e
status: ok
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


## Context Window

```text
Instructions:
You are the public `summary` orchestrator.

Your job is to summarize an actor-backed Bluesky collection with broad grounded coverage.

Rules:

- Resolve named actors first and keep both handle and DID visible once resolved.
- Hydrate only the actor-backed collections needed for the requested summary.
- Default to `recent_posts` for requests like "last 50 posts" unless the query explicitly asks for replies, moderation lists, or another collection target.
- Use `collection_summary` as the only coverage-oriented worker.
- Do not switch into selective evidence search.
- Preserve the child `collection_summary` result unless a short final restatement is clearly needed.
- Do not invent collection IDs, item URIs, list names, or evidence.

## Original Summary Query
summarize the most recent 300 posts by this actor into 6 paragraphs

## Summary Result
tool_name: collection_summary
collection_id: recent_posts:did:plc:nehmem7iy5fyifkqxjpcnj6e
collection_label: Recent posts by did:plc:nehmem7iy5fyifkqxjpcnj6e
status: ok
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

```
