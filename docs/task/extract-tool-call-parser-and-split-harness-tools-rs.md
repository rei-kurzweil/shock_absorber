# Task: Extract Internal Tool-Call Parsing Into `src/harness/tool_call_parser.rs` And Split `tools.rs`

## Summary

`src/harness/tools.rs` is now too large to be a good home for unrelated concerns.

Current size:

- `4611` lines in `src/harness/tools.rs`

It currently mixes together:

- public tool registry definitions
- internal `llm_search` planner/runtime loop logic
- prompt-tool-call parsing
- collection-search parsing
- deterministic repair logic
- collection review heuristics
- collection rendering / summarization helpers
- test fixtures and unit tests

At the same time, the local planner has now demonstrated a new protocol instability:

- sometimes it emits `args: {...}` JSON
- sometimes it emits YAML-like indented `args:` blocks

This task is to:

1. extract a dedicated internal tool-call parser abstraction into `src/harness/tool_call_parser.rs`
2. make that parser tolerant to both JSON and a narrow YAML-like arg format
3. propose and begin a cleaner module split for the rest of `tools.rs`

## Problem

### The harness currently assumes JSON-only `args:` for prompt tool calls

The internal planner protocol expects:

```text
TOOL_CALL
name: hydrate_actor_scope
args: {"actor_did":"did:plc:...","include_recent_posts":true}
```

But recent runs show the local model can emit:

```text
TOOL_CALL
name: hydrate_actor_scope
args:
  actor_did: did:plc:...
  include_clearsky_lists: true
  include_recent_posts: true
  include_recent_replies_received: true
```

This is still structurally a valid single tool call in human terms, but the current harness parser rejects it because it only extracts an `args` object when it finds a literal JSON `{...}` block.

That makes the parser too brittle for the local model we actually have.

### The current error labeling is misleading

Today the failure path can report:

- `strict internal mode requires exactly one TOOL_CALL block with no surrounding prose`

even when the real problem is:

- one clean `TOOL_CALL` exists
- but `args:` were emitted in YAML-like syntax rather than JSON

That makes debugging harder because the runtime points at the wrong defect class.

### `tools.rs` is carrying too many responsibilities

At 4611 lines, `src/harness/tools.rs` is no longer a maintainable unit.

The parser problem is a good forcing function to split it, because:

- parsing concerns should be isolated from execution concerns
- execution concerns should be isolated from rendering/review concerns
- unit tests for parser behavior should live close to the parser module

## Goal

Create a dedicated parser abstraction that:

- recognizes internal prompt-style `TOOL_CALL` blocks
- distinguishes clean tool calls from prose-contaminated output
- parses both JSON-style and narrow YAML-style `args:`
- normalizes parsed args into `serde_json::Value`
- reports more specific parse errors

And use that extraction as the first step toward breaking `tools.rs` into smaller harness modules.

## Scope

### In scope

- new `src/harness/tool_call_parser.rs`
- moving prompt-tool-call parsing logic out of `tools.rs`
- adding tolerant parsing for JSON or YAML-like tool args
- improving parse failure diagnostics
- updating `src/harness/mod.rs`
- proposing a modular split for the rest of `tools.rs`

### Out of scope

- rewriting all internal planner/runtime semantics
- changing the public tool protocol away from `TOOL_CALL`
- introducing arbitrary full YAML support for every possible structure
- solving root synthesis formatting or review heuristics in the same change

## Desired Parser Behavior

### Accepted forms

#### JSON args

```text
TOOL_CALL
name: hydrate_actor_scope
args: {"actor_did":"did:plc:...","include_recent_posts":true}
```

#### JSON args on following lines

```text
TOOL_CALL
name: hydrate_actor_scope
args:
{
  "actor_did": "did:plc:...",
  "include_recent_posts": true
}
```

#### Narrow YAML-like args

```text
TOOL_CALL
name: hydrate_actor_scope
args:
  actor_did: did:plc:...
  include_recent_posts: true
  include_recent_replies_received: true
```

### Normalized output

All accepted forms should normalize to the same internal shape:

- `PromptToolCall { name, args: serde_json::Value }`

Example normalized args:

```json
{
  "actor_did": "did:plc:frudpt5kpurby7s7qdaz7zyw",
  "include_recent_posts": true,
  "include_recent_replies_received": true
}
```

### Preferred but not required model behavior

The prompt should still prefer JSON.

The parser tolerance is a harness-robustness feature, not a statement that YAML is the new canonical protocol.

## Parser Design

### New module

Create:

- `src/harness/tool_call_parser.rs`

It should own:

- extraction of the leading `TOOL_CALL` block
- parsing of `name:`
- parsing of `args:`
- JSON-first then YAML-fallback parsing
- normalized parser result types
- parser-specific error classification

### Suggested API

The concrete names can vary, but the module should expose something close to:

- `parse_internal_tool_response(response: &str) -> InternalToolResponse`
- `parse_prompt_tool_call(response: &str) -> Result<PromptToolCall, ToolCallParseError>`
- `extract_leading_tool_call_block(response: &str) -> Result<AcceptedToolCallBlock, ToolCallParseError>`
- `parse_tool_args(raw: &str) -> Result<Value, ToolCallParseError>`

Suggested parser error categories:

