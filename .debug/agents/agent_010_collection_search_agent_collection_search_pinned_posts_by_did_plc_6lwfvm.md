# Agent Debug

- agent_id: 10
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Pinned posts by did:plc:6lwfvmss45d7j7fot34v2kw5
- status: completed
- parent_agent_id: 7
- child_agent_ids: <none>
- collection_id: pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5

## Result Summary

post: LLM-selected post in Pinned posts by did:plc:6lwfvmss45d7j7fot34v2kw5
summary: The single pinned post by `schizanon.bsky.social` conveys a positive, action-oriented sentiment regarding Direct Relief mobilizing medical aid deliveries to address immediate and near-term health requests. The post is strongly associated with the tags: **#Venuzuela #Caracas #Earthquake #LaGuaira**. The collection does not show any replies to contrast the sentiment with, but the post itself is a direct update from the author, linking to `www.directrelief.org/2026/06/vene...`.
search_result_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mp3h5kbfvc25
search_result_1_source_collection_id: pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 580
- truncated: false

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return a grounded result block with `title:`, `summary:`, and up to four `uri:` lines for the chosen search results.

Every `uri:` must be a real item from the collection.

The `summary:` field is required. It should be dense and grounded: include the main repeated themes, the strongest exact phrases or list names, and any meaningful split or ambiguity inside the collection.

Use the `uri:` lines to point at the strongest supporting records.

If fewer than four real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not add higher-level interpretation beyond grouping repeated evidence and short contrasts that are directly supported by the collection text.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
label: Pinned posts by did:plc:6lwfvmss45d7j7fot34v2kw5
collection_kind: pinned_posts
item_count: 1
last_refreshed_at: 0
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
refresh_ttl_seconds: 900
metadata.actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
metadata.collection_kind: pinned_posts

item[0]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mp3h5kbfvc25
author: schizanon.bsky.social
body: Direct Relief is in communication with local and regional organizations about medical needs and will mobilize medical aid deliveries to help address immediate and near-term health requests.

#Venuzuela #Caracas #Earthquake #LaGuaira 

www.directrelief.org/2026/06/vene...
tag: Venuzuela
tag: Caracas
tag: Earthquake
tag: LaGuaira
link: https://www.directrelief.org/2026/06/venezuela-earthquake-caracas-damage/


## Search Prompt
what is the sentiment toward schizanon.bsky.social and what lists are they on, especially negative ones, and how does this contrast with the people they reply to?
```
