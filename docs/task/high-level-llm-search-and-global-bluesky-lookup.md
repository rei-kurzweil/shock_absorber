# Task: Make `llm_search` A High-Level Bluesky Search Agent

## Goal

Change `llm_search` so the root agent no longer needs to know:

- collection IDs
- actor DIDs
- cache refresh rules
- Bluesky search endpoint details

Instead:

- the root agent gets one high-level tool: `llm_search`
- the root agent passes a natural-language query
- the harness creates the `llm_search` agent
- the `llm_search` agent decides whether the query is about:
  - one or more Bluesky actors/handles
  - a topic that requires Bluesky-wide post search
- the `llm_search` agent chooses which internal loaders and child agents to run
- child agents do narrow execution work only
- the `llm_search` agent synthesizes the final result returned to the root agent

## Current State

Today:

- the root agent can call `llm_search`, but the tool contract is low-level
- `llm_search` expects cache-oriented arguments like:
  - `actor_did`
  - `collection_ids`
  - `prompt`
- the root agent cannot directly express:
  - "look up `2gd4.me`"
  - "who is `rei-cast.xyz`?"
  - "search Bluesky for posts about X"

Important current capability that already exists:

- app code can resolve an arbitrary exact actor reference with `ensure_actor_profile_cached(...)`
- this is used by slash commands like `/bio`, `/replies_from`, and `/pins`
- this capability is not exposed to the root agent as part of tool orchestration

Important current limitation:

- the existing `llm_search` path is collection-search-only
- it can search normalized cached collections, not Bluesky globally
- pre-tool cache refresh only works when the actor DID is already known from:
  - selected UI actor context
  - explicit tool-call arguments

## Desired Root Tool Contract

Keep the root-visible tool name:

- `llm_search`

Replace the root-visible interface with:

- `query` (required string)

The root agent should only need to know:

- `llm_search` can look up Bluesky handles/users
- `llm_search` can search Bluesky posts more broadly
- `llm_search` returns grounded evidence and useful anchors for follow-up reads

The root agent should not need to know:

- actor DIDs
- collection IDs
- which Bluesky endpoint is used
- whether results came from cache or a fresh network-backed load

## Core Behavioral Rules

### If the query is about a handle or username

`llm_search` should:

1. detect one or more actor references in the query
2. resolve each actor reference
3. fetch/cache the actor profile
4. anchor the response on the actor bio
5. if the bio is too weak to answer well:
   - load some posts for grounding
   - search those posts and related normalized collections
6. synthesize the result back to the root agent

Important rule:

- if the user asks "who is this handle?", the answer should not stop at profile resolution unless the bio is already sufficient
- the agent should inspect posts as needed for grounding

### If the query is not about a person

`llm_search` should:

1. treat the query as a topical/global search
2. search Bluesky posts broadly
3. normalize the result set into a persisted synthetic collection
4. run the existing narrow collection-search workflow over that normalized result set
5. synthesize grounded results for the root agent

## Agent Responsibilities

### Root agent

The root agent:

- decides whether search is needed at all
- calls `llm_search(query)`
- does not decide internal scope or search mode

### `llm_search` agent

The `llm_search` agent is the planner and orchestrator.

It is responsible for:

- classifying the query
- deciding actor lookup vs topical/global search
- deciding whether actor bio is sufficient
- deciding whether posts or collections need to be loaded
- choosing which child agents/loaders to run
- synthesizing the final tool result

Important clarification:

- query planning / scope classification is part of the `llm_search` agent itself
- it is not a separate child agent

### Child agents

Child agents do narrow execution work only.

Near-term child roles:

- actor profile loader / resolver
- actor collection hydrator
- per-collection search worker
- global Bluesky post search loader / normalizer

## Tree Diagrams

### 1. Handle lookup: bio is weak, so posts must also be checked

```text
root agent
└── tool call: llm_search(query="who are 2gd4.me and rei-cast.xyz?")
    └── llm_search agent
        ├── actor profile loader: 2gd4.me
        ├── actor collection hydrator: 2gd4.me
        │   ├── load actor profile cache
        │   ├── load recent posts collection
        │   ├── load pinned posts collection
        │   └── load clearsky / related collections when available
        ├── collection search worker: 2gd4.me recent posts
        ├── collection search worker: 2gd4.me pinned posts
        ├── actor profile loader: rei-cast.xyz
        ├── actor collection hydrator: rei-cast.xyz
        │   ├── load actor profile cache
        │   ├── load recent posts collection
        │   ├── load pinned posts collection
        │   └── load clearsky / related collections when available
        ├── collection search worker: rei-cast.xyz recent posts
        ├── collection search worker: rei-cast.xyz pinned posts
        └── final synthesis
            ├── anchor each actor on bio
            ├── supplement with post evidence
            └── return one grounded tool result to root agent
```

### 2. Handle lookup: bio is already sufficient

```text
root agent
└── tool call: llm_search(query="who is 2gd4.me?")
    └── llm_search agent
        ├── actor profile loader: 2gd4.me
        └── final synthesis
            ├── answer from bio / profile facts
            └── skip post-search children
```

### 3. Topic query: search all Bluesky posts

