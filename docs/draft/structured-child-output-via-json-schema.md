# Draft: Structured Child Outputs Via JSON Schema

## Goal

Replace the current fragile plain-text parsing for leaf child LLM calls with provider-constrained structured output.

Near-term scope:

- `collection_search`
- `collection_review`
- optional later: parent `llm_search` synthesis

Out of scope for the first pass:

- replacing the internal planner `TOOL_CALL` loop
- provider-native function calling for all harness tools
- changing the user-facing root run protocol

## Why This Draft Exists

Right now child search calls ask the model to emit plain text like:

```text
title: ...
summary: ...
uri: ...
uri: ...
```

That is easy for the model to drift away from.

When drift happens, the harness often still finds some grounded records, but it cannot extract a usable `summary:` line. It then falls back to synthetic text assembled from matched records.

That fallback is useful for resilience, but it is clearly carrying too much of the system.

## Current Failure Mode

Current `collection_search` parsing depends on:

- explicit `summary:` extraction
- explicit or inferred `uri:` recovery
- a review pass that expects one grounded summary paragraph

Current code path:

- `LlmApiClient::complete_chat(...)` only sends basic chat-completions fields
- `parse_llm_search_result(...)` parses plain assistant text
- `extract_llm_search_summary(...)` only accepts a narrow set of summary shapes
- if no usable summary is found, `fallback_llm_search_summary(...)` synthesizes one from recovered records

Relevant files:

- `src/harness/llm_api.rs`
- `src/harness/tools.rs`
- `src/harness/prompts/agents/collection_search.md`
- `src/harness/prompts/agents/collection_review.md`

## What "Real Structured Output" Should Mean Here

For child calls, "real structured output" should mean:

1. the request includes a response schema on the wire
2. the backend constrains generation to that schema
3. the harness parses JSON, not ad hoc plain text
4. the prompt still explains the semantic task, but the shape is enforced by the backend

This is different from prompt-only "please output JSON".

## Feasibility

### Backend Side

The current local path is already close enough to support this.

`shock_absorber` sends requests to an OpenAI-compatible `POST /v1/chat/completions` endpoint via `src/harness/llm_api.rs`.

`llama.cpp` documents support for JSON Schema constrained decoding on `/chat/completions` via `response_format`, and says the schema is converted to GBNF under the hood.

That is enough for leaf structured output if we stay within the supported JSON Schema subset.

### Model Side

Gemma 4 itself appears suitable for this use case.

The official Gemma 4 function-calling guide explicitly includes `google/gemma-4-E4B-it` in the supported model list and shows passing tool schemas to the model in the Hugging Face path.

That does not prove identical behavior through `llama.cpp`, but it is strong evidence that Gemma 4 E4B is expected to follow structured interface contracts reasonably well.

### Important Limitation

The constrained schema only guarantees output shape, not judgment quality.

For example, a schema can force:

- there is a `summary`
- there is a `selected_results` array
- each selected result has a `uri`

But it cannot guarantee:

- the summary is actually grounded
- the selected URIs are semantically the best ones
- the sentiment interpretation is good

So review logic still matters.

## Proposed Request Shape

Extend `ChatCompletionRequest` with an optional `response_format`.

Suggested Rust shape:

```rust
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: usize,
    stream: bool,
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ResponseFormat {
    #[serde(rename = "json_schema")]
    JsonSchema {
        json_schema: JsonSchemaSpec,
    },
}

#[derive(Serialize)]
struct JsonSchemaSpec {
    name: String,
    schema: serde_json::Value,
    strict: Option<bool>,
}
```

If the local backend rejects `response_format`, the harness should record that and fall back to the current plain-text path.

## Proposed Child Schemas

### 1. `collection_search`

Suggested output shape:

```json
{
  "title": "string",
  "summary": "string",
  "selected_results": [
    {
      "uri": "string",
      "reason": "string"
    }
  ]
}
```

Constraints:

- `summary` required
- `selected_results` required
- `selected_results` length between 1 and 10
- `uri` required
- `additionalProperties: false`

Why include `reason`:

- helps debug why each result was chosen
- gives review a smaller per-result hook than the full summary
- remains easy to ignore downstream if noisy

### 2. `collection_review`

Suggested output shape:

```json
{
  "status": "pass",
  "reason": "string",
  "repair_needed": false,
  "repair_instructions": null
}
```

or

```json
{
  "status": "fail",
  "reason": "string",
  "repair_needed": true,
  "repair_instructions": "string"
}
```

