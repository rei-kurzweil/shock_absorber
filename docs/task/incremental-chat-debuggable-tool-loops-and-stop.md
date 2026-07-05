# Task: Make Root-Agent Runs Inspectable, Incremental, And Stoppable

## Goal

Document and then implement the runtime refactor needed to make root-agent runs:

- inspectable while they are in progress
- debuggable after they finish
- incremental in the AI chat output
- cancellable from the UI with a stop action such as `/stop`

This task is driven by the current behavior of slow `llm_search` runs:

- the root agent can repeat the same high-level search multiple times
- the user cannot clearly see provider output and tool output arrive as separate live sections
- the run cannot be interrupted once the root-agent loop is active

## What The Latest `.debug` Run Shows

The latest captured run is not one endless search loop.

It is three separate `llm_search` tool rounds over the same actor-backed collections, followed by a final answer.

Observed artifacts:

- `.debug/agents/agent_000_root_agent.md`
  - root agent child IDs are `1, 7, 13`
- `.debug/agents/agent_001_tool_agent_llm_search_tool_agent.md`
- `.debug/agents/agent_007_tool_agent_llm_search_tool_agent.md`
- `.debug/agents/agent_013_tool_agent_llm_search_tool_agent.md`

Each of those tool agents searched the same collection set:

- `actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5`
- `clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5`
- `pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5`
- `recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5`
- `recent_replies_sent:did:plc:6lwfvmss45d7j7fot34v2kw5`

The root prompt snapshot confirms the repeated root-level tool rounds:

- `.debug/root_prompt_snapshot.md`
  - `Tool Request #1`
  - `Tool Result #1`
  - `Tool Request #2`
  - `Tool Result #2`
  - `Tool Request #3`
  - `Tool Result #3`
  - `Tool Request #4`
  - `Round Limit Prompt`

Important measured cost from that snapshot:

- root input used: `6857 / 7168`
- tool output alone: `4818` estimated input tokens

So the current problem is:

- repeated high-level search calls
- huge repeated tool-result reinjection into the root prompt
- weak live visibility into what changed between rounds

## Current Runtime Shape

The current root-agent loop lives in `App::run_evil_gemma_query(...)` in `src/main.rs`.

Important current behavior:

1. build one root context window
2. call the provider with the full root prompt
3. if a tool call appears:
   - run tool prep
   - execute the tool
   - append the tool transcript
   - append a compact tool-result message back into the conversation
4. call the provider again
5. repeat up to `MAX_TOOL_CALL_ROUNDS`

Important current UI limitation:

- AI chat output is updated by repeatedly replacing one `DetailView::AiChat`
- it is not modeled as a structured sequence of streaming or incremental entries
- provider output, user messages, tool requests, tool prep logs, and tool results are flattened into one text blob

Important current cancellation limitation:

- the root-agent run happens inside one awaited async flow
- there is no shared cancellation token or stop flag
- there is no `/stop` command
- there is no UI button path to request cancellation
- child tool/subagent work has no parent-linked cancellation path

## Desired Behavior

### 1. AI chat output must be structured and incremental

The user should be able to see a live run as ordered chat-like sections, not just as one replaced panel.

The display should distinguish:

- user chat input
- root-agent/provider output in progress
- tool requests
- tool prep steps
- tool results
- final assistant answer

These should be visible in order as they happen.

Important requirement:

- user chat and AI chat happen sequentially, but both belong in one visible run transcript
- tool output from `llm_search`, `read_collection_item`, and similar tools should also appear inline in that same transcript

### 2. Root-agent runs must be stoppable

The user should be able to stop a running root-agent tree from the UI.

Support at least:

- `/stop`

Optional later UI affordance:

- a visible stop button

Stopping must affect:

- the root agent
- any active provider request
- any tool execution currently in progress
- any subordinate collection-search or tool-agent work spawned by that root run

### 3. `.debug` and live UI must explain repeated work

If the root agent runs the same high-level search multiple times, the system should make that obvious.

We need to be able to answer:

- did the root agent repeat the same tool call?
- did the same collection set run again?
- did the follow-up run differ in query, scope, or result?
- how much prompt budget did that repeated work consume?

## Implementation Changes

### A. Introduce a structured run transcript model

