# Task: Add Invalid Internal Tool-Call Repair To `llm_search`

## Summary

The internal `llm_search` planner can emit malformed tool calls that are semantically close to correct, but not executable.

A real failure showed:

- tool: `summary`
- invalid arg: `collection_id: "did:plc:..."`

That was not a missing-information problem.

It was a repair problem.

The harness already had enough context to recover:

- the root query
- the resolved actor DID
- the requested summary scope
- the available cached collection ids and labels
- the validation error explaining why the call failed

Today the planner either:

- retries the same invalid call
- or falls back to broader harness behavior after too many failures

That is the wrong recovery boundary.

## Problem

The current invalid-tool-call path is too weak for semantically close failures.

Examples:

- bare DID used where exact cached `collection_id` is required
- wrong actor-backed collection kind chosen for a clearly actor-scoped query
- malformed but almost-correct paging args
- tool name is plausible but args do not line up with available cached collections

The current harness can validate the error, but not intelligently repair it.

That means:

- internal rounds are wasted
- the planner may get stuck repeating the same malformed call
- `llm_search` may degrade into the wrong fallback mode
- the root caller gets a grounded but off-scope result

## Desired Behavior

When the internal planner emits an invalid tool call, `llm_search` should get one structured repair chance before broader fallback.

### Repair order

Preferred order:

1. validate internal tool call
2. if valid, execute it
3. if invalid, gather repair context
4. run one harness-owned `tool_call_repair` step
5. if repair emits one valid executable tool call, execute it
6. if repair refuses or still fails validation, fall back to narrow deterministic repair only when mechanically obvious
7. otherwise stop and use existing harness fallback behavior

The important point is:

- semantic interpretation should not rely only on regex
- the repair step should see real runtime context

## Why This Should Be LLM-Backed First

The weak point is not syntax.

The weak point is intent interpretation against currently available collections.

For example:

- `summarize the 25 most recent posts by rei-cast.xyz`

and invalid planner output:

- `summary(collection_id="did:plc:...", page=0, prompt=...)`

The repair step should be able to inspect:

- the user query
- the invalid tool call
- the exact validation error
- the resolved actor DID
- the available cached collections, such as:
  - `recent_posts_unaddressed:did:...`
  - `recent_replies_sent:did:...`
  - `recent_replies_received:did:...`
  - `clearsky_lists:did:...`
  - `actor_profile:did:...`

and then choose:

- `recent_posts_unaddressed:did:...`

That is semantic repair, not string cleanup.

## Proposed Repair Contract

### Inputs

The repair step should receive:

- root query
- current internal requested summary scope
- invalid `TOOL_CALL`
- validation error
- resolved actor refs with handles and DIDs
- available cached collection ids and labels
- current internal tool inventory for `llm_search`

### Output

The repair step should emit exactly one of:

- a repaired `TOOL_CALL`
- `CANNOT_REPAIR`

It must not:

- add explanation prose around the output
- invent collection ids
- invent actors
- invent paging semantics not supported by the tool

### Validation

The repaired call must still pass the normal internal tool validator before execution.

If it fails validation:

- do not run another repair round
- treat the repair attempt as failed

## Deterministic Fallback Role

Deterministic repair still has a place, but it should be second-line.

Good fallback cases:

- exact one-to-one mechanical rewrite
- obvious bare-DID to cached actor collection mapping
- page/offset normalization when semantics are already known

Bad primary cases:

- deciding whether the query implies posts vs replies vs lists
- deciding which actor-backed collection best matches an ambiguous prompt

Those cases should be handled by the LLM repair step first.

## Relationship To Other Work

This task is adjacent to, but distinct from:

- root context compaction
- summary pagination
- summary sufficiency accounting
- future nested `summary` agent loops

Those changes improve later stages of execution.

This task is specifically about repairing malformed internal tool calls before execution.

## Acceptance Criteria

- `llm_search` gets one harness-owned repair attempt after an invalid internal tool call
- the repair step sees real runtime collection inventory, not just the raw query
- repaired calls must pass the existing internal tool validator before execution
- repeated malformed planner calls do not consume most of the internal loop budget when a safe repair was available
- a bare DID in `collection_id` can be repaired to an exact cached actor collection id when the query and cached inventory make that choice unambiguous
- repair output is visible in `.debug` and internal diagnostics

## Open Questions

- Should `tool_call_repair` be implemented as a dedicated prompt template or as a generic harness repair primitive?
- Should deterministic repair run before the LLM repair step for trivial mechanical cases, or strictly after?
- Should repair be limited to `summary` and `search` first, then generalized to all internal tools later?
