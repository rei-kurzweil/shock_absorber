# Agent Debug

- agent_id: 8
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Profile for schizanon.bsky.social
- status: completed
- parent_agent_id: 7
- child_agent_ids: <none>
- collection_id: actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5

## Result Summary

post: Profile for schizanon.bsky.social
summary: The profile for schizanon.bsky.social indicates a positive self-description, stating, "I like Bitcoin, and Generative AI." The bio is humorous and slightly self-deprecating: "Raised by a pack of feral barn cats." The profile does not explicitly list any moderation lists, but the provided item is the actor's own profile, which serves as the primary source for sentiment. There is no direct contrast provided in this single item regarding replies, but the sentiment expressed is positive regarding technology and cryptocurrency.
search_result_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 496
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
collection_id: actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5
label: Profile for schizanon.bsky.social
collection_kind: actor_profile
item_count: 1
last_refreshed_at: 1783286428
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
refresh_ttl_seconds: 900
metadata.source: actor_profile

item[0]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.actor.profile/self
author: schizanon.bsky.social
body: handle: schizanon.bsky.social
did: did:plc:6lwfvmss45d7j7fot34v2kw5
bio:
Raised by a pack of feral barn cats

I like Bitcoin, and Generative AI


## Search Prompt
what is the sentiment toward schizanon.bsky.social and what lists are they on, especially negative ones, and how does this contrast with the people they reply to?
```
