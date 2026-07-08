# Coverage-Oriented Collection Summary Agent Draft

Goal: define a second internal `llm_search` worker that is optimized for exhaustive window coverage rather than relevance-ranked evidence picking.

This draft is intentionally pre-implementation.
It describes a tool/agent shape for queries like:

- analyze the last 50 posts by actor X
- summarize the first 25 replies in this collection
- compare page 0 and page 1 of this actor's recent posts

## Why `collection_search` Is The Wrong Primitive

The current `collection_search` worker is deliberately selective.

Its contract is roughly:

- inspect one 25-item collection window
- choose the strongest supporting records
- return a grounded paragraph plus a small set of cited URIs

That is good for:

- reputation questions
- finding strongest examples
- narrow evidence gathering

That is bad for:

- exhaustive review of N items
- proving how much of the requested scope was actually covered
- user requests that explicitly ask for 25, 50, or 100 recent authored items

The core mismatch is:

- `collection_search` is a ranking tool
- exhaustive post analysis needs a coverage tool

## Problem Statement

Today, a request like:

- analyze the last 50 posts by did:...

can fail in multiple ways:

- the planner may treat one grounded summary as complete
- the child worker may cite only 5-10 records from a larger window
- the verifier may accept "summary + some anchors" without checking requested coverage
- the root run may block a follow-up query like "next 50 posts" as duplicate scope

None of those are necessarily bugs within a ranking-oriented search tool.

They become bugs only because the wrong tool contract is being used for a coverage request.

## Proposed Internal Tool

Recommended internal tool name:

- `collection_window_summary`

Alternative names:

- `collection_coverage_review`
- `collection_window_audit`

`collection_window_summary` is the clearest because:

- it is tied to one explicit collection window
- it sounds different from `collection_search`
- it implies summary over a bounded slice, not a whole corpus

## Intended Contract

This tool should summarize one concrete window and explicitly account for all items in that window.

Unlike `collection_search`, it should not be free to silently pick a handful of "best" items and stop.

Minimum behavior:

- inspect one exact collection window
- account for every item in that window
- return a grounded summary over the whole window
- return explicit coverage metadata

## Proposed Arguments

- `collection_id`: string
- `prompt`: string
- `page`: integer, optional
- `offset`: integer, optional
- `limit`: integer, optional

Semantics:

- default window size should remain 25 unless overridden intentionally
- `offset` takes precedence over `page`
- `limit` should be capped by harness policy

The prompt should be allowed to say things like:

- analyze the tone and recurring topics
- summarize what this actor is posting about
- identify notable contrasts or abrupt topic shifts

But the tool should still be required to account for every item in the window.

## Proposed Result Shape

Suggested structured result:

```json
{
  "title": "short title",
  "summary": "grounded paragraph or short multi-paragraph summary",
  "window_start": 0,
  "window_size": 25,
  "window_total_items": 25,
  "covered_item_uris": ["at://..."],
  "omitted_item_uris": [],
  "coverage_notes": "all 25 items were grouped into recurring topics; no omissions",
  "theme_buckets": [
    {
      "label": "topic or pattern",
      "item_uris": ["at://..."]
    }
  ]
}
```

Not all fields need to be exposed in the first slice.

The most important fields are:

- `summary`
- `window_start`
- `window_size`
- `window_total_items`
- `covered_item_uris`
- `omitted_item_uris`

## Key Difference From `collection_search`

`collection_search` should remain allowed to say:

- these 6 records were the strongest evidence

`collection_window_summary` should be required to say:

- this summary covers items 0-24
- these URIs were covered directly
- these items were omitted or collapsed
- here is the grounded aggregate interpretation of the whole window

That difference should be visible in both the prompt and the verifier.

## Verification Loop

This tool needs a stricter verifier than the current collection-search review pass.

Current groundedness is not enough.

The verifier should reject outputs when:

- `covered_item_uris.len() + omitted_item_uris.len()` does not match the window item count
- listed URIs are not real items from the searched window
- the summary clearly discusses fewer items than the accounting claims
- the model returns only a small hand-picked subset without explicit omissions

The verifier should accept outputs when:

- every item in the window is explicitly covered
- or omissions are explicit, real, and justified

## Planner-Level Usage

This tool should be internal to `llm_search`, not a root-visible public tool at first.

Recommended planner behavior:

- user asks for reputation or strongest evidence -> use `collection_search`
- user asks for exact last 25/50/100 posts -> use `collection_window_summary`

For "last 50 posts" the planner should naturally decompose into:

1. summarize page 0
2. summarize page 1
3. synthesize across both summaries

That is better than asking the root agent to invent vague follow-up queries like:

- analyze the next 50 posts

## Interaction With Existing 25-Item Paging

This draft does not require changing the existing 25-item paging model.

It can reuse:

- `page`
- `offset`
- 25-item windows

The important change is the child contract, not the paging primitive.

## Important Non-Goal

This draft does not solve incorrect collection contents.

If `recent_posts_unaddressed` contains repost/feed items by other authors, then an exhaustive summary over that collection is still wrong.

So this draft should be implemented alongside a separate fix for actor-authored collection modeling.

## Acceptance Criteria

This design is ready for implementation when we agree that:

- `collection_search` remains relevance-oriented
- a separate internal worker is introduced for exhaustive window coverage
- the new worker emits structured coverage/accounting fields
- the harness verifier checks actual coverage counts
- `llm_search` can compose multiple windows for 50-item and 100-item requests
