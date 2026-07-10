# Spec: `llm_search`

## Goal

Document the current `llm_search` and public `summary(query)` execution shape as it exists in code today.

This spec is intentionally concrete and code-facing. It should match:

- `src/harness/loop/root.rs`
- `src/harness/loop/search.rs`
- `src/harness/loop/collection_summary.rs`
- `src/harness/root_run.rs`
- `src/harness/tools.rs`

## Scope

This spec covers:

- the root loop boundary
- the internal `llm_search` planner loop
- the public `summary(query)` orchestration path
- the harness-owned `collection_summary` loop
- review, repair, and notes/planner synthesis steps
- current parent handoff behavior

This spec does not define:

- the full public root tool protocol
- future generic loop unification beyond the current node tables
- all `.debug` artifact formats

## Current Loop Owners

Current loop owners:

- `root`
- `search`
- `collection_summary`

Current one-shot harness or LLM-backed steps inside those loops:

- root protocol repair
- internal planner protocol repair
- internal tool-call repair
- collection page summarization
- per-page summary review
- `collection_summary_planner`
- `collection_summary_planner_review`
- `collection_summary_planner_repair`
- `collection_summary_notes`
- `collection_summary_notes_review`
- `collection_summary_notes_repair`

Important boundary:

- `search` is a real planner loop
- `collection_summary` is now a real paginated coverage loop
- review/planner/notes steps are harness-owned states inside that loop, not separate top-level agents

## Root Loop

The root loop is defined in `src/harness/loop/root.rs`.

```text
root_prompt
  -> tool_execution
  -> root_prompt

root_prompt
  -> protocol_repair
  -> root_prompt

root_prompt
  -> return
```

Semantics:

- `root_prompt` is the root LLM round
- `tool_execution` is harness-owned public tool execution
- `protocol_repair` repairs malformed public tool-call protocol
- the root loop continues until a final answer is emitted or the loop fails

## Internal `llm_search` Planner Loop

The internal planner loop is defined in `src/harness/loop/search.rs`.

```text
planner_decide
  -> execute_internal_tool
  -> planner_decide

planner_decide
  -> planner_protocol_repair
  -> planner_decide

planner_decide
  -> return
```

Semantics:

- `planner_decide` is the internal LLM step that either emits one internal tool call or final synthesis
- `execute_internal_tool` is harness-owned execution of the chosen internal tool
- `tool_call_repair` repairs invalid internal tool calls before execution
- `planner_protocol_repair` repairs malformed planner output when it is not a valid tool call or usable final answer

Current planner-visible internal tools include:

- `resolve_actor_refs`
- `hydrate_actor_scope`
- `set_summary_scope`
- `search`
- `summary`
- `search_global_posts`

Important nuance:

- `summary` is planner-visible as one internal tool
- but that tool now delegates into the separate harness-owned `collection_summary` loop when the request is coverage-oriented

## Public `summary(query)` Path

The public root-visible `summary(query)` flow is not itself a planner loop.

Today it is a harness-owned orchestration path in `src/harness/tools.rs` that does:

1. parse the natural-language summary request into:
   - `requested_summary_scope`
   - `collection_target_hint`
2. resolve the actor anchor
3. hydrate the actor-backed collections needed for the requested scope
4. choose one concrete collection id
5. run `collection_summary` on that collection
6. return the rendered `collection_summary` result upward

For actor-backed recent-post summaries, hydration currently computes:

- `recent_posts_feed_fetch_limit`
- `recent_posts_min_top_level_posts`

from the requested scope, with current limits derived by:

- requested count or page scope
- doubled feed fetch budget
- minimum overall fetch of 100
- maximum overall fetch of 200

## `collection_summary` Loop

The coverage loop is defined in `src/harness/loop/collection_summary.rs`.

```text
init_window
  -> summarize_page

summarize_page
  -> review_page
  -> repair_summary_output

repair_summary_output
  -> review_page

review_page
  -> collection_summary_planner

collection_summary_planner
  -> collection_summary_planner_review
  -> collection_summary_planner_repair

collection_summary_planner_review
  -> advance_cursor
  -> collection_summary_notes
  -> collection_summary_planner_repair

advance_cursor
  -> summarize_page

collection_summary_notes
  -> collection_summary_notes_review
  -> collection_summary_notes_repair

collection_summary_notes_review
  -> return_summary
  -> collection_summary_notes_repair
```

Current responsibilities by state:

- `init_window`
  - compute the initial offset from the requested scope
  - compute the max page budget from the requested scope
