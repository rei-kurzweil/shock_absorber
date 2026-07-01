# AI Harness Draft

Goal: add a read-only AI harness that can inspect the notification graph, search cached actor content, and assemble a bounded context window for an LLM.

## Scope

Near-term target modules:
- `src/harness/context_window.rs`
- `src/harness/tools.rs`
- `src/harness/llm_api.rs`
- `src/net_backend.rs` extensions for recent-post fetch and cache

Non-goal for now:
- writing back to Bluesky
- committing to one prompt format forever
- building a durable database before the in-memory shapes settle

## Working Model

The root graph is still notification-driven:
- notifications identify which actors matter right now
- each actor DID becomes an anchor node
- attached data is fetched on demand and cached in memory

Current attached cache:
- notifications
- reply text and reply thread fragments
- bios
- pinned posts plus thread replies
- Clearsky moderation-list memberships

Planned attached cache:
- recent posts by actor, capped to the latest 20 posts

## `net_backend` additions

Needed backend behavior:
- fetch recent `n` posts by actor through `app.bsky.feed.getAuthorFeed`
- cache the latest 20 posts per DID in `NotificationStore`
- expose read-only accessors so harness tools can search them without touching network code

Important constraint:
- the harness should search cached collections first
- network fetch should be an explicit cache-fill step, not something hidden inside every tool call

## Context Window Design

`context_window.rs` should define a reusable `LLMContext`:
- a context has an instruction header and ordered sections
- multiple contexts can exist at once
- child or scratch contexts are cheap to create for search/planning substeps

Target flow:
1. parent context describes the overall task
2. a search tool spins up a fresh child context with a narrow instruction header
3. that child context searches a collection like recent posts
4. only useful search results are serialized back into the parent context

Context budgeting rules:
- each provider advertises max context size
- reserve output tokens before building the prompt
- trim sections to fit instead of failing late
- do only approximate token budgeting at first; exact tokenizer integration can wait

Likely section types:
- actor summary
- notification summary
- recent-post search results
- pinned-post excerpts
- moderation-list facts
- explicit task/instruction section

## Tools Design

`tools.rs` should hold read-only harness tools the Bluesky agent can call.

First useful tool:
- search cached recent posts for an actor

Expected behavior:
- input: actor DID, query, result limit
- search space: cached recent posts only
- rank matches with a simple lexical heuristic first
- return compact excerpts and URIs, not whole posts by default
- load only selected or top-ranked results back into the active context

Likely next tools:
- search pinned-post threads
- summarize an actor anchor from cached facts
- explain why a notification appears relevant

## LLM API Design

`llm_api.rs` should always talk to providers over HTTP/REST using an OpenAI-style API surface.

That means:
- target request format should be `POST /v1/chat/completions`
- message payloads should use OpenAI-style `role` plus `content`
- streaming, when added, should follow OpenAI-style chunk semantics closely enough that provider swaps are cheap

First provider target:
- local `llama.cpp` over REST
- use the OpenAI-compatible `/v1/chat/completions` path
- model local testing constraints explicitly
- assume a small context window, around 8k, for the first provider profile

The provider abstraction should carry:
- provider name
- model name
- max context tokens
- reserved output tokens

That lets `context_window.rs` stay provider-aware without knowing anything about transport.

## Open Questions

- whether recent-post cache freshness should be time-based, count-based, or both
- whether search result ranking should stay heuristic or use an LLM reranker later
- whether parent/child contexts should share a common fact registry
- whether prompt sections should be markdown, XML-ish tags, or plain text blocks
- where human review fits before any future action-taking agent layer
