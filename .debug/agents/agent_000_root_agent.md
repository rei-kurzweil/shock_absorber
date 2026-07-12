# Agent Debug

- agent_id: 0
- agent_type: RootAgent
- label: Root Unit
- lifecycle_status: completed
- parent_agent_id: <none>
- child_agent_ids: 1

## Result Summary

Overall commentary across Recent posts by did:plc:nehmem7iy5fyifkqxjpcnj6e:
Grounded raw-window fallback preserved 5 page(s), but no final synthesis was produced.

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 832
- truncated: false

## Included Sections

- Tools [tool_definitions]: used 635 / estimated 635
- Search Hints [ui_context]: used 74 / estimated 74
- Current UI Context [ui_context]: used 80 / estimated 80
- Current Task [local_task]: used 22 / estimated 22

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

Tool: search
Description: Search Bluesky at a high level for selective grounded evidence about handles/users or broader topics.
Arguments:
- query (string, required): The user's search request in natural language, such as who a handle is or what Bluesky posts say about a topic.
When To Use: Use when you need selective Bluesky-grounded evidence about one or more handles/users, or about a broader topic that requires searching posts.
Notes:
- The root agent only supplies the high-level query; the harness decides whether to do handle lookup, actor-centric collection search, or broader Bluesky post search.
- If the query names a handle or user, the search should anchor on that actor's profile and may inspect posts for grounding.
- If the query is topical rather than person-centric, the search may use Bluesky-wide post search and normalize the results into a collection before running narrower LLM search.
- When a collection contains structured fields such as `list_name` or `list_description`, use those exact fields as evidence instead of inventing new labels or categories.
- Returns one synthesized block with a chosen URI plus grounded evidence snippets or repeated themes from the matching items.

Tool: summary
Description: Summarize an actor-backed Bluesky collection with broad grounded coverage, such as the last 50 posts by a handle.
Arguments:
- query (string, required): The user's coverage-oriented summary request in natural language.
When To Use: Use when the user explicitly asks for broad coverage such as summarizing recent posts, replies, pages, or the last N posts by an actor.
Notes:
- The root agent only supplies the high-level query; the harness resolves the actor, hydrates the actor scope, picks the target collection, and delegates coverage work to `collection_summary`.
- The first summary slice is actor-centric and defaults to the actor's `recent_posts` collection unless the query explicitly asks for replies or another collection target.
- Returns a grounded coverage summary with covered item URIs and source-exhaustion metadata when applicable.

## Search Hints
The selected actor is did:plc:nehmem7iy5fyifkqxjpcnj6e. Use `search` with a natural-language `query` when you need selective grounded evidence about this actor or related topics. Use `summary` when you need broad coverage such as summarizing the actor's recent posts or replies.

## Current UI Context
reason: like
author_handle: sam.bunnylabs.dev
author_did: did:plc:nehmem7iy5fyifkqxjpcnj6e
uri: at://did:plc:nehmem7iy5fyifkqxjpcnj6e/app.bsky.feed.like/3mqdqzcwqwi2u
indexed_at: 2026-07-11 03:49:29.875 +00:00
reason_subject: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.feed.post/3mqdqpwdjh22g

## Current Task
summarize the most recent 300 posts by this actor into 6 paragraphs
```
