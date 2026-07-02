# Fullscreen Collections And Context Visualizations Draft

Goal: add fullscreen read-only visualizations for cached collections and active LLM context usage, with simple slash commands that temporarily replace the normal split-pane view until `Escape` is pressed.

This draft is intentionally pre-implementation.
It describes the target behavior before the next round of code changes.

## Why Add Fullscreen Visualizations

The current UI can show text output in the detail pane, but that is not enough for two things we now care about:

- inspecting the collection cache as a first-class system object
- inspecting how prompt budget is being consumed inside the LLM harness

We want these views to be visible as dedicated fullscreen modes rather than squeezed into the existing detail panel.

## New User Commands

We should add or change these commands:

- `/collections`
  - opens a fullscreen collection inventory view

- `/context`
  - opens a fullscreen context-usage visualization view

- `/pins <actor>`
  - same behavior as the existing pins command, but slash-prefixed

Important change:

- the existing `pins` command should be renamed to `/pins`
- command help and parser behavior should move toward slash-prefixed UI commands consistently

Near-term scope does not require renaming every existing command yet, but `/pins`, `/collections`, and `/context` should use the slash form.

## Fullscreen Mode Behavior

Both new views should behave as modal fullscreen screens:

- they replace the standard notification/detail layout while active
- they remain visible until the user presses `Escape`
- `Escape` returns to the normal main UI
- they are read-only views
- they do not invoke the LLM

This should be implemented as explicit app modes, not as a special detail-pane rendering trick.

## Proposed Module Layout

We should introduce:

- `src/visualization/collections.rs`
- `src/visualization/context.rs`
- `src/visualization/mod.rs`

The intent is to keep these renderers separate from:

- command parsing
- app state transitions
- harness logic

The visualization modules should focus on:

- shaping data for display
- rendering fullscreen widgets
- scroll and selection behavior specific to those views

## `/collections` View

The `/collections` screen should show a scrolling inventory of all cached collections.

Each visible row or card should include at least:

- collection label
- collection ID
- collection kind
- actor handle when known
- actor DID when handle is not known or when extra precision is useful
- item count
- last refreshed time
- refresh TTL when available

The main use case is to quickly answer:

- what collections exist right now
- which actor they belong to
- how large they are
- how stale or fresh they are

## Collection View Data Requirements

The collection visualization should not only show raw collection IDs.
It should also show actor handles when possible.

That implies the renderer or its data-preparation path should be able to resolve:

- collection `actor_did`
- cached handle for that DID from `NotificationStore`

If no handle is known:

- show the DID directly

If a collection is not actor-scoped:

- show no actor handle field or a compact placeholder such as `global`

## Collection View Interaction

The `/collections` view should support vertical scrolling.

Near-term behavior:

- Up and Down move through the list
- PageUp and PageDown can jump faster
- Home and End are optional later
- `Escape` exits the mode

This first version does not need:

- sorting controls
- filtering
- refresh actions
- drill-down into collection items

Those can be added later if the fullscreen mode proves useful.

## `/context` View

The `/context` screen should visualize how the current effective prompt context is divided across major section categories.

This should be a fullscreen horizontal bar visualization using full-width colored blocks.

The display should include:

- one main horizontal band made of colored segments
- segment width proportional to how much of the total context window each category consumes
- a legend below explaining the colors
- numerical labels where practical

## Context Color Mapping

The target color mapping should be:

- violet = system prompt
- blue = tool definitions
- green = UI context
- yellow = user and AI chat
- orange = tool results

Additional rule:

- the last 25% of the total context window should be rendered as red regardless of category

Intent:

- the bar should make “danger zone” prompt usage obvious at a glance
- red is not a separate semantic category
- red is an overlay meaning “this portion is in the final quarter of available context”

## Context Visualization Semantics

The context view should represent proportion of the total available context window, not just proportion of currently used text.

That means:

- if the model has an 8k context window and current rendered prompt uses 4k, only half the width should be category-colored
- the remaining unused portion should still be visible distinctly

Open display choice:

- unused context can be black, dark gray, or another neutral background

The important behavior is:

- used vs unused capacity must be visually distinguishable
- the final 25% capacity zone must be obvious

## What Counts As Each Context Segment

For the first version, the view should categorize rendered prompt content approximately like this:

- system prompt
  - the instruction header or system message sent to the LLM

- tool definitions
  - rendered tool inventory and protocol instructions

- UI context
  - selected notification summary, search hints, and similar app-provided state

- user and AI chat
  - recent conversational turns included in the request

- tool results
  - any rendered tool outputs loaded back into the active context

The exact internal accounting can evolve later, but the visualization should be tied to the real rendered context path rather than estimated from unrelated strings.

## Need For Provider-Reported Context Window Size

The context visualization is only useful if `shock_absorber` knows the actual context window size of the active model.

Today there is a hardcoded or local provider limit assumption.
That is not good enough for this feature.

We want:

- the `evil_gemma` server to report its context window size
- `shock_absorber` to ingest that value
- the visualization and trimming logic to use the reported value

This should be treated as a real integration requirement, not a UI-only guess.

## Suggested Backend Shape For Context Limits

We likely need a provider metadata path that can expose at least:

- model name
- max context tokens
- maybe max output tokens later

Possible shapes:

- an OpenAI-style model metadata endpoint
- a lightweight custom metadata endpoint on `evil_gemma`
- a startup handshake when `shock_absorber` initializes the LLM client

The important point is:

- `shock_absorber` should stop pretending the context limit is fixed when the server can tell us the true limit

## Rendering Strategy

For the first version, the context visualization does not need perfect tokenization.

A practical first pass is acceptable if it:

- uses the same rendered prompt content that is actually sent to the model
- measures section size consistently
- uses one approximation method everywhere

Possible approximations:

- byte count
- char count
- rough token estimate already used by the harness

Preferred direction:

- use the same budgeting metric already used by `context_window.rs`, or move both features onto one shared estimate

The key requirement is internal consistency between:

- trimming behavior
- displayed proportions

## Relationship To Existing Context Harness

The new context visualization should be built on top of the existing harness flow, not parallel to it.

Relevant existing pieces already include:

- `src/harness/context_window.rs`
- tool inventory rendering
- UI context serialization
- provider context limits

The visualization should expose that machinery rather than inventing a second unrelated prompt-building path.

## App State Expectations

The app likely needs explicit UI state for:

- normal notification view
- fullscreen collections view
- fullscreen context view

Those modes should control:

- draw behavior
- keyboard handling
- status line text

This is cleaner than encoding fullscreen screens as special command output.

## Recommended Near-Term Sequence

1. add this draft
2. promote it to a task doc after naming and scope are stable
3. add explicit fullscreen app modes
4. add `src/visualization/collections.rs`
5. add `src/visualization/context.rs`
6. rename `pins` to `/pins`
7. implement `/collections`
8. implement `/context`
9. add provider-reported context window support from `evil_gemma`
10. connect the context visualization to the real prompt-building path

## Non-Goals For This Draft

Not part of this immediate change:

- interactive refresh or mutation of collections from the visualization
- per-item collection drilldown
- mouse support
- arbitrary chart theming
- exact final keyboard map beyond `Escape` and basic scrolling
- migrating every existing local command to slash-prefixed form in one pass

The goal here is to settle the fullscreen visualization model and required data plumbing before code changes begin.
