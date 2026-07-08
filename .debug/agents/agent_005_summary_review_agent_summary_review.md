# Agent Debug

- agent_id: 5
- agent_type: SummaryReviewAgent
- agent_kind: SummaryReview
- label: summary review
- status: failed
- parent_agent_id: 4
- child_agent_ids: <none>

## Result Summary

status: fail
grounded: false
sufficient: true
reason: Grounded summary coverage reaches all 1 available item(s), exhausting the available collection even though 25 item(s) were requested.
repair_needed: false
additional_pages_needed: false
required_total_items: 1

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 32768
- reserved_output_tokens: 1024
- used_input_tokens: 907
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 16 / estimated 16
- Harness Scope Assessment [local_task]: used 67 / estimated 67
- Collection Evidence [review_evidence]: used 143 / estimated 143
- Proposed Summary [parent_search_results]: used 298 / estimated 298

## Rendered Context Window

```text
Instructions:
You are the internal `summary_review` agent.

Your job is to review one coverage-oriented `summary` result before it is used by parent `llm_search` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `grounded: true` or `grounded: false`
- `sufficient: true` or `sufficient: false`
- `reason: ...`
- optional `additional_pages_needed: true` or `additional_pages_needed: false`
- optional `next_page: <integer>`
- optional `next_offset: <integer>`
- optional `required_total_items: <integer>`

Rules:

- Review the summary against the actual collection window evidence provided.
- Fail if the summary is missing, unsupported, metadata-heavy, or does not match the provided window accounting.
- Fail if `window_offset`, `window_size`, `page_index`, `page_size`, `collection_total_items`, or `has_more` contradict the provided collection window facts.
- Fail if the claimed coverage accounting is incomplete, duplicated, or references URIs outside the provided window.
- Distinguish grounded from sufficient. A page summary can be grounded but still insufficient for the parent task.
- Treat user-facing phrases like `page 1` as the first page, even though internal `page_index` remains zero-based.
- For explicit scope requests like "last 50 posts", "page 1", or "pages 1-2", prefer failing with `grounded: true` and `sufficient: false` when more pages are still required.
- Do not request repair instructions. This step should either pass or explain why more coverage is required.

## Search Prompt
How do people reply to this actor's posts?

## Harness Scope Assessment
requested_scope: count 25
required_total_items: 1
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 1
current_window_offset: 0
current_window_size: 1

## Collection Evidence
collection_id: recent_replies_received:did:plc:hzijw7nigriwppf7eeb3k7ar
collection_label: Recent replies received by did:plc:hzijw7nigriwppf7eeb3k7ar
collection_kind: recent_replies_received
search_window_offset: 0
search_window_total_items: 1

matched_item[0] uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mpsomsylrc26
body: source_post_uri: at://did:plc:hzijw7nigriwppf7eeb3k7ar/app.bsky.feed.post/3mprw73b2uy2e
reply_text: You know, I think I have that exact same "failure" mode and I don't think I'd give it up if I could xD

## Proposed Summary
post: Replies to Actor's Posts
summary: This collection window, which contains a single item, showcases a direct reply to one of the actor's posts. The reply comes from the user jcorvinus.bsky.social and is directed at the post with the URI at://did:plc:hzijw7nigriwppf7eeb3k7ar/app.bsky.feed.post/3mprw73b2uy2e. The content of the reply is conversational and relatable, stating, "You know, I think I have that exact same 'failure' mode and I don't think I'd give it up if I could xD." This suggests the original post likely discussed a recurring "failure" mode or issue that resonated strongly with the replier. The collection itself is sourced from post reply threads and is related to four specific posts by the actor, indicating a pattern of engagement around these topics. The presence of only one item in this window suggests that the replies are varied, but this specific example highlights agreement and perseverance in the face of technical or conceptual difficulties.
window_offset: 0
window_size: 1
page_index: 0
page_size: 25
collection_total_items: 73
has_more: true
covered_item_1_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.feed.post/3mpsomsylrc26
```
