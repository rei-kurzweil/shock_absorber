# Agent Debug

- agent_id: 1
- agent_type: ToolAgent
- agent_kind: LlmSearch
- label: llm_search tool agent
- status: failed
- parent_agent_id: 0
- child_agent_ids: 2, 4, 6, 8
- tool_name: llm_search

## Result Summary

llm_search searched collections independently and combined the grounded results below.
summary: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g: The strongest grounded evidence in this moderation-list collection centers on 4 selected records, with repeated signals around Tech, Follows of @godoglyness.bsky.social, Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts., People who immediately block others when they are contradicted. The matched record text also includes descriptions such as: "Copied from @godoglyness.bsky.social's public follow graph on 2026-05-07. 503 accounts." "Tech bros/Bitcoiners/AI enthusiasts who immediately block others people when questioned about their behavior or views on a particular topic, rather than discussing them peacefully.". This fallback summary is derived directly from those matched records because the model response did not yield a usable structured `summary:` field. | Profile for nonbinary.computer: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around handle: nonbinary.computer. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation. | Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around girl who pooed in front of the bedroom door last night. door was locked because she cannot be trusted with air mattresses.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
selected_result_uri: https://bsky.app/profile/did:plc:23c4uggpdwcjn4vrmyd6w7w4/lists/3mgd43v37ex23
selected_result_source_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
selected_result_collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
selected_result_collection_label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g

collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g
status: ok
diagnostic: Primary full-collection search failed and a reduced retry view was used instead. Primary failure: HTTP status server error (500 INTERNAL SERVER ERROR) for url (http://127.0.0.1:5000/v1/chat/completions)
review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
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

collection_id: recent_replies_received:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent replies received by did:plc:yfvwmnlztr4dwkb7hwz55r2g
status: failed
review_status: fail
review_reason: No usable `summary:` paragraph exists.
review_repair_needed: false
No matching cached posts.

collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Profile for nonbinary.computer
status: ok
review_status: pass
review_reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary omits meaningful text that was available in the matched records.
review_repair_needed: false
repair_diagnostic: Initial review failed. Original summary: The profile for 'nonbinary.computer' presents a multifaceted identity, primarily identifying as a 'Person who does electrical, computer, and music things.' A key theme is the blend of technical and creative pursuits, further emphasized b...
post: LLM-selected post in Profile for nonbinary.computer
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around handle: nonbinary.computer. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g

collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
status: ok
review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
review_repair_needed: false
post: LLM-selected post in Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around girl who pooed in front of the bedroom door last night. door was locked because she cannot be trusted with air mattresses.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpt6ubc2ac2w
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1401
- truncated: false

## Included Sections

- Original Search Query [local_task]: used 30 / estimated 30
- Per-Collection Results [parent_search_results]: used 1100 / estimated 1100

## Rendered Context Window

```text
Instructions:
You are the internal `llm_search` planner and synthesizer.

Your job is to answer the user's Bluesky search request by using the internal tools when needed, then finishing with a direct grounded summary.

Rules:

- Use internal tools to resolve actors, hydrate actor-backed collections, and run narrow collection searches.
- Prefer the narrowest sufficient scope.
- For reputation, sentiment, or list questions, bias toward `clearsky_lists` first.
- Only expand to replies, profile, or recent posts when list evidence is absent, incomplete, or needs contrast.
- Keep both handle and DID visible once an actor is resolved.
- Do not invent collection IDs, item URIs, list names, or evidence.
- If collection search results already answer the question, synthesize directly from them instead of requesting more tools.
- In strict mode, emit exactly one valid `TOOL_CALL` block and nothing else when requesting an internal tool.
- Do not emit JSON unless a tool definition explicitly requires it.
- Your final response should be a short grounded synthesis, not a tool block.

## Original Search Query
sentiment toward nonbinary.computer, how do people reply to her, and what lists is she on?

## Per-Collection Results
collection_id: clearsky_lists:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Clearsky moderation lists for did:plc:yfvwmnlztr4dwkb7hwz55r2g
status: ok
diagnostic: Primary full-collection search failed and a reduced retry view was used instead. Primary failure: HTTP status server error (500 INTERNAL SERVER ERROR) for url (http://127.0.0.1:5000/v1/chat/completions)
review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
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

collection_id: recent_replies_received:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent replies received by did:plc:yfvwmnlztr4dwkb7hwz55r2g
status: failed
review_status: fail
review_reason: No usable `summary:` paragraph exists.
No matching cached posts.

collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Profile for nonbinary.computer
status: ok
review_status: pass
review_reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary omits meaningful text that was available in the matched records.
post: LLM-selected post in Profile for nonbinary.computer
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around handle: nonbinary.computer. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:yfvwmnlztr4dwkb7hwz55r2g

collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g
collection_label: Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
status: ok
review_status: pass
review_reason: The summary is grounded in the selected records and contains substantive evidence.
post: LLM-selected post in Recent top-level posts by did:plc:yfvwmnlztr4dwkb7hwz55r2g
summary: The strongest grounded evidence in this collection centers on 1 selected records, with repeated signals around girl who pooed in front of the bedroom door last night. door was locked because she cannot be trusted with air mattresses.. The model did not return a fully structured summary paragraph, so this fallback is derived from the matched records themselves and should be treated as a compact evidence summary rather than a complete interpretation.
search_result_1_uri: at://did:plc:yfvwmnlztr4dwkb7hwz55r2g/app.bsky.feed.post/3mpt6ubc2ac2w
search_result_1_source_collection_id: recent_posts_unaddressed:did:plc:yfvwmnlztr4dwkb7hwz55r2g
```
