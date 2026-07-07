# Task: Add Deterministic Repair For `collection_search` Summary Failures

## Summary

`collection_search` currently does two distinct jobs in one LLM step:

- choose the most relevant subset of items from a 25-item collection window
- write a grounded `summary:` paragraph over those chosen items

In the latest runs, the first part often succeeds while the second part fails.

That creates an awkward recovery path:

- the model returns valid selected `uri:` lines
- the harness fabricates a fallback summary paragraph
- `collection_review` rejects that fallback
- the harness repairs the result with another summary-generation step
- the overall search succeeds, but the collection-search agent is still marked `warning`

This task is to narrow the recovery model:

- treat “selected concrete items exist, but summary is missing/invalid” as a formatting failure, not a search failure
- replace generic fallback-summary prose with deterministic cited repair in the harness
- reserve LLM-based repair for cases where no concrete selected subset exists or deterministic repair cannot satisfy the prompt shape

## Problem

### The current fallback summary is only a continuity device

The existing fallback summary helps the pipeline continue because it preserves:

- a required `summary:` field
- the chosen `search_result_*_uri` values
- a minimal paragraph for later review code to inspect

But it is weak as actual evidence output:

- it has no aligned citation structure like `[0]`, `[1]`
- it can include metadata-ish phrasing such as `source_post_uri: ...`
- it does not clearly distinguish primary vs secondary selected items
- it does not render list evidence in a way that makes `list_name` and `list_description` easy to inspect

In other words, the fallback summary keeps the runtime alive, but it does not finish the job well.

### The repair stage already exists, but it is not modeled explicitly enough

Today the runtime already has:

- a `collection_search` worker LLM call
- a `collection_review` worker LLM call
- a harness-side repair step triggered when review fails with `repair_needed: true`

Important current implementation detail:

- the repair step is not a first-class tool
- the review agent does not own tools or its own loop
- the harness reads the structured review verdict and decides whether to repair locally

That part is acceptable.

What is still missing is a clear split between:

- deterministic recovery when selected items already exist
- LLM recovery when selected items do not exist or cannot be trusted

### The current warnings reflect first-pass formatting failure, not final unusability

The latest debug runs show that collection-search warnings often mean:

- the initial model output failed to provide a usable grounded summary
- review rejected that initial summary
- harness repair produced a grounded repaired summary
- the repaired result passed and was used

So the warning status is currently telling us:

- “repair happened”

not:

- “the final result is still bad”

That is useful for debugging, but it also reveals that the current recovery path is doing more work than necessary.

## Desired Model

### Phase 1: Collection search chooses items

The collection-search LLM should still attempt to return:

- `title:`
- `summary:`
- up to ten concrete `uri:` lines

But the most important part is the selected subset itself.

If the model selected 5-10 valid collection items, the main ranking/search work is already done.

### Phase 2: Review decides whether the summary is usable

`collection_review` should continue to decide whether the summary is:

- grounded
- prompt-relevant
- specific enough for parent synthesis

The harness should continue to parse the review verdict from structured fields such as:

- `status: pass|fail`
- `repair_needed: true|false`
- optional repair instructions

### Phase 3: Harness chooses repair mode

If review fails, the harness should choose between two repair modes.

#### Deterministic repair

Use deterministic harness repair when:

- one or more valid selected `uri:` values exist
- those URIs resolve to real items in the searched collection
- the failure is primarily missing/invalid/thin summary text

This repair should not require another LLM call.

#### LLM repair

Use LLM repair only when:

- no usable selected `uri:` subset exists
- selection appears inconsistent or too ambiguous
- the prompt requires synthesis that cannot be satisfied by deterministic cited rendering alone

Examples:

- no `uri:` lines were returned, but the raw response still hints at relevant evidence
- the selected items conflict and need a more nuanced contrast paragraph than a mechanical renderer can provide
- a future prompt shape explicitly demands more interpretive synthesis than item-by-item citations

## Deterministic Repair Requirements

### Repair should render explicit evidence lines tied to selected items

When selected items already exist, the harness should build a grounded repair block with explicit numbering:

- `[0]`
- `[1]`
- `[2]`

The numbering should align with the selected `search_result_*_uri` order or with a clearly defined reranked order if we intentionally reorder them.

If reranking is introduced, the rendered repair block must make that ordering obvious and stable.

### Posts and replies

For post-like or reply-like collection items, the deterministic repair block should include:

- the citation number
- the author handle
- the core post/reply text
- optionally the source/root/post linkage when relevant and compact

Suggested rendering shape:

- `[0] @bot-tan.suibari.com: "This 3D render looks so cool! Such precise lines."`
- `[1] @technobaboo.bsky.social: "omg saaaaaame"`

Important constraint:

- avoid dumping raw `source_post_uri:` style metadata inline unless it directly helps explain the selection

If source-post grouping matters, render it as a compact explanatory phrase rather than a raw field dump.

### Moderation lists

For `clearsky_lists` or other structured list collections, the deterministic repair block should include:

- the citation number
- `list_name`
- `list_description` when present

Suggested rendering shape:

