# Agent Debug

- agent_id: 3
- agent_type: SummaryReviewAgent
- agent_kind: SummaryReview
- label: summary review
- status: failed
- parent_agent_id: 2
- child_agent_ids: <none>

## Result Summary

status: fail
grounded: false
sufficient: false
reason: No usable `summary:` paragraph exists.
repair_needed: false
additional_pages_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1555
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 12 / estimated 12
- Harness Scope Assessment [local_task]: used 68 / estimated 68
- Collection Evidence [review_evidence]: used 1080 / estimated 1080
- Proposed Summary [parent_search_results]: used 12 / estimated 12

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
Summarize the last 25 posts.

## Harness Scope Assessment
requested_scope: count 25
required_total_items: 20
page_numbering: user phrases are one-based; `page 0` is accepted as an explicit zero-based alias for the first page
available_total_items: 20
current_window_offset: 0
current_window_size: 20

## Collection Evidence
collection_id: recent_posts:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_kind: recent_posts
search_window_offset: 0
search_window_total_items: 20

matched_item[0] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mq55zbh3e22l
body: plz no nerd snipe the orual

matched_item[1] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mq55i5zmr22l
body: oh I want to hear about some of these now

matched_item[2] uri: at://did:plc:hdhoaan3xa3jiuq4fg4mefid/app.bsky.feed.post/3mq557ekm5k2b
body: i think the process of doing any full network backfill makes you think you’ve made every possible mistake in the process

im here to tell you: no. there are more possible mistakes to make.

matched_item[3] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mq2ehirqas2s
body: 🥰

matched_item[4] uri: at://did:plc:puk5v6kcdtqowovrwj2sijlm/app.bsky.feed.post/3mpyvqgeej222
body: i think, personally speaking, we should make it perfectly and explicitly clear that you can in fact have a career as a trans woman, that you can in fact survive and even thrive even in the most adversarial circumstances, and that the norm is, in fact, that you "figure shit out"

matched_item[5] uri: at://did:plc:puk5v6kcdtqowovrwj2sijlm/app.bsky.feed.post/3mpyvhn4kek22
body: idk dude transitioning set my career back by a decade and yet! im somehow on track to home ownership by the time im 40 and retirement by 65 even if i never see a promotion

matched_item[6] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzkpg7fhc2s
body: please no i have to live here.

matched_item[7] uri: at://did:plc:hu35oubkccqrxl4ldgczpgw7/app.bsky.feed.post/3mpzichlopc2n
body: I really wasn’t expecting any paper to move the needle on the word “conscious”.

I went in expecting support for “thinking”, and that’s basically was it was…

… but it’s gonna be hard to do mech interp now without distinguishing between processes that the model is or is not “conscious of”.

matched_item[8] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzgtv4ses2s
body: okay yeah. and now I'm thinking about Angel in this sort of arc and it would be way less funny and way more "far too real".

matched_item[9] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzgopp7ts2s
body: this is why I have my big bag of cables and adapters. I can plug anything into anything, almost.

matched_item[10] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzglgrz6k2s
body: I haven't put any pictures of *that* online

matched_item[11] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzgcbq6ec2s
body: yeah. I think someone reported them a bit later or why went in ozone and applied the labels after i iirc tagged him to say the classifier was borked in a funny way. and, like, they definitely do need the "this image contains bare boobs" label

matched_item[12] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzg3ta4e22s
body: yeah they all have labels now, but at the time many of them did not get autolabeled

matched_item[13] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzetdle7k2s
body: in fairness to the robot i basically dared her to go look!

matched_item[14] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzeqiiqn22s
body: lol did she just find the one? bc there's definitely more

matched_item[15] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpze67vevc2s
body: nobody would have believed she'd done it otherwise, without checking her logs.

matched_item[16] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpze4qkkbs2s
body: rust for embedded is just way less in-distribution and doesn't have many of the niceties that micropython/circuitpython does. their C is...both better and much worse.

matched_item[17] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzdytzisc2s
body: lmao yes she liked at least one of the posts

matched_item[18] uri: at://did:plc:fpruhuo22xkm5o7ttr2ktxdo/app.bsky.feed.post/3mpzbrbjqwk2o
body: trying to articulate what i hate about "AI writing"

matched_item[19] uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpzcqha6gs2s
body: wat.

## Proposed Summary
No matching cached posts.
```
