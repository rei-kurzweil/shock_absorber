# Spec: `llm_search`

## Goal

Document the current internal `llm_search` execution shape, including:

- the root loop boundary
- the internal `llm_search` planner loop
- the harness-owned review steps
- the one-shot internal tools
- the current and planned role of `summary`

This spec is intentionally concrete and code-facing.

It should explain what exists now and what is planned next.

## Scope

This spec covers:

- public root-visible `llm_search(query: "...")`
- internal planner-visible tools inside `llm_search`
- current one-shot collection workers
- current review steps
- future nested `summary` subloop
- parent handoff back to the root agent

This spec does not define:

- the public root tool protocol in full
- future generic multi-agent runtime unification
- all context-window logging details

## Current Public Surface

The root agent sees `llm_search` as one tool:

```text
TOOL_CALL
name: llm_search
args: {"query":"..."}
```

The root agent does not directly choose:

- actor hydration details
- collection ids
- page offsets
- summary scope objects

Those are internal to the harness-owned `llm_search` workflow.

## Current Internal Inventory

### Loop owners

Current loop owners:

- root agent
- internal `llm_search` planner

Current non-loop review steps:

- `search_review`
- `summary_review`

Current collection workers:

- `search`
- `summary`

Today both `search` and `summary` are still executed as one-shot collection workers from the planner's perspective.

### Internal one-shot tools

The current planner-visible internal tools are:

- `resolve_actor_refs`
- `hydrate_actor_scope`
- `set_summary_scope`
- `search`
- `summary`
- `search_global_posts`

Important nuance:

- `summary` is planner-visible as if it were one-shot
- but the intended future shape is agentic and paginated

## Current Execution Tree

### High-level tree

```text
RootAgent
`-- ToolCall(llm_search)
    `-- LlmSearchPlannerLoop
        |-- OneShotTool(resolve_actor_refs)        [optional]
        |-- OneShotTool(hydrate_actor_scope)       [optional]
        |-- OneShotTool(set_summary_scope)         [optional, at most once, before summary]
        |-- OneShotTool(search_global_posts)       [optional]
        |-- CollectionWorker(search)               [0..N]
        |   `-- ReviewStep(search_review)
        |-- CollectionWorker(summary)              [0..N]
        |   `-- ReviewStep(summary_review)
        `-- ParentSynthesis
```

### Current root and child loop boundaries

```text
+-------------------+
| Root agent loop   |
| choose tool       |
| receive handoff   |
+---------+---------+
          |
          v
+-------------------------+
| llm_search planner loop |
| choose internal tool    |
| receive tool result     |
| continue or finish      |
+-------------------------+
```

There is not yet a third owned loop inside `summary`.

That is the main architectural gap for multi-page coverage.

## Current `llm_search` Internal Flow

### Step-by-step

```text
1. root agent calls llm_search(query)
2. harness enters internal llm_search planner loop
3. planner may:
   - resolve actor refs
   - hydrate actor scope
   - set summary scope once
   - call search_global_posts
   - call search on one exact collection window
   - call summary on one exact collection window
4. harness executes the requested one-shot worker
5. harness runs the matching review step
6. planner resumes with reviewed result
7. planner eventually emits final synthesis
8. harness renders parent llm_search result
9. root loop receives a compact parent handoff
```

### Current one-shot collection worker shape

```text
CollectionWorker(search or summary)
|-- build collection window
|-- run one LLM call on that window
|-- parse result
`-- run review step
```

That shape is sufficient for selective `search`.

It is only partially sufficient for coverage-oriented `summary`.

## Current Review Ownership

The review steps are harness-owned postprocessing, not independent loop owners.

```text
planner -> collection worker -> review step -> planner resumes
```

That means:

- `search_review` does not choose tools
- `summary_review` does not choose tools
- they verify results and emit verdicts

This matters because `summary_review` can already say:

- `grounded: true`
- `sufficient: false`
- `additional_pages_needed: true`
- `next_page: ...`
- `next_offset: ...`

but it does not itself own the continuation loop yet.

## Current `summary` Limitations

### What exists now

Current `summary` already has:

- explicit requested summary scope object
- one-shot planner-visible `set_summary_scope`
- processed-window metadata
- summary sufficiency accounting
- exact `next_page` and `next_offset` recommendations

### What is missing

Current `summary` does not yet own:

- repeated page fetching
- repeated summary-review continuation
- multi-page accumulation
- final merged collection-summary synthesis across pages

So today the planner can ask for:

- `summary(collection_id=..., page=0)`

and the harness can determine:

- that page 1 is still required

