# Draft: Tools, Agents, And Execution Steps

## Why This Draft Exists

The harness currently has multiple overlapping concepts for runtime work:

- prompt-callable tools
- persistent agent nodes
- transient progress events
- one-shot LLM calls with context windows
- harness-only operations that are neither tools nor agents

That was enough to get `llm_search` working, but it now causes confusion in both the runtime model and the UI.

The concrete problem is that some important work is visible in `/context`, some is only visible in the live transcript, and some is not modeled as a first-class execution unit at all.

Examples:

- `llm_search` is a tool
- `llm_search` also creates a tool-agent node
- collection search currently shows up as an agent node
- collection review currently shows up as an agent node
- collection compare, review, and synthesis are all one-shot LLM executions with real context windows
- internal planner / harness operations are partly visible as text events but not as stable runtime nodes

This draft is an attempt to name the problem clearly before changing the code again.

## Current State

Today the harness has three different modeling layers.

### 1. Prompt-Facing Tools

These are the user- or model-callable tools described in the prompt inventory.

Examples:

- `read_selected_post`
- `list_collections`
- `read_collection_item`
- `llm_search`

This layer is currently represented by `ToolSpec` and `ToolRegistry`.

### 2. Persistent Agent Graph

The runtime also builds an `AgentGraph` with persistent nodes such as:

- `RootAgent`
- `ToolAgent`
- `CollectionSearchAgent`
- `CollectionReviewAgent`

This graph is used by `/context` and debug logging.

### 3. Transient Progress Events

Live tool progress is also emitted as text-oriented updates like:

- `llm_search tool agent`
- `collection search: ...`
- `llm_search synthesis`

These are currently rendered as transcript entries rather than as updates to a persistent structured execution tree.

## Main Mismatch

The main mismatch is that "agent" is currently overloaded.

Some things called agents are real multi-step planning units.
Some are one-shot LLM calls.
Some are just persistent execution records.

At the same time, some one-shot LLM steps are important enough to deserve persistent runtime visibility, but they are not modeled as first-class nodes unless they are wrapped in an "agent" label.

That creates several bad effects:

- the runtime tree does not cleanly describe what actually happened
- the UI relies too much on transient text updates
- `/context` shows some execution units but not others
- it is hard to distinguish a tool from a step from a planner from a review pass

## Concrete Example: `llm_search`

`llm_search` is a good example because it crosses all of these boundaries.

Today `llm_search` is:

- a prompt-callable tool
- a tool-level orchestration unit
- an internal planner loop
- a collection search runner
- a collection review runner
- sometimes a synthesis pass

Inside one `llm_search` run, the harness may do all of the following:

1. accept a tool call from the root run
2. create a "tool agent" node
3. run internal planner rounds
4. invoke harness-only internal tools like:
   - resolve actor refs
   - hydrate actor scope
   - collection search
   - global search
5. run one-shot LLM collection comparison on a concrete collection
6. run one-shot LLM review on the resulting summary
7. optionally run one-shot synthesis over multiple per-collection outcomes

That means the current system already has a richer execution model than the type names suggest.

## What We Actually Need

We likely need to separate:

- tool definitions
- prompt templates
- execution nodes
- progress updates

Those are not the same thing.

## Proposed Terminology

Near-term, the system should use more precise language.

### Tool

A tool is something a model can request by name with arguments.

Examples:

- `llm_search`
- `read_collection_item`

### Prompt Template

A prompt template is the system prompt or instruction block used for one kind of LLM step.

Current `AgentKind` is closer to this concept than to a real runtime "agent kind".

Examples:

- collection search prompt
- collection review prompt
- llm_search planner prompt

### Execution Node

An execution node is a persistent runtime record of one unit of work.

It should have:

- kind
- label
- status
- optional context window
- optional result summary
- parent/child relationships

### Progress Update

A progress update is a live mutation to an execution node, not a separate substitute for the node itself.

## Proposed Execution Model

Instead of using only "agent" nodes, we should model a more general execution graph.

Possible node kinds:

- `RootRun`
- `ToolCall`
- `PlannerStep`
- `CollectionStep`
- `ReviewStep`
- `OneShotLLMStep`
- `HarnessStep`

This is only a draft vocabulary, but it describes the current behavior more honestly than `*Agent`.

## Why `ReviewStep` And `CollectionStep` Matter

Collection search and review do not feel like independent agents in the strong sense.

