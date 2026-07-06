# Task: Replace `llm_search` Heuristics With Internal Tool Definitions

## Goal

Move internal `llm_search` planning away from Rust-only heuristics like:

- `detect_actor_refs(query)`
- `collections_for_actor_refs(...)`
- eager unconditional actor-collection hydration

and into explicit internal tool calls owned by the `llm_search` agent subtree.

The root-visible tool remains:

- `llm_search(query: string)`

But inside that tool run, the `llm_search` agent should decide when to call narrower internal tools.

## Why This Needs To Exist

The current internal flow still decides too much in harness code.

Today, once a query looks actor-centric, the harness immediately:

1. resolves actor refs
2. hydrates several actor-backed collections
3. searches those collections
4. synthesizes the result

This creates three problems:

- planning behavior is brittle and hard to tune
- the model cannot choose narrower or broader search scope intentionally
- transcript output can be confusing because harness-driven fanout looks like hidden planner behavior

We now have enough live runtime visibility that this planning layer should become explicit.

## Important Non-Goal

This task is not the same as root-level duplicate tool-call prevention.

If the root agent runs the same `llm_search(query)` three times, that is still a root wrapper problem.

This task is about the internal behavior of one `llm_search` execution.

## Important Clarification About The Current Transcript

When the chat view shows:

- `collection search: X` with `status: running`
- then `collection search: X` with `status: completed`

that currently reflects two progress updates for one child search label.

It should not be treated as proof that the same child search executed twice.

The internal-tool-definition refactor should still happen, but for the real planning problem, not because of that specific UI artifact.

## Desired Internal Model

The `llm_search` agent should get a private internal tool registry.

Initial internal tools should be:

- `resolve_actor_refs`
- `hydrate_actor_scope`
- `collection_search`
- `finish_search`

`finish_search` may remain an implicit final-response step rather than a literal tool call if that is simpler.

## Recommended First Internal Tool Shapes

### `resolve_actor_refs`

Purpose:

- resolve one or more actor references from the query
- return both handle and DID

Arguments:

- `query`: string

Result should include at least:

- `actor_ref`
- `resolved_handle`
- `resolved_did`
- whether resolution succeeded

This makes the actor identity explicit instead of hiding it in Rust planning code.

### `hydrate_actor_scope`

Purpose:

- ensure the relevant actor-backed collections exist before search

Arguments:

- `actor_did`: string
- `include_profile`: bool
- `include_recent_posts`: bool
- `include_pinned_posts`: bool
- `include_recent_replies_received`: bool
- `include_clearsky_lists`: bool

Result should include:

- collection IDs created or refreshed
- collection labels
- any load failures

This lets the `llm_search` agent choose whether it actually needs all collections.

### `collection_search`

Purpose:

- run the existing narrow collection search worker over one named collection

Arguments:

- `collection_id`: string
- `prompt`: string

Result should include:

- collection ID
- collection label
- grounded summary
- selected result URIs when available

This becomes the main reusable worker tool for the `llm_search` subtree.

## Minimal First Refactor

The first implementation slice does not need a perfect general planner.

It only needs to move the current hard-coded branch:

- actor-centric query -> auto resolve actor -> auto hydrate many collections -> search all of them

into an explicit internal tool loop.

That minimum slice is:

1. create an internal tool registry for the `llm_search` agent
2. add an internal prompt that explains those tools
3. run a local tool-call loop for `llm_search`
4. let `collection_search` remain the existing narrow LLM worker underneath

This is enough to remove the most brittle hidden planning heuristics.

## Execution Model

One `llm_search(query)` should look like:

```text
root agent
└── tool call: llm_search(query="what lists is jcorvinus.bsky.social on? are any negative?")
    └── llm_search agent
        ├── TOOL_CALL resolve_actor_refs {query: "..."}
        ├── TOOL_CALL hydrate_actor_scope {actor_did: "...", include_profile: true, include_clearsky_lists: true}
        ├── TOOL_CALL collection_search {collection_id: "actor_profile:...", prompt: "..."}
        ├── TOOL_CALL collection_search {collection_id: "clearsky_lists:...", prompt: "..."}
        └── final synthesis
```

This should be visible in:

- the live transcript
- the agent graph
- `.debug` agent logs

## Why This Is Better

This gives us:

- explicit planner intent
- better debuggability
- less hidden fanout
- clearer future duplicate protection at the internal-tool level

It also creates the right seam for later improvements:

- more selective actor hydration
- arbitrary post-reply collection reads
- global Bluesky search tools
- internal duplicate-call guards for the `llm_search` subtree

## Implementation Notes

The internal tools do not need provider-native JSON tool calling yet.

They can use the same text tool protocol pattern already used by the root agent:

- emit one internal `TOOL_CALL`
- parse it
- execute it in harness code
- append the tool result back into the `llm_search` agent conversation

That is sufficient for the first implementation.

## Acceptance Criteria

This task is complete when:

- `llm_search` no longer relies on `detect_actor_refs(...)` plus immediate eager collection fanout as its main planning path
- the `llm_search` agent can explicitly resolve actors and choose which collections to hydrate
- `collection_search` is invoked through explicit internal tool calls rather than only from harness heuristics
- both handles and DIDs are visible to the internal planner after actor resolution
- live transcript and `.debug` output clearly show the internal tool-call sequence
