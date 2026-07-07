# Agent Debug

- agent_id: 5
- agent_type: CollectionReviewAgent
- agent_kind: CollectionReview
- label: collection review
- status: warning
- parent_agent_id: 4
- child_agent_ids: <none>

## Result Summary

status: pass
reason: Initial review failed but the repaired summary is now grounded in the selected records. Original reason: The summary is dominated by identifiers or metadata placeholders.
repair_needed: false
repair_diagnostic: Initial review failed. Applied deterministic cited repair when possible. Original summary: The collection presents a diverse set of moderation lists curated by Clearsky, covering themes ranging from content moderation to specific community interests and ideological stances. A major recurring theme is the categorization of user...

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 2046
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 32 / estimated 32
- Collection Evidence [review_evidence]: used 778 / estimated 778
- Proposed Summary [parent_search_results]: used 961 / estimated 961

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
what are people saying about rei-cast.xyz? how do people reply to rei-cast.xyz? and what do they post about?

## Collection Evidence
collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
collection_label: Clearsky moderation lists for did:plc:frudpt5kpurby7s7qdaz7zyw (items 1-25 of 30)
collection_kind: clearsky_lists

matched_item[0] uri: https://bsky.app/profile/did:plc:3ykw5fx5blvcs3xl6vofmsd7/lists/3k7vncwdnxr25
type: moderation_list
list_name: Toxic
list_description: Accounts which in the view of our moderators or automated tools, post violent, abusive, hateful, racist, or otherwise contentious content for the purpose of engagement.

matched_item[1] uri: https://bsky.app/profile/did:plc:6ijlu2cpferlpfyxwhskh4o4/lists/3ldvg266egm23
type: moderation_list
list_name: AI Users
list_description: 

matched_item[2] uri: https://bsky.app/profile/did:plc:7tymugifj3pvr3f3ng6gjzte/lists/3m3nbifgm3z2k
type: moderation_list
list_name: /vg/
list_description: people in and around games, mostly nontechnical

matched_item[3] uri: https://bsky.app/profile/did:plc:7tymugifj3pvr3f3ng6gjzte/lists/3m3ncdmatjw25
type: moderation_list
list_name: /art/
list_description: visual art, writing, music, film

matched_item[4] uri: https://bsky.app/profile/did:plc:af2hjqd3y4wpjfwkmbgtht34/lists/3me7etfd2x62h
type: moderation_list
list_name: Hijacked #cat, Bot,AiSlop,…
list_description: Hijacked #cat, Bot, AiSlop, Perv, or Sketchy

matched_item[5] uri: https://bsky.app/profile/did:plc:edzlnzvoztauuygch4z5fvl3/lists/3lca4s5edud24
type: moderation_list
list_name: genderfuck transphobe
list_description: Uses or supports "genderfluid", xeno "genders" or neo "pronouns" (including "it" for people and multiple sets), which are derived from "I sexually identify as an Apache attack helicopter", and delegitimize earnest requests for trans people to be recognized as their gender identity.

matched_item[6] uri: https://bsky.app/profile/did:plc:idibrltidcndvydbxezr3qwt/lists/3lyd6c2mhjv2m
type: moderation_list
list_name: AI freaks
list_description: Personal mute list, do not subscribe

matched_item[7] uri: https://bsky.app/profile/did:plc:ieep5jii7rhqkja2gnaplqmx/lists/3l6slrldzu42u
type: moderation_list
list_name: Conservative-Following Blocklist
list_description: If added to this list, it’s because you’re following a right winger. This includes but isn’t limited to:

matched_item[8] uri: https://bsky.app/profile/did:plc:ns5pb6kszjfznt44rypyl2ba/lists/3lbmmahpkok2y
type: moderation_list
list_name: AI enjoyers
list_description: This list is manually curated. It includes AI spam bots, people that egregiously post AI images and AI enjoyers. It should not be assumed everyone on this list uses AI in their art, that's not what this list is for.

matched_item[9] uri: https://bsky.app/profile/did:plc:pxusq4eselfmt3ajvpbbpi4e/lists/3lmx2emyd3u2c
type: moderation_list
list_name: Supports ai slop
list_description: This is a list of users who openly and  unironically post any ai generated imagery  including but not limited to using ai banners and profile pictures or posting ai generated content feel free to block anyone here or the whole list as always the purpose of these lists are to Inform nothing more

