# Task: Split `llm_search` Into Per-Collection Grandchild Agents

## Goal

Change `llm_search` so that searching multiple collections no longer creates one merged search scope.

Instead:

- the main agent calls `llm_search`
- the `llm_search` tool agent spawns one narrower search agent per collection
- each per-collection search agent gets its own `LLMContext` and provider budget
- each per-collection search agent returns grounded evidence for that one collection only
- the parent `llm_search` tool agent synthesizes across those per-collection results
- the parent `llm_search` tool agent returns one combined result to the main agent

This is effectively:

- root agent
- child tool agent: `llm_search`
- grandchild search agents: one per searched collection

## Why Change This

Current behavior merges many collections into one synthetic collection and runs one child search over the combined text.

That causes several problems:

- one collection can crowd out another in the shared child context budget
- the prompt can mix unrelated collection kinds into one ranking pass
- `/context` only shows one child search window for a multi-collection `llm_search`
- the system loses the distinction between:
  - “search this one collection narrowly”
  - “combine evidence across many collections”

We want multi-collection search to behave more like a real hierarchy:

- search each collection independently
- preserve separate evidence and prompt budgets
- synthesize only after the separate searches complete

## Current State

Today `llm_search` works like this:

1. resolve `collection_ids` or `actor_did`
2. gather matching `LabeledPostCollection`s
3. merge them into one synthetic collection
4. build one child `LLMContext`
5. run one search call
6. return one grounded result

Important current implementation detail:

- `resolve_search_collection(...)` in `src/harness/tools.rs` returns one merged `LabeledPostCollection`
- `merged_collection_from_refs(...)` injects `source_collection_id` and `source_collection_label` into each post body

That merged path is what this task should replace for multi-collection search.

Important current limitation:

- subagents are not currently managed as first-class runtime objects
- there is no durable agent tree or agent graph in app state
- there is no typed node model for:
  - root agents
  - tool agents
  - per-collection search agents
- `/context` and tool transcript behavior are derived from prompt-building and tool-call flow, not from a true source-of-truth agent graph

So today:

- “subagent” mostly means “one narrower LLM call happened here”
- ownership and hierarchy are implicit in control flow
- introspection is partial and reconstructive rather than authoritative

## Desired Runtime Model

This task should move the system toward a real agent graph, not just more nested helper calls.

### Single-Collection Search

If `llm_search` is given exactly one collection:

- behavior may remain effectively the same
- one child search window is built
- one grounded result is returned

This is still useful and should not become more complex than necessary.

### Multi-Collection Search

If `llm_search` is given multiple collections:

1. the parent `llm_search` tool call resolves the collection list
2. for each collection:
   - spawn one grandchild search agent
   - build a dedicated context window containing only that collection plus the search prompt
   - run the search
   - capture grounded output and context-window telemetry
3. aggregate all per-collection outputs
4. ask the parent `llm_search` tool agent to combine the results
5. return one combined result to the root agent

Important constraint:

- grandchild agents should not share one merged search-space prompt

## Agent Graph Source Of Truth

The implementation should introduce a real runtime source of truth for agent ownership.

Recommended model:

- top level: a list or slotmap of root agents
- each root agent owns a typed tree of agent nodes
- every agent node has:
  - stable runtime ID
  - parent ID except for root nodes
  - agent type
  - child IDs
  - current state metadata
  - most recent context-window telemetry

Near-term important agent types:

- `root_agent`
- `tool_agent_llm_search`
- `collection_search_agent`

This means the hierarchy for one multi-collection search should be representable as:

- root agent
  - `llm_search` tool agent
    - collection search agent for recent posts
    - collection search agent for recent replies
    - collection search agent for moderation lists

## Why The Agent Graph Matters

We want introspection and tooling to be built on one authoritative model.

Without that graph:

- `/context` has to reconstruct hierarchy from incidental telemetry
- transcripts cannot reliably explain ownership
- debugging nested agent behavior becomes ad hoc
- future tooling has no stable place to attach:
  - agent inspection
  - tree visualization
  - per-agent status
  - per-agent transcripts
  - cancellation or replay later

With that graph:

- `/context` can render from real nodes
- transcripts can reference stable agent identities
- future slash commands can inspect agent trees directly
- parent/child relationships stop being implicit in stack flow

## Root-Agent Registry

Near-term, the system should conceptually maintain:

- a registry of promptable root agents

These are distinct from subagents.

Root agents are:

- directly promptable by the user
- durable enough to own conversations, context, and descendants

Subagents are:

- owned by another agent
- spawned for narrower work
- not directly promptable in the normal UI path

This distinction matters because the top-level container should not just be “all agents.”
It should be:

- promptable root agents at the top
- each root agent owns its own descendant tree

## Minimum Node Data

Each node in the agent graph should carry enough information to answer:

- what kind of agent is this
- who owns it
- what work is it doing
- what context window did it build
- what result did it produce

Recommended minimum fields:

- `agent_id`
- `agent_type`
- `parent_agent_id`
- `child_agent_ids`
- `label`
- `status`
- `tool_name` when relevant
- `collection_id` when relevant
- `context_window_report`
- `result_summary`

This does not need to be the final schema, but the system should stop relying on anonymous nested calls.

## Expected Search Semantics

Per-collection search agents should answer only:

- what evidence in this collection matches the prompt
- what one anchor item best represents that collection hit
- what repeated themes appear inside that collection