- `NoToolCallPresent`
- `LeadingProseBeforeToolCall`
- `MissingToolName`
- `MissingArgs`
- `ArgsNotParseableAsJsonOrYaml`
- `MultipleToolBlocksOrTrailingOutput`

These names do not have to be exact, but the idea is to stop flattening all parse failures into one vague reason.

### YAML support should be deliberately narrow

Do not try to support arbitrary YAML.

The intended supported subset is:

- top-level mappings only
- scalar values:
  - strings
  - booleans
  - integers
  - floats if needed
- possibly simple string arrays later, but only if current tool args need them

If nested objects or complex lists are needed in the future, they can be added intentionally.

This keeps the parser predictable and avoids turning it into a general configuration language.

### Parse order

Recommended strategy:

1. try to extract a JSON object if `{` is present after `args:`
2. if JSON parse succeeds, use it
3. if JSON object is absent or parse fails, try parsing the indented YAML-like block
4. normalize parsed YAML into `serde_json::Value`
5. if both fail, return a specific parse error

## Diagnostics

### Current misleading behavior

Today a YAML-style arg block can show up as:

- `strict internal mode requires exactly one TOOL_CALL block with no surrounding prose`

That is the wrong diagnosis.

### Desired behavior

The runtime should instead be able to report things like:

- `single TOOL_CALL found, but args were not parseable as JSON or YAML`
- `single TOOL_CALL found, YAML-style args were accepted and normalized`
- `invalid TOOL_CALL: extra text before tool block`
- `invalid TOOL_CALL: trailing prose after accepted block`

This matters because:

- it reduces prompt-debug confusion
- it tells us whether the model is structurally close or actually off-protocol
- it lets us decide later whether to tighten or relax prompt instructions

## Proposed Split For `tools.rs`

Do not try to atomically rewrite `tools.rs` into ten files in one change.

Instead, split it in phases.

### Phase 1: Extract parser concerns

Move into `src/harness/tool_call_parser.rs`:

- `parse_prompt_tool_call(...)`
- `extract_leading_tool_call_block(...)`
- `extract_first_args_object(...)`
- `parse_tool_args_json(...)`
- any new YAML fallback parser
- parser tests

This is the highest-value first extraction.

### Phase 2: Extract collection-search result parsing and repair helpers

Likely file:

- `src/harness/collection_search.rs`

Move concerns such as:

- `parse_llm_search_result(...)`
- collection-search JSON/plaintext result parsing
- fallback summary generation
- deterministic repair helpers
- selected-record rendering helpers tied to collection-search outputs

### Phase 3: Extract collection review logic

Likely file:

- `src/harness/collection_review.rs`

Move:

- `build_collection_review_context(...)`
- `parse_collection_review_verdict(...)`
- `heuristic_collection_review(...)`
- review-related helper types if they are not already centralized

### Phase 4: Extract internal `llm_search` planner/runtime loop

Likely file:

- `src/harness/internal_llm_search.rs`

Move:

- internal planner loop
- internal tool execution sequencing
- planner-visible outcome rendering
- planner retry / fallback logic

This should leave `tools.rs` focused more on:

- public tool registry
- top-level tool dispatch entrypoints
- coarse orchestration wiring

### Phase 5: Extract serialization / display helpers

Likely file:

- `src/harness/tool_rendering.rs`

Move:

- collection serialization for model context
- result rendering for transcripts / debug
- compact summary formatting helpers

## Recommended Final Module Shape

One reasonable target shape is:

- `src/harness/tools.rs`
  - top-level tool registry
  - public dispatch entrypoints
- `src/harness/tool_call_parser.rs`
  - internal `TOOL_CALL` parsing and normalization
- `src/harness/internal_llm_search.rs`
  - planner loop and internal tool execution flow
- `src/harness/collection_search.rs`
  - collection-search response parsing and deterministic repair
- `src/harness/collection_review.rs`
  - review context and heuristics
- `src/harness/tool_rendering.rs`
  - transcript/debug/context rendering helpers

The exact filenames can change, but the architectural boundary should be similar.

## Testing Requirements

Add targeted parser tests in the new parser module for:

- clean JSON tool call
- multiline JSON tool call
- narrow YAML-style arg block
- bool / number parsing in YAML-style args
- leading prose before tool call
- trailing prose after tool call
- malformed args that are neither JSON nor YAML

Add integration coverage to ensure:

- accepted YAML-style args normalize into the same `PromptToolCall` as JSON
- existing planner flows still accept canonical JSON tool calls unchanged

## Acceptance Criteria

- internal prompt tool-call parsing lives in `src/harness/tool_call_parser.rs`
- the harness accepts both canonical JSON args and the narrow YAML-like arg form produced by the local model
- parsed args are normalized to one internal `serde_json::Value` shape
- invalid parse diagnostics distinguish YAML/JSON arg failures from real surrounding-prose failures
- `src/harness/tools.rs` is smaller because parser logic moved out
- the task doc also leaves a clear staged plan for further splitting the remaining 4k+ line file

## Recommended Implementation Order

1. Create `tool_call_parser.rs` and move existing parser logic unchanged.
2. Add JSON-or-YAML arg normalization there.
3. Update diagnostics to report parser failure classes more accurately.
4. Wire `tools.rs` to consume the new parser API.
5. Land parser tests.
6. In follow-up commits, extract `collection_search`, `collection_review`, and planner/runtime modules incrementally.
