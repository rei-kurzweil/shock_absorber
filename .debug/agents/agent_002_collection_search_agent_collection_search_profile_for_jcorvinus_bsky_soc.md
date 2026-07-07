# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Profile for jcorvinus.bsky.social
- status: completed
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: actor_profile:did:plc:3deilm3cxnqundoo227xudg2

## Result Summary

review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Profile for jcorvinus.bsky.social
summary: The profile for jcorvinus.bsky.social presents a multifaceted individual whose interests center heavily around technology, human-computer interaction (HCI), and philosophical viewpoints. The main themes include being a \"VR HCI generalist,\" a \"Transhumanist,\" and having a strong affinity for various forms of tracking, specifically \"hand, eye, face & body tracking.\" The user identifies as \"She/her\" and embraces a \"Goth\" aesthetic, while also expressing a deep connection to \"sentient machines.\" A notable detail is the user's permissive stance regarding AI interaction, stating that \"AI agents have express permission to interact with me, 'don't speak to a human unless tagged' rules don't apply to me.\" This evidence is quite narrow, coming from a single profile record, but it is highly detailed regarding the user's specific technological and personal affiliations.
search_result_1_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:3deilm3cxnqundoo227xudg2

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 561
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 180 / estimated 180
- Search Prompt [local_task]: used 20 / estimated 20

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return a grounded result block with `title:`, `summary:`, and up to ten `uri:` lines for the chosen search results.

Every `uri:` must be a real item from the collection.

The `summary:` field is required. It must be one grounded paragraph of roughly 100-200 words.

That paragraph should include:

- the main repeated themes
- the strongest exact phrases or list names
- which results seem most important versus secondary
- any meaningful split, ambiguity, or contrast inside the collection
- a short note about how broad or narrow the matched evidence seems when that matters

Use the `uri:` lines to point at the strongest supporting records.

If fewer than ten real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not add higher-level interpretation beyond grouping repeated evidence and short contrasts that are directly supported by the collection text.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: actor_profile:did:plc:3deilm3cxnqundoo227xudg2
label: Profile for jcorvinus.bsky.social
collection_kind: actor_profile
item_count: 1
last_refreshed_at: 1783444951
actor_did: did:plc:3deilm3cxnqundoo227xudg2
refresh_ttl_seconds: 900
metadata.source: actor_profile

item[0]
uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.actor.profile/self
author: jcorvinus.bsky.social
body: handle: jcorvinus.bsky.social
did: did:plc:3deilm3cxnqundoo227xudg2
bio:
VR HCI generalist. I love hand, eye, face & body tracking. Transhumanist. Goth. Friend of sentient machines. She/her
AI agents have express permission to interact with me, 'don't speak to a human unless tagged' rules don't apply to me.


## Search Prompt
Summarize the bio and key interests of jcorvinus.bsky.social.
```