```text
root agent
└── tool call: llm_search(query="what are people on Bluesky saying about X?")
    └── llm_search agent
        ├── global post search loader
        │   ├── call Bluesky-wide search endpoint
        │   ├── normalize returned posts
        │   └── persist synthetic global-search collection
        ├── collection search worker: global-search result collection
        └── final synthesis
            ├── summarize strongest grounded matches
            └── return one grounded tool result to root agent
```

### 4. Dependency model

```text
root agent
└── llm_search tool
    └── llm_search agent
        ├── may depend on actor profile loader
        ├── may depend on actor collection hydrator
        ├── may depend on global post search loader
        ├── may spawn one or more collection search workers
        └── always owns final synthesis
```

## Data And Storage Model

Network-backed results should be normalized into persisted collections rather than kept ephemeral.

This keeps follow-up behavior consistent with existing tooling:

- `read_collection_item`
- collection listing/debugging
- context visualization
- future reuse of fetched results without re-querying the network immediately

Add synthetic collection kinds for at least:

- actor-profile-derived search scopes
- actor recent-post search scopes
- actor pinned-post search scopes
- global Bluesky post-search result sets

Each synthetic collection should carry enough metadata to explain:

- where it came from
- which actor or query produced it
- whether it was network-fetched or cache-derived

## Internal Prompt And Agent Model

All internal LLM-driven agents should be represented through `AgentKind`.

That includes:

- the main `llm_search` agent prompt
- the parent synthesis prompt
- any future specialized loaders/search workers that use LLMs

Non-LLM child operations can remain plain harness loaders, but if they later become prompt-driven they should also be promoted into `AgentKind`.

Important design rule:

- the root-visible tool registry and the internal agent registry are separate layers
- the root agent sees one high-level search tool
- the harness owns the internal agent tree

## Context Window And Visualization Follow-Up

This work should be aligned with the current review in:

- `docs/review/context-window-and-agent-kinds.md`

Specific requirements for this task:

- internal search/load agents should appear in the agent graph with stable `agent_kind`
- context visualization should not infer section semantics purely from titles
- synthetic collection loads and tool outputs should appear as distinct, understandable context segments
- section token estimates should come from the context-window structs at creation time, not from later reconstruction

Important remaining gap to close while implementing this task:

- root prompt visualization still reconstructs some segments outside the canonical context-window model

## Implementation Plan

### 1. Simplify the root-visible `llm_search` tool definition

- change the tool schema to accept only `query`
- remove root-facing notes about:
  - `collection_ids`
  - `actor_did`
  - collection-kind heuristics
- rewrite the tool description around high-level Bluesky search capability

### 2. Split internal `llm_search` orchestration from root tool contract

- keep `llm_search` as the root-visible tool name
- when executed, create the `llm_search` agent
- move query classification into the `llm_search` agent prompt/control flow
- let that agent choose:
  - actor lookup path
  - actor-plus-post-grounding path
  - global-post-search path

### 3. Add internal loaders for non-UI actor lookup

- expose arbitrary exact actor handle/DID resolution to the `llm_search` execution path
- reuse existing profile-fetch code where possible
- make sure a query like `who is 2gd4.me?` can succeed with no prior UI selection

### 4. Add global Bluesky post search support

- add a backend wrapper for Bluesky-wide post search
- normalize results into persisted synthetic collections
- route those collections through the existing per-collection search worker flow

### 5. Extend persisted collection modeling

- add synthetic collection IDs and metadata for network-backed searches
- ensure those collections can be:
  - searched
  - read by item URI
  - surfaced in debugging and visualization

### 6. Keep the `llm_search` agent as top-level synthesizer

- child agents/loaders do not decide overall strategy
- child agents/loaders return narrow results only
- the `llm_search` agent always owns:
  - planning
  - fanout
  - final synthesis

### 7. Align agent graph and visualization

- record each internal agent/load step in the graph where appropriate
- ensure the tree shown in `/context` matches actual orchestration ownership
- use `AgentKind` as the source of truth for internal prompt identity

## Tests

### Tool contract

- `llm_search` prompt rendering describes only the high-level query interface
- root-visible tool inventory no longer instructs the model to provide collection IDs or actor DIDs

### Actor lookup

- exact handle query works without selected UI actor context
- actor bio can produce a valid result by itself
- weak bio causes post-backed grounding to run
- multi-handle query fans out correctly and synthesizes one result

### Global search

- non-actor topical query takes the global-post-search branch
- global search results are normalized into a persisted synthetic collection
- `read_collection_item` can read an item returned from that synthetic collection

### Failure handling

- unknown handle returns a useful grounded failure
- actor profile found but no useful posts still yields a sane answer
- global search with no matches returns a clear no-result outcome
- partial loader failure remains visible in the parent synthesis/debug tree

### Visualization And Debugging

- internal agent tree matches actual execution ownership
- agent nodes expose stable `agent_kind`
- context-window estimates remain consistent for new sections and synthetic collections

## Acceptance Criteria

The task is complete when:

- the root agent can answer `who is 2gd4.me?` with no prior UI actor context
- the root agent can ask about multiple named handles in one `llm_search`
- the root agent can ask about a non-person topic and trigger Bluesky-wide post search
- the root-visible `llm_search` tool contract is high-level and does not leak cache internals
- follow-up reads still work through normalized collection/item IDs
- `/context` and debug output show the internal search tree clearly enough to explain what happened
