# Happy Path: `llm_search` Collection Summary

## Goal

Create a testable end-to-end path where the root agent can effectively delegate to `llm_search` and answer a request like:

- "summarize the last 50 posts by `<handle>`"

The result should be grounded in at least those 50 posts and should return:

- a concatenated set of grounded per-window summaries
- a final 1-3 paragraph scope-level summary
- short exact quotes pulled from the collection text

Inline citations are not required in the final summary.

## Success Condition

The happy path is ready when one root query can reliably do all of the following:

1. resolve the actor
2. hydrate the relevant authored-post collection
3. detect that this is a coverage request, not a selective-evidence search request
4. route into `collection_summary`
5. cover two 25-item windows for a 50-post request
6. produce one summary for each page or query-result window it reads
7. concatenate those summaries as harness-owned state
8. run a harness-managed `collection_summary_planner` inner review/repair subloop after each accepted page summary
9. run a terminal harness-managed `collection_summary_notes` inner review/repair subloop only after the source is exhausted or the requested scope has actually been covered by accepted windows
10. merge them into one combined child result
11. send that combined result back to parent `llm_search`
12. let the public `summary` path expose that grounded combined result directly without inventing any extra evidence

## Current State

Several pieces already exist:

- root can call `llm_search`
- `llm_search` already distinguishes `search` from coverage-oriented `summary`
- `RequestedSummaryScope` already supports count scopes like `last 50 posts`
- `collection_summary` already loops through pages and applies sufficiency gates
- loop ownership has already been split into:
  - `src/harness/loop/root.rs`
  - `src/harness/loop/llm_search.rs`
  - `src/harness/loop/collection_summary.rs`

But the current child contract is still too narrow for this happy path:

- the `collection_summary` prompt still asks for one paragraph
- the child is explicitly told not to emit URI arrays or page bookkeeping
- the current flow returns per-page outcomes rather than one combined child payload
- there is not yet an explicit inner planner node that sees the concatenated per-window summaries at each step
- there is not yet a final `collection_summary_notes` pass for cross-window themes that runs only at real terminal coverage or exhaustion
- parent `llm_search` still reasons mostly from summary text plus the old result shape
- there is not yet one clean end-to-end test that proves "50 covered posts in, quoted 2-3 paragraph summary out"

## Required Target Contract

## Parent Query Shape

The root-visible query remains simple:

- `llm_search(query: "summarize the last 50 posts by mara.x0f.nl")`

The root should not manage page math.
The harness and internal loops should own:

- scope detection
- page traversal
- child result merging
- final synthesis

## `llm_search` Responsibility

For this happy path, `llm_search` should act as the parent orchestrator, not as the component that manually manages page-by-page prose reasoning.

It should:

- treat `last 50 posts` as a coverage request
- choose the actor-authored recent-posts collection
- start one `collection_summary` run for that collection
- receive one combined child result back
- expose that combined child result as the final public answer by default

It should not need to:

- manually issue page 0 and page 1 as separate planner choices
- infer progress from LLM prose
- stitch together raw page summaries itself
- paraphrase a complete child summary into a shorter root-level answer

That page traversal belongs in the harness-owned `collection_summary` loop.

## `collection_summary` Responsibility

`collection_summary` should become the harness-owned worker that:

- traverses the required windows
- tracks coverage state
- produces one summary for each page/query-result window
- concatenates those summaries as harness-owned state
- runs a planner inner review/repair subloop after each accepted page summary
- runs a final notes inner review/repair subloop for cross-window patterns only at terminal coverage or exhaustion
- returns one combined payload for the requested scope

For a 50-post request over a 25-item page size, that means:

- summarize page 0
- review page 0
- update concatenated summary state
- run `collection_summary_planner`
- run `collection_summary_planner_review`
- run `collection_summary_planner_repair` only if needed
- summarize page 1
- review page 1
- update concatenated summary state
- run `collection_summary_planner`
- run `collection_summary_planner_review`
- run `collection_summary_planner_repair` only if needed
- run `collection_summary_notes` only after requested coverage is reached or the source is exhausted
- run `collection_summary_notes_review`
- run `collection_summary_notes_repair` only if needed
- stop only when at least 50 authored posts are covered or the collection ends
- return one aggregated child result

## Internal `collection_summary` Loop Shape

