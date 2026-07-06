# Agent Debug

- agent_id: 5
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: failed
- parent_agent_id: 4
- child_agent_ids: <none>

## Result Summary

status: fail
reason: No usable `summary:` paragraph exists.
repair_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 595
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 21 / estimated 21
- Collection Evidence [review_evidence]: used 287 / estimated 287
- Proposed Summary [parent_search_results]: used 12 / estimated 12

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
Sentiment toward nonbinary.computer's posts (nonbinary.computer)?

## Collection Evidence
collection_id: recent_replies_received:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent replies received by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: recent_replies_received

matched_item[0] uri: at://did:web:jb.waf.moe/app.bsky.feed.post/3mpr5piz6622a
body: source_post_uri: at://did:plc:2zmxikig2sj7gqaezl5gntae/app.bsky.feed.post/3mpr5mber622f
reply_text: @jollywhoppers.com i believe in you
mention: did:plc:lwckcyzhyrufq4ytg2abji7d

matched_item[1] uri: at://did:web:jb.waf.moe/app.bsky.feed.post/3mpr62lioe226
body: source_post_uri: at://did:plc:2zmxikig2sj7gqaezl5gntae/app.bsky.feed.post/3mpr5mber622f
reply_text: please i need witchsky native app

matched_item[2] uri: at://did:plc:xgvzy7ni6ig6ievcbls5jaxe/app.bsky.feed.post/3mpr6paked22n
body: source_post_uri: at://did:plc:2zmxikig2sj7gqaezl5gntae/app.bsky.feed.post/3mpr5mber622f
reply_text: 

matched_item[3] uri: at://did:plc:q7suwaz53ztc4mbiqyygbn43/app.bsky.feed.post/3mprbja5u2c23
body: source_post_uri: at://did:plc:2zmxikig2sj7gqaezl5gntae/app.bsky.feed.post/3mpr5mber622f
reply_text: i asked for help with fixing performance first 🌝

## Proposed Summary
No matching cached posts.
```
