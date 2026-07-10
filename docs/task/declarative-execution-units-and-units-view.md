# Task: Add Declarative Execution Units And A New `/units` View

## Summary

Introduce one declarative runtime model centered on `execution units` and `steps`, then use that model to power a new `/units` view.

This replaces the fuzzy "everything is an agent" mental model with a more accurate seam:

- an `execution unit` is any inspectable runtime work item
- a `step` is an ordered piece of work inside a unit
- some execution units are `loop owners`
- some steps invoke child execution units

`/context` should keep focusing on context windows. The new `/units` view should focus on execution structure and active progress. It should show units nested under the unit that owns them, with sibling units sharing screen width. Each unit renders its ordered colored step blocks, and the active step uses the brighter stored color. Nested loop-owning units may be visually boxed in grey to make the ownership boundary obvious.

## Desired Seam

The universal visible runtime concept should be `execution unit`, not `agent`.

An execution unit:

- is any runtime work item the harness wants to expose in `/units`, `/context`, `/agents`, or `.debug`
- has status
- may have a context window
- may have ordered child execution units
- may have ordered steps

A loop owner is a kind of execution unit that owns iterative control flow.

Examples:

- root run
- internal search planner
- `collection_summary`

A harness step is an ordered step inside an execution unit's workflow.

Examples:

- prep
- hydration
- refresh
- review
- repair
- selection
- synthesis

A step may invoke a child execution unit. That child unit should remain a real nested ownership boundary rather than being flattened into the parent timeline.

This means:

- `collection review` is visible because it is inspectable runtime work, not because it is a true agent
- `collection_summary` is visible as a child execution unit because it owns its own workflow
- a parent step may invoke a child workflow, and that child workflow then exposes its own steps and active state

## Runtime Model Changes

Add a shared declarative execution model for all inspectable harness work.

Core types:

- `ExecutionUnitDefinition`
- `ExecutionUnitKind`
- `ExecutionWorkflowDefinition`
- `ExecutionStepDefinition`
- `ExecutionStepKind`
- `ExecutionStepExecutor`
- `ExecutionStepColor`

The model should cover:

- existing loop nodes already expressed in `LoopDefinition`
- non-loop harness orchestration such as root tool prep, summary prep, hydration, initial refresh, collection selection, and review or repair handoffs

Step metadata should include:

- stable step id
- human label
- owning workflow or unit kind
- executor kind: `Harness` or `Llm`
- step kind: prompt, prep, hydration, refresh, tool, review, repair, synthesize, branch, return, selection, invocation
- declared normal color
- declared active or bright color
- ordering within the workflow

Execution unit runtime state should include:

- unit id
- parent unit id
- unit kind
- label
- status
- optional context window
- declared workflow definition when applicable
- per-step runtime status
- active step id
- child execution unit ids

## Relationship To Existing Structures

This should become the source-of-truth runtime model for visible execution.

The current `LoopDefinition` layer should either:

- be expanded so it directly carries the richer unit and step metadata, or
- map losslessly into the richer execution-unit model

The existing runtime tree should stop relying on the idea that every visible node is an agent.

Near-term:

- `AgentGraph` may remain as a compatibility wrapper if necessary
- execution units should become the authoritative semantic model
- `/agents`, `/context`, and `.debug` should render from execution-unit ownership and step metadata

The model must preserve the nuance that a harness step can invoke a child execution unit. A child workflow is not just another sibling step in the parent unit.

## Workflow Coverage

The declarative model should cover all inspectable harness work, not only current loop definitions and not only LLM-backed work.

That includes:

- root workflow
- search workflow
- summary workflow
- `collection_summary` workflow
- root tool preparation
- summary preparation and scope resolution
- hydration and refresh work
- collection selection
- review and repair phases
- parent synthesis handoff

Public summary orchestration should no longer be described as partially implicit control flow hidden inside handler bodies. Its visible runtime work should be declarative.

## `/units` View

Add a new `/units` view as the primary visualization for runtime structure and active harness progress.

The purpose of `/units` is to show:

- ownership
- nesting
- sibling relationships
- ordered steps
- current active step

It is not primarily a context-budget view.

Layout rules:

- each visible box or lane is an execution unit
- children render below the unit that owns them
- sibling units share the width available under their parent
- child units remain visually constrained to the width of the parent unit

Unit rendering:

- header with unit label and status
- ordered step strip inside the unit
- active step uses the brighter stored variant of the step color
- loop-owning nested child units may be boxed in grey using block characters so the ownership boundary is obvious

Selection and interaction:

- arrow keys move selection among visible execution units
- selected unit should receive stronger emphasis
- scrolling remains separate from unit selection

The view should make these distinctions obvious:

- this is a parent unit step
- this step invoked a child execution unit
- these blocks belong to the child unit's own workflow, not to the parent timeline

## `/context` Integration

`/context` should remain focused on context windows and token composition.

However, it should source ownership metadata from the same execution-unit model so each context window knows:

- which execution unit owns it
- which workflow it belongs to
- which step was active when it was captured, if relevant

This task does not require graph rendering inside `/context`. The new structural runtime visualization belongs in `/units`.

## `.debug` And `/agents`

Update `.debug` and `/agents` rendering so they distinguish:

- loop-owning execution units
- non-loop inspectable steps
- child unit invocation boundaries

Current visible runtime work such as collection review should remain inspectable, but it should no longer be mislabeled as a peer agent if it is actually a harness step or review unit.

## Acceptance Criteria

- all inspectable harness work is representable through one declarative execution-unit model
- loop-owning workflows and non-loop steps are both visible without conflating them
- steps can invoke nested child execution units while preserving ownership boundaries
- every declared step stores its UI color and active color
- `/units` renders execution units, nested ownership, and ordered step strips
- sibling units share width under their parent
- active steps render in a brighter version of the stored step color
- nested workflow or child-unit boundaries are visually distinct
- `/context` continues to work and can attribute windows to execution units
- `.debug` and `/agents` reflect execution-unit ownership rather than overloading `agent`

## Test Plan

Declarative model tests:

- existing loop definitions map cleanly into execution-unit workflow definitions
- non-loop harness paths can declare ordered steps without pretending to be agents
- child execution unit invocation is represented explicitly
- every step definition includes declared color data

Runtime tests:

- active step updates correctly as execution progresses
- nested workflows attach as child execution units under the invoking parent
- review and repair remain visible as inspectable runtime work without being flattened into fake peer agents

`/units` view tests:

- sibling units share width correctly
- child units render below their owner and remain within parent width
- selected unit changes with arrow keys
- active step uses the brighter stored color
- nested workflow boxes render distinctly from ordinary unit step strips

Regression tests:

- `/context` still renders context windows correctly
- existing debug capture still works with execution-unit ownership metadata
- current `collection_summary` and review visibility remain inspectable

## Assumptions

- `execution unit` is the new source-of-truth runtime concept
- `loop owner` is a subtype or role of execution unit, not a separate parallel model
- a step may invoke a child execution unit, and that child remains a real nested boundary
- `/units` is the primary view for runtime structure
- `/context` remains the primary view for context windows
- step colors are stored in declarative definitions, not inferred in the UI
- grey boxed nested units are the default visual treatment for nested workflows in this phase
