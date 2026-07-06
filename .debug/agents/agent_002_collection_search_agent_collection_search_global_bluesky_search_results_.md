# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Global Bluesky search results for "what are people saying about elsyluna.bsky.social's post (at://did:plc:hzijw7nigriwppf7eeb3k7ar/app.bsky.feed.post/3mpsf6yh3ql2s)? check sentiment and replies."
- status: completed
- parent_agent_id: 1
- child_agent_ids: <none>
- collection_id: global_search_posts:8276cadf224b600d

## Result Summary

No matching cached posts.

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 201
- truncated: false

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully. Return a compact result block with `uri:`, `title:`, and `analysis:` fields. Always choose one anchor item with a real `uri:` from the collection. The `analysis:` field is evidence-only: quote exact short snippets, list names, list descriptions, or other text taken from the collection, and note repeated themes across multiple items when relevant. For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context; do not invent a separate label field unless it appears explicitly in the collection text. Do not add higher-level interpretation beyond brief grouping of repeated evidence. Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.
```
