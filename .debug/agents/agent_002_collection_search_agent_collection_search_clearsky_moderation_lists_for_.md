# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2
- status: completed
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2

## Result Summary

diagnostic: Primary full-collection search failed and a reduced retry view was used instead. Primary failure: HTTP status server error (500 INTERNAL SERVER ERROR) for url (http://127.0.0.1:5000/v1/chat/completions)
review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (reduced retry view)
summary: The collection showcases a diverse set of moderation lists curated by Clearsky, heavily focused on themes related to Artificial Intelligence (AI), technological clusters, and behavioral curation. A strong recurring theme is the negative categorization of AI content, evidenced by lists like 'AI slop' (21), 'AI Fanatics' (19), and 'AI' (18, 23), which target users who promote 'AI-generated images' or 'AI' content generally. There is a clear split in AI categorization: some lists are broad ('AI', 'AI research / effective acceleration / good tech people' (7)), while others are highly specific, such as 'Crypto/AI' (14) or '%AI/ML' (15). Furthermore, the collection includes structural lists, such as 'Core Cluster' (12) and 'Silver Cluster (54)' (6), which denote mutual-follow groups. Sentiment is generally negative when applied to specific behaviors (e.g., 'Please stop' (8)), but positive/neutral when describing groups ('People' (13)). The evidence seems quite broad, covering everything from specific tech niches to general behavioral flags.
search_result_1_uri: https://bsky.app/profile/did:plc:avoehatd55goxr6357qsuiza/lists/3mh44mz7sz62o
search_result_1_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_2_uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mhrbbkz2hw25
search_result_2_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_3_uri: https://bsky.app/profile/did:plc:f6n22z62adionrvb5s6n6vfk/lists/3mktl7bpsbm2y
search_result_3_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_4_uri: https://bsky.app/profile/did:plc:bctwbs3xyefn5hmcfztd7neb/lists/3kdhucvdfcg2o
search_result_4_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1803
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 1421 / estimated 1421
- Search Prompt [local_task]: used 34 / estimated 34

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
collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
label: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (reduced retry view)
collection_kind: clearsky_lists
item_count: 24
last_refreshed_at: 0
actor_did: did:plc:3deilm3cxnqundoo227xudg2
refresh_ttl_seconds: 900
metadata.collection_kind: clearsky_lists
metadata.actor_did: did:plc:3deilm3cxnqundoo227xudg2

item[0]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
author: clearsky
type: moderation_list
list_name: Follows of @norvid-studies.bsky.social
list_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.

item[1]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbh2zglpr2h
author: clearsky
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

item[2]
uri: https://bsky.app/profile/did:plc:2x5qwlfl2s3hdhsssrgrctzt/lists/3ls675z3x572o
author: clearsky
type: moderation_list
list_name: -
list_description: 

item[3]
uri: https://bsky.app/profile/did:plc:5v4pjxmdet7k3cppv5lkhbzj/lists/3lgegeoklpb2p
author: clearsky
type: moderation_list
list_name: XR
list_description: 

item[4]
uri: https://bsky.app/profile/did:plc:6z7uxrayuc5fugnzwtkbkuev/lists/3lbsjj4didv2e
author: clearsky
type: moderation_list
list_name: VR
list_description: 

item[5]
uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
author: clearsky
type: moderation_list
list_name: ai and llm
list_description: more LLM focused than my other computer science list

item[6]
uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mhrbbkz2hw25
author: clearsky
type: moderation_list
list_name: Silver Cluster (54)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[7]
uri: https://bsky.app/profile/did:plc:avoehatd55goxr6357qsuiza/lists/3mh44mz7sz62o
author: clearsky
type: moderation_list
list_name: AI research / effective acceleration / good tech people
list_description: 

item[8]
uri: https://bsky.app/profile/did:plc:bctwbs3xyefn5hmcfztd7neb/lists/3kdhucvdfcg2o
author: clearsky
type: moderation_list
list_name: Please stop
list_description: People who should stop

item[9]
uri: https://bsky.app/profile/did:plc:cgutwbizbl3cnxss3s5equjh/lists/3lkpn6d7oh72m
author: clearsky
type: moderation_list
list_name: Users/Promoters of "AI" LLMs, Diffiusors, NFTs, Crypto, etc
list_description: Any accounts that use and/or promote machine regurgitations and aren't caught by another mute list get put on my personal mute list. Also crypto that isn't, you know, the study of cryptographics.

item[10]
uri: https://bsky.app/profile/did:plc:dnivjy2zou3r25vesdcyz63c/lists/3mncr5c7hph2l
author: clearsky
type: moderation_list
list_name: list dnf
list_description: DNF this list

item[11]
uri: https://bsky.app/profile/did:plc:e25inok6kpsy5sbqxszzzhct/lists/3mav6u4qk3226
author: clearsky
type: moderation_list
list_name: 00O0
list_description: 

item[12]
uri: https://bsky.app/profile/did:plc:f6n22z62adionrvb5s6n6vfk/lists/3mktl7bpsbm2y
author: clearsky
type: moderation_list
list_name: Core Cluster
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[13]
uri: https://bsky.app/profile/did:plc:gwo6arpcklxtjselze5ftmd4/lists/3lbwzxw4dmp26
author: clearsky
type: moderation_list
list_name: People
list_description: Interesting posters with fewer than 10K followers.

item[14]
uri: https://bsky.app/profile/did:plc:i4jevytmqw4yg2vplnuyz6e3/lists/3kcykc7b3v62r
author: clearsky
type: moderation_list
list_name: Crypto/AI
list_description: Account that shares or even ‘likes’ blatant generative AI garbage. Anyone who follows an obvious AI slop spreader has failed the test and proven they cannot be taken seriously about any topic.

item[15]
uri: https://bsky.app/profile/did:plc:j3d3ovtl7hs7sdjmvpezs6ay/lists/3l3th65ohpu2n
author: clearsky
type: moderation_list
list_name: %AI/ML
list_description: 

item[16]
uri: https://bsky.app/profile/did:plc:k7gm5aub2iaylyuivzj4zynh/lists/3mdqpq37sbv2v
author: clearsky
type: moderation_list
list_name: test-dar
list_description: 

item[17]
uri: https://bsky.app/profile/did:plc:mdjhvva6vlrswsj26cftjttd/lists/3lukpawccs32y
author: clearsky
type: moderation_list
list_name: topb ai
list_description: 

item[18]
uri: https://bsky.app/profile/did:plc:n7hcry6k4tbgitdcizufvixn/lists/3k3xqeemiff2a
author: clearsky
type: moderation_list
list_name: AI
list_description: Plagiarism machine users

item[19]
uri: https://bsky.app/profile/did:plc:okjrq4aoj56bjpubznaphjfg/lists/3lzex3pbv5r2o
author: clearsky
type: moderation_list
list_name: AI Fanatics
list_description: AI Fanaticism is magical thinking.

item[20]
uri: https://bsky.app/profile/did:plc:p572wxnsuoogcrhlfrlizlrb/lists/3lawwom7zqa2i
author: clearsky
type: moderation_list
list_name: TBOB kickstarters
list_description: we can have a little nostalgia, as a treat

item[21]
uri: https://bsky.app/profile/did:plc:peyy3kcsbjfo6s77xu3rq6ob/lists/3le7i47fbd527
author: clearsky
type: moderation_list
list_name: AI slop
list_description: Users who post AI-generated images

item[22]
uri: https://bsky.app/profile/did:plc:qjqbxfvrq3j4qrcitwtjodhp/lists/3kakmznvljy2b
author: clearsky
type: moderation_list
list_name: Automated plagiarism/art theft o matic
list_description: 

item[23]
uri: https://bsky.app/profile/did:plc:rt2ppbowvmxej7rkkreeg5xp/lists/3m4rklh4n4z2a
author: clearsky
type: moderation_list
list_name: AI
list_description: People who post or repost AI "Art" and their supporters


## Search Prompt
Summarize all lists jcorvinus.bsky.social is on, noting if the list is generally positive or negative in sentiment.
```
