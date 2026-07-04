# Event Handlers And Persistent Agents Draft

Goal: add a first-class event system plus long-lived named agent instances that can react to notifications, coordinate with each other, and be inspected or chatted with from the main UI.

This draft is intentionally pre-implementation.
It describes the target behavior before the next round of code changes.

## Why This Needs To Exist

The current harness has only one durable interactive agent:

- the root `evil_gemma` conversation in the main UI

Everything else is transient:

- `llm_search` is effectively a narrow one-shot child agent
- there is no concept of a persistent worker agent with a named role
- there is no event bus
- there is no queue of work items for background or semi-background agents
- there is no way to register notification-triggered behavior

That means we cannot express flows like:

- when a reply arrives, ask one agent whether it matters
- if it matters, let another agent investigate and prepare a response
- keep those agents alive with explicit goals, memory, and state
- inspect or chat with those agents directly from the UI

We want a model where:

- notifications become events
- handlers can subscribe to subsets of events
- handlers are named runtime objects
- agents can be created from predefined templates
- agents can stay active across multiple events
- the root UI can talk to a running agent directly

## Terminology

We need to distinguish four things clearly.

### 1. Agent Template

A predefined agent type that can be created on demand.

Examples:

- `triage_reply`
- `investigate_actor`
- `draft_response`
- `moderation_watchdog`

A template defines at least:

- template name
- system prompt
- tool access policy
- optional default goal
- optional model/provider config
- optional queue behavior

### 2. Running Agent

A live instance of an agent template.

A running agent has at least:

- instance name
- template name
- status
- queue of pending messages or work items
- recent transcript
- current goal
- optional bound actor or conversation context

Examples:

- `reply_triage_1`
- `watchdog_alice`
- `investigate_current_thread`

### 3. Event

A normalized internal occurrence that agents may react to.

Near-term events should include:

- `reply`
- `follow`
- `like`

Later events may include:

- `mention`
- `quote`
- `repost`
- `collection_refreshed`
- `agent_message`

### 4. Handler

A runtime subscription that listens for one or more event kinds and coordinates two agents:

- a filter agent
- a response agent

The filter agent decides whether the response agent should run.

## Proposed Module Shape

We likely need at least:

- `src/events/mod.rs`
- `src/events/types.rs`
- `src/events/bus.rs`
- `src/events/handlers.rs`
- `src/harness/agents.rs`

The intent is to separate:

- event normalization and delivery
- persistent agent runtime state
- slash-command management
- root UI rendering and interaction

## Event Model

Each event should have a normalized shape.

Required fields should include:

- `event_id`
- `event_kind`
- `created_at`
- `source`
- optional `actor_did`
- optional `notification_uri`
- optional `post_uri`
- compact event payload

Examples of event sources:

- `notification_poll`
- `manual_ui_action`
- `agent_delegate`

Important constraint:

- handlers should not consume raw Bluesky notification records directly
- they should consume normalized event objects

That keeps event routing stable even if notification payload details change.

## Notification Subsets

The user request specifically wants selective subscriptions.

That means handlers should be able to subscribe to:

- one event kind
- many event kinds
- later: filtered subsets of one event kind

Near-term subscription syntax should support at least:

- `reply`
- `follow`
- `like`
- comma-separated combinations such as `reply,follow`

Later we may add richer filters like:

- only replies from unknown actors
- only follows from accounts on certain lists
- only likes on selected posts

But the first version should keep subscription matching simple.

## Persistent Agent Model

Running agents should be first-class runtime objects rather than ad hoc LLM calls.

Each running agent should track at least:

- instance name
- template name
- human-readable goal
- current status
- active or paused flag
- pending queue length
- recent input messages
- recent output messages
- tool transcript history
- creation time
- last activity time

Status values should likely include:

- `idle`
- `queued`
- `running`
- `waiting`
- `errored`
- `stopped`

Important design point:

- these agents should not be tied to a visible terminal pane
- the UI should be able to inspect them, but they must exist independently of the current screen

## Agent Goals

The current root agent has conversational context but not an explicit durable goal object.

For persistent agents, we should make the goal explicit.

Each agent instance should have:

- a goal string
- optional fixed role text
- optional bounded scope

Examples:

- “triage incoming replies for harassment or urgent follow-up”
- “decide whether this notification deserves investigation”
- “draft a response recommendation for the root agent”

This is different from:

- the transient search prompt used by `llm_search`

We want a persistent agent to keep its role across many events.

## Handler Model

A handler should be a named runtime registration.

Each handler should contain:

- handler name
- subscribed event kinds
- filter agent template or instance reference
- response agent template or instance reference
- enabled flag
- optional concurrency limit
- optional dedup policy

The filter agent and response agent may be:

- newly spawned per event
- pooled long-lived instances

Near-term default:

- allow handlers to spawn fresh instances for each event
- optionally reuse a named long-lived agent later

## Filter Agent And Response Agent Split

The filter agent exists to decide:

- is this event interesting enough to escalate
- what narrow question should the response agent answer
- what initial context should be passed along

The response agent exists to:

- investigate with tools
- inspect collections
- inspect recent posts or replies
- inspect list membership or moderation context
- prepare a recommendation or response block for the root agent

This split is important because it keeps expensive investigation from running on every event automatically.

## Event Flow

Near-term target flow:

1. notifications are polled
2. new notifications are normalized into events
3. the event bus finds matching handlers
4. each matching handler enqueues a filter-agent work item
5. the filter agent decides whether to continue
6. if accepted, the response agent receives a derived work item
7. the response agent can use tools and produce a result
8. the result is delivered to the root agent or stored in an agent inbox

Important behavior:

- many handlers may listen to the same event
- one handler declining an event should not block another handler

## Root Agent Integration

The response agent should not directly post to Bluesky in the first version.

