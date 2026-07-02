# Task: Add OpenAI-Style Tool Calling to `shock_absorber`

## Goal

Add a tool-aware agent loop to `shock_absorber` that:
- enumerates available harness tools
- explains what each tool does and when to use it
- loads that tool inventory into prompt context after the system prompt
- supports a path from prompt-only tool orchestration to real OpenAI-style tool calling

This should fit the current local testing setup:
- `shock_absorber` builds the prompt and orchestration state
- `evil_gemma` serves a local OpenAI-compatible `/v1/chat/completions`
- tool execution happens in Rust inside `shock_absorber`

## Current State

Current harness pieces:
- `src/harness/context_window.rs`
  - `LLMContext` is a prompt builder with a header and ordered sections
  - it serializes context into plain text with `build_context_window(...)`
- `src/harness/llm_api.rs`
  - sends basic OpenAI-style chat completion requests over REST
  - currently supports `model`, `messages`, `max_tokens`, and `stream`
  - does not support `tools`, `tool_choice`, or parsing tool-call responses
- `src/harness/tools.rs`
  - contains harness search logic
  - now exposes an LLM-mediated collection search path:
    - `llm_search`
  - does not yet expose a formal tool registry
- `src/model/mod.rs`
  - defines generic `PostRecord`
  - defines generic `LabeledPostCollection`

Current message submission path:
- `shock_absorber/system_prompt.md` is loaded as a real `system` message
- rendered `LLMContext` is sent as a real `user` message
- this preserves the role boundary on the wire

## Important Clarification

`LLMContext` is not the agent.

Right now:
- `LLMContext` is only a prompt/state container
- there is no first-class `Agent` type
- there is no first-class `Subagent` type
- there is no runtime tool loop

So at the moment:
- an "agent" is only implicit
- a "subagent" is only an extra narrower LLM call, such as `llm_search`

## Why Tool Inventory Needs To Be In Context

The model needs explicit guidance about:
- what tools exist
- what each tool is for
- what inputs each tool expects
- when to use direct post inspection vs an LLM-mediated collection search

The immediate prompt shape should be:
1. system prompt
2. tools section
3. user task section
4. later: selected notification context, actor context, search results, and tool outputs

## Desired Tool Registry

`src/harness/tools.rs` should grow a registry layer, not just tool implementations.

Suggested types:
- `ToolSpec`
- `ToolArgumentSpec`
- `ToolRegistry`

`ToolSpec` should include:
- `name`
- `description`
- `input schema`
- `when_to_use`
- `notes`

Example search tool to register first:

### `llm_search`

Purpose:
- search a named `LabeledPostCollection` using a synthesized prompt

Behavior:
- the calling agent generates the search prompt
- the tool performs a narrower LLM pass over the collection
- it chooses one post
- it returns a synthesized block that quotes and explains why that post is relevant

Expected inputs:
- `collection_id`
- `prompt`

When to use:
- when relevance depends on semantics rather than literal token overlap
- when the compact UI preview is insufficient

## How OpenAI-Style Tool Calling Works

There are two related but different patterns:

### 1. Chat Completions function calling

The client sends:
- `messages`
- `tools`
- optionally `tool_choice`

The model may respond with tool calls instead of a final answer.

Then the application:
1. inspects the returned tool call
2. executes the requested tool in application code
3. appends the tool output back into the conversation
4. calls the model again for the next step or final answer

Important implication:
- tool execution is still your application’s responsibility
- the model only chooses the tool and arguments

### 2. Responses API tool calling

The newer Responses API uses a similar loop, but models tool calls and tool outputs as distinct response items correlated by `call_id`.

Conceptually it is the same orchestration pattern:
- model requests tool use
- application executes tool
- application returns tool output
- model continues

## Important Constraint In This Repo

Tool calling is not implemented yet just because:
- `evil_gemma` exposes `/v1/chat/completions`
- messages are sent in an OpenAI-like shape

Current local stack does not yet support real function calling end to end:
- `src/harness/llm_api.rs` does not send `tools`
- it does not parse `tool_calls`
- `evil_gemma/app.py` currently supports basic chat-completions semantics, not a full tool-calling loop

So for this repo, "OpenAI-style tool calling" is still a feature to build.

## Recommended Implementation Path

Start with prompt-level orchestration first, then add real wire-level tool calling later.

### Phase 1: Prompt-only tool protocol

Add a tool inventory section to context and instruct the model to emit a strict tool request block such as:

```text
TOOL_CALL
name: llm_search
args: {"collection_id":"recent_posts:did:plc:...","prompt":"find posts about harassment accusations or public disputes"}
```

Then `shock_absorber`:
1. parses the tool request
2. runs the Rust tool locally
3. appends the tool output as a follow-up chat turn
4. makes another LLM call

Why this is the best first step:
- works with the current local Gemma bridge
- avoids needing immediate server/runtime changes
- lets prompt and tool semantics stabilize before wire-protocol work

### Phase 2: Real Chat Completions function calling

Extend `src/harness/llm_api.rs` to:
- send `tools`
- send `tool_choice`
- parse returned `tool_calls`

Extend `evil_gemma` to:
- accept tool definitions on input
- preserve tool-related fields
- return tool-call objects in the OpenAI chat-completions shape

Then implement the Rust orchestration loop in `shock_absorber`.

## Prompt Construction Plan

Near-term prompt structure should look like:

### System message

Loaded from:
- `system_prompt.md`

Contains:
- persona / instruction style
- high-level agent behavior

### User message

Rendered from `LLMContext`

Sections should include:
- `Tools`
- `User Query`
- later: `Selected Notification`
- later: `Actor Facts`
- later: `Tool Results`

The `Tools` section should be rendered from the registry, not hand-written in `main.rs`.

## Suggested New Abstractions

The harness is now large enough that a proper agent layer should exist.

Suggested additions:
- `src/harness/agent.rs`
- `AgentDefinition`
- `AgentSession`
- `ToolRegistry`
- `ToolSpec`

Clean separation:
- `LLMContext`
  - prompt/state serialization only
- `ToolRegistry`
  - available tools and their schemas
- `AgentDefinition`
  - system prompt, tool access policy, provider config
- `AgentSession`
  - active context, tool results, orchestration loop state

Then:
- main agent = broad task + full tool access
- subagent = narrower task + reduced tool access + child context

## Concrete Next Steps

1. Add `ToolSpec` and `ToolRegistry` to `src/harness/tools.rs` or a nearby module.
2. Implement a renderer that serializes the available tools into a `Tools` context section.
3. Load that section immediately after the system prompt and before the user task.
4. Define a strict text protocol for prompt-only tool requests.
5. Add a parser for that protocol in `shock_absorber`.
6. Add an orchestration loop that:
   - calls the model
   - executes one requested tool
   - appends the tool result
   - calls the model again
7. Only after that is stable, extend the REST layer for real OpenAI-style function calling.

## Acceptance Criteria

This task is complete when:
- available tools can be enumerated from code, not manually described
- a `Tools` section is inserted into the LLM context after the system prompt
- the model can request `llm_search` through a defined protocol
- `shock_absorber` can execute that request and feed the result back into the conversation
- the design leaves room for later migration to true OpenAI-style `tools` / `tool_calls`
