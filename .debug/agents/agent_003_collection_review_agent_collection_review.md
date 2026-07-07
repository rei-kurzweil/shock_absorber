# Agent Debug

- agent_id: 3
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 2
- child_agent_ids: <none>

## Result Summary

status: pass
reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary is dominated by identifiers or metadata placeholders.
repair_needed: false
repair_diagnostic: Initial review failed. Applied deterministic cited repair when possible. Original summary: The collection features numerous moderation lists covering a broad range of topics, with strong recurring themes in technology, social commentary, and specific ideological groups. Key thematic areas include technology, exemplified by lis...

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1790
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 14 / estimated 14
- Collection Evidence [review_evidence]: used 616 / estimated 616
- Proposed Summary [parent_search_results]: used 885 / estimated 885

## Rendered Context Window

```text
Instructions:
You are the internal collection-summary review agent.

Your job is to review one `collection_search` summary before it is used by parent `llm_search` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `reason: ...`
- `repair_needed: true` or `repair_needed: false`
- optional `repair_instructions: ...`

Rules:

- Review the summary against the actual collection evidence provided.
- Fail if the summary is missing, mostly metadata, mostly identifiers, unsupported by the selected records, or too thin to support parent synthesis.
- Pass only when the summary is one grounded paragraph and uses real phrases, quotes, list names, descriptions, or post/reply text from the matched records.
- When the prompt asks for sentiment, reputation, contrast, or list interpretation, expect the summary to preserve that distinction with grounded evidence.
- If the summary can likely be fixed by rewriting it from the existing selected records, set `repair_needed: true` and provide short repair instructions.
- Do not rewrite the summary yourself in this step.

## Search Prompt
List names nonbinary.computer is on

## Collection Evidence
collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (items 1-25 of 100)
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
type: moderation_list
list_name: Tech
list_description: 

matched_item[1] uri: https://bsky.app/profile/did:plc:2j7aolyu2ys3umoikzy2ikpo/lists/3lbdylnqnsx2u
type: moderation_list
list_name: Bots
list_description: 

matched_item[2] uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
type: moderation_list
list_name: People who immediately block others when they are contradicted
list_description: Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.

matched_item[3] uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lxecpau4ek2g
type: moderation_list
list_name: Aggressive people/Rage bait
list_description: People who verbally attack others without justifiable reason. People who make false accusations. Trolls in general. People who spend all day fighting and arguing on the Internet.

matched_item[4] uri: https://bsky.app/profile/did:plc:34jy6uzsekpjaem566fcjpcj/lists/3lyb7harust2a
type: moderation_list
list_name: GenAI
list_description: 

matched_item[5] uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q
type: moderation_list
list_name: Media
list_description: 

matched_item[6] uri: https://bsky.app/profile/did:plc:477rnpqffrg4vayxgmu22v5u/lists/3lch7kaubmv27
type: moderation_list
list_name: Zionists Genociders Cops Agents MAGA Bots
list_description: Israel is a genocidal dictatorship and so is America, both must be abolished.

matched_item[7] uri: https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74dqwfqs2n
type: moderation_list
list_name: Gold Cluster (113)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

matched_item[8] uri: https://bsky.app/profile/did:plc:4ysnxi6vujpjhovgtn5k4ztr/lists/3lb3pdqomsc2j
type: moderation_list
list_name: Tech
list_description: Nerds, geeks and atproto enthusiasts

matched_item[9] uri: https://bsky.app/profile/did:plc:565zvqotj5odnawlgvbqoapt/lists/3lbrdfgh7l62i
type: moderation_list
list_name: AI glazers
list_description: 

## Proposed Summary
post: LLM-selected post in Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (items 1-25 of 100)
summary: The collection features numerous moderation lists covering a broad range of topics, with strong recurring themes in technology, social commentary, and specific ideological groups. Key thematic areas include technology, exemplified by lists such as \"Tech\" (appearing twice, once without description and once defined as \"Nerds, geeks and atproto enthusiasts\" [https://bsky.app/profile/did:plc:4ysnxi6vujpjhovgtn5k4ztr/lists/3lb3pdqomsc2j]), \"GenAI\" [https://bsky.app/profile/did:plc:34jy6uzsekpjaem566fcjpcj/lists/3lyb7harust2a], and \"AI glazers\" [https://bsky.app/profile/did:plc:565zvqotj5odnawlgvbqoapt/lists/3lbrdfgh7l62i]. There is a notable contrast between general interest lists like \"Media\" [https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q] and highly specific ones like \"Zionists Genociders Cops Agents MAGA Bots\" [https://bsky.app/profile/did:plc:477rnpqffrg4vayxgmu22v5u/lists/3lch7kaubmv27]. Secondary themes involve behavioral critiques, such as \"Aggressive people/Rage bait\" [https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lxecpau4ek2g] and \"People who immediately block others when they are contradicted\" [https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b]. The evidence appears quite broad, encompassing everything from \"Bots\" [https://bsky.app/profile/did:plc:2j7aolyu2ys3umoikzy2ikpo/lists/3lbdylnqnsx2u] to specific clusters like \"Gold Cluster (113)\" [https://bsky.app/profile/did:plc:4hawmtgzjx3vclfyphbhfn7v/lists/3mi74dqwfqs2n].
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
```
