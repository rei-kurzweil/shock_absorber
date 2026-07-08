# Agent Debug

- agent_id: 0
- agent_type: RootAgent
- label: Root Agent
- status: completed
- parent_agent_id: <none>
- child_agent_ids: 1

## Result Summary

This collection of recent posts by nonbinary.computer covers a diverse range of topics, heavily featuring commentary from the author. Key themes include the challenges of technical work, such as the complexities of "full network backfill" and the nuances of Rust for embedded systems compared to MicroPython/Circui...

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 628
- truncated: false

## Included Sections

- Tools [tool_definitions]: used 426 / estimated 426
- Search Hints [ui_context]: used 76 / estimated 76
- Current UI Context [ui_context]: used 81 / estimated 81
- Current Task [local_task]: used 24 / estimated 24

## Rendered Context Window

```text
Instructions:
Use the available tools only when they materially improve the answer.

## Tools
Tool: read_selected_post
Description: Read the full text and facets for the post or reply currently selected in the UI.
Arguments:

When To Use: Use when the current UI context suggests the selected notification matters, but you need the actual post or reply text before answering.
Notes:
- Reads the currently selected notification from the detail view.
- Returns reply text, reply parent/root URIs, facets, and quoted text when present.

Tool: llm_search
Description: Search Bluesky at a high level, including looking up handles/users or searching posts by topic, then return grounded evidence anchored to real records.
Arguments:
- query (string, required): The user's search request in natural language, such as who a handle is or what Bluesky posts say about a topic.
When To Use: Use when you need Bluesky-grounded evidence about one or more handles/users, or about a broader topic that requires searching posts.
Notes:
- The root agent only supplies the high-level query; the harness decides whether to do handle lookup, actor-centric collection search, or broader Bluesky post search.
- If the query names a handle or user, the search should anchor on that actor's profile and may inspect posts for grounding.
- If the query is topical rather than person-centric, the search may use Bluesky-wide post search and normalize the results into a collection before running narrower LLM search.
- When a collection contains structured fields such as `list_name` or `list_description`, use those exact fields as evidence instead of inventing new labels or categories.
- Returns one synthesized block with a chosen URI plus grounded evidence snippets or repeated themes from the matching items.

## Search Hints
The selected actor is did:plc:yfvwmnlztr4dwkb7hwz55r2g. Use `llm_search` with a natural-language `query` when you need grounded evidence about this actor or related topics. The harness may reuse cached actor collections, load more actor data, or search Bluesky posts globally as needed.

## Current UI Context
reason: like
author_handle: nonbinary.computer
author_did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.like/3mptpp674kc2l
indexed_at: 2026-07-04 18:43:19.470 +00:00
reason_subject: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.feed.post/3mn2n56unyk2v

## Current Task
lets see what this user posts about. summarize the last 25 things she posted
```
