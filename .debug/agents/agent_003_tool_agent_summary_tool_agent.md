# Agent Debug

- agent_id: 3
- agent_type: ToolAgent
- agent_kind: Summary
- label: summary tool agent
- status: completed
- parent_agent_id: 0
- child_agent_ids: <none>
- tool_name: summary

## Result Summary

status: failed
reason: no summary pages were processed

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 233
- truncated: false

## Included Sections

- Original Summary Query [local_task]: used 25 / estimated 25
- Summary Result [upstream_results]: used 19 / estimated 19

## Rendered Context Window

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
summarize the last 400 posts by schizanon.bsky.social into 4 paragraphs

## Summary Result
status: failed
reason: no summary pages were processed
```
