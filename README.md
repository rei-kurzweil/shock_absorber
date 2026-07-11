# shock_absorber

`shock_absorber` is a terminal Bluesky client with a local AI harness attached to it.

It does two things at once:

- it polls and caches Bluesky notifications plus related actor data
- it lets a local model reason over that cached state through a bounded tool/loop runtime

The project is not just "a chat UI with tools". The important part is the harness structure:

- root-visible tools
- harness-owned internal tools
- explicit loop definitions
- modeled context windows with token budgeting
- persistent debug artifacts under `.debug/`

This README is a practical summary of those primitives and how to run the TUI.

## What The App Does

At runtime the app logs into Bluesky, polls notifications, caches actor-backed collections, and exposes a root AI loop that can answer questions using those cached collections.

Examples of cached or hydrated data:

- notifications
- selected post text
- actor bios
- pinned posts
- recent posts
- recent replies received
- Clearsky moderation-list memberships

The root model talks to the harness through a small public tool surface, and the harness decides how much extra work is needed to answer a query safely.

## Main Primitives

### Public tools

The root model sees a small prompt-facing tool inventory defined in `src/harness/tools.rs`.

Current public tools:

- `read_selected_post`
  - read the currently selected notification post/reply from the UI
- `search`
  - run selective Bluesky-grounded evidence search for a person or topic
- `summary`
  - run coverage-oriented summarization over an actor-backed collection such as recent posts

The root agent only supplies high-level intent. The harness owns:

- actor resolution
- collection choice
- pagination
- grounding and sufficiency checks

For `summary`, the public return path is intentionally more deterministic than `search`:

- `collection_summary` produces the grounded multi-window payload
- the public `summary` tool formats that payload as:
  - one scope-level commentary block
  - one concatenated per-window summaries block
- root context compaction preserves that block shape
- the root run returns that `summary` result directly when it is the latest grounded answer, instead of asking the root model to paraphrase it again

### Internal tools

Inside `llm_search`, the planner can request a narrower internal tool set.

Current internal tools include:

- `resolve_actor_refs`
- `hydrate_actor_scope`
- `set_summary_scope`
- `collection_search`
- `search_global_posts`

These are not exposed to the user directly. They are harness-level building blocks used by the internal planner loop.

### Agent graph

The runtime also maintains a persistent `AgentGraph` for inspection and debug output.

Current node kinds include:

- root agent
- tool agent
- collection search tool
- collection summary tool
- search review agent
- summary review agent

This graph is visible in `/agents`, used by `/context`, and written to `.debug/agents/`.

### Loops

Loop definitions live under `src/harness/loop/`.

Current real loop owners:

- `root`
- `search`
- `collection_summary`

The important distinction is:

- `root` owns top-level prompting and public tool execution
- `search` owns the internal `llm_search` planner protocol
- `collection_summary` owns multi-page coverage-oriented summarization

That means coverage-oriented summaries are no longer one-shot leaf calls. They run as a harness-managed loop with page review, planner synthesis, and final notes synthesis.

## Loop Model

### Root loop

The root loop is the user-facing orchestration boundary.

Shape:

```text
root_prompt
  -> tool_execution
  -> root_prompt

root_prompt
  -> protocol_repair
  -> root_prompt

root_prompt
  -> return
```

The root model either:

- emits a public tool call
- emits a final answer
- or gets repaired if it breaks the expected protocol

When a grounded public `summary` result already answers the query, the harness now short-circuits the last step:

- the root transcript still records the tool call and tool result
- but the final answer reuses the grounded `summary` result directly
- this avoids a second lossy root-level paraphrase over an already-complete coverage summary

### Internal `search` loop

The internal planner loop for `llm_search` decides which harness-owned internal tool to run next.

Shape:

```text
planner_decide
  -> execute_internal_tool
  -> planner_decide

planner_decide
  -> planner_protocol_repair
  -> planner_decide

planner_decide
  -> return
```

This loop handles:

- actor resolution
- actor-scope hydration
- exact collection targeting
- topical global search when needed
- final search synthesis