The internal shape should be explicit.

Near-term target graph:

```text
init_window
  -> summarize_page
  -> review_page
  -> collection_summary_planner
  -> collection_summary_planner_review
  -> collection_summary_planner_repair?
  -> advance_cursor
  -> summarize_page
  -> review_page
  -> collection_summary_planner
  -> collection_summary_planner_review
  -> collection_summary_planner_repair?
  -> collection_summary_notes
  -> collection_summary_notes_review
  -> collection_summary_notes_repair?
  -> return_summary
```

Where:

- `summarize_page` is the per-page or per-query-result LLM step
- `review_page` verifies groundedness and coverage facts for that page
- `collection_summary_planner` is an LLM node inside a harness-managed inner review/repair subloop that runs after each accepted page summary
- `collection_summary_planner_review` is a harness-owned verifier for the interim planner synthesis
- `collection_summary_planner_repair` is a harness-owned single repair path for malformed or truncated interim planner synthesis
- `advance_cursor` is harness-owned pagination state
- `collection_summary_notes` is a terminal LLM node inside a harness-managed inner review/repair subloop
- `collection_summary_notes_review` is a harness-owned verifier for the final notes synthesis
- `collection_summary_notes_repair` is a harness-owned single repair path for malformed or truncated final notes synthesis
- `return_summary` returns one combined child payload to parent `llm_search`

`collection_summary_planner` is the new incremental piece.

It should receive:

- the accepted page summaries
- the covered offsets
- the concatenated summary text so far
- the requested scope and actual covered-post count

It should produce:

- interim result-set synthesis over accepted pages so far
- no paging decision authority; the harness still decides whether to continue, stop at requested coverage, or continue until exhaustion
- updated result-set summary state

`collection_summary_notes` is the final synthesis piece.

It should receive:

- the full concatenated summary text across accepted windows
- the requested scope and actual covered-post count
- terminal loop state showing either requested coverage completion or source exhaustion

It should produce:

- a final 1-3 paragraph scope-level summary
- cross-window notes about recurring patterns, shifts, or contrasts
- the final combined child result metadata

Important ownership rule:

- neither `collection_summary_planner` nor `collection_summary_notes` is a standalone loop owner
- both are LLM-backed nodes wrapped by harness-managed inner review/repair subloops inside the outer `collection_summary` loop

## Combined Child Result Shape

This is the key contract change.

Instead of returning raw result lists or page-local summary text only, `collection_summary` should return one combined summary object to parent `llm_search`.

Minimum desired fields:

```text
concatenated_window_summaries: <harness-owned combined text>
summary: 1-3 grounded paragraphs with short exact quotes
covered_window_offsets:
- 0
- 25
covered_post_count: 50
required_post_count: 50
collection_total_items: <n>
coverage_complete: true
source_exhausted: false
```

The public `summary` tool should then render that payload into a deterministic root-facing block:

```text
Overall commentary across <collection label>:
<final scope-level summary>

Concatenated page summaries for <collection label>:
<accepted page summary 1>

<accepted page summary 2>
...
```

The root run should preserve that block and, when it is already grounded and sufficient, return it directly instead of asking the root model to compress it again.

The exact Rust field names can vary, but the payload must express both:

- the concatenated window-level summaries accumulated by the harness
- the final summary

This is intentionally summary-first rather than raw-result-first.

## Summary Rules

The returned summary should be:

- 2-3 paragraphs
- grounded in the covered posts
- written over the combined 50-post scope, not one page at a time
- allowed to quote short exact snippets from the posts

It should not:

- pretend to cover more posts than the harness actually traversed
- rely on invented quotes
- collapse into a raw item dump
- answer with only page-local observations when the request was for 50 posts

## What Must Change

## 1. Make `collection_summary` Return An Aggregated Child Result

Current behavior is close to:

- run one page
- produce one page outcome
- append another page outcome if needed

For this happy path we need:

- a loop-local accumulator
- one incremental planner node that consumes the concatenated accepted page summaries and is wrapped by harness review/repair
- one final notes node that synthesizes across the full concatenated summary state and is wrapped by terminal harness review/repair
- one final aggregated result emitted after the requested scope is satisfied

That accumulator should track at least:

- covered page offsets
- covered post count
- all page summaries
- concatenated summary text
- collection total items

