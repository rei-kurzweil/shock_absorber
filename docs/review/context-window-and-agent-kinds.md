# Context Window And Agent Kinds Review

## What Changed

- Internal agent prompts now come from `AgentKind` instead of mixing `AgentKind` with ad hoc string constants.
- `llm_search` now has two explicit internal agent kinds:
  - `LlmSearch`
  - `LlmSearchParent`
- `LLMContext` now stores rendered header and section text plus estimated token counts at insertion time.
- `BuiltContextSection` now keeps both:
  - `estimated_tokens`: the full section estimate before truncation
  - `used_tokens`: the amount that actually fit into the built context window

## Current Data Flow

1. Code creates an `LLMContext` with a header.
2. Each `push_section(...)` call renders the section immediately and stores its estimated token count on the section struct.
3. `build_context_window_report(...)` consumes those stored estimates instead of recomputing them from raw title/body pairs.
4. If a section is truncated, the built report preserves both the original estimate and the included token count.
5. Visualizations still render from `BuiltContextWindow` or from a root snapshot assembled in `main.rs`.

## Improvements From This Refactor

- Prompt definition is more consistent across internal agents.
- Token estimation is now closer to the write path, not reconstructed later from raw strings.
- Truncated sections can now distinguish "how large the section was" from "how much fit".
- Agent graph debug output can now identify the specific internal agent kind.

## Remaining Issues

### 1. Root visualization still reconstructs part of the prompt outside `LLMContext`

`build_root_context_snapshot(...)` in `src/main.rs` still estimates token counts for:

- top-level system prompt
- tool protocol instructions
- follow-up chat/tool-result messages

Those segments are not represented as first-class `ContextSection` or `BuiltContextSection` records. That means the root visualization is still partly built from side calculations rather than one canonical context structure.

### 2. Header accounting is still special-cased

The context builder now stores header estimates on `LLMContext`, but `BuiltContextWindow` still exposes `header_tokens` separately from `sections`. That is workable, but it means header and section accounting are still structurally different.

### 3. Visualization categories are still title-driven in places

`src/visualization/context.rs` maps categories from section titles like `Tools`, `Current Task`, and `Per-Collection Results`. That is fragile. A better long-term direction is for the producer of each section to declare its semantic category directly.

### 4. Root and subagent snapshots do not share one segment schema

Subagent windows come directly from `BuiltContextWindow`.

Root snapshots combine:

- prompt metadata outside the built window
- built-window sections
- later conversation turns

That makes the root path harder to reason about than the subagent path.

## Recommended Follow-Up

1. Introduce a semantic section kind on `ContextSection` or `BuiltContextSection`.
2. Represent root-only prompt pieces with the same section model used elsewhere.
3. Consider replacing `header_tokens` with a first-class built header segment if the visualization should treat headers and sections uniformly.
4. If internal agents continue to grow, use `AgentKind` as the canonical source for:
   - prompt file
   - debug label
   - visualization/system-instruction label
