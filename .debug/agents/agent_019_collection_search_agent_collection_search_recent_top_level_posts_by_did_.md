# Agent Debug

- agent_id: 19
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Recent top-level posts by did:plc:3deilm3cxnqundoo227xudg2
- status: completed
- parent_agent_id: 15
- child_agent_ids: <none>
- collection_id: recent_posts_unaddressed:did:plc:3deilm3cxnqundoo227xudg2

## Result Summary

post: LLM-selected post in Recent top-level posts by did:plc:3deilm3cxnqundoo227xudg2
summary: The collection shows recent posts from `jcorvinus.bsky.social` focusing on technical development, specifically creating a \"neat axis moving sphere constrained gizmo thingie\" and working on making it movable via \"grab handles.\" Another post mentions \"Mitigating the ram shortage by switching to the alternative: write only memory.\" While the collection does not explicitly show replies *to* jcorvinus.bsky.social, it does show posts from other authors that are thematically related to technical/creative work (e.g., face tracking, Gaussian splatting). There is no explicit negative sentiment or negative label directed at jcorvinus.bsky.social in these five items, but the themes are highly technical.
search_result_1_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mplp23axos2j
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:3deilm3cxnqundoo227xudg2
search_result_2_uri: at://did:plc:m2642o675ocuajh3qteocwjs/app.bsky.feed.post/3mplnintri22d
search_result_2_source_collection_id: recent_posts_unaddressed:did:plc:3deilm3cxnqundoo227xudg2
search_result_3_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mpjdhcnyns23
search_result_3_source_collection_id: recent_posts_unaddressed:did:plc:3deilm3cxnqundoo227xudg2

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 865
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
collection_id: recent_posts_unaddressed:did:plc:3deilm3cxnqundoo227xudg2
label: Recent top-level posts by did:plc:3deilm3cxnqundoo227xudg2
collection_kind: recent_posts_unaddressed
item_count: 5
last_refreshed_at: 1783011273
actor_did: did:plc:3deilm3cxnqundoo227xudg2
refresh_ttl_seconds: 900
related_collections: pinned_posts:did:plc:3deilm3cxnqundoo227xudg2
metadata.source_feed: author_feed

item[0]
uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mplp23axos2j
author: jcorvinus.bsky.social
body: Gm everyone. Managed to make a neat axis moving sphere constrained gizmo thingie last night. It's not done but it is neat. Today I work on making it so you can move a distance along said axis using grab handles

item[1]
uri: at://did:plc:m2642o675ocuajh3qteocwjs/app.bsky.feed.post/3mplnintri22d
author: hashe.bsky.social
body: Super excited to share my face tracking for #SIUSIU3D !

The add-on comes with a slider to change the style of the expressions and secrets :o
I hope you'll have as much fun with it as I've had making it :>

Avaliable now:
hashedits.booth.pm/items/8560768
ko-fi.com/s/71dcabc8e7

#HashFT
tag: SIUSIU3D
link: https://hashedits.booth.pm/items/8560768
link: https://ko-fi.com/s/71dcabc8e7
tag: HashFT

item[2]
uri: at://did:plc:cp5hnfgqbgjdbizyqyp4zgdl/app.bsky.feed.post/3mpjhllhles2n
author: antiali.as
body: man goes to library and asks for books about paranoia

librarian whispers, "they're right behind you"

item[3]
uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mpjdhcnyns23
author: jcorvinus.bsky.social
body: Mitigating the ram shortage by switching to the alternative: write only memory

item[4]
uri: at://did:plc:5goqcqeqnn4wjycr2bblfl6a/app.bsky.feed.post/3mpi4wou6nk2l
author: danybittel.bsky.social
body: High magnification Gaussian splatting is now working! My first attempts all failed, now with a proper lens it just works. Still need to improve diffraction (blur / haze) and pick a nicer subject. #3dgs
tag: 3dgs


## Search Prompt
how people tend to reply to jcorvinus.bsky.social and what lists jcorvinus.bsky.social is on, specifically looking for negative sentiment or negative labels directed at jcorvinus.bsky.social
```