but there is no dedicated nested `summary` agent loop that keeps going until the requested scope is satisfied.

## Planned `summary` Subloop

### Intended shape

`summary` is the only internal `llm_search` worker that should become agentic by default.

Planned shape:

```text
RootAgent
`-- ToolCall(llm_search)
    `-- LlmSearchPlannerLoop
        `-- SummaryAgent(collection_id, requested_summary_scope, prompt)
            |-- SummaryPageTool(page 0 / offset 0)
            |   `-- ReviewStep(summary_review)
            |-- SummaryPageTool(page 1 / offset 25)         [if needed]
            |   `-- ReviewStep(summary_review)
            |-- SummaryPageTool(page 2 / offset 50)         [if needed]
            |   `-- ReviewStep(summary_review)
            `-- SummaryMergeStep
```

### Why `summary` gets the extra loop

`summary` is coverage-oriented.

Its job is not merely:

- find strong evidence

Its job is:

- satisfy requested coverage over a collection window or range

That means it needs its own private state:

- requested summary scope
- pages already covered
- current `next_page` / `next_offset`
- accumulated window summaries
- final merged grounded result

### Why `search` does not necessarily get the same loop

`search` is selective.

It often only needs:

- one exact window
- one review pass
- done

Future iterative `search` refinements are possible, but they are not required for the same reasons as `summary`.

### Expected actor-backed summary targets

When `summary` becomes agentic, the main actor-backed collection targets should be:

- authored posts:
  - `recent_posts_unaddressed:<did>`
- replies from the actor:
  - `recent_replies_sent:<did>`
- replies to the actor:
  - `recent_replies_received:<did>`
- list membership / moderation-list coverage:
  - `clearsky_lists:<did>`
- profile-only summary when explicitly requested:
  - `actor_profile:<did>`

The parent `llm_search` planner should choose which collection to summarize.

The nested `SummaryAgent` should own satisfying the requested page/count scope on that one chosen collection.

## Invalid Tool-Call Repair

### Current problem

The internal planner can emit malformed tool calls, especially:

- bare DID used where exact cached `collection_id` is required

Example:

```text
TOOL_CALL
name: summary
args: {
  collection_id: "did:plc:...",
  prompt: "...",
  page: 0
}
```

### Planned repair boundary

Invalid internal tool calls should be handled inside `llm_search` before broader fallback.

Planned shape:

```text
planner emits TOOL_CALL
        |
        v
validate internal tool call
        |
   +----+----+
   |         |
 valid     invalid
   |         |
 execute   ToolCallRepairStep
             |
        +----+----+
        |         |
      repaired   cannot repair
        |         |
     validate     fallback / fail
        |
      execute
```

### Role of the repair step

The repair step should see:

- root query
- invalid tool call
- validation error
- requested summary scope
- resolved actor refs
- cached collection inventory

It should emit:

- one repaired `TOOL_CALL`
- or `CANNOT_REPAIR`

This is a semantic repair step, not just a string patch step.

## Parent Handoff Back To Root

The root caller should not receive the full raw internal transcript.

`llm_search` should produce a parent-facing handoff artifact with a strict budget.

Current implementation still partly derives this from rendered tool output.

Planned shape is more explicit:

```text
llm_search internal result
        |
        +-- full debug / transcript artifact
        |
        `-- compact parent handoff
             |-- status
             |-- grounded summary
             |-- selected anchor uri
             |-- key collection ids
             |-- compact failures / diagnostics
             `-- next-step hint
```

That parent handoff should be what the root loop receives, not a lossy truncation of the full rendered child result.

## Current vs Planned Summary

### Current

```text
root loop
`-- llm_search planner loop
    |-- one-shot internal tools
    |-- one-shot search worker + review
    |-- one-shot summary worker + review
    `-- parent synthesis
```

### Planned

```text
root loop
`-- llm_search planner loop
    |-- one-shot internal tools
    |-- one-shot search worker + review
    |-- agentic summary subloop
    |   |-- summary page tool
    |   |-- summary review
    |   |-- continue while insufficient
    |   `-- merge final coverage result
    |-- tool_call_repair step on invalid calls
    `-- parent handoff compaction
```

## Open Questions

- Should `tool_call_repair` be LLM-first with deterministic fallback, or deterministic-first with LLM fallback?
- Should `summary` remain planner-visible as a pseudo-tool while being agentic internally, or should it become a first-class internal agent node explicitly?
- Should parent handoff compaction be purely harness-driven, or optionally use an LLM one-shot when multi-collection synthesis is too large?
- Should future iterative `search` flows reuse the same loop/verify runtime primitives as `summary`, even if they do not paginate by default?
