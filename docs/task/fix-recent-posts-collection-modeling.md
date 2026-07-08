# Task: Fix `recent_posts_unaddressed` Collection Modeling

## Goal

Make actor-scoped recent-post collections actually represent posts authored by the target actor.

This task is separate from planner changes and separate from any new coverage-oriented summary worker.

The collection itself must be trustworthy first.

## Why This Needs To Exist

The current `recent_posts_unaddressed` collection is labeled like:

- recent top-level posts by actor X

But recent debug output shows entries authored by other accounts appearing inside that collection.

That breaks the meaning of downstream queries like:

- analyze the last 50 posts by this actor
- what does this actor post about
- summarize their recent top-level posts

If the source collection is wrong, then:

- `collection_search` is wrong
- any future `collection_window_summary` is wrong
- root answers can look grounded while still analyzing the wrong author's posts

## Observed Failure Mode

In the latest debug run:

- the query asked for the last 50 posts by one DID
- the searched collection was `recent_posts_unaddressed:<did>`
- that collection contained only 7 items
- several of those items were authored by other accounts

This strongly suggests that the current author-feed ingestion path is treating feed entries as if every `item.post` were authored by the requested actor.

That assumption is unsafe for repost-heavy feeds and similar author-feed structures.

## Current Likely Source Of The Bug

The current fetch path:

1. calls `app.bsky.feed.getAuthorFeed`
2. maps each feed entry to `item.post`
3. partitions those posts into:
   - reply records
   - non-reply records
4. stores the non-reply records as `recent_posts_unaddressed`

What appears to be missing is a strict author check.

The collection should not trust:

- feed inclusion alone

It should require:

- post author DID equals the requested actor DID

before the item is considered part of an actor-authored collection.

## Required Behavioral Clarification

We need to be explicit about what each collection means.

### `recent_posts_unaddressed`

Should mean:

- posts authored by the actor
- top-level only
- not replies

It should not mean:

- feed items shown on the actor's author feed
- reposted third-party posts
- quoted or otherwise associated third-party posts unless the actor authored the post record itself

### `recent_replies_sent`

Should mean:

- posts authored by the actor
- reply records

It should not include:

- replies authored by other people that merely appear in feed context

## Scope Of This Task

This task should:

- audit the exact structure returned by `getAuthorFeed`
- distinguish original authored posts from repost/feed-wrapper entries
- filter actor-scoped collections to records actually authored by the target DID
- preserve the reply vs top-level split only after author filtering

This task should also verify naming clarity:

- if the collection is truly actor-authored top-level posts, the current label is acceptable
- if not, the collection kind and label may need to change

## Non-Goals

This task is not primarily about:

- adding exhaustive 50-item analysis
- changing the root rerun guard
- replacing `collection_search`

Those may depend on this fix, but they are not the same problem.

## Recommended Debugging Steps

1. Inspect raw `getAuthorFeed` entry shapes for repost cases.
2. Confirm which field identifies the feed actor versus the embedded post author.
3. Add or tighten filtering so only posts authored by the requested DID enter actor-authored collections.
4. Re-run the failing case and inspect `.debug` artifacts.
5. Confirm that the collection no longer contains third-party authored items.

## Acceptance Criteria

This task is complete when:

- `recent_posts_unaddressed:<did>` contains only top-level posts authored by that exact DID
- `recent_replies_sent:<did>` contains only replies authored by that exact DID
- reposted or otherwise surfaced third-party posts are excluded from those actor-authored collections
- debug artifacts for a known repost-heavy actor no longer show foreign authors inside those collections
- downstream summaries over those collections are analyzing the correct actor's authored content
