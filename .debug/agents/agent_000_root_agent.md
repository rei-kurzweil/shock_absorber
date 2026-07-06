# Agent Debug

- agent_id: 0
- agent_type: RootAgent
- label: Root Agent
- status: completed
- parent_agent_id: <none>
- child_agent_ids: 1, 10

## Result Summary


The sentiment toward nonbinary.computer appears generally positive and engaged, based on the search results.

**How people reply to her:**
Replies cover a wide range of topics, including:
*   Discussions about **AI/LLMs** and technical topics (e.g., accessibility provisions, AI thumbnails indicating "terrible slop content").
*   Comments on her personal life, such as her finding a **full-time job** at NBCUniversal in Montreal.
*   General commentary on her posts, such as the anecdote about the **girl who pooed in front of the bedroom door**.
*   Replies often involve constructive feedback or detailed opinions on the content she shares.

**What lists she is on (and their sentiment):**
She is on several moderation lists, which indicate the themes she engages with:
*   **Tech:** This is a major list, and the evidence suggests a nuanced approach, as one related list is explicitly marked **"NOT A MUTE LIST."**
*   **AI and LLMs:** This is a prominent technical category, with specific lists like 'ai and llm' (noted as "more LLM focused than my other computer science list") and 'llms' and 'Crypto/AI'.
*   **Behavioral/Social Commentary:** Lists like **'People who immediately block others when they are contradicted'** and **'Be Kind'** suggest she is involved in discussions about social conduct.

**Overall Sentiment Notes:**
The lists themselves suggest she is part of communities that value **technical depth** (Tech, AI/LLMs) and **constructive social interaction** (Be Kind, blocking behavior), which points toward a positive and engaged presence.

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 910
- truncated: false

## Included Sections

- Tools [tool_definitions]: used 693 / estimated 693
- Search Hints [ui_context]: used 76 / estimated 76
- Current UI Context [ui_context]: used 81 / estimated 81
- Current Task [local_task]: used 39 / estimated 39

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

Tool: list_collections
Description: List cached collections, either globally or for one actor.
Arguments:
- actor_did (string, optional): Optional actor DID. If provided, only collections related to that actor are listed.
When To Use: Use when you need to know what cached collections exist before searching or reading an item.
Notes:
- Returns compact collection summaries.
- If `actor_did` is omitted, all cached collections are listed.
- Actor-scoped reply collections may include recent inbound replies from other actors when they have been cached.

Tool: read_collection_item
Description: Read one specific item from a collection in a richer form suitable for loading into context.
Arguments:
- collection_id (string, required): The collection containing the item.
- item_uri (string, required): The URI of the item to read.
When To Use: Use after search when you want one item and its richer details in the active context.
Notes:
- Reads the exact item selected from a search result.
- For reply-oriented collections, returns the synthesized reply record body.

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
what is the sentiment toward this user. how do people reply to her? what lists is she on? (if lists sound negative or positive take note)
```
