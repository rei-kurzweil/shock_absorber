# Agent Debug

- agent_id: 14
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 13
- child_agent_ids: <none>

## Result Summary

status: pass
reason: The summary is grounded in the selected records and contains substantive evidence.
repair_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1085
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 16 / estimated 16
- Collection Evidence [review_evidence]: used 295 / estimated 295
- Proposed Summary [parent_search_results]: used 499 / estimated 499

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
how do people reply to nonbinary.computer?

## Collection Evidence
collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
type: moderation_list
list_name: People who immediately block others when they are contradicted
list_description: Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.

matched_item[1] uri: https://bsky.app/profile/did:plc:4uity4bhgo5oxdsgajfzk7y5/lists/3lhwljowezk2h
type: moderation_list
list_name: Genocide simps and most hated by Edward Said
list_description: 

matched_item[2] uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
type: moderation_list
list_name: ai and llm
list_description: more LLM focused than my other computer science list

matched_item[3] uri: https://bsky.app/profile/did:plc:dlslhehdki6232cy6uxdphog/lists/3k7a64b32452s
type: moderation_list
list_name: Tech
list_description: NOT A MUTE LIST

## Proposed Summary
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
```
