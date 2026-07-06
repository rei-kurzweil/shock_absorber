# Task: Reframe Review Steps Vs Agent Loops

## Summary

We currently call `collection review` an agent, but semantically it is not the same kind of thing as:

- the root agent
- the internal `llm_search` planner

Those components own an iterative decision loop around user input, tool calls, or future verification calls.

`collection review` does not.

Today it is a harness-run post-processing step that executes after a `collection_search` result exists and before parent `llm_search` synthesis uses that result.

This task is to make that distinction explicit in runtime structure, naming, and `/context` visualization.

## Current State

### Real loop owners

The following components behave like real agents:

- root agent
  - owns the main user/tool loop
- internal `llm_search` planner
  - owns an internal tool loop across:
    - `resolve_actor_refs`
    - `hydrate_actor_scope`
    - `collection_search`
    - `search_global_posts`

### Review step

`collection review` currently:

- runs after `collection_search` completes
- consumes:
  - the local search prompt
  - selected collection evidence
  - the proposed summary
- emits:
  - pass/fail verdict
  - optional repair request
- does not choose tools
- does not own an iterative loop
- is invoked by harness logic rather than by an autonomous sub-loop

This means it is closer to:

- a review tool
- a verification step
- a harness-owned post-tool check

than to a first-class agent loop.

## Why This Matters

Treating every LLM call as an agent blurs important runtime boundaries.

It makes it harder to reason about:

- which components truly decide what to do next
- which components merely transform or verify existing outputs
- where future generic verification hooks should live
- how `/context` and `.debug` should present ownership

It also makes the current `collection review` subtree look more autonomous than it really is.

## Desired Model

### Agent

A thing should be called an agent when it owns an iterative decision loop such as:

- user/tool loop
- tool/verify loop
- tool-only planning loop

Examples:

- root agent
- internal `llm_search` planner
- future agents with explicit verification-tool loops

### Review step

A review step should be modeled as something that:

- runs after a tool result or proposed summary exists
- evaluates or repairs that result
- may be called automatically by the harness
- may later become callable by an agent loop
- does not itself imply independent loop ownership

Examples:

- current `collection review`
- future generic verification/review primitives

## Short-Term Goal

Keep existing behavior, but reframe runtime structure and UI around the real ownership model:

- `llm_search` planner remains the internal loop owner
- `collection_search` remains planner-requested work
- `collection review` is represented as a harness review step over collection-search output

This can be mostly a modeling and visualization cleanup first.

## Medium-Term Goal

Introduce a reusable review primitive that can be invoked in either of two ways:

1. automatically by harness policy after a tool step completes
2. explicitly by a future agent that owns a verification loop

That would let review logic be reused without pretending every review invocation is a peer agent.

## Concrete Follow-Up Work

### Runtime naming

Audit names such as:

- `CollectionReviewAgent`
- `collection review`
- any debug labels implying full agent autonomy

Prefer names that distinguish:

- loop-owning agent
- harness review step

### Agent graph / runtime graph

Decide whether the runtime graph should:

- keep one unified node type but include a node role like:
  - `AgentLoop`
  - `ReviewStep`
- or split the graph model more explicitly between:
  - loop owners
  - harness-managed post-tool steps

### `/context` and `.debug`

Update presentation so it is clear:

- the root agent and `llm_search` planner are loop owners
- `collection review` is a review/verification step
- indentation and labels do not imply false autonomy

### Review invocation contract

Make the harness boundary explicit:

- planner emits tool request
- harness executes tool
- harness may run review step on the tool result
- planner resumes with reviewed output

That contract should be visible in code and docs.

## Relationship To Existing Behavior

This task does not require changing the current review logic itself.

The important behavior that already exists is still correct:

- the planner pauses between tool calls
- `collection_search` runs
- review runs on the produced summary
- parent `llm_search` synthesizes only after that step

The cleanup is about classification and ownership, not about removing review.

## Acceptance Criteria

- docs clearly distinguish loop-owning agents from harness review steps
- runtime naming no longer implies that every LLM call is a peer agent
- `/context` and `.debug` make review-step ownership explicit
- current `llm_search` review behavior still works
- future verification-tool work has a clean conceptual home

## Open Questions

- Should `collection_search` itself eventually stop being called an agent and become a planner-dispatched tool step?
- Do we want one generic review-step abstraction, or several narrow review primitives?
- Should review-step nodes appear in the same visible tree as agents, or in a separate post-tool verification lane?
- When the root agent gains its own verification tool, should that verification step share the same primitive as `collection review`?