Instead it should report back to the root agent.

Possible output forms:

- a message appended to the root agent inbox
- a queued recommendation visible in UI
- a “handler result” object with summary plus drill-down transcript

Near-term target:

- response agents produce recommendations for the root agent
- the root UI can inspect or accept them

## Slash Command Surface

We need a command surface for:

- discovering templates
- creating agents
- listing running agents
- chatting with a chosen running agent
- registering handlers
- unregistering handlers

### `/list_agents`

This should list available agent templates that can be created.

Each row should include:

- template name
- short description
- default goal
- major tools used

This command is about templates, not running instances.

### `/create_agent <template> [name]`

This should create a running agent instance from a template.

Returned fields should include:

- instance name
- template used
- initial goal
- status

If no explicit name is provided:

- generate a unique instance name

### `/running_agents`

This should list currently active or available runtime instances.

Fields should include:

- instance name
- template name
- status
- queue length
- current goal
- last activity time

### `/chat <agent_name>`

This should switch the main detail/context pane so the user is effectively talking to that running agent instead of the root agent.

Target behavior:

- the UI context becomes that agent’s visible state
- user input is placed into that agent’s queue
- the agent can pick it up in its own tool loop
- the user can inspect its tool transcript and responses

This does not mean:

- the root agent disappears

Instead:

- the UI temporarily focuses a chosen agent session

### `/add_handler <name> <filter_agent> <response_agent> <events>`

Examples:

- `/add_handler urgent_replies triage_reply investigate_reply reply`
- `/add_handler social_watch generic_filter reply_worker reply,follow,like`

This should register a named handler.

Important behavior:

- event kinds may be one or many
- a handler name should be explicit
- duplicate handler names should be rejected unless later we add update semantics

### `/remove_handler <name>`

This should remove a handler by name.

This is preferable to removing by event kind alone because:

- many handlers may listen to the same event kinds

### `/list_handlers`

We likely also need this, even though it was not explicitly requested.

Fields should include:

- handler name
- event kinds
- filter agent template
- response agent template
- enabled status

Without this command:

- the handler system will be hard to inspect

## Queue Model

Running agents need inboxes.

Each queue item should include at least:

- work item ID
- source
- enqueue time
- type
- payload

Near-term queue item types:

- user chat message
- notification event
- delegated investigation request
- root-agent follow-up

Important behavior:

- user messages sent via `/chat` and event-driven work should use the same queue abstraction
- the agent runtime decides what to process next

## Concurrency And Scheduling

We should not assume every agent runs continuously in parallel with its own thread immediately.

First version can use cooperative scheduling inside the main app loop.

Possible strategy:

- each tick, give one runnable agent one step
- if an agent is waiting on an LLM/tool call, mark it busy
- when it completes, move to the next queued item

This is enough to model persistent agents without designing a full distributed runtime.

## Tool Access For Persistent Agents

Persistent agents should likely use the same core tool registry as the root agent, but with per-template restrictions.

Examples:

- a filter agent may only need:
  - `list_collections`
  - maybe `read_selected_post`

- a response agent may need:
  - `list_collections`
  - `llm_search`
  - `read_collection_item`

Later, if reply or posting tools are added, those should be permissioned explicitly per template.

## Relationship To Current Collections

The response-agent part of a handler should be able to inspect:

- recent posts by the actor in the event
- recent replies sent by that actor
- list membership collections
- future custom/labeled collections

This suggests that event-driven agents must be able to trigger the same collection refresh path already used by root-agent tool calls.

So the event/agent system should integrate cleanly with:

- collection loading
- collection refresh
- tool-prep refresh behavior

## Persistence

We should distinguish two levels of persistence.

### 1. Template Persistence

Templates are code/config assets and naturally persist.

### 2. Runtime Persistence

Running agents and handlers may or may not persist across app restarts.

Near-term recommendation:

- persist handler registrations
- optionally persist running agents later

Why:

- handlers feel like configuration
- running agents feel more like process state

First version can reasonably support:

- handler persistence
- non-persistent running agent queues

## Minimal First Version

The first implementation should stay narrow.

Recommended first slice:

- introduce `src/events/mod.rs`
- normalize notification events for:
  - `reply`
  - `follow`
  - `like`
- add agent templates
- add running agent registry
- add handler registry
- add commands:
  - `/list_agents`
  - `/create_agent`
  - `/running_agents`
  - `/chat`
  - `/list_handlers`
  - `/add_handler`
  - `/remove_handler`
- support filter-agent then response-agent execution
- deliver response-agent output back to the root agent inbox or a visible handler results pane

Out of scope for first version:

- automatic posting/replying
- cross-process worker runtime
- rich filter expressions
- durable queue replay
- full agent persistence

## Open Questions

We should keep these explicit.

### Should running agents be named manually or generated automatically?

Likely:

- support both

### Should handlers reference templates or concrete running agents?

Near-term recommendation:

- handlers should reference templates
- runtime instances can be spawned per event or pooled later

### Should `/chat <agent_name>` change the visible context window accounting?

Likely yes:

- the focused agent’s context should become the visible one in the UI

### Should event handlers be able to send results directly into root chat history?

Probably not raw history entries.

Better:

- a separate inbox or recommendation feed

### Should likes be supported immediately?

They should be supported as events if they already arrive in notifications.

But searchable authored-like collections are a different feature and may need separate cache design.

## Acceptance Criteria

This draft should be considered implemented when:

- there is a real event module
- notification events are normalized and routed
- named handler registrations exist
- named persistent running agent instances exist
- slash commands can create, list, and inspect agents
- slash commands can add and remove handlers
- handlers can subscribe to reply, follow, and like events
- filter agents can decide whether response agents should run
- response agents can use harness tools and report results back to the root agent