Add a typed model for in-progress and completed AI chat output instead of building one text blob directly.

Minimum entry kinds:

- user message
- provider thought/output chunk
- tool request
- tool prep log
- tool result
- system/runtime notice
- final assistant answer

The `AiChat` detail view should render this transcript as ordered sections.

The current helpers:

- `set_evil_gemma_progress(...)`
- `set_ai_chat_output(...)`
- `build_evil_gemma_output_lines(...)`

should be replaced or wrapped so they append/update structured entries instead of reserializing everything from scratch each time.

### B. Separate provider output from tool transcript state

Right now the root loop accumulates `tool_transcript: Vec<String>` and only stores the final provider answer string.

Refactor this so provider output and tool output are tracked independently:

- provider output buffer for the current round
- persisted tool-request/result entries for prior rounds
- final answer entry when the run completes

This is required so the UI can show:

- what the provider most recently said
- what tool was requested
- what the tool returned
- what changed on the next provider round

### C. Record root-run rounds explicitly

Add explicit root-run round metadata so repeated searches are first-class runtime facts.

Minimum per-round fields:

- round index
- provider request prompt summary
- parsed tool call
- tool arguments
- tool result summary
- whether the same tool/query/scope repeated

This should be reflected in:

- live AI chat transcript
- agent graph or root-run state
- `.debug` artifacts

### D. Add cancellation plumbing for root and child work

Introduce a root-run cancellation primitive shared across:

- root provider call
- tool prep steps
- tool execution
- subordinate agent work

Preferred design:

- one cancellation token or atomic stop flag owned by the active root run
- every awaited phase checks it before starting the next step
- long-running loops and fanout points check it between units of work

When cancellation is requested:

- mark the root agent as stopped/cancelled
- stop spawning new child work
- stop rendering the run as active
- append a visible transcript entry indicating cancellation

### E. Add `/stop`

Add a command handler for `/stop` that:

- cancels the currently active root run if one exists
- returns a clear command/AI-chat message if no run is active

Important runtime implication:

- the current input/event loop must remain responsive while a root run is active

If the current architecture blocks that, then the root-agent run must move into a background task owned by app state, with UI polling or message passing for progress updates.

### F. Improve `.debug` artifacts for repeated tool calls

Extend `.debug` output so repeated searches are easier to audit.

Add at least:

- one per-round root-run summary file or section
- repeated-call detection note when the same `llm_search(query)` is invoked again
- collection-set fingerprint or explicit collection list per tool-agent run
- prompt-token contribution by each tool-result reinjection

### G. Keep current agent tree, but expose live status better

The current agent graph is already useful for post-run inspection.

For this task it should also support live status fields such as:

- ready
- running
- completed
- failed
- cancelled

And the UI should be able to show running progress from that state while the root run is active.

## Acceptance Criteria

This task is complete when:

- a running root-agent query can be stopped with `/stop`
- stopping cancels both the root agent and any child work still in flight
- AI chat output shows user input, provider output, tool requests, tool prep, and tool results as ordered incremental sections
- repeated `llm_search` rounds are obvious both live and in `.debug`
- the system can distinguish slow single-run work from repeated root-level tool re-invocation
- the final `.debug` output makes it easy to verify whether the same collections were searched multiple times

## Test And Verification Scenarios

### Incremental transcript

- run a query that triggers one `llm_search`
- verify the UI shows:
  - user query
  - provider/tool request
  - tool prep
  - tool result
  - final answer

### Repeated root-level search

- run a query that currently tends to produce repeated `llm_search` calls
- verify repeated rounds are visible as distinct transcript segments
- verify `.debug` shows each round and its collection set clearly

### Stop during provider call

- issue `/stop` while the provider request is in progress
- verify the run ends in cancelled/stopped state

### Stop during tool execution

- issue `/stop` while `llm_search` or collection-search children are active
- verify no new child work is started after cancellation

### Post-run audit

- inspect `.debug` after a completed run
- verify root-level repeated work and tool-result prompt cost are easy to identify

## Important Notes

- This task is about runtime structure and observability first, not about changing provider backends.
- OpenRouter fallback support should be documented separately in another `docs/task` item.
- The latest debug evidence strongly suggests that repeated high-level tool calls are the bigger prompt-budget problem than any single collection-search child.
