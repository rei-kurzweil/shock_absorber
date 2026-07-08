# Task: Cap Root `llm_search` Context Spill And Harden Invalid Planner Fallback

## Summary

A real run exposed two adjacent failures in the internal `llm_search` workflow:

- the root context window accepted an oversized compacted `llm_search` tool result
- the internal planner repeated the same invalid `summary` tool call with a bare DID as `collection_id`

The immediate user-visible symptom was a sparse but still partially grounded answer, paired with an unexpectedly large `tool_output` segment in the next root prompt.

## Observed Failure

Example invalid internal planner call:

- `summary(collection_id="did:plc:...", page=0, prompt=...)`

That is invalid because `summary.collection_id` must be an exact cached collection id such as:

- `recent_posts_unaddressed:did:plc:...`
- `actor_profile:did:plc:...`

In the observed run, the planner repeated the same invalid call multiple times, consumed internal rounds, and then fell back to broad `search` over actor collections. That produced a grounded but off-scope answer for a coverage-oriented summary request.

## Required Fixes

### Root context compaction

The root-visible compacted `llm_search` result must not carry:

- an unbounded combined `summary:` line
- repeated `diagnostic:` lines from planner failure loops

Desired behavior:

- truncate long `summary:` lines aggressively for root context
- cap or count repeated diagnostics instead of replaying them verbatim
- preserve only the minimum fields needed for the root agent to continue coherently

### Invalid planner retry handling

The internal planner loop must not spend most of its budget repeating the same invalid tool args.

Desired behavior:

- cap repeated invalid internal tool-call argument retries
- provide a clearer correction hint when a bare DID is used as `collection_id`
- bail out to a deterministic harness-side fallback after the retry cap

### Summary-oriented fallback

When no valid internal outcomes were produced and the query is clearly coverage-oriented, the harness should prefer a summary fallback over a broad selective-search fallback.

Examples:

- `summarize the 25 most recent posts by ...`
- `analyze page 1 of recent posts`
- `write a blog post based on the last 50 posts`

For actor-backed summary requests, the preferred fallback collection is usually:

- `recent_posts_unaddressed:*`

before broader actor collection search.

## Acceptance Criteria

- a compacted root `llm_search` tool result cannot consume thousands of tokens from one long summary line
- repeated internal planner diagnostics do not flood the next root context window
- repeated invalid internal tool args are capped and do not consume most of the internal tool budget
- a bare DID in `collection_id` produces a correction hint pointing to exact cached collection ids
- coverage-oriented actor summary requests fall back to `summary` on recent posts rather than broad `search` across unrelated collections