### `collection_summary` loop

Coverage-oriented summaries use a dedicated paginated loop.

Shape:

```text
init_window
  -> summarize_page
  -> review_page
  -> collection_summary_planner
  -> advance_cursor?
  -> collection_summary_notes
  -> return_summary
```

With repairs and reviews included, the real loop has dedicated states for:

- malformed page output repair
- planner synthesis repair
- final notes repair

The default page size is 25 items.

The public output contract for that loop is now:

- `Overall commentary across <collection label>:`
- one grounded scope-level synthesis paragraph block
- `Concatenated page summaries for <collection label>:`
- the accepted per-window summaries concatenated in coverage order

That is the block shape root sees and preserves.

## Context Window Model

Context windows are modeled explicitly in `src/harness/context_window.rs`.

The core types are:

- `LLMContext`
  - header plus ordered sections before budgeting
- `ContextSectionKind`
  - semantic labels for each section
- `BuiltContextWindow`
  - rendered prompt plus token accounting after budgeting
- `ProviderContextLimits`
  - provider/model context limits and reserved output budget

Each context window has:

- an instruction header
- ordered titled sections
- approximate token accounting
- truncation when the input budget is exceeded

The budgeting rule is simple:

- available input tokens = `max_context_tokens - reserved_output_tokens`

The current implementation uses approximate token estimation instead of a model-specific tokenizer. That is deliberate for now.

### Section kinds

Current section kinds include:

- `ToolDefinitions`
- `UiContext`
- `CurrentTask`
- `UserAiChat`
- `CollectionEvidence`
- `ReviewEvidence`
- `ParentSearchResults`
- `Generic`

This lets the harness distinguish:

- root prompt/tool inventory
- currently selected UI state
- live chat history
- collection evidence used by search or summary steps
- reviewed parent results reused by later synthesis

### Root context construction

When a root run starts, the app builds a tool-aware root context window from:

- tool inventory
- search hints
- current UI context
- current task
- recent chat

That root window becomes:

- the initial user-context payload for the root model
- the basis for `/context`
- part of `.debug/root_prompt_snapshot.md`

### Child context construction

Child steps build narrower windows.

Examples:

- a collection-summary page step gets one collection window plus the local prompt
- a planner/notes step gets accepted page summaries plus current coverage state
- review steps receive the candidate output plus the evidence needed to validate it

The harness tries to keep child windows tight rather than inheriting broad root chat/UI state.

## TUI Model

The TUI has two presentation modes:

- the main split-view terminal UI
- the stdout-style AI chat view

Use `Tab` to switch between them.

### Main TUI layout

The default terminal layout has:

- a main body
- a `Command` input area
- a status line

The main body is either:

- a list/detail split
- or a fullscreen overlay

In split mode:

- the left pane is either `Notifications` or `Agents`
- the right pane is the detail view for the currently active left pane item

In overlay mode:

- `/context`, `/help`, `/task`, and similar command output take over the main body

### Notifications view

The notifications split is the default operator view.

Left pane:

- cached notifications

Right pane:

- notification metadata
- selected actor bio
- pinned posts
- Clearsky moderation lists when loaded

Press `Enter` with an empty command line to open the selected notification detail.

### Agents view

Open with `/agents`.

Left pane:

- the current agent graph in display order

Right pane:

- selected node metadata
- node status
- tool/collection identifiers
- built context window stats
- rendered child context
- result summary

This is the quickest way to inspect what a root run actually spawned.

### Context view

Open with `/context`.

This is a fullscreen visualization of prompt/context usage across the current root run and child agent nodes.

It is useful for:

- verifying what the root model saw
- inspecting child context shapes
- spotting truncation or token-budget issues

### AI chat view

Use `Tab` to switch into the stdout-style AI chat transcript.

This view shows:

- root transcript
- tool activity
- subagent progress events
- final model output

Input behavior in this mode:

- `Enter` submits
- `Shift+Enter` inserts a newline
- `Ctrl+J` inserts a newline
- `Esc` clears the editor, or leaves the view if the editor is empty

