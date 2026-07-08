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
reason: The summary is grounded and the coverage accounting matches the requested collection window.
repair_needed: false

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1489
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 18 / estimated 18
- Collection Evidence [review_evidence]: used 397 / estimated 397
- Proposed Summary [parent_search_results]: used 767 / estimated 767

## Rendered Context Window

```text
Instructions:
You are the internal collection-summary review agent.

Your job is to review one `search` or `summary` result before it is used by parent `llm_search` synthesis.

Return a compact verdict block with:

- `status: pass` or `status: fail`
- `reason: ...`
- `repair_needed: true` or `repair_needed: false`
- optional `repair_instructions: ...`

Rules:

- Review the summary against the actual collection evidence provided.
- Fail if the summary is missing, mostly metadata, mostly identifiers, unsupported by the selected records, or too thin to support parent synthesis.
- Pass only when the summary is one grounded paragraph and uses real phrases, quotes, list names, descriptions, or post/reply text from the matched records.
- When the proposed result is coverage-oriented, also fail if the claimed coverage does not match the actual window contents.
- When the prompt asks for sentiment, reputation, contrast, or list interpretation, expect the summary to preserve that distinction with grounded evidence.
- If the summary can likely be fixed by rewriting it from the existing selected records, set `repair_needed: true` and provide short repair instructions.
- Do not rewrite the summary yourself in this step.

## Search Prompt
Analyze the content and themes of the last 50 posts.

## Collection Evidence
collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
collection_label: Recent top-level posts by did:plc:uv76n3a4zrgxzo45cgpemkve
collection_kind: recent_posts_unaddressed
search_window_offset: 0
search_window_total_items: 7

matched_item[0] uri: at://did:plc:qn3cmxkfisazt25du36o3txj/app.bsky.feed.post/3mq25ij3i4224
body: ooo www.youtube.com/watch?v=GYdx...
link: https://www.youtube.com/watch?v=GYdxmZvYvMg

matched_item[1] uri: at://did:plc:cjf5cyja7mbog73soxw7y5lt/app.bsky.feed.post/3mpxwi4j4mk2d
body: 📸abdul :
''same image stacked differently. i capture about an hour of starlight and align them to get more detail out of the milky way. but if i don’t align them it demonstrates the rotation of the earth.''
x.com/Advil/status...
link: https://x.com/Advil/status/2073858316011286618

matched_item[2] uri: at://did:plc:mo6di3tejom2usriqbnrkh2w/app.bsky.feed.post/3mpxnrgkhec2u
body: Separate sound channels with independently adjustable volume and ability to mute would be really convenient in meatspace.

matched_item[3] uri: at://did:plc:7cgn3czdxcpsas5a6kp5bt2x/app.bsky.feed.post/3mpxi3uh6gs2j
body: ok give me some idea what do with fable since it's supposed to run out tonorrow

matched_item[4] uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpw6ramgzc2p
body: fallow_irl

matched_item[5] uri: at://did:plc:huocg7zsbthk54vjai56wd27/app.bsky.feed.post/3mptlcs3ops2n
body: A good workman always praises his tools.

matched_item[6] uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mptlvdrf5s2z
body: TIL

## Proposed Summary
post: Recent Unaddressed Posts: Media, Tech, and General Updates
summary: This collection of recent, unaddressed posts covers a diverse range of topics, including media consumption, technological feature requests, and general life updates. There is a clear theme of visual and auditory media, exemplified by a post sharing a YouTube link, another detailing astrophotography techniques—specifically aligning stacked images to reveal Milky Way detail while demonstrating Earth's rotation—and a third suggesting a desired feature for 'meatspace' (likely a platform interface): 'Separate sound channels with independently adjustable volume and ability to mute.' Other posts touch upon creative endeavors and personal reflections. One user is seeking inspiration for a project, asking, 'ok give me some idea what do with fable since it's supposed to run out tonorrow.' Furthermore, there are brief acknowledgments of content, such as 'fallow_irl' and a simple 'TIL,' alongside a philosophical nod to craftsmanship: 'A good workman always praises his tools.'
window_start: 0
window_total_items: 7
covered_item_1_uri: at://did:plc:qn3cmxkfisazt25du36o3txj/app.bsky.feed.post/3mq25ij3i4224
covered_item_2_uri: at://did:plc:cjf5cyja7mbog73soxw7y5lt/app.bsky.feed.post/3mpxwi4j4mk2d
covered_item_3_uri: at://did:plc:mo6di3tejom2usriqbnrkh2w/app.bsky.feed.post/3mpxnrgkhec2u
covered_item_4_uri: at://did:plc:7cgn3czdxcpsas5a6kp5bt2x/app.bsky.feed.post/3mpxi3uh6gs2j
covered_item_5_uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpw6ramgzc2p
covered_item_6_uri: at://did:plc:huocg7zsbthk54vjai56wd27/app.bsky.feed.post/3mptlcs3ops2n
covered_item_7_uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mptlvdrf5s2z
search_result_1_uri: at://did:plc:qn3cmxkfisazt25du36o3txj/app.bsky.feed.post/3mq25ij3i4224
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_2_uri: at://did:plc:cjf5cyja7mbog73soxw7y5lt/app.bsky.feed.post/3mpxwi4j4mk2d
search_result_2_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_3_uri: at://did:plc:mo6di3tejom2usriqbnrkh2w/app.bsky.feed.post/3mpxnrgkhec2u
search_result_3_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_4_uri: at://did:plc:7cgn3czdxcpsas5a6kp5bt2x/app.bsky.feed.post/3mpxi3uh6gs2j
search_result_4_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_5_uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mpw6ramgzc2p
search_result_5_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_6_uri: at://did:plc:huocg7zsbthk54vjai56wd27/app.bsky.feed.post/3mptlcs3ops2n
search_result_6_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
search_result_7_uri: at://did:plc:uv76n3a4zrgxzo45cgpemkve/app.bsky.feed.post/3mptlvdrf5s2z
search_result_7_source_collection_id: recent_posts_unaddressed:did:plc:uv76n3a4zrgxzo45cgpemkve
```
