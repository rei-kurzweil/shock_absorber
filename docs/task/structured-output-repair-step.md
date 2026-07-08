# Task: Add A Harness-Owned `StructuredOutputRepairStep`

## Summary

Some internal worker outputs are semantically close to valid, but fail strict parsing.

A recent `summary` run showed:

- `SUMMARY_RESULT_START`
- a grounded `summary:` paragraph
- many valid `covered_item_uri:` lines
- but no trailing `SUMMARY_RESULT_END`

That is not a review problem.

It is also not a pure deterministic parsing problem.

The harness already knows:

- which structured shape was expected
- which worker produced the output
- the raw model text
- the parse failure reason
- the original collection window
- the original prompt and requested scope

Today that malformed-but-meaningful output collapses into:

- parse failure
- no typed result
- weak or misleading review fallback
- parent/root synthesis still seeing raw diagnostic text

That is the wrong boundary.

We should add a harness-owned `StructuredOutputRepairStep` between strict parsing and review.

## Problem

The current flow assumes a worker output is either:

- strictly parseable
- or unusable

That is too coarse.

Real failures often land in the middle:

- start marker exists
- most required fields are present
- the content is clearly attempting the expected schema
- but one structural defect prevents typed parsing

Examples:

- missing end marker
- malformed but recoverable tagged field layout
- mixed prose plus near-valid structured block
- truncated output where the intended schema is still inferable from the raw text and evidence context

In those cases, the current review steps are poorly placed to help:

- `summary_review` is designed to verify groundedness and sufficiency
- `search_review` is designed to verify groundedness and request repair instructions
- neither step is supposed to rewrite or reconstruct structured output

At the same time, the current parser layer is too strict to perform semantic recovery on its own.

## Desired Behavior

When a worker emits malformed structured output, the harness should get one repair chance before review.

### Preferred order

1. worker emits raw text
2. strict parser attempts typed parse
3. if parse succeeds, continue to review
4. if parse fails, check whether repair is eligible
5. if eligible, run one `StructuredOutputRepairStep`
6. if repair emits one valid structured result, continue to review
7. if repair emits `CANNOT_REPAIR` or still fails validation, treat the worker result as failed

The key point is:

- review should not be overloaded into a hidden rewrite step
- parent synthesis should not consume malformed raw output as if it were an accepted typed result

## Why This Should Be A Separate Step

The current architecture already distinguishes:

- worker generation
- harness validation
- harness-owned review
- planned harness-owned repair for invalid internal tool calls

`StructuredOutputRepairStep` fits that same model.

It should be a sibling concept to tool-call repair, not a hidden extension of `summary_review`.

That keeps responsibilities clean:

- worker: generate
- parser: validate strict shape
- structured-output repair: recover intended typed output when strict parse fails
- review: judge groundedness and sufficiency

## Scope

### First slice

Start with malformed `summary` outputs only.

Target example:

- raw output contains `SUMMARY_RESULT_START`
- strict tagged parse fails
- JSON fallback fails
- raw output still appears to be a genuine attempt at the required summary schema

The repair step should try to recover a valid typed `LlmSummaryResult`.

### Later generalization

If the shape works, generalize it to other internal structured outputs, such as:

- `search` result blocks
- future structured review outputs
- other harness-owned child result schemas

## Repair Eligibility

The harness should not call repair for every parse failure.

Good initial eligibility rules:

- a recognizable schema marker exists, such as `SUMMARY_RESULT_START`
- the failing worker is expected to return a structured schema
- the raw output is non-empty and not obviously unrelated free-form prose
- strict parse has already failed

Bad initial candidates:

- completely empty response
- output with no schema marker and no plausible structure
- cases where deterministic validation already proves the output cannot be trusted

## Proposed Repair Contract

### Inputs

The repair step should receive:

- worker kind, such as `summary`
- expected schema description
- raw model output
- strict parse failure diagnostic
- original prompt
- requested summary scope when relevant
- source collection window or selected evidence
- any known collection/page metadata required by the schema

### Output

The repair step should emit exactly one of:

- one repaired strict structured result
- `CANNOT_REPAIR`

It must not emit:

- explanatory prose around the result
- parent-facing synthesis
- tool calls

### Validation

The repaired output must still pass the same typed validation path the original worker output would have passed.

If the repaired output still fails validation:

- do not run a second repair round
- mark the repair attempt failed

## Relationship To Review

After repair:

- `summary_review` should receive a typed `summary` result and evaluate groundedness/sufficiency
- `search_review` should receive a typed `search` result and evaluate groundedness/usefulness

Review should remain a verifier.

It may explain:

- grounded but insufficient
- unsupported by evidence
- invalid accounting

It should not become the place where malformed structured text is reconstructed into typed output.

## Debugging And Visibility

The repair step should be visible in runtime/debug output as its own step.

At minimum, debug artifacts should show:

- original parse failure reason
- whether repair was attempted
- whether repair succeeded
- whether the final reviewed result came from original parse or repaired parse

This matters because a repaired result is materially different from:

- first-pass strict success
- deterministic parser tolerance
- unrepaired hard failure

## Suggested Implementation Slices

### Slice 1: Add summary-only repair path

- add a `StructuredOutputRepairStep` concept in the harness
- trigger it only for `summary` parse failures with recognizable markers
- require repaired output to validate into a real `LlmSummaryResult`

### Slice 2: Add repair-specific debug/runtime reporting

- new agent/debug node naming
- explicit diagnostics for original parse failure and repair outcome
- distinguish repaired success from first-pass success

### Slice 3: Tighten parent/root handoff

- prevent malformed raw worker output from being treated as accepted parent-facing evidence
- only reviewed typed results should feed parent synthesis

### Slice 4: Generalize beyond `summary`

- evaluate whether `search` or other structured child outputs should use the same repair primitive

## Acceptance Criteria

- malformed `summary` worker output with recognizable schema markers gets one harness-owned repair attempt before review
- repaired structured output must pass the normal typed validation path
- `summary_review` continues to judge groundedness and sufficiency, not perform structured reconstruction
- failed parse plus failed repair does not silently degrade into parent synthesis over malformed raw output
- runtime/debug output clearly distinguishes original parse success, repaired success, and unrepaired failure

## Open Questions

- Should `StructuredOutputRepairStep` be implemented as one generic prompt template plus per-schema context, or as schema-specific repair prompts?
- Should repair output be required to use the same tagged text protocol as the original worker, or can it emit JSON first and normalize afterward?
- Should the repair step receive the full source evidence window, or only the raw malformed output plus minimal schema metadata?
- After this exists, should we keep adding deterministic parser tolerance for trivial cases, or prefer routing all semantic recovery through the repair step?
