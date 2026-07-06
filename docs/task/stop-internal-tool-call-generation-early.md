# Task: Stop Internal `llm_search` Tool-Call Generation As Soon As The First Valid Tool Block Is Emitted

## Summary

The internal `llm_search` planner can emit a valid tool request and then keep generating speculative prose, hypothetical results, and self-correction text in the same response.

That is bad for two reasons:

- it wastes provider/model time after the useful output is already available
- it creates avoidable protocol failures when strict internal tool mode expects exactly one tool block

This task is to change runtime behavior so the system stops generation, or behaves as if it stopped generation, at the first valid internal tool call.

## Current Failure Shape

Observed sequence:

1. parent `llm_search` agent emits a valid leading internal tool block such as:
   - `TOOL_CALL`
   - `name: hydrate_actor_scope`
   - `args: {...}`
2. the same response continues with:
   - self-correction prose
   - hypothetical hydration results
   - invented list/post sentiment summaries
3. the harness detects surrounding prose and may log a planner protocol error
4. even if the first tool call is later accepted, the provider/model was held longer than necessary

## Goal

Once the first valid internal tool request is available, the runtime should:

- stop generation immediately when possible
- otherwise truncate/ignore everything after the valid tool block
- execute the tool without forcing a planner retry for trailing prose

The key requirement is resource release:

- free `evil_gemma` or any future provider/model as soon as the valid internal tool request is known

## Non-Goals

- redesigning the public root-visible `llm_search(query)` interface
- changing the semantics of collection review or root rerun guards
- switching the whole app to provider-native tool calling in one step

## Proposed Fix Directions

### Option 1: Streaming parser with early cancellation

If the provider path can stream partial tokens/chunks:

- scan the response incrementally
- detect the first complete valid internal `TOOL_CALL`
- once the closing args object is fully parsed:
  - cancel/close the provider request
  - return the parsed tool call immediately

This is the best behavior because it truly frees the model early.

### Option 2: Prompt-level truncation semantics with non-streaming providers

If true streaming cancellation is not available yet:

- parse the first valid internal `TOOL_CALL` block from the completed response
- treat any trailing prose as ignored extra output, not as a hard planner failure
- surface a transcript/debug warning that extra planner text was discarded
- do not ask the planner to regenerate just because it kept talking

This does not free the model mid-generation, but it does stop retry loops and preserves the useful tool call.

### Option 3: Provider stop-sequence / grammar constraints

If the local provider supports it:

- add stop sequences or output constraints around the internal tool-call protocol
- target stopping right after the parsed args object closes

This may reduce trailing prose even before full streaming support exists.

## Recommended Direction

Implement in phases:

1. Short term:
   - accept the first valid internal `TOOL_CALL`
   - ignore trailing prose
   - log discarded trailing text as a warning
2. Medium term:
   - add streaming parsing and active cancellation for internal tool calls
3. Long term:
   - prefer provider-native structured tool calling when the runtime architecture is ready

## Runtime Semantics

For one internal planner response:

- if no valid `TOOL_CALL` exists:
  - treat it as final synthesis or planner failure, depending on content
- if exactly one valid leading `TOOL_CALL` exists:
  - execute it
  - discard trailing prose
  - log `planner_trailing_output_discarded`
- if multiple tool blocks or ambiguous parse occurs:
  - reject the response as invalid
  - request a clean retry or fall back after the configured cutoff

## Transcript / Debug Requirements

Add explicit visibility for:

- raw planner response
- parsed accepted tool block
- whether trailing text was discarded
- whether provider generation was actively cancelled or merely ignored after completion

Suggested event labels:

- `internal_tool_call_accepted`
- `planner_trailing_output_discarded`
- `internal_tool_generation_cancelled`

## Likely Code Touchpoints

- `src/harness/tools.rs`
  - internal `llm_search` loop
  - internal tool-call parser/validator
  - transcript/debug diagnostics for planner events
- `src/harness/llm_api.rs`
  - streaming or cancellation support if added
- `src/main.rs`
  - if root transcript/debug event types need to surface new runtime notices

## Acceptance Criteria

- a planner response that starts with one valid internal `TOOL_CALL` is accepted even if the model keeps talking afterward
- trailing prose is visible as discarded diagnostic output, not as the sole reason for retry
- repeated planner retries caused only by trailing prose no longer occur
- if streaming cancellation is implemented:
  - provider generation is actively stopped once the first valid tool block closes
- transcript/debug output makes it clear whether the tool call was:
  - accepted cleanly
  - accepted with discarded trailing text
  - rejected as ambiguous or malformed

## Test Plan

- planner emits one valid `TOOL_CALL` followed by explanatory prose:
  - tool call is accepted
  - trailing prose is discarded
  - no retry occurs
- planner emits one valid `TOOL_CALL` followed by invented hypothetical results:
  - tool call is accepted
  - transcript/debug records discarded trailing text
- planner emits two tool blocks:
  - response is rejected as ambiguous
- planner emits malformed args before closing the object:
  - response is rejected
- if streaming cancellation exists:
  - generation stops immediately after the first complete valid tool block
  - provider/model occupancy time drops relative to the current behavior

## Open Questions

- Does the local `evil_gemma` path expose chunk streaming or request cancellation hooks today?
- Can we reliably stop on the closing JSON object without accidentally truncating nested braces inside strings?
- Should trailing prose after an accepted tool block still count as a protocol-quality warning for future prompt tuning?
- Should future `llm_search` subtree cleanup also flatten child ownership so collection reviews are parent-owned siblings of collection searches rather than nested underneath them?
