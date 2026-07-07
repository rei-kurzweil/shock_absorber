# Agent Debug

- agent_id: 2
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (items 1-25 of 100)
- status: completed
- parent_agent_id: 1
- child_agent_ids: 3
- collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Result Summary

review_status: pass
review_reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary is dominated by identifiers or metadata placeholders.
review_repair_needed: false
repair_diagnostic: Initial review failed. Applied deterministic cited repair when possible. Original summary: The collection features numerous moderation lists covering a broad range of topics, with strong recurring themes in technology, social commentary, and specific ideological groups. Key thematic areas include technology, exemplified by lis...
post: LLM-selected post in Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (items 1-25 of 100)
summary: Selected moderation-list evidence is drawn from 10 cited record(s). [0] "Bots" - no description. [1] "People who immediately block others when they are contradicted" - "Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular top...". [2] "Aggressive people/Rage bait" - "People who verbally attack others without justifiable reason. People who make false accusations. Trolls in general. People who spend all ...". [3] "GenAI" - no description. [4] "Media" - no description. [5] "Zionists Genociders Cops Agents MAGA Bots" - "Israel is a genocidal dictatorship and so is America, both must be abolished.". [6] "Gold Cluster (113)" - "Mutual-follow cluster with shells, found by mino.mobi/cluster". [7] "Tech" - "Nerds, geeks and atproto enthusiasts". [8] "AI glazers" - no description. [9] "Tech" - no description.
search_result_1_uri: https://bsky.app/profile/did:plc:2j7aolyu2ys3umoikzy2ikpo/lists/3lbdylnqnsx2u
search_result_1_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_2_uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
search_result_2_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_3_uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lxecpau4ek2g
search_result_3_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_4_uri: https://bsky.app/profile/did:plc:34jy6uzsekpjaem566fcjpcj/lists/3lyb7harust2a
search_result_4_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_5_uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q
search_result_5_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_6_uri: https://bsky.app/profile/did:plc:477rnpqffrg4vayxgmu22v5u/lists/3lch7kaubmv27
search_result_6_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_7_uri: https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74dqwfqs2n
search_result_7_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_8_uri: https://bsky.app/profile/did:plc:4ysnxi6vujpjhovgtn5k4ztr/lists/3lb3pdqomsc2j
search_result_8_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_9_uri: https://bsky.app/profile/did:plc:565zvqotj5odnawlgvbqoapt/lists/3lbrdfgh7l62i
search_result_9_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_10_uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
search_result_10_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 2026
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 1547 / estimated 1547
- Search Prompt [local_task]: used 14 / estimated 14

## Rendered Context Window

