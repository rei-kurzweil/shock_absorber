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
- used_input_tokens: 699
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 32 / estimated 32
- Collection Evidence [review_evidence]: used 112 / estimated 112
- Proposed Summary [parent_search_results]: used 280 / estimated 280

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
collection_id: actor_profile:did:plc:frudpt5kpurby7s7qdaz7zyw
collection_label: Profile for rei-cast.xyz
collection_kind: actor_profile

matched_item[0] uri: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.actor.profile/self
body: handle: rei-cast.xyz
did: did:plc:frudpt5kpurby7s7qdaz7zyw
bio:
#!/usr/bin/rei
making a cat engine powered by Vulkan.

# https://github.com/rei-kurzweil
# 5/9 lives left, 🇨🇦, 🏳️‍⚧️ (she/her)

## Proposed Summary
post: LLM-selected post in Profile for rei-cast.xyz
summary: The provided evidence is narrow, consisting of a single actor profile for rei-cast.xyz, which details the creator's identity and primary focus. The main theme revolves around the development of a "cat engine powered by Vulkan," which is the core project mentioned in the bio. The profile explicitly states the author's handle is "rei-cast.xyz" and provides a link to their GitHub repository: "https://github.com/rei-kurzweil." Further biographical details include the creator being from Canada ("🇨🇦"), identifying as transgender ("🏳️‍⚧️"), and using she/her pronouns. The profile also includes a playful status update: "5/9 lives left." Since this is a self-profile, the evidence shows what rei-cast.xyz posts about, but it does not show direct replies or external commentary, though the bio itself functions as a statement about the creator's identity and work.
search_result_1_uri: at://did:plc:frudpt5kpurby7s7qdaz7zyw/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:frudpt5kpurby7s7qdaz7zyw
```
