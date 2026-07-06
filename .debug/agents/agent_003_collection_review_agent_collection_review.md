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
reason: The summary is grounded in the selected records and contains substantive evidence.
repair_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1056
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 34 / estimated 34
- Collection Evidence [review_evidence]: used 264 / estimated 264
- Proposed Summary [parent_search_results]: used 483 / estimated 483

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
Summarize all lists jcorvinus.bsky.social is on, noting if the list is generally positive or negative in sentiment.

## Collection Evidence
collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
collection_label: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mhrbbkz2hw25
type: moderation_list
list_name: Silver Cluster (54)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

matched_item[1] uri: https://bsky.app/profile/did:plc:avoehatd55goxr6357qsuiza/lists/3mh44mz7sz62o
type: moderation_list
list_name: AI research / effective acceleration / good tech people
list_description: 

matched_item[2] uri: https://bsky.app/profile/did:plc:bctwbs3xyefn5hmcfztd7neb/lists/3kdhucvdfcg2o
type: moderation_list
list_name: Please stop
list_description: People who should stop

matched_item[3] uri: https://bsky.app/profile/did:plc:f6n22z62adionrvb5s6n6vfk/lists/3mktl7bpsbm2y
type: moderation_list
list_name: Core Cluster
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

## Proposed Summary
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
```
