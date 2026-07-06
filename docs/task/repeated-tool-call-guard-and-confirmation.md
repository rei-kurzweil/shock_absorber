# Task: Add A Repeated Tool-Call Guard For Root-Agent Runs

## Goal

Prevent the root agent from silently rerunning the exact same tool call multiple times in one run, especially:

- `llm_search` with the exact same `query`
- repeated follow-up calls that add no new scope or evidence

Instead of executing the duplicate call immediately, the harness should intercept it and return a warning/confirmation result back into the conversation.

## Why This Needs To Exist

The latest `.debug` run shows the root agent invoked:

- `llm_search(query="what is the sentiment about elsyluna.bsky.social and how do they reply / respond to people generally? use their posts and replies for grounding")`

three separate times in one root-agent run.

Observed artifacts:

- `.debug/chat_transcript.md`
  - the exact same `llm_search` `args` block appears 3 times
- `.debug/agents/agent_000_root_agent.md`
  - root agent child IDs are `1, 7, 13`
- `.debug/root_prompt_snapshot.md`
  - `Tool Result #1`, `Tool Result #2`, and `Tool Result #3` together consume `5151` input tokens

The collection set was effectively the same on each invocation:

- `actor_profile:did:plc:hzijw7nigriwppf7eeb3k7ar`
- `clearsky_lists:did:plc:hzijw7nigriwppf7eeb3k7ar`
- `pinned_posts:did:plc:hzijw7nigriwppf7eeb3k7ar`
- `recent_posts_unaddressed:did:plc:hzijw7nigriwppf7eeb3k7ar`
- `recent_replies_sent:did:plc:hzijw7nigriwppf7eeb3k7ar`

This latest run is important because the internal collection fanout is already de-duped.
The remaining repetition is now clearly root-level duplicate tool invocation, not duplicated child collection search expansion.

This is not a child-agent search loop.
It is repeated root-level tool invocation.

That means the fix belongs in the root tool-call wrapper, not inside `llm_search` itself.

## Current State

The tool wrapper already has one narrow prevention path:

- repeated `llm_search` after a failed `read_collection_item`

That logic lives in `run_evil_gemma_query(...)` in `src/main.rs`, but it is:

- special-cased
- collection-ID-specific
- not a general repeated-tool-call guard

There is currently no generic mechanism that detects:

- same tool name
- same normalized arguments
- repeated invocation count within the active root run

## Desired Behavior

### Duplicate call detection

For every tool call in an active root run, the harness should compute a canonical tool-call fingerprint:

- `tool_name`
- normalized arguments JSON

If the root agent asks for the same fingerprint again in the same run, the harness should treat it as a repeated call attempt.

### First duplicate attempt

On the first exact duplicate attempt:

- do not execute the tool
- append a tool-result-style warning to the conversation
- tell the root agent that this exact call was already executed
- instruct it to:
  - answer from existing results
  - narrow or change the query
  - or explicitly confirm the rerun

### Explicit override

If the root agent really wants to rerun the same call, it must say so explicitly in tool args.

Near-term override shape:

- `confirm_repeat: true`

Optional extra field:

- `repeat_reason: string`

If `confirm_repeat: true` is present, the duplicate call may execute.

### Repetition count

The warning should tell the model how many times it has already requested that exact call.

Example:

- “You already ran this exact `llm_search` query 2 times in this run.”

## Recommended Scope

The guard should be generic across tools, not `llm_search`-only.

However, `llm_search` is the primary motivating case, so the messaging should be especially clear for:

- repeated broad actor-sentiment searches
- repeated global-topic searches

## Implementation Changes

### 1. Add per-run tool-call history

Track tool-call history inside the active root run.

Minimum per-entry fields:

- tool name
- normalized args JSON
- fingerprint
- first round seen
- times requested
- whether it actually executed

### 2. Add canonical argument normalization

Before comparison:

- serialize tool args into stable JSON form
- ignore field ordering differences

This is required so semantically identical calls hash the same way.

### 3. Add generic duplicate-call interception

In `run_evil_gemma_query(...)`, after parsing the tool call and before tool execution:

- compute the fingerprint
- look up prior count in the current run
- if duplicate and no override:
  - skip execution
  - inject a tool-result-style warning
  - continue the loop with that warning as the latest tool result

### 4. Add override support in tool protocol

Document that tool calls may include:

- `confirm_repeat: true`

This is not a domain argument for the tool itself.
It is a harness-level execution override.

The wrapper should strip or ignore that field before domain-specific validation if needed.

### 5. Extend `.debug`

When a repeated call is intercepted, `.debug` should make that obvious.

Add at least:

- duplicate-call warning entry in chat transcript
- repeated-call note in the root-run round summary
- count of how many identical calls were attempted

## Acceptance Criteria

This task is complete when:

- exact duplicate tool calls are detected inside one root-agent run
- duplicate calls are blocked by default
- the model receives a warning telling it the call already ran
- the model can explicitly override with `confirm_repeat: true`
- `.debug` shows both executed calls and blocked duplicate attempts clearly

## Important Notes

- This guard should live in the root-agent tool wrapper, not inside `llm_search`.
- This is complementary to, not a replacement for, the broader incremental-transcript and cancellation work.
- This guard is a cheap and high-value safety brake even before a fuller internal agent/tool-calling architecture exists.