They should not be responsible for the final cross-collection conclusion.

That final combination step belongs to the parent `llm_search` tool agent.

## Output Shape

The final outward-facing `llm_search` tool result can still remain a single tool result block for the main agent.

But internally it should preserve:

- one result per collection searched
- one anchor item per collection when available
- one context-window report per collection search agent
- one synthesis step across those collection-local results

Near-term acceptable final shape:

- a combined result block that includes compact per-collection evidence sections
- plus one selected anchor URI that the main agent can inspect further

Longer-term better shape:

- explicit structured sub-results
- explicit collection-local anchor items
- explicit final synthesized summary

## `/context` Expectations

After this change, `/context` should show:

- one root-agent window
- one `llm_search` tool-agent window when relevant
- one grandchild search window per searched collection

Example:

If the main agent asks to search:

- `recent_posts_unaddressed`
- `recent_replies_sent`
- `clearsky_lists`

Then `/context` should show separate child/grandchild search windows for:

- recent posts search
- recent replies search
- moderation lists search

This is one of the main reasons to do the refactor.

Important requirement:

- `/context` should render from the agent graph, or from telemetry attached to graph nodes
- it should not need to infer the existence of subagents only from a flat transcript

## Synthesis Step

We need to be explicit about where synthesis happens.

Recommended model:

- each per-collection grandchild search agent produces a grounded evidence block
- the parent `llm_search` tool agent receives those blocks
- the parent `llm_search` tool agent synthesizes across them

This parent synthesis step should be able to:

- compare evidence across collection kinds
- note when one collection has no meaningful matches
- keep moderation-list evidence distinct from authored-post evidence
- prefer a more representative anchor item if one collection result is clearly stronger

## Prompt Construction Requirements

Per-collection grandchild search prompts should include:

- fixed search-agent instructions
- exactly one collection serialization
- the search prompt passed in by the caller

They should not include:

- other collection payloads
- unrelated source collections
- aggregated merged post bodies from other collection kinds

The parent synthesis prompt should include:

- the original search prompt
- compact outputs from each per-collection search agent
- enough metadata to identify which collection each sub-result came from

## Error Handling

If one per-collection search agent fails:

- the overall `llm_search` tool call should not necessarily fail immediately
- the parent tool agent should be allowed to synthesize from successful collection searches
- the failed collection should be reported explicitly in the tool result

Near-term acceptable behavior:

- partial results are allowed
- failure in one collection is surfaced in the combined output

## Tool Contract

Near-term, the external tool contract can remain:

- `llm_search`
  - optional `actor_did`
  - optional `collection_ids`
  - optional `label`
  - required `prompt`

The change is internal orchestration, not necessarily public tool shape.

That means this task does not require changing the main-agent prompt protocol first.

## Recommended Implementation Direction

### 1. Stop Merging Collections For Search

Refactor `resolve_search_collection(...)` so that multi-collection search returns a list of collections rather than one merged collection.

Single-collection search may still reuse the current narrow path.

### 2. Add Per-Collection Search Execution

Add a helper that:

- accepts one `LabeledPostCollection`
- builds one child `LLMContext`
- executes one search pass
- returns grounded evidence plus telemetry

### 3. Add Parent `llm_search` Aggregation

Add a parent aggregation step that:

- receives the per-collection results
- synthesizes them into one final `llm_search` result

This may be:

- a second LLM call inside `llm_search`
- or a deterministic formatter first, then optional LLM synthesis

Near-term preference:

- use a narrow LLM synthesis call only after grounded per-collection results are collected

### 4. Extend Context Telemetry

Update `/context` plumbing so that it can represent:

- root agent
- `llm_search` tool agent
- one grandchild search agent per searched collection

This should be attached to real agent nodes rather than only transient local variables.

### 5. Add Agent Registry / Graph State

Add a runtime structure that:

- stores promptable root agents in a top-level registry
- stores each root agent’s descendant tree
- records typed parent/child relationships
- exposes enough information for `/context` and later tooling

### 6. Update Tool Transcript

The visible tool transcript should ideally make clear:

- which collections were searched
- that they were searched independently
- which results came back from each search agent

## Open Questions

These should be resolved during implementation:

- Should the parent `llm_search` tool agent itself get a separately visualized context window, or is the current root-plus-grandchildren view sufficient?
- Should the parent synthesis step always use an LLM, or only when more than one collection has meaningful results?
- How should the final anchor URI be chosen when different collections each have strong candidates?
- Should the system cap the number of per-collection grandchild searches per `llm_search` call?

## Acceptance Criteria

This task is complete when:

- multi-collection `llm_search` no longer relies on one merged collection search
- subagents are represented as typed nodes in a real agent graph
- promptable root agents exist as top-level graph owners
- each searched collection gets its own narrower search agent/context window
- per-collection grounded outputs are collected independently
- the parent `llm_search` layer synthesizes across those outputs
- `/context` shows separate search-agent windows for each searched collection
- `/context` can identify which agent node produced each shown context window
- tool transcript output makes the independent collection searches visible

## Non-Goals

Not part of this immediate change:

- changing the public `llm_search` tool signature
- implementing a generic multi-agent framework for all tools
- parallel execution across threads or tasks unless it falls out naturally
- redesigning collection serialization for unrelated features

The narrow goal is:

- make multi-collection `llm_search` hierarchical instead of merged
