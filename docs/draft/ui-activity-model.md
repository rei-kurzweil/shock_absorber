# UI Activity Model Draft

Goal: define a single app-state model for what the user is primarily looking at, so commands like `Tab`, `Escape`, and slash views do not keep accreting special cases.

This draft is intentionally pre-implementation.
It describes the target state model before the next round of UI refactors.

## Why This Draft Exists

The current UI state is spread across concepts like:

- notification pane selection
- detail view mode
- fullscreen overlays
- deferred command output

That was enough to get `/context` and command output working, but it is now making navigation semantics unclear.

The concrete pressure is:

- pressing `Tab` should toggle between AI chat output and whatever primary UI activity the app was previously showing
- `/context` and `/collections` should be modeled as real app states, not just special render branches
- notifications plus its detail pane should remain coherent as one thing from the user’s perspective

We need one state model that answers:

- what is the current primary activity
- is the app in split-screen or fullscreen presentation
- if the current primary activity has secondary detail, where is it stored
- what view should `Tab` return to

## Core Idea

The app should model:

- `layout_mode`
- `primary_activity`

Near-term, `primary_activity` is the authoritative answer to “what screen am I on?”

The current `DetailView` approach should eventually become an implementation detail or be replaced by activity-specific state.

## Proposed Top-Level State

We should describe the UI with two top-level concepts:

- `layout_mode`
  - `split_screen`
  - `full_screen`

- `primary_activity`
  - `notifications`
  - `ai_chat_output`
  - `context_view`
  - `collections_view`

This means the app can say things like:

- split-screen + notifications
- full-screen + context_view
- full-screen + ai_chat_output

This is a cleaner model than “normal view plus some overlay enum plus deferred detail buffer.”

## Primary Activity

`primary_activity` should mean:

- the main thing the user is intentionally looking at
- the thing `Tab` toggles away from or back to
- the thing keyboard behavior should primarily target

Candidate activities for the near term:

- `notifications`
  - the standard notification browsing experience

- `ai_chat_output`
  - the command/chat transcript and final answer view

- `context_view`
  - the `/context` visualization

- `collections_view`
  - the `/collections` visualization

Future activities could include:

- collection drilldown
- agent/thread inspection
- dedicated actor profile view

## Notifications As One Primary Activity

Near-term simplification:

- treat the notifications list and its right-hand detail pane as one primary activity

This matters because otherwise we would need a more general split-screen state model immediately:

- left activity
- right activity
- focus owner
- per-pane navigation state

That is more machinery than we need right now.

Instead, the notifications primary activity should own:

- list selection
- opened notification
- detail scroll
- selected actor summary derived from that notification when relevant

In other words:

- the right-side detail pane is not a separate primary activity yet
- it is part of the `notifications` activity state

This lets us keep “what mode am I in?” in one place without implementing generic pane composition first.

## Secondary Activity

If we need the concept at all in the near term, it should be narrow.

For now:

- `notifications` may have an internal secondary detail surface
- other activities do not need a separate secondary activity yet

That means we should avoid prematurely modeling a general-purpose `secondary_activity` field for every screen.

Better near-term rule:

- only the `notifications` primary activity renders split-screen
- every other primary activity is fullscreen

This gives us the effect we want without inventing an overly abstract pane system.

## Layout Rules

Near-term layout rules should be simple:

- `notifications` uses `split_screen`
- `ai_chat_output` uses `full_screen`
- `context_view` uses `full_screen`
- `collections_view` uses `full_screen`

Later we may allow:

- `context_view` in split-screen
- `collections_view` in split-screen
- a dedicated detail pane for chat output

But that should be a later step after the activity model is stable.

## `Tab` Behavior

`Tab` should toggle between:

- `ai_chat_output`
- the previous non-chat `primary_activity`

This implies the app needs to remember:

- `last_non_chat_primary_activity`

Examples:

1. user is on `notifications`
2. user runs a prompt
3. app switches to `ai_chat_output`
4. user presses `Tab`
5. app returns to `notifications`
6. user presses `Tab` again
7. app returns to `ai_chat_output`

Same expectation for:

- `context_view`
- `collections_view`

If the user runs a prompt while viewing `/context`, `Tab` should still conceptually switch between:

- the pending or completed `ai_chat_output`
- `context_view`

This is cleaner than treating `/context` as a fragile overlay over some hidden detail buffer.

## `Escape` Behavior

`Escape` semantics should become activity-based rather than overlay-based.

Near-term:

- if input is non-empty, `Escape` clears input
- otherwise, if the current primary activity is a transient fullscreen activity opened from somewhere else, `Escape` returns to the previous primary activity
- if the current primary activity is `ai_chat_output`, `Escape` can return to the previous non-chat activity

Exact policy can be tuned later, but it should route through primary-activity transitions rather than ad hoc `DetailView` dismissal.

## Slash Command Semantics

Slash commands that open UI surfaces should set `primary_activity` directly.

Examples:

- `/context` -> `context_view`
- `/collections` -> `collections_view`

Regular prompts should generally target:

- `ai_chat_output`

But if we intentionally support “run prompt while staying on `/context`”, that should be represented as:

- current visible `primary_activity` remains `context_view`
- `ai_chat_output` becomes the alternate tab target

That means chat output should be a tracked activity even when hidden.

## Relationship To Existing Code

Today the main code roughly mixes together:

- `DetailView`
- fullscreen-or-not checks
- deferred output behavior

This draft suggests replacing that mental model with:

- stable top-level activity state
- per-activity state payloads
- layout derived from activity kind

In practice this probably means:

- `DetailView::Notification` becomes part of `notifications` activity state
- `DetailView::Command` maps more closely to `ai_chat_output`
- `DetailView::ContextVisualization` maps to `context_view`
- deferred command output becomes “chat activity exists but is not currently primary”

## Recommended Near-Term Refactor Shape

We should probably move toward something like:

- `AppViewState`
  - `primary_activity`
  - `last_non_chat_primary_activity`

- `PrimaryActivity`
  - `Notifications(NotificationsActivityState)`
  - `AiChatOutput(ChatActivityState)`
  - `ContextView(ContextActivityState)`
  - `CollectionsView(CollectionsActivityState)`

This keeps all mode-specific state attached to the activity that owns it.

## Non-Goals For This Draft

Not part of this immediate change:

- implementing a fully generic multi-pane layout engine
- introducing arbitrary nested activities
- redefining every keybinding now
- solving collection drilldown
- solving multiple simultaneous visible agents

The immediate goal is smaller:

- define a clean activity model
- support `Tab` toggling cleanly
- stop encoding “current screen” implicitly across multiple unrelated fields