- `summarize_page`
  - build one 25-item page window with `paged_search_collection`
  - run the coverage-oriented summary leaf on that page
- `repair_summary_output`
  - perform one harness-owned repair pass when the page result is malformed but repairable
- `review_page`
  - apply groundedness and summary-scope sufficiency checks
  - compute `next_page` / `next_offset` when more coverage is needed
- `collection_summary_planner`
  - synthesize accepted page summaries so far into one interim grounded paragraph
- `collection_summary_planner_review`
  - validate the interim planner paragraph
  - choose either `advance_cursor` or terminal notes synthesis
- `advance_cursor`
  - move to the next offset chosen by the harness
- `collection_summary_notes`
  - synthesize one final scope-level summary from accepted page summaries and planner notes
- `collection_summary_notes_review`
  - validate the final scope summary
- `return_summary`
  - emit one aggregated `CollectionToolOutcome`

## Windowing And Coverage Rules

Current page size:

- `COLLECTION_SEARCH_PAGE_SIZE = 25`

Current summary-loop state tracks:

- accepted page outcomes
- accepted page summaries
- planner updates
- accepted `(offset, window_size)` ranges
- `covered_post_count`
- `collection_total_items`
- `source_exhausted`

Coverage sufficiency is harness-owned.

For count scopes:

- the harness computes contiguous coverage across accepted windows
- the harness compares that contiguous coverage against `requested_items.min(available_total_items)`
- the harness emits `next_page` / `next_offset` until coverage is sufficient or the source is exhausted

This means:

- page-local summary review can fail with `grounded: true` and `sufficient: false`
- that is a normal continuation signal, not a fatal error by itself

## Review Ownership

Review is harness-owned postprocessing over a page result or synthesis result.

For coverage-oriented summary pages:

- the summary leaf may produce a grounded page summary
- `summary_review` may still mark it insufficient for the full requested scope
- the page can still remain admissible as accepted evidence for later planner/notes synthesis

This is the key ownership rule:

- the harness decides whether one page is grounded
- the harness decides whether overall requested coverage has been satisfied
- the model does not decide paging completion on its own

## Repairs

There are currently three distinct repair boundaries:

- root public protocol repair
- internal search planner protocol repair / tool-call repair
- `collection_summary` page-output / planner / notes repair

These are not interchangeable.

In particular:

- planner protocol repair fixes malformed planner output
- tool-call repair fixes semantically invalid internal tool calls
- summary-output repair fixes malformed page-level summary payloads
- planner/notes repair fixes malformed plain-prose synthesis at later `collection_summary` stages

## Current Parent Handoff

The root caller does not receive the full internal transcript.

Current parent-visible behavior is still mostly rendered-text handoff:

- the child loop produces a rendered summary result block
- the public `summary(query)` wrapper largely preserves that rendered child result
- the root loop then sees a compact tool result rather than the full child state machine transcript

Debug detail lives separately in:

- `.debug/chat_transcript.md`
- `.debug/current_task.txt`
- `.debug/summary_trace.md`
- `.debug/agents/...`

## Current Limits And Caveats

The current implementation is concrete, but not yet perfect.

Known constraints:

- `collection_summary` pages are fixed at 25 items each
- actor recent-post hydration caps feed fetch budget at 200 items
- public summary flows are still rendered-text-first at the parent boundary
- some debug artifacts still use legacy `agent` naming even when the runtime unit is really a loop state or harness step

Known behavioral caveat from current debugging:

- a run can hydrate a 200-item `recent_posts` collection and start multiple pages
- but later grounded pages may still fail to be retained correctly by the final accumulator
- in that failure mode, the final notes synthesis can collapse back to the first 25-post window even though the loop touched later offsets

That caveat should be treated as a current bug, not intended behavior.

## Current Concrete Trees

### Root calling `llm_search`

```text
root
`-- llm_search
    `-- search loop
        |-- planner_decide
        |-- execute_internal_tool
        |-- tool_call_repair?
        `-- final synthesis
```

### Root calling public `summary(query)`

```text
root
`-- public summary orchestration
    |-- resolve actor
    |-- hydrate actor scope
    |-- choose collection
    `-- collection_summary loop
        |-- summarize_page
        |-- review_page
        |-- collection_summary_planner
        |-- advance_cursor?
        |-- collection_summary_notes
        `-- return_summary
```

## Source Of Truth

If this spec and the code diverge, treat these files as the source of truth:

- `src/harness/loop/root.rs`
- `src/harness/loop/search.rs`
- `src/harness/loop/collection_summary.rs`
- `src/harness/tools.rs`
- `src/harness/root_run.rs`
