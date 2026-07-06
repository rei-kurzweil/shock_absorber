# Agent Debug

- agent_id: 1
- agent_type: ToolAgent
- agent_kind: LlmSearch
- label: llm_search tool agent
- status: completed
- parent_agent_id: 0
- child_agent_ids: 2
- tool_name: llm_search

## Result Summary

No matching cached posts.

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 224
- truncated: false

## Rendered Context Window

```text
Instructions:
Synthesize grounded per-collection search results. Keep collection boundaries explicit, compare what each collection supports, and retain failures as diagnostics. Return a compact combined result block with a cross-collection `summary:` plus the strongest real `selected_result_*` anchor when available. Do not invent evidence beyond the provided child results.

## Original Search Query
what are people saying about elsyluna.bsky.social's post (at://did:plc:hzijw7nigriwppf7eeb3k7ar/app.bsky.feed.post/3mpsf6yh3ql2s)? check sentiment and replies.

## Per-Collection Results
collection_id: global_search_posts:8276cadf224b600d
collection_label: Global Bluesky search results for "what are people saying about elsyluna.bsky.social's post (at://did:plc:hzijw7nigriwppf7eeb3k7ar/app.bsky.feed.post/3mpsf6yh3ql2s)? check sentiment and replies."
status: ok
No matching cached posts.
```
