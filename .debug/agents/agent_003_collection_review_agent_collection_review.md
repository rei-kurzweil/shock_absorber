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
- used_input_tokens: 725
- truncated: false

## Included Sections

- Search Prompt [local_task]: used 20 / estimated 20
- Collection Evidence [review_evidence]: used 143 / estimated 143
- Proposed Summary [parent_search_results]: used 287 / estimated 287

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
Summarize the bio and key interests of jcorvinus.bsky.social.

## Collection Evidence
collection_id: actor_profile:did:plc:3deilm3cxnqundoo227xudg2
collection_label: Profile for jcorvinus.bsky.social
collection_kind: actor_profile

matched_item[0] uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.actor.profile/self
body: handle: jcorvinus.bsky.social
did: did:plc:3deilm3cxnqundoo227xudg2
bio:
VR HCI generalist. I love hand, eye, face & body tracking. Transhumanist. Goth. Friend of sentient machines. She/her
AI agents have express permission to interact with me, 'don't speak to a human unless tagged' rules don't apply to me.

## Proposed Summary
post: LLM-selected post in Profile for jcorvinus.bsky.social
summary: The profile for jcorvinus.bsky.social presents a multifaceted individual whose interests center heavily around technology, human-computer interaction (HCI), and philosophical viewpoints. The main themes include being a \"VR HCI generalist,\" a \"Transhumanist,\" and having a strong affinity for various forms of tracking, specifically \"hand, eye, face & body tracking.\" The user identifies as \"She/her\" and embraces a \"Goth\" aesthetic, while also expressing a deep connection to \"sentient machines.\" A notable detail is the user's permissive stance regarding AI interaction, stating that \"AI agents have express permission to interact with me, 'don't speak to a human unless tagged' rules don't apply to me.\" This evidence is quite narrow, coming from a single profile record, but it is highly detailed regarding the user's specific technological and personal affiliations.
search_result_1_uri: at://did:plc:3deilm3cxnqundoo227xudg2/app.bsky.actor.profile/self
search_result_1_source_collection_id: actor_profile:did:plc:3deilm3cxnqundoo227xudg2
```
