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
- used_input_tokens: 616
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 41 / estimated 41
- Collection Evidence [review_evidence]: used 288 / estimated 288
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
Analyze the sentiment of replies received from jcorvinus.bsky.social. Look for strong positive or negative indicators in the replies themselves.

## Collection Evidence
collection_id: recent_replies_received:did:plc:3deilm3cxnqundoo227xudg2
collection_label: Recent replies received by did:plc:3deilm3cxnqundoo227xudg2
collection_kind: recent_replies_received

matched_item[0] uri: at://did:plc:vivdsh7kvkb4iqiwcjt4odvx/app.bsky.feed.post/3lqkt4ctxdk24
body: source_post_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3lqks73aop227
reply_text: this is rad 😎

matched_item[1] uri: at://did:plc:fccqluwn4zrklddjvcrkxssv/app.bsky.feed.post/3lqkswfb5nc2e
body: source_post_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3lqks73aop227
reply_text: and we are HERE FOR IT

matched_item[2] uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpjds2bj422u
body: source_post_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mpjdhcnyns23
reply_text: WMRN (Write Many Read Never) memory

matched_item[3] uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpjec3aq7k2b
body: source_post_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mpjdhcnyns23
reply_text: yess
at last, computers with hardware-accelerated /dev/null device

## Proposed Summary
No matching cached posts.
```
