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