Keep this schema narrow. The review step should not be allowed to invent extra fields.

## Parsing Changes

### `collection_search`

Instead of:

- parse free text
- search for `summary:`
- recover `uri:` lines or infer matches from hints

Do:

1. parse assistant content as JSON
2. validate required fields
3. verify every returned `uri` exists in the searched collection
4. map into `LlmSearchResult`
5. if validation fails, fall back to the existing text parser

This keeps the current fallback path available while dramatically improving the happy path.

### `collection_review`

Instead of:

- parse compact text verdict blocks

Do:

1. parse JSON verdict
2. validate fields
3. map into `CollectionReviewVerdict`
4. fall back to the current text parser if needed

## Prompt Changes

Even with constrained decoding, the prompt should still describe the expected semantics clearly.

`llama.cpp` notes that the JSON Schema constrains output but is not injected into the prompt for ordinary structured output.

So the prompt still needs to explain:

- what counts as grounded evidence
- what the summary should emphasize
- how many results to select
- what distinctions matter for reputation/sentiment queries

But the prompt no longer needs to explain textual output syntax like `summary:` or `uri:` lines.

That should reduce one class of avoidable failures.

## Why Child Calls First

Structured output is a better fit for leaf steps than for the planner loop.

Leaf child steps:

- have stable small schemas
- are easy to validate
- already return object-like results
- have obvious fallback behavior

The internal planner loop is harder:

- it mixes tool-choice semantics with reasoning
- it may eventually want native tool calling rather than plain JSON output
- its schema is less stable than the leaf result structs

So the recommended order is:

1. `collection_review`
2. `collection_search`
3. optional parent synthesis
4. only later revisit planner-native tool calling

## Risks

### 1. Backend Schema Support Is A Subset

`llama.cpp` explicitly supports only a subset of JSON Schema when converting to grammars.

So the first schemas should avoid:

- nested `$ref`
- `oneOf` / `anyOf` mixes with `properties`
- complex pattern constraints
- conditional schema logic

Flat objects and simple arrays are the safest starting point.

### 2. Shape Compliance Does Not Fix Weak Summaries

The model may still produce:

- generic summaries
- semantically wrong summaries
- shallow `reason` fields

Structured output reduces parser breakage, not reasoning error.

### 3. Performance

Grammar-constrained decoding may be somewhat slower.

That is probably acceptable for child search/review calls because those calls are already narrower and slower than plain deterministic parsing.

## Recommended Initial Implementation

### Phase 1

Add optional structured-output support to `LlmApiClient`.

- add `response_format` to request serialization
- keep response parsing unchanged at the wire level
- the OpenAI-compatible backend still returns assistant content as text, which will now be JSON text

### Phase 2

Add a structured-output path for `collection_review`.

Reason:

- simpler schema
- smallest blast radius
- immediate reduction in text verdict parsing failures

### Phase 3

Add a structured-output path for `collection_search`.

- parse JSON first
- validate selected URIs against the current collection
- keep current text parsing as fallback

### Phase 4

Improve debug artifacts.

For each child LLM call, log:

- raw assistant content
- whether structured mode was requested
- whether JSON parse succeeded
- whether schema validation succeeded
- whether the harness fell back to text parsing

Without that, it will still be hard to tell whether failures come from:

- model semantics
- backend schema rejection
- JSON parsing
- collection validation

## Concrete Example

For `collection_search`, the prompt would become semantic-only:

```text
Inspect the provided collection carefully.

Return a grounded local summary for this one collection.

Choose the strongest supporting records from the collection only.

The summary must stay local to this collection and must not answer the user's overall question directly.
```

The request would attach a schema like:

```json
{
  "type": "object",
  "properties": {
    "title": {"type": "string"},
    "summary": {"type": "string"},
    "selected_results": {
      "type": "array",
      "minItems": 1,
      "maxItems": 10,
      "items": {
        "type": "object",
        "properties": {
          "uri": {"type": "string"},
          "reason": {"type": "string"}
        },
        "required": ["uri", "reason"],
        "additionalProperties": false
      }
    }
  },
  "required": ["title", "summary", "selected_results"],
  "additionalProperties": false
}
```

## Bottom Line

This is feasible now.

As of 2026-07-06, the main blocker is not model capability. It is that `shock_absorber` still uses prompt-only text contracts for child outputs even though the local stack appears capable of schema-constrained JSON generation.

The best next step is to implement structured output for `collection_review` and `collection_search` while keeping the current text parser as fallback during rollout.