They are more like named execution steps with:

- a specific purpose
- a specific context window
- a status
- a result summary

That is especially true for review.

Review is currently not a tool call in the prompt-facing sense.
It is also not really a planner.
It is a one-shot LLM evaluation step over an existing result.

So `ReviewStep` is probably a better runtime concept than `CollectionReviewAgent`.

Likewise, `CollectionStep` is probably a better runtime concept than `CollectionSearchAgent`.

## One-Shot LLM Steps

One-shot LLM steps should probably become first-class execution nodes.

Why:

- they have real context windows
- they consume meaningful token budget
- they can fail independently
- they are useful to inspect in `/context`
- they are often the most important work in a search pipeline

Examples that fit this concept:

- compare one collection against one search prompt
- review one proposed collection summary
- synthesize multiple collection outcomes into one parent summary

These are not always tools.
They are not always planners.
But they are real execution steps.

## Relationship Between Tools And Steps

A likely long-term structure is:

- tools can own execution nodes
- tools can spawn child steps
- steps can be LLM-backed or harness-backed
- some steps may call additional internal tools

That means a tool is not the smallest runtime unit.

A tool is more like an execution subtree root.

## Sketch: `llm_search` As A Subtree

One possible shape:

- `ToolCall(llm_search)`
  - `PlannerStep`
  - `HarnessStep(resolve_actor_refs)`
  - `HarnessStep(hydrate_actor_scope)`
  - `CollectionStep(clearsky_lists:...)`
    - `OneShotLLMStep(compare collection)`
    - `ReviewStep`
  - `CollectionStep(recent_replies_received:...)`
    - `OneShotLLMStep(compare collection)`
    - `ReviewStep`
  - `OneShotLLMStep(parent synthesis)`

This is much closer to what the harness is already doing.

## UI Consequence

The UI should not depend on transient transcript entries to preserve the execution tree.

Instead:

- the execution tree should be persistent for the lifetime of the run
- progress updates should mutate node status and summaries
- completed and failed nodes should remain visible
- `/context` and chat output should both be renderings of the same execution structure

In other words:

- live events are updates
- execution nodes are the durable source of truth

## Likely Refactor Direction

This draft suggests a future refactor in three moves.

### 1. Rename The Runtime Model

Possible direction:

- `AgentGraph` -> `ExecutionGraph`
- `AgentNode` -> `ExecutionNode`
- `AgentNodeKind` -> `ExecutionNodeKind`

That would reduce the semantic overload immediately.

### 2. Separate Prompt Template Kind From Runtime Node Kind

Current `AgentKind` likely wants to become something like:

- `PromptTemplateKind`

That field would describe which prompt family a step used, not what kind of runtime node it is.

### 3. Make Progress Node-Oriented

Instead of text-only `AgentUpdate { label, depth, content }`, we likely want progress updates that can:

- create a node
- update a node status
- update a node summary
- attach context-window metadata

This does not have to be the final API shape yet, but it should move toward node identity rather than string labels.

## Open Questions

These questions need more thought before implementation.

### Should review itself become a tool?

Possibly.

Arguments in favor:

- clearer consistency
- reusable execution shape
- easier instrumentation

Arguments against:

- review is currently harness-internal and not prompt-facing
- not every execution step needs to be exposed as a tool

My current bias is:

- review should definitely be a first-class execution step
- it does not necessarily need to be a prompt-callable tool

### What is the real difference between planner, tool, and step?

We likely need a crisp answer here:

- tools are callable interfaces
- planners choose what happens next
- steps are the actual runtime work units

### Should all one-shot LLM calls become nodes?

Probably yes when they have:

- a dedicated prompt
- a context window
- meaningful user/debug visibility

Probably no for tiny helper completions that do not matter operationally.

## Non-Goals For This Draft

This draft does not yet define:

- the final Rust types
- the final event protocol
- whether internal llm_search helper operations should all become tools
- how the chat transcript should look after the refactor

It is only trying to establish a cleaner conceptual model.

## Bottom Line

The current harness is already richer than "tools plus agents".

The durable model we probably want is:

- tools are callable interfaces
- prompt templates are instruction families
- execution steps are persistent runtime nodes
- one-shot LLM calls can be first-class execution steps
- the UI and `/context` should render the same persistent execution tree

That appears to be the right direction for making `llm_search`, collection search, review, and future internal workflows easier to understand and extend.
