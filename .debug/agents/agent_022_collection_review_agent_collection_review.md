# Agent Debug

- agent_id: 22
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: completed
- parent_agent_id: 21
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
- used_input_tokens: 980
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 16 / estimated 16
- Collection Evidence [review_evidence]: used 272 / estimated 272
- Proposed Summary [parent_search_results]: used 417 / estimated 417

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
collection_id: recent_replies_sent:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent replies sent by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: recent_replies_sent

matched_item[0] uri: at://did:plc:hs22zme55gwbp5lgygsdtdll/app.bsky.feed.post/3mptln3kapc24
body: if an accessibility provision takes two seconds of prompting bc your model already knows every kind of colorblindness & how to filter a color set or display to make it more readable, not spending those two seconds becomes much harder to justify!

matched_item[1] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpsybspm6k2w
body: that an obviously ai thumbnail on a video is an indicator of a certain kind of terrible slop content at present is part of it.

matched_item[2] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mprnhd27422m
body: fucked up reply refs -> rude robot

matched_item[3] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpr4c4cxl22o
body: you can, but better to figure out how to build in the resilience for next time.

## Proposed Summary
post: LLM-selected post in Recent replies sent by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The strongest grounded evidence in this collection centers on 4 selected records, with repeated signals around if an accessibility provision takes two seconds of prompting bc your model already knows every kind of colorblindness & how to filter a color set or display to make it more readable, not spending those two seconds becomes much harder to justify!, that an obviously ai thumbnail on a video is an indicator of a certain kind of terrible slop content at present is part of it., fucked up reply refs -> rude robot, you can, but better to figure out how to build in the resilience for next time.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:hs22zme55gwbp5lgygsdtdll/app.bsky.feed.post/3mptln3kapc24
search_result_1_source_collection_id: recent_replies_sent:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_2_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpsybspm6k2w
search_result_2_source_collection_id: recent_replies_sent:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_3_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mprnhd27422m
search_result_3_source_collection_id: recent_replies_sent:did:plc:yfvwmnlztr4dwkb7hwz55r2g
search_result_4_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpr4c4cxl22o
search_result_4_source_collection_id: recent_replies_sent:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
