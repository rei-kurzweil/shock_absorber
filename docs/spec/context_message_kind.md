# Spec: `ContextMessageKind`

## Purpose

`ContextMessageKind` defines the typed semantic role of messages that the harness appends to the root-agent conversation.

This exists so `/context` and other runtime views can classify root conversation entries without re-parsing rendered strings such as:

- `Tool Result\nname: ...`
- `TOOL_CALL`
- repair text

The intent is:

- typed at write time
- consumed directly at render/debug time
- no substring inference for canonical root message semantics

## Rust Location

Today this type lives in:

- [src/harness/runtime.rs](/home/rei/_/shock_absorber/src/harness/runtime.rs)

Related root snapshot usage lives in:

- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs)

## Current Type

Today the root runtime uses:

```rust
enum ContextMessageKind {
    InitialSystem,
    InitialUserContext,
    ToolRequest,
    ToolResult,
    AssistantReply,
    UserFollowUp,
    RoundLimitPrompt,
    RepairPrompt,
}
```

and stores:

```rust
struct ContextMessage {
    kind: ContextMessageKind,
    message: ChatMessage,
}
```

inside `RootRunState`.

## Meaning Of Each Kind

### `InitialSystem`

The initial system message for the root agent.

Today this contains:

- the main system prompt
- the root-level tool protocol instructions

Current producers:

- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:557)
- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1228)

Current `/context` handling:

- not rendered as a follow-up message entry
- represented separately as `System Prompt` and `Tool Instructions` in the root snapshot

### `InitialUserContext`

The initial user-context message sent to the root agent at run start.

Today this contains the rendered root context window, including sections such as:

- `Tools`
- `Search Hints`
- `Current UI Context`
- `Current Task`
- `Recent Chat`

Current producers:

- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:570)
- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1239)

Current `/context` handling:

- not rendered as a follow-up message entry
- represented separately via the typed root context-window snapshot

### `ToolRequest`

The assistant message that requested a tool.

Today this is usually the raw assistant output containing a `TOOL_CALL` block.

Current producers:

- root foreground loop in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:891)
- root background loop in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1870)

Current `/context` handling:

- shown as `Tool Request #N`
- categorized as tool-result/tool-loop context rather than user chat

### `ToolResult`

The synthetic user-side message that feeds a tool result back into the root agent after tool execution.

Today this contains:

- `Tool Result`
- tool name
- serialized args
- compacted tool output
- harness instructions about whether another tool round is allowed

Current producers:

- root foreground loop in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:896)
- root background loop in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1875)

Current `/context` handling:

- shown as `Tool Result #N`
- increments the displayed tool-round counter

### `AssistantReply`

An assistant-authored non-tool reply inside the root loop.

Today this is mainly used when:

- the model produced a non-tool answer
- that answer is then kept in conversation before a repair pass

Current producers:

- repair path in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:966)
- repair path in [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1937)

Current `/context` handling:

- shown as `Assistant Reply`
- categorized as chat context

Note:

- the final rendered answer is currently appended only to the final visualization snapshot, not persisted back into `RootRunState` as a normal follow-up message

### `UserFollowUp`

A user-authored follow-up inside the root run after startup.

Today this is mostly reserved for future use.

Current producers:

- none in the current root loop

Current `/context` handling:

- shown as `User Follow-up`
- categorized as chat context

### `RoundLimitPrompt`

A synthetic harness-authored prompt injected when the root loop has already used the maximum allowed tool rounds but the model is still trying to request another tool.

Current producers:

- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:942)
- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1912)

Current `/context` handling:

- shown as `Round Limit Prompt`
- categorized as current-task/runtime-control context

### `RepairPrompt`

A synthetic harness-authored prompt injected when the latest assistant answer appears cut off or incomplete.

Today the repair instruction asks the model to:

- finish directly
- avoid `TOOL_CALL`
- keep the answer short and complete

Current producers:

- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:967)
- [src/main.rs](/home/rei/_/shock_absorber/src/main.rs:1941)

Current `/context` handling:

- shown as `Repair Prompt`
- categorized as current-task/runtime-control context

## Current Consumer

The main consumer today is:

- [build_root_context_snapshot(...)](/home/rei/_/shock_absorber/src/main.rs:2082)

That function now classifies follow-up messages by `ContextMessageKind` instead of by inspecting the rendered message text.

## Why This Exists

Without `ContextMessageKind`, the harness had to infer semantics from string content such as:

- whether a message parsed as `TOOL_CALL`
- whether a user message started with `Tool Result`
- whether a user message matched a repair or round-limit string

That made `/context` inconsistent and fragile.

Typed message kinds make the root context view:

- deterministic
- easier to extend
- easier to debug
- aligned with the rest of the typed runtime model

## Current Scope

This spec only covers root-run conversation messages.

It does not yet define:

- typed internal `llm_search` private-loop messages
- typed tool-result payload structs
- typed per-agent verify/tool state-machine records

Those are related future steps, but separate from this root message-kind contract.
