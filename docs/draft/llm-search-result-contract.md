# `llm_search` Result Contract Draft

Goal: make the search result shape explicit so the harness, `/context`, and downstream synthesis all agree on what a collection-search agent returns and what the parent `llm_search` agent does with it.

## Core Rule

Chosen items are the search results.

If a collection-search agent selects an item and that item has a stable inspectable identifier such as a `uri` or `did`, that identifier is the search result identity.

We should not model search results as a separate detached evidence layer when the actual chosen records already exist.

## Why This Matters

Right now it is too easy to blur together:

- the local summary for one collection
- the chosen items from that collection
- the parent summary across collections
- extra flat identifier fields that are treated like something distinct from the chosen items

That makes the result shape harder to reason about and easier to break.

The simpler model is:

- a child agent chooses concrete items
- those chosen items are the search results
- the child agent summarizes what those chosen items show
- the parent agent summarizes across children

## Per-Collection Search Agent Contract

A `collection_search_agent` should return:

- `summary`
- `search_results`

The `summary` should be:

- grounded in the chosen items and nearby repeated evidence from the same collection
- local to that one collection
- not a final answer to the user’s overall question

The `search_results` should be:

- one to four concrete items from the searched collection
- ordered strongest-first
- each with a stable identifier such as `uri` or `did`

Suggested shape:

```text
summary: Repeated list labels center on AI/LLM use and several are explicitly negative or dismissive.
uri: https://...
uri: https://...
```

If the item identity is a DID rather than a URI, that is acceptable so long as it is stable and inspectable.

## Parent `llm_search` Tool Agent Contract

The parent `llm_search` tool agent should return:

- `summary`
- `per_collection_results`
- optional `selected_results`

The parent `summary` should be:

- the cross-collection synthesis
- explicit about which collections support or weaken the conclusion
- explicit about missing or failed collections

The parent should synthesize from:

- child summaries
- child chosen items

It should not synthesize from free-floating identifier text with no item identity.

## Coverage-Oriented Child Contract

Coverage-oriented collection summary workers are different from selective `collection_search` workers.

For coverage-oriented child loops:

- the main upward contract should be summaries, not raw result lists
- each covered page/window should produce one grounded local summary
- the harness may concatenate those local summaries as internal state
- a planner node may inspect the concatenated summary state after each step
- a final notes node may write one scope-level summary after coverage completes or the source is exhausted

Near-term implication:

- coverage-oriented child results do not need to expose raw `search_results` in their final returned payload if the harness already preserves coverage truth internally
- parent `llm_search` can synthesize from the final scope-level summary plus harness-owned coverage metadata

## Naming Guidance

Prefer `search_result` over `anchor item`, `citation`, or `chosen_item` in the internal structure.

Reason:

- `search_result` describes what the harness is actually returning
- the item identity then falls out naturally from the search result’s `uri` or `did`

Near-term acceptable naming:

- `summary`
- `search_results`
- `selected_result_uri`

Transitional compatibility is fine where a flat text protocol still repeats `uri:` lines, but the spec should treat those as:

- identifiers for search results

not as a separate evidence abstraction.

## Failure Semantics

If a collection-search agent cannot ground a result:

- it should not invent a summary
- it should return no search results
- the parent should record that collection as failed or empty

If the parent has no grounded child results:

- it should not emit a substantive sentiment/reputation/conclusion summary
- it should return a deterministic failure explanation instead

## `/context` Implication

`/context` should make this layering visible:

- root agent context
- parent `llm_search` tool-agent context
- one child context per searched collection

The child node diagnostics should show:

- the local summary
- coverage/accounting state
- eventually the concatenated summary state for coverage-oriented loops

The parent node diagnostics should show:

- the cross-collection summary
- which child search results were used

## Suggested Next Code Alignment

The current implementation should converge toward:

- child result struct:
  - `summary`
  - optional `search_results` for selective search workers
  - coverage/accounting metadata for coverage workers

- parent result struct:
  - `summary`
  - `per_collection_results`
  - `selected_results`

If compatibility fields remain for now, they should be treated as legacy flat identifiers for search results rather than a separate citation concept.
