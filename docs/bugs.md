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