## 2. Add An Explicit `collection_summary_planner` Inner Subloop

`collection_summary` should not jump directly from "last page reviewed" to "return combined result."

It should have an explicit node such as:

- `collection_summary_planner`

Responsibilities of this node:

- receive each accepted page summary
- inspect the concatenated summary state
- synthesize the accepted coverage so far
- update scope-level result-set summary state
- prepare inputs for the final `collection_summary_notes` pass

This is better than burying the aggregation logic in:

- ad hoc post-loop Rust code
- or parent `llm_search` synthesis

Because it keeps the coverage-oriented planning inside the `collection_summary` loop that already owns the paging truth.

The harness, not this node, decides whether more windows are still needed.

This node should be wrapped by:

- `collection_summary_planner_review`
- optional one-shot `collection_summary_planner_repair`

## 3. Add A Final `collection_summary_notes` Inner Subloop

`collection_summary` also needs a final notes/synthesis step after:

- the requested items were considered
- or the query source was exhausted

It should have an explicit node such as:

- `collection_summary_notes`

Responsibilities of this node:

- read the full concatenated summary state
- identify recurring patterns or themes across all accepted page summaries
- write the final 1-3 paragraph summary returned upward

This node should be wrapped by:

- `collection_summary_notes_review`
- optional one-shot `collection_summary_notes_repair`

It must run only when:

- requested coverage is complete
- or the source is exhausted

## 4. Widen The `collection_summary` Prompt Contract

The current prompt still requests:

- one grounded paragraph

That should become a contract closer to:

- produce page-grounded notes or summaries suitable for concatenation
- allow `collection_summary_planner` to see the concatenated state after each accepted page
- allow `collection_summary_notes` to write the final combined summary only once coverage is complete or the source is exhausted
- allow both planner and notes outputs to be reviewed and repaired by harness-managed inner subloops

Two viable paths:

### Option A

Keep per-page child LLM calls narrow, then add `collection_summary_planner` plus `collection_summary_notes` inside `collection_summary`, each wrapped by harness-managed review/repair steps.

This is the safer path because it preserves page-level verification and gives one explicit incremental planner node plus one explicit final notes node before the final combined summary is produced.

### Option B

Ask each page step to emit richer structured outputs and let the harness concatenate them into the final result.

This is workable, but it risks carrying too much raw page text upward.

Recommended path:

- Option A

## 5. Add A Final Result-Set Notes Step Inside `collection_summary`

After the paging loop reaches actual requested coverage or the source ends, `collection_summary` should run the terminal `collection_summary_notes` inner subloop, which:

- receives the page-level summaries
- receives the concatenated summary text
- writes the final 2-3 paragraph quoted summary
- returns the combined child result

This node can be:

- harness-authored concatenation
- followed by one LLM notes/synthesis pass over the concatenated summary state

That is the cleanest path to "only summaries move upward and the harness handles concatenation."

## 6. Preserve Harness-Owned Coverage Truth

The harness, not the LLM, must remain the source of truth for:

- covered offsets
- covered post count
- whether 50 posts were actually traversed
- whether the collection ended early

This should live in loop runtime state or a near-term accumulator struct.

## 7. Align Parent `llm_search` Result Handling

Parent `llm_search` needs to accept the new combined child result as a first-class result shape.

That means:

- treat the concatenated child summaries as internal harness state
- treat the final 1-3 paragraph text as the child `summary`
- synthesize directly from that child result for a single-collection coverage request

For the single-collection happy path, parent `llm_search` should often be little more than:

- "use the combined child result as the grounded answer"

## 6. Keep Root Pass-Through Strong

The root layer should not rewrite a good `llm_search` answer into something weaker.

For this happy path, root should preserve:

- the grounded summary
- the collection identity
- the evidence post identifiers

This matches the existing direction in `docs/spec/tool-result-pass-through.md`.

## Recommended Implementation Order

## Slice 1

Add a `collection_summary` accumulator and explicit planner/notes node shapes.

Concrete work:

- add an accumulator struct in `src/harness/loop/collection_summary.rs`
- collect page-level outcomes into one scope-level intermediate state
- add `collection_summary_planner`, `collection_summary_planner_review`, and `collection_summary_planner_repair` to the loop definition
- add `collection_summary_notes`, `collection_summary_notes_review`, and `collection_summary_notes_repair` to the loop definition
- route accepted page summaries into the planner inner subloop
- return one final combined result instead of only raw page outcomes