## Proposed Summary
post: LLM-selected post in Clearsky moderation lists for did:plc:frudpt5kpurby7s7qdaz7zyw (items 1-25 of 30)
summary: The collection presents a diverse set of moderation lists curated by Clearsky, covering themes ranging from content moderation to specific community interests and ideological stances. A major recurring theme is the categorization of users based on their engagement with Artificial Intelligence, evidenced by lists such as \"AI Users\" [https://bsky.app/profile/did:plc:6ijlu2cpferlpfyxwhskh4o4/lists/3ldvg266egm23], \"AI freaks\" [https://bsky.app/profile/did:plc:idibrltidcndvydbxezr3qwt/lists/3lyd6c2mhjv2m], \"AI enjoyers\" [https://bsky.app/profile/did:plc:ns5pb6kszjfznt44rypyl2ba/lists/3lbmmahpkok2y], and \"Supports ai slop\" [https://bsky.app/profile/did:plc:pxusq4eselfmt3ajvpbbpi4e/lists/3lmx2emyd3u2c]. Other significant categories include ideological groups like \"Conservative-Following Blocklist\" [https://bsky.app/profile/did:plc:ieep5jii7rhqkja2gnaplqmx/lists/3l6slrldzu42u] and \"genderfuck transphobe\" [https://bsky.app/profile/did:plc:edzlnzvoztauuygch4z5fvl3/lists/3lca4s5edud24], and content types such as visual arts and gaming, seen in \"/art/\" [https://bsky.app/profile/did:plc:7tymugifj3pvr3f3ng6gjzte/lists/3m3ncdmatjw25] and \"/vg/\" [https://bsky.app/profile/did:plc:7tymugifj3pvr3f3ng6gjzte/lists/3m3nbifgm3z2k]. While the evidence is broad, the lists are quite specific, indicating focused community discussions; for instance, \"Hijacked #cat, Bot, AiSlop,…\" [https://bsky.app/profile/did:plc:af2hjqd3y4wpjfwkmbgtht34/lists/3me7etfd2x62h] details specific types of problematic posts. A contrast exists between general toxicity (\"Toxic\" [https://bsky.app/profile/did:plc:3ykw5fx5blvcs3xl6vofmsd7/lists/3k7vncwdnxr25]) and highly specific ideological critiques, such as those targeting \"genderfluid, xeno 'genders' or neo 'pronouns'\" [https://bsky.app/profile/did:plc:edzlnzvoztauuygch4z5fvl3/lists/3lca4s5edud24].
search_result_1_uri: https://bsky.app/profile/did:plc:3ykw5fx5blvcs3xl6vofmsd7/lists/3k7vncwdnxr25
search_result_1_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_2_uri: https://bsky.app/profile/did:plc:6ijlu2cpferlpfyxwhskh4o4/lists/3ldvg266egm23
search_result_2_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_3_uri: https://bsky.app/profile/did:plc:7tymugifj3pvr3f3ng6gjzte/lists/3m3nbifgm3z2k
search_result_3_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_4_uri: https://bsky.app/profile/did:plc:7tymugifj3pvr3f3ng6gjzte/lists/3m3ncdmatjw25
search_result_4_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_5_uri: https://bsky.app/profile/did:plc:af2hjqd3y4wpjfwkmbgtht34/lists/3me7etfd2x62h
search_result_5_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_6_uri: https://bsky.app/profile/did:plc:edzlnzvoztauuygch4z5fvl3/lists/3lca4s5edud24
search_result_6_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_7_uri: https://bsky.app/profile/did:plc:idibrltidcndvydbxezr3qwt/lists/3lyd6c2mhjv2m
search_result_7_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_8_uri: https://bsky.app/profile/did:plc:ieep5jii7rhqkja2gnaplqmx/lists/3l6slrldzu42u
search_result_8_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_9_uri: https://bsky.app/profile/did:plc:ns5pb6kszjfznt44rypyl2ba/lists/3lbmmahpkok2y
search_result_9_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
search_result_10_uri: https://bsky.app/profile/did:plc:pxusq4eselfmt3ajvpbbpi4e/lists/3lmx2emyd3u2c
search_result_10_source_collection_id: clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw
```
