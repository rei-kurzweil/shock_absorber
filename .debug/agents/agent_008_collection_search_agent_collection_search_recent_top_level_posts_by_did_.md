# Agent Debug

- agent_id: 8
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
- status: completed
- parent_agent_id: 1
- child_agent_ids: 9
- collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Result Summary

review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around girl who pooed in front of the bedroom door last night. door was locked because she cannot be trusted with air mattresses.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpt6ubc2ac2w
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 933
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 567 / estimated 567
- Search Prompt [local_task]: used 18 / estimated 18

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return a grounded result block with `title:`, `summary:`, and up to four `uri:` lines for the chosen search results.

Every `uri:` must be a real item from the collection.

The `summary:` field is required. It must be one grounded paragraph of roughly 100-200 words.

That paragraph should include:

- the main repeated themes
- the strongest exact phrases or list names
- any meaningful split, ambiguity, or contrast inside the collection
- a short note about how broad or narrow the matched evidence seems when that matters

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
collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g
label: Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: recent_posts_unaddressed
item_count: 7
last_refreshed_at: 1783190620
actor_did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
refresh_ttl_seconds: 900
related_collections: pinned_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g
metadata.source_feed: author_feed

item[0]
uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpt6ubc2ac2w
author: nonbinary.computer
body: girl who pooed in front of the bedroom door last night. door was locked because she cannot be trusted with air mattresses.

hopefully now that there's no more big changes other than all our furniture showing up, she'll be able to de-stress properly.

item[1]
uri: at://did:plc:zjnskcrzfd35gtlso2agki5y/app.bsky.feed.post/3mpsz4yazbc2k
author: apex.atproto.ceo
body: yak lasering

item[2]
uri: at://did:plc:hvys4nxbae54ntu32tad3t2p/app.bsky.feed.post/3mprjmcmir22l
author: the.moonwit.ch
body: oomf

item[3]
uri: at://did:plc:5wm25vgenhgut3iqfjf4ozj5/app.bsky.feed.post/3mprfgv7bu22b
author: eugenevinitsky.bsky.social
body: As an engineer we're not asking "what does exist", which is more the purview of science, but "what should exist" and so it comes with a corresponding moral requirement to try to bring the right things into existence.

item[4]
uri: at://did:plc:2zmxikig2sj7gqaezl5gntae/app.bsky.feed.post/3mpr5mber622f
author: iame.li
body: atproto community I love you but it's time for a gentle roasting

Streamplace has been live in the App Store and Play Store for TWO YEARS as of this month

One of you could have published a social-app fork by now, I believe in you

item[5]
uri: at://did:plc:h5zkfkc35wkb53udnemrica5/app.bsky.feed.post/3mpqzyo32as27
author: olufemiotaiwo.bsky.social
body: African, can confirm: every moment of your life has been spent during an atrocity somewhere, perpetuated or exacerbated by a larger number of countries and corporations than many care to admit.

item[6]
uri: at://did:plc:h6tcd37yr7vk33uuisbidqvw/app.bsky.feed.post/3mpqt6tsen22m
author: jefferyharrell.bsky.social
body: If you use AI like a hammer, don't be surprised if the output has dents. That's a good line. I'm stealing that.


## Search Prompt
Sentiment toward nonbinary.computer's recent posts?
```