## TUI Commands

Local commands currently supported:

- `/bio handle.bsky.social`
- `/replies_from handle.bsky.social`
- `/pins handle.bsky.social`
- `/agents`
- `/notifications`
- `/context`
- `/stop`
- `/models`
- `/model model-name`
- `/task`
- `/clear`
- `/help`
- `/quit`

Anything else is treated as a root AI query and sent to the local model through the harness.

Useful examples:

- `who is @somebody and why are people mad at them`
- `search for posts about gemma performance on bluesky`
- `summarize the last 50 posts by schizanon.bsky.social`

## Navigation

Main keybindings:

- `Up` / `Down`
  - move the active left-pane selection
- `Enter`
  - with empty command input: activate the selected item
  - with non-empty command input: run the command or root query
- `PageUp` / `PageDown`
  - scroll the current detail/overlay
- `Tab`
  - toggle AI chat view
- `Esc`
  - clear current input, or dismiss the current fullscreen overlay
- `Ctrl+C`
  - quit immediately
- `q`
  - quit from the main TUI when the command input is empty

## Debug Artifacts

The app resets and rewrites `.debug/` on startup.

Important files:

- `.debug/chat_transcript.md`
- `.debug/current_task.txt`
- `.debug/root_prompt_snapshot.md`
- `.debug/summary_trace.md`
- `.debug/agents/agent_*.md`

Use these when:

- a run seems to use the wrong context
- a summary appears to stop early
- you need the exact rendered child prompt
- you want a copyable transcript outside the TUI

## Getting Started

### 1. Prepare Bluesky credentials

Create a `.env` file or export these variables:

```dotenv
BSKY_HANDLE=your-handle.bsky.social
BSKY_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx
```

### 2. Start a local OpenAI-compatible inference server

By default the app expects:

- base URL: `http://127.0.0.1:5000`
- chat endpoint: `/v1/chat/completions`
- default model name: `qwen-3.5-local`

Optional overrides:

- `EVIL_GEMMA_BASE_URL`
- `EVIL_GEMMA_MODEL`
- `SYSTEM_PROMPT_PATH`
- `SHOCK_ABSORBER_DB_PATH`
- `SHOCK_ABSORBER_CONFIG_PATH`

The client will also probe:

- `/v1/capabilities`
- `/v1/models`

if the server exposes them.

### 3. Run the app

```bash
cargo run
```

The app will:

1. load `.env`
2. log into Bluesky
3. open or create `shock_absorber.sqlite3`
4. restore cached state
5. reset `.debug/`
6. enter the alternate-screen TUI

### 4. First-use workflow

A sensible first pass:

1. run `cargo run`
2. wait for notifications to load
3. move through the notification list with `Up` / `Down`
4. press `Enter` on a notification to inspect it
5. run `/pins handle.bsky.social` or `/bio handle.bsky.social` for a known actor
6. type a natural-language query such as:

```text
lets see what this user is accused of based on what lists they are on
```

7. use `/context` and `/agents` after the run to inspect what happened

## Mental Model For Working On The Harness

If you are changing the runtime, keep these boundaries straight:

- a tool is something the model can request by name
- a loop is a harness-owned state machine that decides what happens next
- a context window is the bounded prompt payload actually sent to one LLM step
- an agent node is the persistent runtime/debug record of a unit of work

In practice:

- public tools define the root-visible API
- internal tools support planner execution
- loops own retries, paging, repair, and completion
- context windows are the real prompt boundary
- `.debug` is the source of truth when behavior looks wrong in the UI

## Related Docs

- [docs/spec/llm_search.md](/home/rei/_/shock_absorber/docs/spec/llm_search.md)
- [docs/bugs.md](/home/rei/_/shock_absorber/docs/bugs.md)
- [docs/draft/tools-agents-and-execution-steps.md](/home/rei/_/shock_absorber/docs/draft/tools-agents-and-execution-steps.md)
- [docs/draft/context-window-logging.md](/home/rei/_/shock_absorber/docs/draft/context-window-logging.md)
