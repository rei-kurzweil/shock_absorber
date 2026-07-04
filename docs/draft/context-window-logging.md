# Context Window Logging Draft

Goal: make agent context windows and the visible AI/tool transcript easy to inspect outside the TUI by writing stable debug artifacts to disk.

This draft is intentionally narrow.
It does not try to redesign the full harness or keep every debug artifact permanently in sync.
The near-term goal is:

- make `/context` easier to verify
- make current-task state explicit
- let us inspect root/tool/search windows from disk after an agent finishes
- make copy/paste of the active AI transcript easy without relying on TUI scrolling

## Current Problems

Right now we can inspect context windows only through the fullscreen visualization.

That creates several problems:

- the root visualization is hard to compare against the actual prompt payloads
- the UI is awkward for copy/paste
- when a root query spawns multiple search agents, we need direct access to each built window
- there is no simple command for "what does the harness think the current task is?"
- there is no durable debug record after the UI state changes

We also now have a real agent tree in memory, so the missing piece is logging and inspection rather than inference.

## Near-Term Goals

Add a small debug logging layer that writes:

- one file per finished agent node context window
- one easy-to-copy transcript file for the current visible AI/tool exchange
- one task snapshot file for the current root task

These files should live under:

- `.debug/`

Near-term acceptable behavior:

- overwrite debug files on startup
- overwrite per-agent files when the same agent node is rerun
- sync only when an agent finishes its task or tool step
- no persistence guarantees across app restarts

## Proposed Module

Add a small helper module:

- `src/harness/context_window_logger.rs`

The helper should not own agent execution.
It should only:

- accept completed agent nodes and transcript strings
- render stable debug text/markdown
- write those files to `.debug/`

Near-term this can be called directly from the app loop when:

- the root agent finishes a query
- an `llm_search` tool agent finishes
- a collection-search child agent finishes

It does not need to stream partial updates yet.

## Proposed Files

At startup:

- ensure `.debug/` exists
- clear or recreate the files we own in that directory

Near-term files:

- `.debug/chat_transcript.md`
- `.debug/current_task.txt`
- `.debug/agents/root_agent.md`
- `.debug/agents/tool_agent_llm_search_<n>.md`
- `.debug/agents/collection_search_agent_<n>.md`

Alternative file naming if stable numeric IDs are easier:

- `.debug/agents/agent_000_root.md`
- `.debug/agents/agent_001_tool_llm_search.md`
- `.debug/agents/agent_002_collection_recent_posts.md`

The important part is:

- filenames should be stable enough to compare across one run
- they should include node kind and enough label info to be recognizable

## What To Log Per Agent

Each finished agent file should include at least:

- `agent_id`
- `agent_type`
- `label`
- `status`
- `parent_agent_id`
- `child_agent_ids`
- `tool_name` when relevant
- `collection_id` when relevant
- `result_summary`
- full rendered context window
- context-window token stats

Suggested markdown shape:

```md
# Agent Debug

- agent_id: 3
- agent_type: collection_search_agent
- label: collection search: Recent replies sent by did:plc:...
- status: completed
- parent_agent_id: 1
- child_agent_ids:
- collection_id: recent_replies_sent:did:plc:...

## Result Summary

...

## Context Window Stats

- provider: llama.cpp
- model: gemma-4-local
- max_context_tokens: 8192
- reserved_output_tokens: 1024
- used_input_tokens: 1460
- truncated: false

## Rendered Context Window

```text
...
```
```

## Root Task Visibility

The current root task is presently just the query string passed into the root context builder.

That is too implicit.

We should add a no-argument command:

- `/task`

Near-term `/task` should print:

- the current root task string
- the root agent ID
- whether the root agent is idle, running, or completed
- the latest root result summary if present

Near-term acceptable behavior:

- `/task` only reports the active root query/task
- no task history yet

We should also mirror the same data to:

- `.debug/current_task.txt`

Suggested shape:

```text
root_agent_id: 0
status: completed
current_task: lets see what sentiment exists about this user by looking at which lists they're on
```

## Transcript Logging

The TUI transcript is useful but awkward to copy because it lives behind scrolling and mixed overlays.

We should write a single markdown file:

- `.debug/chat_transcript.md`

This file should contain only the user-visible AI/tool transcript for the active root exchange:

- user query
- tool transcript entries
- final answer

It should not include hidden diagnostic structures unless we later choose to add a second debug transcript file.

Suggested shape:

```md
# Chat Transcript

## User Query

...

## Tool Transcript

### Tool Call

...

### Tool Result

...

## Final Answer

...
```

## Logging Timing

We do not need every mutation to be logged instantly.

Near-term preferred sync points:

1. when a collection-search agent completes
2. when an `llm_search` tool agent completes
3. when the root query completes

This gives us useful files without turning the logger into an event stream.

If an agent fails:

- still log the node
- include the failure status and error summary

## Rendering Guidance

The logger should write the real rendered context windows, not reconstructed summaries.

That means:

- use the stored `BuiltContextWindow.rendered`
- include token stats from the stored `BuiltContextWindow`
- avoid re-serializing the source sections in a different format for the main artifact

The markdown wrapper can add metadata, but the prompt body itself should be the exact rendered window used for the LLM call.

## Open Questions

- whether the root system prompt and tool protocol should also be logged as standalone files
- whether every agent node should get both a `.md` wrapper and a raw `.txt` prompt file
- whether `.debug/` should be gitignored explicitly
- whether `/task` should eventually support inspecting child agents too
- whether transcript logging should happen after every tool round or only after final answer

## Suggested First Implementation

1. Add `src/harness/context_window_logger.rs`.
2. Create `.debug/` and `.debug/agents/` on startup, clearing files we manage.
3. Log finished agent nodes from the in-memory agent graph.
4. Log `.debug/chat_transcript.md` when the root query finishes.
5. Add `/task` to print the active root task and status.

This is enough to make `/context` debugging practical without committing to a permanent diagnostics architecture yet.
