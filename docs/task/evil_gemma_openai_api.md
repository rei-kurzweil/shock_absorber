# Task: Refactor `gemma-4` Flask Server to Serve `shock_absorber` via OpenAI-Compatible REST

## Goal

Refactor `/home/rei/_/gemma-4` so it can serve `shock_absorber` as a local LLM backend over an OpenAI-compatible HTTP API.

This is specifically for the harness work in this repo:
- `shock_absorber` should talk to providers only through HTTP/REST
- the wire format should look like OpenAI chat completions
- local `gemma-4` should be usable as a drop-in testing backend

## Why

Current `gemma-4` behavior is close, but not compatible enough:
- it exposes `POST /chat`, not `POST /v1/chat/completions`
- it accepts a custom payload with `message`, `image`, and `image_mime_type`
- it streams custom SSE payloads like `{"content": "..."}`
- it does not currently expose a standard non-streaming OpenAI-style response object

Internally, the model layer is already near the target:
- `vision_inference.py` already constructs OpenAI-style `messages`
- it already calls `llama_cpp` through `create_chat_completion(...)`
- it already supports streaming and multimodal requests

So this task is mainly a Flask API-surface refactor, not a model-runtime rewrite.

## Source Context

Relevant files in `/home/rei/_/gemma-4`:
- `app.py`
- `vision_inference.py`
- `docs/draft/streaming-responses.md`

Observed current state:
- `app.py` preloads the model singleton at startup
- `app.py` serves `/chat` for browser use
- `vision_inference.py` wraps the model and exposes `generate_response(...)`
- `generate_response(...)` already calls `self.llm.create_chat_completion(...)`

## Required Outcome

`gemma-4` should support:
- `POST /v1/chat/completions`
- OpenAI-style request bodies
- OpenAI-style non-streaming JSON responses
- OpenAI-style SSE streaming responses

The intent is that `shock_absorber` can point `src/harness/llm_api.rs` at the local Flask server without needing provider-specific request translation.

## API Requirements

### 1. Endpoint

Add:
- `POST /v1/chat/completions`

Optional but useful:
- `GET /v1/models`
- `GET /health` or `GET /ready`

`/chat` may remain temporarily as a compatibility shim for the browser UI, but it should not be the primary interface for the harness.

### 2. Request Shape

Accept an OpenAI-style chat completion request body with fields such as:
- `model`
- `messages`
- `stream`
- `max_tokens`
- `temperature`
- `stop`

Minimum supported message formats:

Text-only:
```json
{
  "model": "gemma-4-local",
  "messages": [
    {"role": "system", "content": "You are concise."},
    {"role": "user", "content": "Summarize this actor."}
  ],
  "stream": false
}
```

Multimodal user turn:
```json
{
  "model": "gemma-4-local",
  "messages": [
    {"role": "system", "content": "Describe the image briefly."},
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "What is happening here?"},
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/jpeg;base64,..."
          }
        }
      ]
    }
  ],
  "stream": true
}
```

Notes:
- the `model` field can be accepted and ignored initially if only one model is loaded
- system messages should pass through unchanged
- user `content` must support both plain string and array forms

### 3. Non-Streaming Response Shape

When `stream` is false or omitted, respond with an OpenAI-style JSON object:

```json
{
  "id": "chatcmpl-local-123",
  "object": "chat.completion",
  "created": 1782921600,
  "model": "gemma-4-local",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "..."
      },
      "finish_reason": "stop"
    }
  ]
}
```

If practical, also include:
- `usage.prompt_tokens`
- `usage.completion_tokens`
- `usage.total_tokens`

If exact usage data is hard to obtain from the current wrapper, usage can be omitted initially.

### 4. Streaming Response Shape

When `stream` is true:
- return `text/event-stream`
- emit OpenAI-style chunk objects
- terminate with `data: [DONE]`

Expected chunk pattern:
```text
data: {"id":"chatcmpl-local-123","object":"chat.completion.chunk","created":1782921600,"model":"gemma-4-local","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

data: {"id":"chatcmpl-local-123","object":"chat.completion.chunk","created":1782921600,"model":"gemma-4-local","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-local-123","object":"chat.completion.chunk","created":1782921600,"model":"gemma-4-local","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

This is the main compatibility gap in the current Flask implementation.

## Proposed Refactor

### `vision_inference.py`

Refactor `generate_response(...)` so it can accept already-normalized OpenAI-style inputs instead of only the current convenience parameters:
- current API is roughly `text + optional image + optional system prompt`
- target API should accept `messages`, plus generation settings like `stream`, `max_tokens`, `temperature`, and `stop`

Possible direction:
- keep the current convenience method for the browser UI if helpful
- add a lower-level method like `create_chat_completion(messages, stream, max_tokens, temperature, stop)`

That avoids making Flask route handlers reconstruct chat turns in multiple places.

### `app.py`

Refactor the Flask layer so the OpenAI-compatible route becomes the primary route.

Needed changes:
- parse request JSON for `/v1/chat/completions`
- validate `messages`
- pass normalized parameters into the model wrapper
- support both streaming and non-streaming code paths
- emit OpenAI-compatible JSON/SSE envelopes

Recommended structure:
- one helper to validate/normalize request payload
- one helper to build a standard completion id and metadata
- one helper for non-streaming response formatting
- one helper for SSE chunk formatting

### Backward Compatibility

The browser-facing `/chat` route can remain for now, but should become a thin compatibility shim:
- translate `{message, image, image_mime_type}` into OpenAI-style `messages`
- call the same internal implementation as `/v1/chat/completions`

That keeps the web UI working while reducing divergence.

## Implementation Steps

1. Add an OpenAI request parser in `app.py`.
2. Add an internal message normalization layer.
3. Extend `vision_inference.py` to accept normalized `messages` directly.
4. Implement non-streaming `POST /v1/chat/completions`.
5. Implement OpenAI-style SSE streaming for `stream=true`.
6. Optionally keep `/chat` as a compatibility shim.
7. Optionally add `GET /v1/models`.
8. Test with a simple `curl` request from the command line.
9. Test with `shock_absorber` once `llm_api.rs` is wired into a call path.

## Acceptance Criteria

The task is complete when all of the following are true:
- a local `curl` request to `POST /v1/chat/completions` returns a valid OpenAI-style response
- `stream=true` returns OpenAI-style SSE chunks and ends with `[DONE]`
- text-only requests work
- multimodal requests with `image_url` data URLs work
- the server can still be used locally by `shock_absorber` without custom request translation

## Non-Goals

Not required for the first pass:
- full OpenAI API coverage beyond chat completions
- tool calling
- embeddings
- function calling
- authentication
- multi-model routing
- exact token accounting if the current runtime does not expose it cleanly

## Risks / Notes

- `llama-cpp-python` may not return token usage in the exact shape OpenAI clients expect
- multimodal message validation must be strict enough to reject malformed `image_url` inputs
- OpenAI-compatible streaming is easy to get almost right and still break clients if chunk framing is off
- if `shock_absorber` stays text-only at first, multimodal support can still remain in `gemma-4` without being exercised immediately

## Suggested Follow-On

After `gemma-4` exposes `POST /v1/chat/completions`, wire a single end-to-end harness test path in this repo:
- build a small `LLMContext`
- serialize it into messages
- send it through `src/harness/llm_api.rs`
- confirm the local backend can answer through the standardized interface
