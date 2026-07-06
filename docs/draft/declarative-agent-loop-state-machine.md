# Draft: Declarative Agent Loops And Verification States

## Why This Draft Exists

The current internal `llm_search` loop now works, but it is still encoded as imperative Rust control flow in:

- `src/harness/tools.rs`
  - `BlueskyTools::execute_internal_llm_search(...)`

That is a good first implementation step, but it leaves the structure of the loop implicit in code rather than explicit in data.

This draft describes a better long-term model:

- define agent loops declaratively
- make loop units explicit
- separate `tool`, `verify`, `complete`, and `fail` as runtime states

## Current State

Today the internal `llm_search` loop is effectively:

1. build an internal prompt
2. ask the model for the next action
3. if it emits `TOOL_CALL`, execute the named internal tool
4. append the tool result back into the conversation
5. repeat up to a hard-coded round limit
6. if no tool call is emitted, treat the response as final synthesis

That means the loop is currently:

- imperative
- text-protocol-driven
- generic in spirit
- but not yet represented as a first-class runtime machine

## Key Question

What are the real units inside an agent loop?

The useful answer is:

- `tool`
- `verify`
- `complete`
- `fail`

Optionally:

- `plan`
- `wait`
- `cancelled`

But the smallest durable model is still:

- `tool`
- `verify`
- terminal success or failure

## Why `verify` Should Be First-Class

Right now most loops effectively do:

- ask model what to do
- run a tool
- trust the result is enough or ask again

That misses an important middle state:

- verify whether the last tool result actually satisfied the requirement

Examples:

- a collection search ran, but returned no useful evidence
- actor resolution succeeded, but the wrong actor was chosen
- a search result exists, but no exact item URI was preserved for follow-up read
- a tool result partially failed and should change next-step planning

Those are not new tool executions.

They are verification decisions.

## Proposed Runtime Model

An agent loop should be represented as a declarative state machine.

## Core Concepts

### Loop Definition

A loop definition says:

- what states exist
- what transitions are allowed
- what kind of work happens in each state
- what terminates the loop

### Loop Unit

A loop unit is one step in the machine.

Each unit should have a type such as:

- `tool`
- `verify`
- `complete`
- `fail`

### Loop Step Record

Each executed step should produce a durable runtime record:

- step index
- state kind
- input summary
- output summary
- transition reason
- timing / cancellation status

That record should be visible in:

- live chat transcript
- `/context`
- `.debug`

## Minimal Declarative Shape

```text
AgentLoopDefinition
- loop_kind
- initial_state
- states[]
- max_rounds
- termination_policy
```

Each state:

```text
LoopStateDefinition
- state_id
- unit_kind: tool | verify | complete | fail
- executor_kind: llm | harness
- allowed_transitions[]
```

Each executed step:

```text
LoopStepRecord
- run_id
- agent_id
- step_index
- state_id
- unit_kind
- status: running | completed | failed | cancelled
- input
- output
- next_state_id
```

## Example: `llm_search`

The current `llm_search` tool agent could be expressed as:

```text
loop_kind: llm_search_internal
initial_state: decide_next_action
max_rounds: 6
```

States:

```text
decide_next_action
- unit_kind: verify
- executor_kind: llm
- outputs:
  - next tool call
  - or final synthesis

run_tool
- unit_kind: tool
- executor_kind: harness
- outputs:
  - tool result
  - tool failure

check_tool_result
- unit_kind: verify
- executor_kind: llm
- outputs:
  - request another tool
  - finish
  - fail

complete
- unit_kind: complete
- executor_kind: llm or harness

fail
- unit_kind: fail
- executor_kind: harness
```

## Why This Is Better Than Today

It makes visible what is currently hidden in imperative control flow:

- the model is not always “tool-calling”
- some steps are evaluation steps
- some transitions are policy decisions
- failure is a state, not just an error string

It also gives us cleaner control over:

- duplicate-tool prevention
- retry policy
- cancellation boundaries
- per-state token budgets
- future provider-native tool calling

## Recommended State Semantics

### `tool`

Use when the unit executes an external capability:

- resolve actor refs
- hydrate actor collections
- search a collection
- read a specific item

### `verify`

Use when the unit decides whether prior evidence is sufficient or whether the last step was valid.

This includes:

- “did the last tool result answer the question?”
- “do we need another collection?”
- “did the tool fail in a recoverable way?”
- “should we narrow or broaden scope?”

### `complete`

Use when the loop has enough grounded evidence and should finalize.

This state should produce:

- final user-facing synthesis
- final tool result
- or a structured completion artifact for the parent agent

### `fail`

Use when the loop can no longer make valid progress.

Examples:

- max rounds exceeded
- repeated invalid tool requests
- unresolved actor and no fallback path
- cancellation requested

## Declarative Transition Rules

Transitions should be explicit.

Example:

```text
decide_next_action -> run_tool
decide_next_action -> complete
decide_next_action -> fail

run_tool -> check_tool_result
run_tool -> fail

check_tool_result -> run_tool
check_tool_result -> complete
check_tool_result -> fail
```

This is better than inferring transitions from ad hoc `if` chains in code.

## Imperative Vs Declarative

### Current Implementation

Current code is imperative:

- the Rust function owns the loop structure
- the model only chooses tool calls or final text
- verification is implicit in prompt wording and loop condition branches

### Proposed Future

Future code should be declarative:

- the Rust runtime executes a generic loop engine
- each agent kind provides a loop definition
- state transitions are recorded as first-class runtime events
- prompts become per-state behavior, not the whole loop definition

## Important Design Rule

The loop structure should be data-driven even if some executors remain hard-coded.

That means:

- tool execution can still be normal Rust functions
- verification can still be LLM-based
- but the allowed states and transitions should not live only in bespoke control flow

## Possible Rust Shape

```text
enum LoopUnitKind {
    Tool,
    Verify,
    Complete,
    Fail,
}

struct LoopStateDefinition {
    state_id: String,
    unit_kind: LoopUnitKind,
    executor_kind: LoopExecutorKind,
    allowed_transitions: Vec<String>,
}

struct AgentLoopDefinition {
    loop_kind: String,
    initial_state: String,
    max_rounds: usize,
    states: Vec<LoopStateDefinition>,
}
```

Then each agent kind could register one:

- root tool loop
- internal `llm_search` loop
- future provider-native tool loop

## Relationship To Existing Work

This draft builds on:

- `docs/task/internal-llm-search-tool-definitions.md`
- `docs/task/runtime-sequencing-for-root-runs-and-internal-search-agents.md`
- `docs/task/openai-style-tool-calling.md`

It is especially relevant now because `llm_search` has crossed the line from:

- one direct harness operation

to:

- a nested internal agent loop with private tools

## Open Questions

1. Should `plan` be separate from `verify`, or is it just one kind of verification state?
2. Should a `tool` unit represent one tool execution only, or may it include tool retries?
3. Should duplicate-tool guards be expressed as:
   - transition guards
   - or tool execution policies?
4. Should provider calls themselves be modeled as explicit units, separate from `verify`?
5. Should terminal cancellation be represented as:
   - `fail`
   - or its own `cancelled` terminal state?

## Near-Term Recommendation

Do not rewrite the runtime around this immediately.

Instead:

1. keep the current imperative internal `llm_search` loop
2. add runtime step records that distinguish `tool` from `verify`
3. define one declarative loop schema in Rust
4. migrate `llm_search` onto that schema first
5. only later unify the root loop and nested loops on the same engine

## Acceptance For This Draft

This draft is useful if it clarifies:

- that the current loop is imperative code
- that `tool` and `verify` are the primary non-terminal unit kinds
- that `complete` and `fail` should become first-class terminal states
- that future nested agent loops should be represented declaratively rather than only as custom Rust control flow
