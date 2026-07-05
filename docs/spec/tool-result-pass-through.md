# Tool Result Pass-Through Spec

## Goal

Document the current root/tool/subagent prompt chain in code and define the expected behavior when a tool result, especially `llm_search`, already answers the user's question.

This spec is intentionally about the current prompt-driven harness, not a future generic agent framework.

## Current Prompt Sources

The relevant prompt and projection layers currently live in these files:

- [system_prompt.md](/home/rei/_/shock_absorber/system_prompt.md)
  - root persona and top-level answering style
- [src/harness/prompts/tool_prompt.md](/home/rei/_/shock_absorber/src/harness/prompts/tool_prompt.md)
  - root-agent tool protocol and post-tool instructions
- [src/harness/prompts/agents/collection_search.md](/home/rei/_/shock_absorber/src/harness/prompts/agents/collection_search.md)
  - per-collection child-search instructions
- [src/harness/tools.rs](/home/rei/_/shock_absorber/src/harness/tools.rs)
  - child result parsing
  - parent `llm_search` synthesis
  - outward-facing tool result rendering
- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs)
  - root conversation loop
  - tool result compaction before feeding results back to the root model

## Current Execution Path

### 1. Root agent request

The app sends:

- `system`: `system_prompt.md` plus `tool_prompt.md`
- `user`: rendered root context window

That happens in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:520).

### 2. Root tool choice

The root model may emit a strict text `TOOL_CALL` block.

That protocol is defined in [src/harness/prompts/tool_prompt.md](/home/rei/_/shock_absorber/src/harness/prompts/tool_prompt.md).

### 3. Tool execution

For `llm_search`, the harness:

- resolves the requested collections
- runs one child search agent per collection
- synthesizes a parent `llm_search` result for multi-collection searches

This logic is implemented in [src/harness/tools.rs](/home/rei/_/shock_absorber/src/harness/tools.rs:379), [src/harness/tools.rs](/home/rei/_/shock_absorber/src/harness/tools.rs:1172), and [src/harness/tools.rs](/home/rei/_/shock_absorber/src/harness/tools.rs:1310).

### 4. Tool result projection back to root

The root model does not receive the raw full tool transcript. It receives a compacted follow-up message:

`Tool Result`
`name: ...`
`args: ...`
`...compacted result...`

That handoff is built in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:847).

The compaction rule for `llm_search` lives in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1487).

## Why The Root Answer Can Get Worse Than The Tool Result

There are two separate failure modes.

### Weak root prompt

Historically, the root prompt only said:

- answer directly or request one more tool

That leaves too much freedom for the root model to:

- rephrase a strong tool answer into something vaguer
- add persona fluff
- summarize when no extra synthesis is actually needed

### Lossy tool-result projection

Historically, the root compaction step kept metadata lines but could drop the actual `summary:` from `llm_search`.

That means the root model might be asked to answer from:

- collection labels
- status lines
- selected URI metadata

without seeing the grounded natural-language summary that the child or parent search agent already produced.

## Desired Default Behavior

When the latest `llm_search` result already answers the user’s question:

- the root agent should treat that result as the answer by default
- the root agent should preserve the substance of the tool `summary:`
- if the root agent has nothing material to add, it should answer with a minimal restatement or near pass-through

Examples of valid additions:

- combining multiple tool results
- directly answering a narrower user question using the tool findings
- stating uncertainty or missing evidence

Examples of invalid additions:

- decorative persona framing
- more abstract paraphrase that removes concrete findings
- replacing specific grounded themes with generic labels like “negative reinforcement” unless the tool result itself supports that framing

## Projection Contract

The intended contract from `llm_search` to the root caller is:

1. `llm_search` returns grounded `summary:` text and concrete `search_result_*` or `selected_result_*` identifiers.
2. The root compaction layer must preserve the grounded `summary:` when feeding the result back into the root context.
3. The root prompt must explicitly tell the caller to reuse that grounded result by default.
4. The root model should only perform additional synthesis if it is adding information, not merely rewording it.

## Current Code Direction

The current code should converge on these principles:

- prompts live in code-backed files, not hidden string fragments in random call sites
- child agents produce grounded summaries
- parent tool agents may synthesize across child results
- root callers should not degrade a better subagent answer during final rendering

## Discussion Questions

These are the main review points worth critiquing next:

1. Should the root caller ever rewrite a single strong `llm_search` result, or should it default to a stricter pass-through mode?
2. Should `Tool Result` follow-up messages preserve the full `summary:` plus all selected URIs, even if that costs more tokens?
3. Should the harness add an explicit post-tool instruction like:
   `If the tool result already answers the question, answer by re-emitting that result with only minimal adaptation`?
4. Should persona/style instructions be kept separate from tool-result handling rules so they cannot override evidence fidelity?
