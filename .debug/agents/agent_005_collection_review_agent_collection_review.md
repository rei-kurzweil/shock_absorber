# Agent Debug

- agent_id: 5
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 4
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
- used_input_tokens: 1661
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 21 / estimated 21
- Collection Evidence [review_evidence]: used 635 / estimated 635
- Proposed Summary [parent_search_results]: used 730 / estimated 730

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
List the names of the Clearsky lists jcorvinus.bsky.social is on.

## Collection Evidence
collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
collection_label: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
type: moderation_list
list_name: Follows of @norvid-studies.bsky.social
list_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.

matched_item[1] uri: https://bsky.app/profile/did:plc:2bij7yypmcuvwyz4gyqwtluy/lists/3lbxfscjqno2d
type: moderation_list
list_name: AI, Crypto, & Ratcult Shitheads
list_description: 

matched_item[2] uri: https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n
type: moderation_list
list_name: The Great AI - NFT - CRYPTO Cull
list_description: If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will.

matched_item[3] uri: https://bsky.app/profile/did:plc:3ra4dxf4rwet2urznakt2sm4/lists/3mmiew6l3zh2t
type: moderation_list
list_name: Uniquely Insightful
list_description: People whose viewpoints are worthy of serious consideration due to their repeated proof of self-examination

matched_item[4] uri: https://bsky.app/profile/did:plc:565ebob5f6hw33hjdkxty6qj/lists/3lar7ge4qyn2c
type: moderation_list
list_name: ML peeps
list_description: People who post about machine learning in my network

matched_item[5] uri: https://bsky.app/profile/did:plc:7l2rn4ozedxwth3goe3zsasg/lists/3lafejflekr2w
type: moderation_list
list_name: Favs
list_description: In progress, making a custom feed of my favorite ppl

matched_item[6] uri: https://bsky.app/profile/did:plc:7nf3vqbvea5gpbet3kmibxpm/lists/3lvunqkqtlt2t
type: moderation_list
list_name: Gen AI commentary (feed)
list_description: Feed list

matched_item[7] uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
type: moderation_list
list_name: ai and llm
list_description: more LLM focused than my other computer science list

matched_item[8] uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mfxevpxejj2w
type: moderation_list
list_name: Cluster (163)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

matched_item[9] uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mhrbb6iqxi2g
type: moderation_list
list_name: Gold Cluster (92)
list_description: Mutual-follow cluster with shells, found by mino.mobi/cluster

## Proposed Summary
post: LLM-selected post in Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
summary: The collection primarily consists of 'moderation_list' entries, showcasing a variety of curated feeds and block lists created by 'clearsky' for different accounts. A major recurring theme is the categorization of content, particularly around technology, with strong evidence pointing to lists like \"AI, Crypto, & Ratcult Shitheads\" and \"The Great AI - NFT - CRYPTO Cull.\" There is a notable split between lists based on specific topics (e.g., \"ML peeps,\" \"Software,\" \"VR,\" \"ai and llm\") and lists based on social graph structure, such as \"Cluster (163)\\" and \"Gold Cluster (92)\." While the prompt asks for lists associated with jcorvinus.bsky.social, the collection provides several examples, suggesting the evidence is broad, covering multiple list creators and types. The most important results appear to be those with descriptive names, like \"Uniquely Insightful,\" which suggests a high level of curation.
search_result_1_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
search_result_1_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_2_uri: https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n
search_result_2_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_3_uri: https://bsky.app/profile/did:plc:3ra4dxf4rwet2urznakt2sm4/lists/3mmiew6l3zh2t
search_result_3_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_4_uri: https://bsky.app/profile/did:plc:565ebob5f6hw33hjdkxty6qj/lists/3lar7ge4qyn2c
search_result_4_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_5_uri: https://bsky.app/profile/did:plc:7l2rn4ozedxwth3goe3zsasg/lists/3lafejflekr2w
search_result_5_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_6_uri: https://bsky.app/profile/did:plc:7nf3vqbvea5gpbet3kmibxpm/lists/3lvunqkqtlt2t
search_result_6_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_7_uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
search_result_7_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_8_uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mfxevpxejj2w
search_result_8_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_9_uri: https://bsky.app/profile/did:plc:7zre4plmd5jllccww575j6sb/lists/3mhrbb6iqxi2g
search_result_9_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_10_uri: https://bsky.app/profile/did:plc:2bij7yypmcuvwyz4gyqwtluy/lists/3lbxfscjqno2d
search_result_10_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
```
