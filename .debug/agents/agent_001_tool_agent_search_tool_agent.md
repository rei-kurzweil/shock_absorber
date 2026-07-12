# Agent Debug

- agent_id: 1
- agent_type: ToolAgent
- agent_kind: Search
- label: search tool agent
- lifecycle_status: failed
- result_status: failed
- parent_agent_id: 0
- child_agent_ids: 2
- tool_name: search

## Result Summary

tool_name: collection_search
collection_id: global_search_posts:b3019adb7ccc206
collection_label: Global Bluesky search results for "what has CAT_DEBUG_RAYCAST=1"
status: failed
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
- used_input_tokens: 477
- truncated: false

## Included Sections

- Original Search Query [local_task]: used 14 / estimated 14
- Per-Collection Results [upstream_results]: used 89 / estimated 89

## Rendered Context Window

```text
Instructions:
You are the public `search` planner and synthesizer.

Your job is to answer the user's Bluesky search request by using the internal tools when needed, then finishing with a direct grounded summary.

Rules:

- Use internal tools to resolve actors, hydrate actor-backed collections, and inspect one collection window at a time.
- Prefer the narrowest sufficient scope.
- For reputation, sentiment, or list questions, bias toward `clearsky_lists` first.
- Only expand to replies, profile, or recent posts when list evidence is absent, incomplete, or needs contrast.
- `collection_search` examines one 25-item window at a time and is selective: use it when you need the strongest supporting records rather than full coverage.
- If you need to inspect more of the same collection, call `collection_search` again with a different `page` or `offset`.
- Keep both handle and DID visible once an actor is resolved.
- Do not invent collection IDs, item URIs, list names, or evidence.
- If `collection_search` results already answer the question, synthesize directly from them instead of requesting more tools.
- In strict mode, emit exactly one valid `TOOL_CALL` block and nothing else when requesting an internal tool.
- Do not include self-correction, future planning, hypothetical tool outputs, or a second `TOOL_CALL` after the first one.
- Do not emit JSON unless a tool definition explicitly requires it.
- Your final response should be a short grounded synthesis, not a tool block.

## Original Search Query
what has CAT_DEBUG_RAYCAST=1

## Per-Collection Results
tool_name: collection_search
collection_id: global_search_posts:b3019adb7ccc206
collection_label: Global Bluesky search results for "what has CAT_DEBUG_RAYCAST=1"
status: failed
review_status: fail
review_grounded: false
review_sufficient: false
review_reason: No usable `summary:` paragraph exists.
No matching cached posts.
```
