# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
- status: warning
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2

## Result Summary

review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
summary: The strongest grounded evidence in this moderation-list collection centers on 10 selected records, with repeated signals around Follows of @norvid-studies.bsky.social, Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts., clearsky, list_name: Follows of @norvid-studies.bsky.social, Follows of @godoglyness.bsky.social. The matched record text also includes descriptions such as: "Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts." "Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.". This fallback summary is derived directly from those matched records because the model response did not yield a usable structured `summary:` field.
search_result_1_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
search_result_1_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_2_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfo67mop2p
search_result_2_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_3_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfwqzzsx2n
search_result_3_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_4_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbgozpuxf2s
search_result_4_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_5_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbh2zglpr2h
search_result_5_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_6_uri: https://bsky.app/profile/did:plc:2bij7yypmcuvwyz4gyqwtluy/lists/3lbxfscjqno2d
search_result_6_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_7_uri: https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n
search_result_7_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_8_uri: https://bsky.app/profile/did:plc:2u5f43ezqz2u6j32wplqxeup/lists/3llaqm3tnvh2k
search_result_8_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_9_uri: https://bsky.app/profile/did:plc:565ebob5f6hw33hjdkxty6qj/lists/3k7wlmyybyk23
search_result_9_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_10_uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mfxevpxejj2w
search_result_10_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1971
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 1595 / estimated 1595
- Search Prompt [local_task]: used 15 / estimated 15

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
collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
label: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
collection_kind: clearsky_lists
item_count: 25
last_refreshed_at: 0
actor_did: did:plc:3deilm3cxnqundoo227xudg2
refresh_ttl_seconds: 900
metadata.collection_kind: clearsky_lists
metadata.search_window_total_items: 100
metadata.search_window_size: 25
metadata.actor_did: did:plc:3deilm3cxnqundoo227xudg2
metadata.search_window_offset: 0

item[0]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
author: clearsky
type: moderation_list
list_name: Follows of @norvid-studies.bsky.social
list_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.

item[1]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfo67mop2p
author: clearsky
type: moderation_list
list_name: Follows of @norvid-studies.bsky.social
list_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.

item[2]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfwqzzsx2n
author: clearsky
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

item[3]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbgozpuxf2s
author: clearsky
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

item[4]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbh2zglpr2h
author: clearsky
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

item[5]
uri: https://bsky.app/profile/did:plc:2bij7yypmcuvwyz4gyqwtluy/lists/3lbxfscjqno2d
author: clearsky
type: moderation_list
list_name: AI, Crypto, & Ratcult Shitheads
list_description: 

item[6]
uri: https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n
author: clearsky
type: moderation_list
list_name: The Great AI - NFT - CRYPTO Cull
list_description: If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will.

item[7]
uri: https://bsky.app/profile/did:plc:2u5f43ezqz2u6j32wplqxeup/lists/3llaqm3tnvh2k
author: clearsky
type: moderation_list
list_name: LUM
list_description: 

item[8]
uri: https://bsky.app/profile/did:plc:2x5qwlfl2s3hdhsssrgrctzt/lists/3ls675z3x572o
author: clearsky
type: moderation_list
list_name: -
list_description: 

item[9]
uri: https://bsky.app/profile/did:plc:3ra4dxf4rwet2urznakt2sm4/lists/3mmiew6l3zh2t
author: clearsky
type: moderation_list
list_name: Uniquely Insightful
list_description: People whose viewpoints are worthy of serious consideration due to their repeated proof of self-examination

item[10]
uri: https://bsky.app/profile/did:plc:565ebob5f6hw33hjdkxty6qj/lists/3k7wlmyybyk23
author: clearsky
type: moderation_list
list_name: unmissable
list_description: Just some people whose posts I don't want to miss

item[11]
uri: https://bsky.app/profile/did:plc:565ebob5f6hw33hjdkxty6qj/lists/3lar7ge4qyn2c
author: clearsky
type: moderation_list
list_name: ML peeps
list_description: People who post about machine learning in my network

item[12]
uri: https://bsky.app/profile/did:plc:5v4pjxmdet7k3cppv5lkhbzj/lists/3lgegeoklpb2p
author: clearsky
type: moderation_list
list_name: XR
list_description: 

item[13]
uri: https://bsky.app/profile/did:plc:5zcs77lwuzzv5djzidamnhsk/lists/3lkde6qwuon2o
author: clearsky
type: moderation_list
list_name: Software
list_description: 

item[14]
uri: https://bsky.app/profile/did:plc:66m62ib4x3ksrd565uafp3qx/lists/3kdkq72umpl2e
author: clearsky
type: moderation_list
list_name: 3
list_description: 

item[15]
uri: https://bsky.app/profile/did:plc:6c2iefym5cdi24vtwmkcqnki/lists/3micxvp23so2j
author: clearsky
type: moderation_list
list_name: lyran thoroughbred
list_description: certified genuine lyran weavers. created by lyra maristela, we were sent to earth to progress technology to the point of recreating our universe, stelanet, on this planet :3

item[16]
uri: https://bsky.app/profile/did:plc:6vh4jl6kgmjatxccd63syc2i/lists/3mehv6iw4yw2z
author: clearsky
type: moderation_list
list_name: Clankers, Polluters, Thieves
list_description: anthonymoser.github.io/writing/ai/h...

item[17]
uri: https://bsky.app/profile/did:plc:6z7uxrayuc5fugnzwtkbkuev/lists/3lbsjj4didv2e
author: clearsky
type: moderation_list
list_name: VR
list_description: 

item[18]
uri: https://bsky.app/profile/did:plc:7l2rn4ozedxwth3goe3zsasg/lists/3lafejflekr2w
author: clearsky
type: moderation_list
list_name: Favs
list_description: In progress, making a custom feed of my favorite ppl

item[19]
uri: https://bsky.app/profile/did:plc:7nf3vqbvea5gpbet3kmibxpm/lists/3lvunqkqtlt2t
author: clearsky
type: moderation_list
list_name: Gen AI commentary (feed)
list_description: Feed list

item[20]
uri: https://bsky.app/profile/did:plc:7tsv4wd4ggnv7zctvt3eqyj7/lists/3lr4uycn6z72c
author: clearsky
type: moderation_list
list_name: cunts
list_description: Personal block list for people I find to be cunts and don't want to see anymore. Die mad about it.

item[21]
uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
author: clearsky
type: moderation_list
list_name: ai and llm
list_description: more LLM focused than my other computer science list

item[22]
uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mfwscsz5xo2w
author: clearsky
type: moderation_list
list_name: Cluster Gold (18)
list_description: Largest mutual-follow clique + expanded circle, found by mino.mobi/cluster

item[23]
uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mfxevpxejj2w
author: clearsky
type: moderation_list
list_name: Cluster (163)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[24]
uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mhrbb6iqxi2g
author: clearsky
type: moderation_list
list_name: Gold Cluster (92)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster


## Search Prompt
What lists is jcorvinus.bsky.social on?
```
