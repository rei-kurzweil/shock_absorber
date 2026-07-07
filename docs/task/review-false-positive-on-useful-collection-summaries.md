# Task: Fix `collection_review` False Positives On Useful `collection_search` Summaries

## Summary

The current `collection_review` heuristic is rejecting some clearly useful first-pass `collection_search` summaries as:

- `The summary is dominated by identifiers or metadata placeholders.`

That verdict is too coarse for the current failure shape.

In the latest run, both:

- `clearsky_lists`
- `recent_replies_received`

produced useful grounded prose on the first pass, but each was still flagged and repaired into the deterministic cited-evidence format.

This task is to tighten the review logic so it distinguishes:

- genuinely metadata-heavy garbage
- from grounded prose that merely includes raw URI citations inline

## Problem

### Current review bucket is conflating useful URI citations with metadata spam

The existing heuristic in `src/harness/tools.rs` fails summaries when it sees enough metadata-like markers such as:

- `collection_id:`
- `did:`
- `item[`

That made sense for earlier malformed outputs.

But after the structured-output change, the current failure mode is different:

- the model writes a real grounded paragraph
- the paragraph includes inline full URI citations like `[https://bsky.app/... ]` or `[at://...]`
- review still classifies the whole paragraph as “dominated by identifiers or metadata placeholders”

So the heuristic is now overfiring on a summary that is useful, even if not in the ideal style.

### The run still succeeds, but only through repair

Because review fails, deterministic repair rewrites the result into explicit cited evidence lines such as:

- `[0] ...`
- `[1] ...`

That means:

- the agent status becomes `warning`
- the parent `llm_search` tool agent also inherits `warning`
- the final output becomes more list-like than summary-like

The system is therefore masking a review-policy problem as if it were a search-quality problem.

## Concrete Example From `.debug`

### Lists example that was flagged

From [agent_003_collection_review_agent_collection_review.md](/home/rei/_/shock_absorber/.debug/agents/agent_003_collection_review_agent_collection_review.md:1):

The proposed summary included useful grounded content such as:

- `"The Great AI - NFT - CRYPTO Cull"`
- `"Uniquely Insightful"`
- `"ML peeps"`
- `"ai and llm"`
- `"Software"`
- `"Favs"`

and it clearly described a real contrast:

- curated interest lists
- social grouping / follow-graph-copy lists

But it also embedded raw list URIs inline, for example:

- `[https://bsky.app/profile/did:plc:2segyv655ckqdgkvsqaiianr/lists/3jxwojift2y2n]`

That summary was still rejected as:

- `The summary is dominated by identifiers or metadata placeholders.`

Even though the actual prose was useful and materially grounded.

### Replies example that was flagged

From [agent_005_collection_review_agent_collection_review.md](/home/rei/_/shock_absorber/.debug/agents/agent_005_collection_review_agent_collection_review.md:1):

The proposed summary included grounded phrases like:

- `"Top-tier work!"`
- `"Sick video, very well explained"`
- `"Nicely explained and illustrated!"`
- `"This 3D render looks so cool! Such precise lines."`

and a useful contrast between:

- visual praise
- mathematical / conceptual praise

But it also embedded raw reply URIs inline, for example:

- `[at://did:plc:qcwhrvzx6wmi5hz775uyi6fh/app.bsky.feed.post/3mpic6w5xkd2j]`

That summary was also rejected under the same generic “metadata placeholders” bucket.

## Desired Behavior

### Useful grounded prose should not fail solely for inline URI citations

If a summary:

- is one paragraph
- uses real grounded phrases from the selected records
- captures meaningful repeated themes or contrasts
- and only “misbehaves” by embedding raw selected-result URIs inline

then review should not treat it the same way as:

- raw field dumps
- `collection_id:` dumps
- `item[0]` garbage
- fallback diagnostic prose

At minimum, that case should become either:

- `pass` with a style warning
- or `fail` with a more specific reason such as `summary uses raw URI citations inline`

### Repair policy should match the real defect

If inline URI citations are undesirable, the repair instructions should say so directly.

Example:

- `Rewrite the same grounded paragraph without raw URI citations in the prose. Keep citations only in the selected search-result fields.`

That is much more precise than:

- `The summary is dominated by identifiers or metadata placeholders.`

## Proposed Fix Directions

### Option 1: Split “metadata-heavy” from “inline URI citation” failures

Extend review heuristics so they differentiate:

- raw metadata markers like `collection_id:` / `source_post_uri:` / `item[`
- from inline URI citations inside otherwise useful grounded prose

This is the most direct fix.

### Option 2: Allow inline URI citations if the rest of the paragraph is grounded

If the summary is otherwise clearly useful, review can pass it and rely on later formatting cleanup.

This is acceptable if:

- root synthesis no longer exposes raw URIs to the user
- later formatting strips or ignores inline URIs

### Option 3: Keep failing, but with a specific reason and repair path

If we still want review to fail these summaries, the reason and repair instructions should be specific:

- `raw URI citations are embedded in the paragraph`
- `rewrite the same grounded summary without raw URI citations`

That would reduce debugging confusion even if repair remains.

## Recommended Direction

Use a two-part change:

1. treat raw `item[` / `matched_item[` / `collection_id:`-style dumps as true metadata-heavy failures
2. treat inline selected-result URIs in otherwise grounded prose as a distinct, less severe style failure

That gives the runtime cleaner observability:

- real malformed output remains obvious
- useful but stylistically wrong output no longer looks like garbage

## Acceptance Criteria

- the review reason for useful URI-citing summaries is no longer `dominated by identifiers or metadata placeholders`
- a grounded one-paragraph summary that quotes real list names or reply text is not lumped into the same bucket as raw field dumps
- deterministic repair is no longer triggered for summaries whose only defect is inline URI citation style, unless we intentionally keep that stricter policy
- debug output makes the actual failure mode legible

## Related Follow-Up

This task pairs with:

- [reply-and-list-attribution-formatting.md](/home/rei/_/shock_absorber/docs/task/reply-and-list-attribution-formatting.md:1)

That task addresses how selected reply and list evidence should be rendered cleanly once review stops over-flagging useful summaries.
