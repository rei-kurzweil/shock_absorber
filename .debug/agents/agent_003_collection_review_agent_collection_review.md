# Agent Debug

- agent_id: 3
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: warning
- parent_agent_id: 2
- child_agent_ids: <none>

## Result Summary

status: pass
reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary is fallback diagnostic text rather than a grounded collection summary.
repair_needed: false
repair_diagnostic: Initial review failed. Original summary: The strongest grounded evidence in this moderation-list collection centers on 10 selected records, with repeated signals around Follows of @norvid-studies.bsky.social, Copied from @norvid-studies.bsky.social's public follow graph on 2026...

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1645
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 27 / estimated 27
- Collection Evidence [review_evidence]: used 657 / estimated 657
- Proposed Summary [parent_search_results]: used 686 / estimated 686

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
Find negative and positive sounding lists and provide examples of the lists themselves.

## Collection Evidence
collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
collection_label: Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
type: moderation_list
list_name: Follows of @norvid-studies.bsky.social
list_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.

matched_item[1] uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfo67mop2p
type: moderation_list
list_name: Follows of @norvid-studies.bsky.social
list_description: Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts.

matched_item[2] uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfwqzzsx2n
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

matched_item[3] uri: https://bsky.app/profile/did:plc:2bij7yypmcuvwyz4gyqwtluy/lists/3lbxfscjqno2d
type: moderation_list
list_name: AI, Crypto, & Ratcult Shitheads
list_description: 

matched_item[4] uri: https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n
type: moderation_list
list_name: The Great AI - NFT - CRYPTO Cull
list_description: If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will.

matched_item[5] uri: https://bsky.app/profile/did:plc:2u5f43ezqz2u6j32wplqxeup/lists/3llaqm3tnvh2k
type: moderation_list
list_name: LUM
list_description: 

matched_item[6] uri: https://bsky.app/profile/did:plc:3ra4dxf4rwet2urznakt2sm4/lists/3mmiew6l3zh2t
type: moderation_list
list_name: Uniquely Insightful
list_description: People whose viewpoints are worthy of serious consideration due to their repeated proof of self-examination

matched_item[7] uri: https://bsky.app/profile/did:plc:7nf3vqbvea5gpbet3kmibxpm/lists/3lvunqkqtlt2t
type: moderation_list
list_name: Gen AI commentary (feed)
list_description: Feed list

matched_item[8] uri: https://bsky.app/profile/did:plc:7tsv4wd4ggnv7zctvt3eqyj7/lists/3lr4uycn6z72c
type: moderation_list
list_name: cunts
list_description: Personal block list for people I find to be cunts and don't want to see anymore. Die mad about it.

matched_item[9] uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
type: moderation_list
list_name: ai and llm
list_description: more LLM focused than my other computer science list

## Proposed Summary
post: LLM-selected post in Clearsky moderation lists for did:plc:3deilm3cxnqundoo227xudg2 (items 1-25 of 100)
summary: The strongest grounded evidence in this moderation-list collection centers on 10 selected records, with repeated signals around Follows of @norvid-studies.bsky.social, Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts., clearsky, list_name: Follows of @norvid-studies.bsky.social, Follows of @godoglyness.bsky.social. The matched record text also includes descriptions such as: "Copied from @norvid-studies.bsky.social's public follow graph on 2026-05-07. 320 accounts." "Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.". This fallback summary is derived directly from those matched records because the model response did not yield a usable structured `summary:` field.
search_result_1_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfjfm6ze2v
search_result_1_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_2_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfo67mop2p
search_result_2_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_3_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbfwqzzsx2n
search_result_3_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_4_uri: https://bsky.app/profile/did:plc:2bij7yypmcuvwyz4gyqwtluy/lists/3lbxfscjqno2d
search_result_4_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_5_uri: https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n
search_result_5_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_6_uri: https://bsky.app/profile/did:plc:2u5f43ezqz2u6j32wplqxeup/lists/3llaqm3tnvh2k
search_result_6_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_7_uri: https://bsky.app/profile/did:plc:3ra4dxf4rwet2urznakt2sm4/lists/3mmiew6l3zh2t
search_result_7_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_8_uri: https://bsky.app/profile/did:plc:7nf3vqbvea5gpbet3kmibxpm/lists/3lvunqkqtlt2t
search_result_8_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_9_uri: https://bsky.app/profile/did:plc:7tsv4wd4ggnv7zctvt3eqyj7/lists/3lr4uycn6z72c
search_result_9_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
search_result_10_uri: https://bsky.app/profile/did:plc:7xkqxg6m4legdq5hzwiobkys/lists/3ltzgvdl5dg2l
search_result_10_source_collection_id: clearsky_lists:did:plc:3deilm3cxnqundoo227xudg2
```