```text
Instructions:
Inspect the provided collection carefully.

Return exactly one JSON object with this shape:

```json
{
  "title": "short title",
  "summary": "one grounded paragraph",
  "uris": ["real item uri 1", "real item uri 2"]
}
```

- `title` is optional but preferred.
- `summary` is required.
- `uris` is required and may contain up to ten real item URIs from the collection.
- Do not return markdown, code fences, or any text outside the JSON object.

Every `uris` entry must be a real item from the collection.

The `summary` field must be one grounded paragraph of roughly 100-200 words.

That paragraph should include:

- the main repeated themes
- the strongest exact phrases or list names
- which results seem most important versus secondary
- any meaningful split, ambiguity, or contrast inside the collection
- a short note about how broad or narrow the matched evidence seems when that matters

Use the `uris` array to point at the strongest supporting records.

If fewer than ten real search results are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not mention `item[...]`, `matched_item[...]`, or raw metadata labels inside the `summary`. Keep citations in `uris`, not in the paragraph.

Do not add higher-level interpretation beyond grouping repeated evidence and short contrasts that are directly supported by the collection text.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.

## Collection
collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (items 1-25 of 100)
collection_kind: clearsky_lists
item_count: 25
last_refreshed_at: 1783190623
actor_did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
refresh_ttl_seconds: 900
metadata.search_window_total_items: 100
metadata.search_window_offset: 0
metadata.search_window_size: 25

item[0]
uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
author: clearsky
type: moderation_list
list_name: Tech
list_description: 

item[1]
uri: https://bsky.app/profile/did:plc:26jy4fpvbwppaoh5onqhwr34/lists/3ldc7ddy5ef2n
author: clearsky
type: moderation_list
list_name: groypers
list_description: 

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
uri: https://bsky.app/profile/did:plc:2j7aolyu2ys3umoikzy2ikpo/lists/3lbdylnqnsx2u
author: clearsky
type: moderation_list
list_name: Bots
list_description: 

item[7]
uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx343fdrap26
author: clearsky
type: moderation_list
list_name: Sutor, ne ultra crepidam
list_description: Ne supra crepidam ("not beyond the shoe") is a Latin expression used to tell others not to pass judgment beyond their expertise. People who approach subjects without due knowledge or expertise.

item[8]
uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
author: clearsky
type: moderation_list
list_name: People who immediately block others when they are contradicted
list_description: Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.

item[9]
uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lxecpau4ek2g
author: clearsky
type: moderation_list
list_name: Aggressive people/Rage bait
list_description: People who verbally attack others without justifiable reason. People who make false accusations. Trolls in general. People who spend all day fighting and arguing on the Internet.

item[10]
uri: https://bsky.app/profile/did:plc:34jy6uzsekpjaem566fcjpcj/lists/3lyb7harust2a
author: clearsky
type: moderation_list
list_name: GenAI
list_description: 

item[11]
uri: https://bsky.app/profile/did:plc:35ekk2ezse66rkfsikfpg46c/lists/3m6hzacibit2z
author: clearsky
type: moderation_list
list_name: PERSONAL USE 2
list_description: X X X X X

item[12]
uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q
author: clearsky
type: moderation_list
list_name: Media
list_description: 

item[13]
uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb5krm6wcr2r
author: clearsky
type: moderation_list
list_name: Tech Research Science
list_description: 

item[14]
uri: https://bsky.app/profile/did:plc:46egr756sr2dbyyhc6ggsyrd/lists/3larubwc4yn2j
author: clearsky
type: moderation_list
list_name: Therapeutic
list_description: Art, culture, fun

item[15]
uri: https://bsky.app/profile/did:plc:477rnpqffrg4vayxgmu22v5u/lists/3lch7kaubmv27
author: clearsky
type: moderation_list
list_name: Zionists Genociders Cops Agents MAGA Bots
list_description: Israel is a genocidal dictatorship and so is America, both must be abolished.

item[16]
uri: https://bsky.app/profile/did:plc:4e3mlfezhmqpi467lahlpn5f/lists/3klaksvwzag27
author: clearsky
type: moderation_list
list_name: 3rd party Bluesky apps
list_description: 

item[17]
uri: https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74dqwfqs2n
author: clearsky
type: moderation_list
list_name: Gold Cluster (113)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[18]
uri: https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74e5oajs2n
author: clearsky
type: moderation_list
list_name: Bronze Cluster (66)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[19]
uri: https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74efolns2n
author: clearsky
type: moderation_list
list_name: All Clusters (145)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[20]
uri: https://bsky.app/profile/did:plc:4nta75huav6senq5xrzu7e3r/lists/3mp3mlsguw326
author: clearsky
type: moderation_list
list_name: AIR POLICE
list_description: Info@creativecommons.org

item[21]
uri: https://bsky.app/profile/did:plc:4uity4bhgo5oxdsgajfzk7y5/lists/3lhwljowezk2h
author: clearsky
type: moderation_list
list_name: Genocide simps and most hated by Edward Said
list_description: 

item[22]
uri: https://bsky.app/profile/did:plc:4whbnldtjuxhupevsqtny7gf/lists/3lh2vcmlvzw2a
author: clearsky
type: moderation_list
list_name: 11001001
list_description: not a huge fan of lists ugh

item[23]
uri: https://bsky.app/profile/did:plc:4ysnxi6vujpjhovgtn5k4ztr/lists/3lb3pdqomsc2j
author: clearsky
type: moderation_list
list_name: Tech
list_description: Nerds, geeks and atproto enthusiasts

item[24]
uri: https://bsky.app/profile/did:plc:565zvqotj5odnawlgvbqoapt/lists/3lbrdfgh7l62i
author: clearsky
type: moderation_list
list_name: AI glazers
list_description: 


## Search Prompt
List names nonbinary.computer is on
```
