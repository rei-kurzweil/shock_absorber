# Agent Debug

- agent_id: 13
- agent_type: CollectionSearchAgent
- agent_kind: CollectionSearch
- label: collection search: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g
- status: completed
- parent_agent_id: 10
- child_agent_ids: 14
- collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Result Summary

diagnostic: Primary full-collection search failed and a reduced retry view was used instead. Primary failure: HTTP status server error (500 INTERNAL SERVER ERROR) for url (http://127.0.0.1:5000/v1/chat/completions)
review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (reduced retry view)
summary: The collection features numerous moderation lists centered around specific themes, most prominently 'Tech' and various facets of Artificial Intelligence (AI) and Large Language Models (LLMs). There is a clear thematic split between technical categorization (e.g., 'Tech', 'ai and llm', 'Computer-Touchers') and behavioral/social commentary (e.g., 'People who immediately block others when they are contradicted', 'Be Kind'). Specific technical lists include 'ai and llm' (which is noted as 'more LLM focused than my other computer science list') and 'llms' and 'Crypto/AI'. A notable contrast exists in the 'Tech' lists, where one is explicitly marked 'NOT A MUTE LIST' (uri: https://bsky.app/profile/did:plc:dlslhehdki6232cy6uxdphog/lists/3k7a64b32452s), suggesting a nuanced approach to categorization. Furthermore, the evidence is quite broad, covering everything from 'Gold Cluster (113)' mutual-follow groups to specific behavioral traits like 'Genocide simps and most hated by Edward Said'. The evidence seems to be a mix of highly specific (like 'Blackrock Containment Cell') and very broad (like 'Media').
search_result_1_uri: https://bsky.app/profile/did:plc:dlslhehdki6232cy6uxdphog/lists/3k7a64b32452s
search_result_1_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_2_uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
search_result_2_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_3_uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
search_result_3_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_4_uri: https://bsky.app/profile/did:plc:4uity4bhgo5oxdsgajfzk7y5/lists/3lhwljowezk2h
search_result_4_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1947
- truncated: false

## Included Sections

- Collection [collection_evidence]: used 1583 / estimated 1583
- Search Prompt [local_task]: used 16 / estimated 16

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
collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (reduced retry view)
collection_kind: clearsky_lists
item_count: 24
last_refreshed_at: 1783190623
actor_did: did:plc:yfvwmnlztr4dwkb7hwz55r2g
refresh_ttl_seconds: 900

item[0]
uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
author: clearsky
type: moderation_list
list_name: Tech
list_description: 

item[1]
uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbh2zglpr2h
author: clearsky
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

item[2]
uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
author: clearsky
type: moderation_list
list_name: People who immediately block others when they are contradicted
list_description: Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.

item[3]
uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q
author: clearsky
type: moderation_list
list_name: Media
list_description: 

item[4]
uri: https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74dqwfqs2n
author: clearsky
type: moderation_list
list_name: Gold Cluster (113)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[5]
uri: https://bsky.app/profile/did:plc:4uity4bhgo5oxdsgajfzk7y5/lists/3lhwljowezk2h
author: clearsky
type: moderation_list
list_name: Genocide simps and most hated by Edward Said
list_description: 

item[6]
uri: https://bsky.app/profile/did:plc:5gbwipbphmu3ku5tz5jewdkb/lists/3lcoh2zciqs27
author: clearsky
type: moderation_list
list_name: Levitation is real 🍞
list_description: powerful beings with unconfirmed abilities. lots of rotating. Masters, adepts, and initiates.

item[7]
uri: https://bsky.app/profile/did:plc:6h3xmaezngsectrcz76eampz/lists/3jznke7wdt72r
author: clearsky
type: moderation_list
list_name: Be Kind
list_description: People who insist we should Be Kind whilst hurling abuse

item[8]
uri: https://bsky.app/profile/did:plc:6xo274js6ywhgftvdmtpmonn/lists/3lcnupzuoax24
author: clearsky
type: moderation_list
list_name: Protocol Pals-Pack
list_description: Imported from the Starter Pack at https://bsky.app/starter-pack/samuel.bsky.team/3kztso5fnic24, powered by https://nws-bot.us/bskyStarterPack.php and @wandrme.paxex.aero.

item[9]
uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
author: clearsky
type: moderation_list
list_name: ai and llm
list_description: more LLM focused than my other computer science list

item[10]
uri: https://bsky.app/profile/did:plc:aidzar5i4mx5hwmwjwtjwh3r/lists/3lf4wqidxma2u
author: clearsky
type: moderation_list
list_name: Blackrock Containment Cell
list_description: Hiss corruption.

item[11]
uri: https://bsky.app/profile/did:plc:ccxl3ictrlvtrrgh5swvvg47/lists/3lmpxplrrqa2v
author: clearsky
type: moderation_list
list_name: Mean Field Theory
list_description: The current effective interaction (reply) on my skyline

item[12]
uri: https://bsky.app/profile/did:plc:cilba7ue5v3ohztrpyjdrbaw/lists/3mcx6vhrwcz2a
author: clearsky
type: moderation_list
list_name: AI Knowers-Pack
list_description: Imported from the Starter Pack at https://bsky.app/starter-pack-short/L71zwey, powered by https://nws-bot.us/bskyStarterPack.php and @wandrme.paxex.aero.

item[13]
uri: https://bsky.app/profile/did:plc:dlslhehdki6232cy6uxdphog/lists/3k7a64b32452s
author: clearsky
type: moderation_list
list_name: Tech
list_description: NOT A MUTE LIST

item[14]
uri: https://bsky.app/profile/did:plc:f3eza2bwrdyrpllqaagktcmw/lists/3kowpbimkt325
author: clearsky
type: moderation_list
list_name: Computer-Touchers
list_description: DevOps, IT, web design, protocols, and whatever other computer related people

item[15]
uri: https://bsky.app/profile/did:plc:fyige73je24j7ppzirz67vwl/lists/3mgldgyuufc2j
author: clearsky
type: moderation_list
list_name: Bronze Cluster (46)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[16]
uri: https://bsky.app/profile/did:plc:gi5ewawxvgtuehrw5naka5gy/lists/3mdfkmozspd23
author: clearsky
type: moderation_list
list_name: llms
list_description: 

item[17]
uri: https://bsky.app/profile/did:plc:gttrfs4hfmrclyxvwkwcgpj7/lists/3mk7h75w2f72z
author: clearsky
type: moderation_list
list_name: Gold Cluster (68+1)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

item[18]
uri: https://bsky.app/profile/did:plc:ho3b2qt3jy6a2ouvuvpbqxlk/lists/3mfit6q63i62x
author: clearsky
type: moderation_list
list_name: Mean Field Theory
list_description: friends and maybe future friends who post good

item[19]
uri: https://bsky.app/profile/did:plc:hqm4ltxbn3wwwfyljbryb77j/lists/3lgjrt2rvun2c
author: clearsky
type: moderation_list
list_name: ⚠️ EXTREME stinkers ⚠️
list_description: BEWARE - u rlly stink ,,,

item[20]
uri: https://bsky.app/profile/did:plc:i4jevytmqw4yg2vplnuyz6e3/lists/3kcykc7b3v62r
author: clearsky
type: moderation_list
list_name: Crypto/AI
list_description: Account that shares or even ‘likes’ blatant generative AI garbage. Anyone who follows an obvious AI slop spreader has failed the test and proven they cannot be taken seriously about any topic.

item[21]
uri: https://bsky.app/profile/did:plc:juutyvd4tzpichqfguswmtlu/lists/3miev53bjzg2b
author: clearsky
type: moderation_list
list_name: meet you at the corner of silly and earnest
list_description: 

item[22]
uri: https://bsky.app/profile/did:plc:k5ufhsh5vidwzllq77xjmijz/lists/3levuoiooxg22
author: clearsky
type: moderation_list
list_name: K-Blocklist
list_description: Accounts included might be blocked for trivial reasons or not. I probably don't remember why an account made it to the list.

item[23]
uri: https://bsky.app/profile/did:plc:lehcqqkwzcwvjvw66uthu5oq/lists/3lac453barl27
author: clearsky
type: moderation_list
list_name: atprotocol.dev Follows
list_description: All of the accounts that @atprotocol.dev is following. The account only follows people with a custom domain name mapped.


## Search Prompt
how do people reply to nonbinary.computer?
```
