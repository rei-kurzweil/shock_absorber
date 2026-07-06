# Task: Sequence The Runtime Refactor For Root Runs And Internal Search Agents

## Goal

Define the implementation order for the planned runtime and search-agent work so we establish the right substrate before adding more internal orchestration.

This document is an aggregate sequencing task.

It answers:

- what has to come first
- which existing task docs depend on other ones
- when internal `collection_search` tool definitions should be introduced

## Short Answer

Yes: first-class run state has to happen first.

Not necessarily as a giant standalone feature, but as the first implementation phase that the other tasks build on.

Without first-class run state:

- `/stop` has nothing stable to cancel
- repeated-call guards only exist as ad hoc loop checks
- incremental AI chat output has no authoritative live run object to render
- internal `llm_search -> collection_search` tool definitions would add more nested work before we can inspect, stop, or explain it

## Current Pressure

The current system already shows three tightly coupled problems:

1. repeated root-level `llm_search` calls
2. weak incremental visibility into provider and tool output
3. no cancellation path for an active root-agent tree

All three point to the same missing substrate:

- one first-class active root-run object with owned state

As of the current implementation, most of this substrate now exists:

- root runs are first-class runtime objects
- `/stop` exists
- tool/subagent progress is emitted into the live chat view

That changes the sequencing question.

We no longer need to defer all internal `llm_search` toolification until after basic observability exists.

## Important Clarification From The Current UI

The current live transcript can make one collection search look like two “agents” because it renders:

- one `status: running` progress event
- then one `status: completed` progress event

with the same agent label.

That is a transcript-modeling issue, not necessarily proof that the same collection search executed twice.

So there are now two separate problems to keep distinct:

1. duplicate-looking progress rendering for one child agent
2. overly heuristic internal planning inside `llm_search`

The first problem should not be mistaken for the second.

## Recommended Order

### Phase 0: First-class root-run state

Establish a real runtime object for an active root-agent run.

Minimum responsibilities:

- stable run ID
- current status
- round history
- live transcript entries
- tool-call history
- active agent graph reference
- cancellation token or stop flag

This is the prerequisite for everything else below.

This phase should absorb the most important runtime pieces from:

- `docs/task/incremental-chat-debuggable-tool-loops-and-stop.md`
- `docs/draft/ui-activity-model.md`
- the relevant runtime ownership ideas from `docs/draft/event-handlers-and-persistent-agents.md`

Important constraint:

- do not try to build the full persistent-agent/event system first
- only establish the root-run state needed for:
  - live progress
  - cancellation
  - round tracking
  - transcript ownership

### Phase 1: Repeated tool-call guard

After the root run exists as a first-class object, implement:

- `docs/task/repeated-tool-call-guard-and-confirmation.md`

This should become a policy attached to the run state, not just a local check in `run_evil_gemma_query(...)`.

Why it comes here:

- duplicate-call history belongs to the run object
- warnings should be added to the run transcript
- `.debug` output should be derived from the run state

### Phase 2: Incremental AI chat transcript and `/stop`

Continue the root-run refactor by implementing the user-visible runtime behaviors from:

- `docs/task/incremental-chat-debuggable-tool-loops-and-stop.md`

Specifically:

- structured incremental transcript entries
- live provider/tool/progress sections
- `/stop`
- cancellation propagation into active child work

Why it comes before deeper internal toolification:

- once `llm_search` starts spawning more internal tool-style work, the UI and debug path must already be able to show and interrupt that work

### Phase 3: Internal `llm_search` orchestration cleanup

Continue:

- `docs/task/high-level-llm-search-and-global-bluesky-lookup.md`

But with one important architectural shift:

- move from Rust-only collection selection toward explicit internal orchestration records owned by the `llm_search` run subtree

Near-term outcome:

- actor/global search planning becomes visible as runtime steps
- collection hydration and search fanout become more inspectable

This phase may still use Rust control flow for some loaders, but they should now be attached to the live run state and transcript.

### Phase 4: Internal `collection_search` tool definitions

Only after the phases above should we introduce internal tool definitions for:

- `collection_search`

Recommended shape:

- root agent sees one high-level tool: `llm_search`
- `llm_search` internally has access to `collection_search`
- `llm_search` may call `collection_search` multiple times with different scoped inputs before synthesizing

Why this is not phase 0:

- without run state and transcript plumbing, nested internal tool calls would be hard to inspect
- without cancellation, nested internal tool calls would be hard to stop
- without duplicate-call tracking, repeated internal fanout could become another opaque prompt-budget problem

However, this phase is now the most important next planning refactor.

The current Rust heuristic path:

- `detect_actor_refs(query)`
- `collections_for_actor_refs(...)`
- implicit eager hydration of actor collections

is now the main source of brittle behavior.

That logic should move behind explicit internal tool definitions owned by the `llm_search` subtree, so the `llm_search` agent can choose when to:

- resolve actor handles / DIDs
- hydrate actor-backed collections
- search a specific collection
- synthesize the final answer

### Phase 5: Wire-level OpenAI/OpenRouter tool calling and provider fallback

Only after the runtime semantics are stable should we continue:

- `docs/task/openai-style-tool-calling.md`

And later add a separate provider-fallback task for:

- local `evil_gemma`
- OpenRouter fallback

Why last:

- provider protocol work should not define the runtime architecture
- the runtime/tool/agent semantics need to stabilize first

## Dependency Map

### Must happen first

- first-class root-run state

### Depends on first-class root-run state

- repeated tool-call guard
- incremental transcript
- `/stop`
- cancellation propagation
- per-round `.debug` audit output
- internal tool-call records for subagents

### Should wait until after the above

- internal `collection_search` tool definitions
- real provider-level tool calling
- OpenRouter fallback/provider-switching work

## Practical Recommendation

If implementation starts immediately, the next concrete coding target should be:

1. create a root-run state object
2. move round history, tool-call history, active transcript, and cancel state into it
3. make the UI render from that object
4. add `/stop`
5. add repeated-call guard
6. only then refactor `llm_search` to use internal tool definitions like `collection_search`

That historical order was correct when the runtime substrate did not yet exist.

Given the current codebase, the updated immediate recommendation is:

1. keep the root-level repeated-call guard work on the list
2. separately fix transcript rendering so one child agent update is not displayed as two sibling “agents”
3. begin the internal `llm_search` tool-definition refactor
4. replace Rust-only actor/collection planning heuristics with explicit internal tool calls

## Related Docs

This sequencing task depends on and orders:

- `docs/task/repeated-tool-call-guard-and-confirmation.md`
- `docs/task/incremental-chat-debuggable-tool-loops-and-stop.md`
- `docs/task/high-level-llm-search-and-global-bluesky-lookup.md`
- `docs/task/openai-style-tool-calling.md`

And it takes input from:

- `docs/draft/ui-activity-model.md`
- `docs/draft/event-handlers-and-persistent-agents.md`

## Acceptance Criteria

This aggregate task is satisfied when future implementation work follows this dependency order:

- first-class root-run state first
- repeated-call guard and incremental transcript before deeper internal toolification
- internal `collection_search` tool definitions only after runs are observable and stoppable
- provider/wire-level tool calling after runtime semantics are stable
