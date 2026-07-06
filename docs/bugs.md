# Bugs

## `/context` does not repaint live during in-flight LLM work

Current behavior:

- `/context` can remain the visible primary screen while a prompt runs
- child and root context accounting is updated in app state as tool rounds complete
- but the screen does not visibly repaint in real time during the long-running async query

Observed effect:

- the `/context` view appears stale until control returns from the active query path
- this makes it look like context accounting is not changing even when state has been updated

Expected behavior:

- `/context` should visibly refresh while a query is in progress
- root and child context windows should update as messages, tool results, and child `llm_search` contexts are added

Likely cause:

- the current query execution path is awaited inline from the UI event loop, so the app is not redrawing continuously while the query is active

## Root agent can hallucinate after a hard `llm_search` failure

Current behavior:

- a root query can request `llm_search`
- tool prep can fail to refresh the actor or collections
- the tool call can then fail with no cached collections or no grounded evidence
- despite that, the root model may still be prompted to answer directly

Observed effect:

- the final answer can invent list contents, replies, sentiment, or quoted evidence that were never loaded
- this is especially visible when the tool transcript shows `Tool execution failed:` and the final answer still claims to have analyzed the data

Example failure shape:

- prep warning includes `Profile not found`
- tool result includes `Tool execution failed: no cached collections matched the requested search scope`
- final answer still presents a sentiment breakdown as if the lists were inspected

Expected behavior:

- if `llm_search` fails without grounded evidence, the harness should return a deterministic failure answer
- the root model should not be asked to synthesize sentiment from missing tool data

Likely cause:

- the tool loop treated a hard tool failure like an ordinary tool result and still asked the root model to answer

Current mitigation:

- the harness now short-circuits this case with a deterministic failure response
- we should keep this bug documented until the broader end-to-end `llm_search` reliability issues are resolved

## Internal `llm_search` planner keeps generating after emitting a valid tool call

Current behavior:

- the parent internal `llm_search` agent can emit a valid leading `TOOL_CALL`
- but it may continue generating trailing prose, hypothetical results, or self-correction text in the same response
- strict internal protocol handling can reject that message or treat it as malformed
- even when the harness later recovers by parsing the first tool block, the provider/model time has already been spent generating irrelevant trailing text

Observed effect:

- `evil_gemma` stays occupied longer than necessary after the useful tool request has already been produced
- transcript output shows planner failures like `strict internal mode requires exactly one TOOL_CALL block with no surrounding prose`
- raw planner output can include invented post/list summaries after the real tool call
- internal planner retries can consume multiple rounds for a problem that should have stopped at the first valid tool block

Example failure shape:

- planner emits:
  - `TOOL_CALL`
  - `name: hydrate_actor_scope`
  - valid `args: {...}`
- then continues with:
  - `Self-Correction/Refinement`
  - hypothetical hydration results
  - invented positive/negative list examples

Expected behavior:

- once the first valid internal tool call has been emitted, generation should stop immediately
- if wire-level stop is not available, the harness should:
  - accept the first valid tool block
  - ignore or truncate everything after the parsed args object
  - avoid asking the planner to regenerate just because of trailing prose
- the system should free the provider/model as early as possible after the valid tool request is known

Likely cause:

- the current prompt-level internal tool protocol still lets the model continue free-form generation after the tool block
- the harness parser runs only after the full provider response arrives, so it cannot yet interrupt the generation mid-response

Current mitigation:

- the harness now surfaces raw planner output and parsed invalid args in transcript/debug output
- repeated invalid internal planner formatting falls back to harness-side collection resolution after a short cutoff
- this bug should remain open until runtime semantics support early stop or equivalent truncation behavior for internal tool calls

## `llm_search` review-agent ownership is visually and semantically nested too low

Current behavior:

- each `collection review` agent is currently attached as a child of the corresponding `collection search` agent
- this makes the visible tree look like the review step is owned by the collection-search child rather than by the parent `llm_search` coordinator

Observed effect:

- the `llm_search` subtree does not present as one flat set of subordinate work units owned by the parent tool agent
- coordination can look more local and implicit than it really is
- this makes it harder to reason about the parent `llm_search` agent as the single orchestrator deciding:
  - which collection searches to run
  - which reviews to run
  - which reviewed results are admitted into parent synthesis

Why this is a problem:

- the review step is logically part of parent-level orchestration
- the parent `llm_search` agent is the component that should decide whether a collection result is accepted, repaired once, or excluded
- nesting review under collection search can blur that ownership and make future runtime controls harder:
  - flat child scheduling
  - parent-level cancellation
  - parent-level replay/review policies
  - transcript ordering across search and review steps

Expected behavior:

- all immediate subordinate work units for one `llm_search` run should be visible as a flat child list owned by the parent `llm_search` tool agent
- collection searches and collection reviews should both be coordinated explicitly by the parent
- the parent should remain the obvious owner of reviewed/finalized result admission into synthesis

Likely cause:

- review was added as an extension of per-collection search execution state, so it was naturally attached under the corresponding collection-search node
- that attachment is convenient for data flow but may not match the desired orchestration model

Current mitigation:

- the runtime behavior still enforces review before parent synthesis
- this should remain documented until the agent graph and transcript structure better reflect parent-owned coordination

## Child `llm_search` agents appear to receive the wrong context shape in `/context`

Current behavior:

- deeper `llm_search` subordinates such as `collection search` agents appear in `/context`
- those child windows can show more UI-context-like material than the root agent itself
- at the same time, they do not clearly show task/chat/tool-result segments that would explain why that extra context is present

Observed effect:

- `/context` makes it look like collection-level agents are inheriting broad root-style context
- the visible context shape does not match the intended ownership model:
  - root agent should own UI context, recent chat, and root tool transcript
  - parent `llm_search` agent may need a narrower task/tool context derived from the root
  - collection-search and collection-review children should receive only tightly scoped search/review inputs

Why this is a problem:

- deeper child agents should not look like they have more ambient UI context than the root
- if `/context` hides task/chat/tool-result segments for those children, it becomes hard to tell whether the runtime is:
  - actually leaking root context too far downward
  - or merely rendering child context categories inaccurately
- this weakens confidence in context budgeting and makes prompt-boundary bugs harder to reason about

Expected behavior:

- root agent:
  - system prompt
  - tool instructions
  - root instructions
  - UI context
  - recent chat
  - tool results
- parent `llm_search` tool agent:
  - its own system prompt
  - original search query
  - selected internal tool results / reviewed collection summaries
  - no broad inherited UI context dump
- `collection search` and `collection review` children:
  - only the local system prompt plus the narrow collection/review payload they need
  - no root chat history
  - no broad UI context
  - no generic root tool transcript

Likely cause:

- the current `/context` rendering categories for child agents are too coarse
- child context windows may be labeling non-UI sections as generic UI context
- or parent/root-derived context may be flowing further down than intended

Current mitigation:

- none beyond manual inspection of `/context` and `.debug`
- this should remain documented until child agent context windows are explicitly scoped and rendered with categories that reflect their real inputs
