# Task: Scope Child-Agent Context Strictly And Fix `/context` Visualization For `llm_search` Subordinates

## Summary

`/context` currently makes deeper `llm_search` subagents, especially `collection search` and `collection review`, look like they receive root-style UI context while not clearly showing task/chat/tool-result segments.

That suggests one of two problems:

- real context leakage: child agents are receiving broader root-derived context than intended
- rendering drift: child context is narrow in reality, but `/context` classifies or displays it misleadingly

This task is to make the runtime boundary explicit and make `/context` show the real scope of each child agent.

## Intended Context Ownership

### Root agent

The root agent is the only place that should own the full ambient app context:

- system prompt
- root tool instructions
- current UI context
- selected actor / selected notification summary
- recent chat
- root-level tool results

### Parent `llm_search` tool agent

The parent internal `llm_search` agent may receive a reduced derived context:

- its own system prompt
- the original search query
- resolved actor hints when relevant
- internal tool results
- reviewed collection outputs for synthesis

It should not receive a broad replay of root UI context unless that is deliberately justified.

### `collection search` and `collection review` children

These should be tightly scoped workers only.

`collection search` should receive:

- its own system prompt
- the specific collection payload
- the specific search prompt

`collection review` should receive:

- its own system prompt
- the specific review prompt
- the specific selected collection evidence
- the produced summary under review

They should not receive:

- root recent chat
- generic app UI context
- unrelated tool transcript history
- broader parent/root context than needed for the local task

## Problems To Solve

### 1. Real context scoping

Audit the actual context-window construction for:

- root agent
- parent `llm_search`
- `collection search`
- `collection review`

Ensure only the intended sections are included at each level.

### 2. Visualization correctness

`/context` should show categories that match the real payload.

For child windows, avoid collapsing unknown sections into generic `UiContext` when they are really:

- collection evidence
- review payload
- parent tool results
- current local task

### 3. Budget sanity

Deeper child agents should never appear to have a wider or more ambient context than the parent/root agent.

If that appears in `/context`, either:

- the runtime is wrong and must be fixed
- or the visualization is wrong and must be fixed

## Proposed Changes

### Runtime scoping

Audit context construction in:

- root query context builder
- parent `llm_search` context builder
- collection-search context builder
- collection-review context builder

Make each builder own only the sections relevant to that agent.

### Visualization categories

Add more precise child-agent categories such as:

- `CollectionEvidence`
- `ReviewEvidence`
- `ParentSearchResults`
- `LocalTask`

Avoid mapping child sections to generic `UiContext` unless they truly came from the app UI.

### Debug output

For each non-root agent, make `.debug` and `/context` clearly show:

- exactly which sections were included
- why each section exists
- that no root chat/UI payload was inherited unless explicitly intended

## Acceptance Criteria

- root agent is the only agent that visibly owns broad UI/chat/tool context
- parent `llm_search` shows only reduced search-specific coordination context
- `collection search` and `collection review` show only narrow local task payloads
- `/context` categories for child agents reflect real content rather than generic UI labels
- no child agent appears to have more ambient context than its parent

## Test Plan

- run one root query that triggers `llm_search` with collection search and review
- inspect `/context`:
  - root shows UI/chat/tool context
  - parent `llm_search` shows search coordination context
  - child `collection search` shows collection plus search prompt only
  - child `collection review` shows review payload only
- inspect `.debug/agents/*` for the same run:
  - rendered context windows match the intended scope for each agent

## Open Questions

- Should the parent `llm_search` agent receive any root UI summary at all once the query has already been normalized into a search request?
- Do we want `/context` to show both semantic categories and raw included section titles for child windows so rendering bugs are easier to spot?