## Slice 2

Add harness-owned summary concatenation.

Concrete work:

- concatenate accepted page summaries in coverage order
- make the concatenated text visible to `collection_summary_planner`
- preserve harness-owned coverage metadata separately from LLM prose

## Slice 3

Add the `collection_summary_planner` and `collection_summary_notes` handler implementations plus their harness review/repair handlers.

Concrete work:

- define the planner prompt or handler
- define the final notes prompt or handler
- define planner review/repair rules
- define notes review/repair rules
- require 1-3 paragraphs in the final notes output
- require short exact quotes
- ensure the final notes step only sees concatenated grounded summaries plus harness-owned coverage state

## Slice 4

Teach parent `llm_search` to consume the combined child result directly.

Concrete work:

- accept concatenated summary state plus final summary from `collection_summary`
- avoid re-expanding into manual page-level synthesis
- prefer direct pass-through for single-collection coverage requests

## Slice 5

Add end-to-end tests for the 50-post happy path.

Concrete work:

- root query fixture
- actor-backed recent-post collection fixture with at least 50 posts
- deterministic mock child outputs if needed for the first test slice
- assert coverage count
- assert covered offsets
- assert concatenated summary state exists
- assert final summary has 1-3 paragraphs
- assert final summary contains exact quotes that exist in the covered posts

## Test Plan

## Minimum Happy-Path Test

One test should prove:

- input query asks for `last 50 posts`
- scope resolves to `RequestedSummaryScope::Count { requested_items: 50 }`
- `collection_summary` covers offsets `0` and `25`
- `collection_summary` produces one accepted page summary for each covered window
- `collection_summary_planner` sees the concatenated summary state after each accepted window but does not own pagination decisions
- planner review/repair runs before the loop proceeds
- `collection_summary_notes` runs only after coverage is complete or the source is exhausted
- notes review/repair runs before the final payload is returned
- `covered_post_count >= 50`
- combined child result contains concatenated summary state plus final summary text
- final summary contains 1 to 3 paragraphs
- every quoted snippet in the summary appears in one of the covered posts

## Early Lower-Cost Test

Before full end-to-end UI coverage, add a narrower harness test around `collection_summary` only.

That test should prove:

- given a 50-post collection and count scope 50
- the loop keeps paging until 50 are covered
- the loop produces one summary per covered page/query result
- the loop runs `collection_summary_planner` after each accepted page summary
- the loop runs planner review/repair before proceeding
- the loop runs `collection_summary_notes` only at terminal coverage/exhaustion, not after partial coverage
- the loop runs notes review/repair before returning the final payload
- the final combined payload includes both:
  - the concatenated summary state
  - the final summary

This is the fastest path to confidence before wiring the whole root flow.

## Useful Negative Tests

- 37-post collection with a 50-post request should return `coverage_complete: false` or an equivalent grounded insufficiency signal
- quoted text in the final summary must be rejected if it does not appear in the covered posts
- a parent `llm_search` response should not claim 50-post coverage if only page 0 ran

## Open Design Decisions

- Should the concatenated summary state be persisted verbatim, or compacted between planner rounds?
- Should page-level summary outputs stay text-first, or should they return richer structured theme buckets later?
- Should `collection_summary_planner` be allowed to rewrite prior concatenated notes, or should it be restricted to producing a fresh interim synthesis over harness-owned state?
- Should the combined child result expose the concatenated window summaries upward, or keep them internal and return only the final summary?

## Recommendation

The cleanest happy-path setup is:

1. keep root simple
2. keep `llm_search` as the parent orchestrator
3. make `collection_summary` own page traversal, page-summary production, and coverage accounting
4. add a `collection_summary_planner` inner harness review/repair subloop after each accepted page
5. add a terminal `collection_summary_notes` inner harness review/repair subloop after coverage completes or the source ends
6. make `collection_summary` return one combined result containing:
   - concatenated summary state
   - a final 1-3 paragraph quoted summary
7. let parent `llm_search` mostly pass that result through for single-collection coverage requests

That is the shortest path to a real test where we can verify both:

- at least 50 posts were actually covered
- the returned summary is grounded enough to check by quote against the collection
