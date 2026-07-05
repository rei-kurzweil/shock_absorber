# Agent Debug

- agent_id: 3
- agent_type: CollectionSearchAgent
- label: collection search: Pinned posts by did:plc:3deilm3cxnqundoo227xudg2
- status: completed
- parent_agent_id: 1
- child_agent_ids: <none>
- collection_id: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2

## Result Summary

post: LLM-selected post in Pinned posts by did:plc:3deilm3cxnqundoo227xudg2
summary: Grounded evidence centers on: I moved all my blog articles from medium to my own site! I didn't like that Medium doesn't have any options for letting AI get access to / train on my articles, so Monday and I vibecoded a place for them..
search_result_1_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3lqks73aop227
search_result_1_source_collection_id: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 540
- truncated: false

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return a compact result block with `title:`, `summary:`, and up to four `uri:` lines for the chosen search results.

Every `uri:` must be a real item from the collection.

The `summary:` field is required. It should stay brief and grounded.

Use the `uri:` lines to point at the strongest supporting records.

If fewer than four real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not add higher-level interpretation beyond brief grouping of repeated evidence in the grounded summary.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2
label: Pinned posts by did:plc:3deilm3cxnqundoo227xudg2
collection_kind: pinned_posts
item_count: 1
last_refreshed_at: 0
actor_did: did:plc:3deilm3cxnqundoo227xudg2
refresh_ttl_seconds: 900
metadata.collection_kind: pinned_posts
metadata.actor_did: did:plc:3deilm3cxnqundoo227xudg2

item[0]
uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3lqks73aop227
author: jcorvinus.bsky.social
body: I moved all my blog articles from medium to my own site! I didn't like that Medium doesn't have any options for letting AI get access to / train on my articles, so Monday and I vibecoded a place for them.

jcorvinus.black

The text is all copyright free. It's entirely a static site, no scripts
link: https://jcorvinus.black/


## Search Prompt
What is the overall sentiment towards this user (did:plc:3deilm3cxnqundoo227xudg2) based on the descriptions of the Clearsky lists they are a member of? Specifically, highlight any lists whose descriptions suggest a negative or critical sentiment towards the user.
```
