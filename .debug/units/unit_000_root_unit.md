# Unit Debug

- instance_id: unit-1
- unit_id: root
- unit_kind: root
- label: Root Unit
- lifecycle_status: completed
- parent_unit: <none>
- active_node: <none>
- visit_count: 0
- visited_nodes: <none>
- selected_output_port: <none>
- blocked_on_child: <none>

## Local State

<none>

## Result Summary

Overall commentary across Recent posts by did:plc:37lcyqfy4d5r25jvgtabe5kn:
Page 1 preserved grounded raw evidence around "KYOOOOT 😸\u{fe0f}"; "these artworks is beautiful btw! :D"; "Halloo everynyan!".

Page 2 preserved grounded raw evidence around "scary! youtu.be/tFAd8jtPceQ?..."; "the samee!! /v♥\u{fe0f}v/"; "did you know you can drag this one if you hold your mouse, at the empty blue border? 🤣\u{fe0f}🤣\u{fe0f}🤣\u{fe0f}".

Page 3 preserved grounded raw evidence around "OH and the last part you wrote? me too. i..proobably wouldnt want to know either."; "so im thinking..maybe one person is hater/spammy,on my list(sadly, i dont know who or what they type i just know they did)"; "oh! maybe but.. to me i have done so only a few can reply".

## Transition History

<none>

## Context Window

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
Use `search` with a natural-language `query` when you need selective Bluesky-grounded evidence about a handle/user or broader topic questions. Use `summary` for coverage-oriented requests like summarizing the last 50 posts by an actor.

## Current UI Context
No actor is currently selected in the UI.

## Current Task
summarize 150 posts by loneicewolf.bsky.social into 3 paragraphs
```
