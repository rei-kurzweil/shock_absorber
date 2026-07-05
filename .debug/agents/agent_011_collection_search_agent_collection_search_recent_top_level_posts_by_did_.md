# Agent Debug

- agent_id: 11
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Recent top-level posts by did:plc:6lwfvmss45d7j7fot34v2kw5
- status: completed
- parent_agent_id: 7
- child_agent_ids: <none>
- collection_id: recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5

## Result Summary

post: LLM-selected post in Recent top-level posts by did:plc:6lwfvmss45d7j7fot34v2kw5
summary: Grounded evidence centers on: I decided that since AI Harnesses can be used with both closed and open LLMs that I shouldn't include them in my "Open Weights LLMs" feed.; What is an AI "harness"?.
search_result_1_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpoimyfzy22b
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5
search_result_2_uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpofi5k3mk2z
search_result_2_source_collection_id: recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 619
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
collection_id: recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5
label: Recent top-level posts by did:plc:6lwfvmss45d7j7fot34v2kw5
collection_kind: recent_posts_unaddressed
item_count: 2
last_refreshed_at: 1783011272
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
refresh_ttl_seconds: 900
related_collections: pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
metadata.source_feed: author_feed

item[0]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpoimyfzy22b
author: schizanon.bsky.social
body: I decided that since AI Harnesses can be used with both closed and open LLMs that I shouldn't include them in my "Open Weights LLMs" feed. 

Instead I made a new feed that's just about "AI Harnesses", pin it!

bsky.app/profile/did:...
link: https://bsky.app/profile/did:plc:6lwfvmss45d7j7fot34v2kw5/feed/AI-Harnesses

item[1]
uri: at://did:plc:6lwfvmss45d7j7fot34v2kw5/app.bsky.feed.post/3mpofi5k3mk2z
author: schizanon.bsky.social
body: What is an AI "harness"?

I know of course, I just want to see if you do...


## Search Prompt
what is the sentiment toward schizanon.bsky.social and what lists are they on, especially negative ones, and how does this contrast with the people they reply to?
```
