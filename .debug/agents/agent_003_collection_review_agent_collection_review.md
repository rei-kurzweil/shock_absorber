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
- used_input_tokens: 1008
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 14 / estimated 14
- Collection Evidence [review_evidence]: used 297 / estimated 297
- Proposed Summary [parent_search_results]: used 422 / estimated 422

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
What lists is nonbinary.computer on?

## Collection Evidence
collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
type: moderation_list
list_name: Tech
list_description: 

matched_item[1] uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbh2zglpr2h
type: moderation_list
list_name: Follows of @godoglyness.bsky.social
list_description: Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts.

matched_item[2] uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
type: moderation_list
list_name: People who immediately block others when they are contradicted
list_description: Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.

matched_item[3] uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q
type: moderation_list
list_name: Media
list_description: 

## Proposed Summary
post: LLM-selected post in Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g (reduced retry view)
summary: The strongest grounded evidence in this moderation-list collection centers on 4 selected records, with repeated signals around Tech, Follows of @godoglyness.bsky.social, Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts., People who immediately block others when they are contradicted. The matched record text also includes descriptions such as: "Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts." "Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.". This fallback summary is derived directly from those matched records because the model response did not yield a usable structured `summary:` field.
search_result_1_uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
search_result_1_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_2_uri: https://bsky.app/profile/did:plc:27u6urclrgh6uijeiqb2wcts/lists/3mlbh2zglpr2h
search_result_2_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_3_uri: https://bsky.app/profile/did:plc:2m7nz3542c4hj4pfxhyxmw2j/lists/3lx6t2qy2ym2b
search_result_3_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_4_uri: https://bsky.app/profile/did:plc:3xbusn6qwgakkgay4xv5p3d2/lists/3lb3kpy5tu32q
search_result_4_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
