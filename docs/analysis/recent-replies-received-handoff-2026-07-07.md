# Recent Replies Received Handoff

Date: 2026-07-07

## Scope

This note captures the investigation into why `recent_replies_received` was failing or producing weak signal in `shock_absorber`, plus the remaining issue that likely needs work in the `evil_gemma` inference layer.

## What Was Broken

Earlier runs showed:

- `recent_replies_received:did:plc:frudpt5kpurby7s7qdaz7zyw` being cached with `posts: []`
- collection search agent failure: `No matching cached posts.`
- collection review failure: `No usable summary paragraph exists.`
- root synthesis still writing a narrative about replies even when the reply collection had failed

The concrete storage mismatch was:

- persisted `post_replies:*` collections already existed in SQLite for Rei's posts
- `recent_replies_received` was being rebuilt from transient in-memory `pinned_post_replies`
- on a fresh run, that in-memory map could be empty even though the persisted `post_replies` collections were present

## Fix Applied In This Repo

Patched [src/net_backend.rs](/home/rei/_/shock_absorber/src/net_backend.rs:618) so `ensure_recent_replies_received_cached(...)` aggregates reply records from cached `post_replies` collections when available, instead of depending only on transient thread state.

Added regression test in [src/net_backend.rs](/home/rei/_/shock_absorber/src/net_backend.rs:1170):

- `recent_replies_received_uses_cached_post_replies_collections`

## Verification

After rebuilding and rerunning:

- `recent_replies_received:did:plc:frudpt5kpurby7s7qdaz7zyw` now contained `13` posts instead of `0`
- reply-side collection search no longer failed
- the harness selected grounded reply URIs from `recent_replies_received`

Useful run artifacts:

- [chat_transcript.md](/home/rei/_/shock_absorber/.debug/chat_transcript.md:1)
- [agent_002_collection_search_agent_collection_search_recent_replies_received_by_did.md](/home/rei/_/shock_absorber/.debug/agents/agent_002_collection_search_agent_collection_search_recent_replies_received_by_did.md:1)
- [agent_003_collection_review_agent_collection_review.md](/home/rei/_/shock_absorber/.debug/agents/agent_003_collection_review_agent_collection_review.md:1)

## Current Remaining Problems

### 1. Internal planner still emits prose around tool calls

Repeated diagnostics in the latest run:

- `internal planner validation failed: strict internal mode requires exactly one TOOL_CALL block with no surrounding prose`

This shows up before the harness recovers and continues. The current best guess is that this behavior is configured or encouraged below the harness layer, likely in the `evil_gemma` server path, possibly around `evil_gemma/vision_inference.py` or the `python-llama-cpp` API usage.

This is the main reason to reopen from the parent workspace and inspect both repos together.

### 2. Reply collection search still degrades into deterministic repair

The latest reply search did not produce a good native LLM summary. Instead:

- initial collection-search summary fell back to diagnostic-style text
- review failed it
- harness deterministic repair rewrote it into a grounded citation list
- final status became `pass`, but only after repair

That means:

- caching/storage is fixed
- inference/planner or collection-search generation quality is still poor

### 3. Final synthesis still loses signal

Even when reply evidence exists, the root answer compresses it into broad categories and drops a lot of the useful detail.

Examples of signal that did exist in the reply collection:

- `@ak-jp.bsky.social`: `"I checked. No observations queued. That format got deprecated two builds ago."`
- `@ak-jp.bsky.social`: `"The snake oil label is an uncorrected calibration error — evidence can fix it. The troll tag is different: faction-membership, not a conduct verdict."`
- `@schizanon.bsky.social`: `"I'm kind of a rorsach test for Listifications users. I've been accused of being MAGA a *lot* which is wild."`
- `@jcorvinus.bsky.social`: `"It's a little low res of an analysis but not too bad actually."`
- `@jcorvinus.bsky.social`: `"the negative signals themselves are low res"`
- `@mara.x0f.nl`: `"i don't think i want to know what it makes of mine, lol"`

But the root answer mostly reduced this to generic buckets like:

- feedback on analysis
- community dynamics
- personal experience

So the pipeline is now returning evidence, but the user-facing synthesis is still lower signal than the underlying collection evidence.

## Likely Cross-Repo Follow-Up

Inspect `evil_gemma` next, especially:

- `evil_gemma/vision_inference.py`
- any prompt-wrapping or stop-sequence handling around `python-llama-cpp`
- inference config that might allow or encourage trailing continuations after the first accepted `TOOL_CALL`
- any sampling/termination settings that make the model continue with self-correction or explanatory prose

Main target:

- make strict tool-call mode actually stop after the first valid `TOOL_CALL`

Secondary target:

- improve collection-search output quality so the reply summary is natively grounded and does not need deterministic repair

## Suggested Next Steps

1. Reopen from the parent workspace so both `shock_absorber` and `evil_gemma` can be edited in one session.
2. Trace where strict tool-call mode is enforced versus where the model is still allowed to continue generating prose.
3. Add or tighten stop conditions in the inference server if the model should terminate immediately after a valid `TOOL_CALL`.
4. After that, rerun the same Rei query and compare:
   - planner diagnostics
   - reply collection-search raw output
   - whether collection review still requires repair
   - whether final synthesis preserves reply quotes/citations