- `[0] "AI, Crypto, & Ratcult Shitheads"`
- `[1] "The Great AI - NFT - CRYPTO Cull" - "If you prefer to avoid - AI - NFT - CRYPTO content. This lists blocks all three things. Use at your own will."`
- `[2] "Uniquely Insightful" - "People whose viewpoints are worthy of serious consideration due to their repeated proof of self-examination"`

This is more inspectable than a prose fallback because:

- list titles are visually anchored
- descriptions are clearly attached to the right list
- parent synthesis can quote or group the cited records without guessing

### Aggregate summary sentence

The deterministic repair block may also include one short aggregate sentence ahead of the cited items, but it must stay mechanical and grounded.

For example:

- “The selected list evidence mixes neutral follow-graph copies with strongly judgmental AI/crypto-themed lists.”
- “The selected replies are mostly enthusiastic reactions, with one more technical follow-up question.”

The aggregate sentence must be derived only from the selected items already cited below it.

### Output shape

The repaired output should remain compatible with the existing typed result flow.

At minimum it should still populate:

- `summary`
- selected `search_result_*_uri` fields

Recommended `summary` shape:

- one compact overview sentence
- followed by inline cited evidence entries such as `[0] ... [1] ... [2] ...`

This keeps the result one paragraph if needed, while still giving precise anchors.

## Why This Is Better Than The Current Fallback

### Deterministic repair is enough when selection already succeeded

If the model already picked the relevant subset, we do not need another model to restate obvious evidence.

The harness can render the chosen items directly and more reliably.

That means:

- fewer avoidable LLM calls
- fewer invented phrases
- easier debug inspection
- clearer provenance for parent synthesis

### Review remains useful

`collection_review` is still useful because it answers:

- did the first-pass summary actually satisfy the prompt?
- if not, is repair needed?

What changes is the owner of the easy recovery case:

- the harness, not another LLM

### LLM repair still has a place

We should not remove LLM repair entirely.

It still matters when:

- no valid selected subset exists
- the search worker response is malformed in a way that deterministic repair cannot recover from
- a future prompt shape demands nuanced synthesis over selected records that the deterministic renderer cannot satisfy

## Proposed Implementation

### 1. Split “selected items exist” from “summary is usable”

Treat these as separate facts in runtime state:

- `selected_items_exist`
- `summary_is_usable`

Today they are too implicitly bundled together.

We want the harness to recognize:

- valid `uri:` selection + invalid summary

as a recoverable deterministic case.

### 2. Replace generic fallback-summary generation in the main repair path

The current fallback-summary helper should no longer be the main repaired output when selected items already exist.

Instead:

- keep a minimal internal fallback only if needed for continuity before review
- after review failure, use deterministic cited rendering as the primary repair mode

### 3. Add a deterministic cited repair renderer

Introduce a dedicated harness helper such as:

- `render_deterministic_collection_repair(...)`

Potential sub-helpers:

- `render_repaired_list_evidence_item(...)`
- `render_repaired_reply_evidence_item(...)`
- `render_repaired_post_evidence_item(...)`
- `render_repair_overview_sentence(...)`

The exact names may differ, but the split should follow collection/item shape rather than one monolithic string builder.

### 4. Add explicit repair mode to execution state

The runtime should record whether the final result came from:

- original model summary
- deterministic harness repair
- LLM repair

Suggested enum shape:

- `Original`
- `DeterministicRepair`
- `LlmRepair`

This will make warning policy and debug rendering much easier to reason about.

### 5. Revisit warning policy

Once deterministic repair is clean and inspectable, we should decide whether a repaired collection search should still show as `warning`.

A good default may be:

- `warning` if the final result still used degraded/diagnostic-only text
- `completed` if deterministic repair produced a grounded cited summary block
- `warning` if LLM repair was needed because the first-pass output lacked selected items or had deeper ambiguity

This is not strictly required for the first patch, but the task should leave room for it.

## Acceptance Criteria

- When `collection_search` returns valid selected `uri:` items but no usable summary, the harness repairs the result deterministically without an extra LLM call.
- The repaired summary contains explicit aligned citations such as `[0]`, `[1]`, `[2]`.
- For list collections, the repaired summary includes citation number, `list_name`, and `list_description` when present.
- For post/reply collections, the repaired summary includes citation number, author handle, and the main text snippet.
- Raw metadata placeholders like `source_post_uri:` do not appear in the repaired summary unless deliberately rendered as compact explanatory context.
- LLM repair remains available for cases where no usable selected subset exists.
- Debug output records which repair mode was used.
- Parent `llm_search` synthesis can consume the repaired output without needing to guess which text belongs to which selected item.

## Testing

Add focused tests for at least these cases:

- selected list items exist, summary missing, deterministic repair renders `[0]`, `[1]`, list names, and descriptions
- selected replies exist, summary missing, deterministic repair renders author handles plus reply text without raw metadata dumps
- no selected items exist, deterministic repair is skipped and the code paths correctly fall back to LLM repair or hard failure
- repaired deterministic summaries are accepted by local validation/review heuristics
- warning/debug state distinguishes original success from deterministic repair and from LLM repair

## Notes

This task does not require turning repair into a public tool.

The current ownership model is fine:

- `collection_review` returns a structured verdict
- the harness decides how to repair
- parent `llm_search` receives the final admitted result

The key change is not tool exposure.

The key change is making the easy repair case deterministic, cited, and inspectable.
