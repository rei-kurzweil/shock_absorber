# Storage Current State

This document describes how `shock_absorber` stores runtime state today, both in memory and on disk.

## Summary

The app currently has two storage layers:

- an in-memory runtime store: `NotificationStore` in `src/net_backend.rs`
- a SQLite snapshot store: `AppDb` in `src/db.rs`

The runtime store is still the source of truth while the app is running.
The SQLite layer currently persists only part of that state so it can be restored on startup.

## In-Memory State

The main in-memory state lives in `NotificationStore` in `src/net_backend.rs`.

Current fields:

- `notifications: Vec<Notification>`
  - loaded from Bluesky polling
  - used to render the left-hand notification list and selected/opened notification views
  - not currently persisted to disk

- `notification_keys: HashSet<String>`
  - de-duplication key for notifications already seen in this process
  - not currently persisted

- `reply_texts: HashMap<String, String>`
  - cached reply text keyed by notification URI
  - not currently persisted

- `bios: HashMap<String, Option<String>>`
  - actor DID -> bio
  - persisted to disk through the `actors` table

- `post_collections: HashMap<String, LabeledPostCollection>`
  - generic searchable cached collections
  - currently includes:
    - `recent_posts:<did>`
    - `pinned_posts:<did>`
    - `clearsky_lists:<did>`
  - persisted to disk through the `post_collections` table

- `pinned_post_replies: HashMap<String, Vec<CachedThreadReply>>`
  - reply trees for pinned posts
  - used for detail rendering
  - not currently persisted

- `clearsky_lists: HashMap<String, Vec<ModerationListEntry>>`
  - actor DID -> raw Clearsky moderation list entries
  - persisted separately to disk through the `clearsky_lists` table

- `did_to_handle: HashMap<String, String>`
  - DID -> handle
  - persisted indirectly through the `actors` table

- `handle_to_did: HashMap<String, String>`
  - handle -> DID
  - rebuilt in memory from restored actor rows on startup

- `clearsky_client: reqwest::Client`
  - network client only
  - not persisted

## Collection Model

Searchable collections use `LabeledPostCollection` in `src/model/mod.rs`.

Shape:

- `id`
- `label`
- `posts: Vec<PostRecord>`
- `related_collection_ids`
- `metadata`

Despite the name, this is now used for more than literal posts:

- Bluesky recent posts are stored as `PostRecord`
- Bluesky pinned posts are stored as `PostRecord`
- Clearsky moderation list entries are mapped into synthetic `PostRecord`s so the same search path can inspect them

That means the current collection abstraction is already acting as a generic searchable record bag, even though the type names still say `Post`.

## How Collections Enter Memory

Collections are populated from a few paths in `src/net_backend.rs`:

- `poll_notifications(...)`
  - fetches notifications
  - for each seen actor DID:
    - `ensure_recent_posts_cached(...)`
    - `ensure_pinned_posts_cached(...)`
    - `ensure_clearsky_lists_cached(...)`

- `pins <actor>`
  - also forces:
    - recent posts cache
    - pinned posts cache
    - Clearsky lists cache

Collection builders:

- `build_recent_posts_collection(...)`
- `build_pinned_posts_collection(...)`
- `build_clearsky_lists_collection(...)`

## On-Disk State

The SQLite persistence layer lives in `src/db.rs`.

Default database path:

- `shock_absorber.sqlite3` in the repo root

Override:

- `SHOCK_ABSORBER_DB_PATH`

Schema currently created by `AppDb::init()`:

- `app_meta`
  - currently only stores `schema_version`

- `actors`
  - `did`
  - `handle`
  - `bio`

- `post_collections`
  - `id`
  - `payload_json`
  - `updated_at_unix`

- `clearsky_lists`
  - `actor_did`
  - `payload_json`
  - `updated_at_unix`

- `sessions`
  - `id`
  - `label`
  - `created_at_unix`
  - `last_selected_notification_uri`
  - `last_opened_notification_uri`
  - currently unused placeholder only

## What Is Persisted Today

Persisted:

- actor DID/handle/bio rows
- all cached `LabeledPostCollection`s as JSON blobs
- raw Clearsky list payloads as JSON blobs

Not persisted:

- notifications
- unread/recent notification session state
- selected notification index
- opened notification index
- reply text cache
- pinned post reply trees
- search result sets
- user-created collections
- tool transcripts

## Load and Save Lifecycle

Startup flow in `src/main.rs`:

1. open `AppDb`
2. call `restore_store_from_db(...)`
3. restore:
   - actor cache
   - cached post collections
   - raw Clearsky lists

Runtime save flow:

- after a poll that loads new notifications, `db.save_store(&app.store)` runs
- after a successful command execution, `db.save_store(&app.store)` runs

`save_store(...)` is currently a full snapshot rewrite for persisted tables:

- `DELETE FROM actors`
- `DELETE FROM post_collections`
- `DELETE FROM clearsky_lists`
- re-insert everything from the current in-memory store

So persistence is currently snapshot-based, not incremental.

## Current Mismatches and Gaps

### 1. Notifications are memory-only

The UI is centered on notifications, but notifications are not persisted at all.
After restart, the searchable actor caches may come back, but the notification list itself does not.

### 2. Sessions table exists but is unused

There is a `sessions` table, but the app does not write or restore any session state yet.
So there is no real resume concept today.

### 3. Some related caches are only partially persisted

`post_collections` are persisted, but `pinned_post_replies` are not.
So a restored pinned-post collection may exist without the associated cached reply tree.

### 4. Clearsky is duplicated in two persisted forms

Clearsky data is stored both as:

- raw entries in `clearsky_lists`
- synthetic searchable records inside `post_collections`

That duplication is convenient right now, but it creates two persisted representations of the same source data.

### 5. User-defined collections do not exist yet

There is no separate persistent table or in-memory type yet for:

- curated/labeled user collections
- saved search result sets
- adding selected search results into named collections

## Practical Current State

Today the storage model is best described as:

- runtime app state lives in memory
- a subset of caches is snapshotted into SQLite
- restored SQLite data is used to warm the in-memory cache at startup

This is not yet a full persistent app model.
It is a cache persistence layer with a placeholder for future session persistence.

## Recommended Next Steps

If the goal is resumable workflows and curated collections, the next pieces should be:

1. persist notifications and a session concept
2. add first-class persistent user collections
3. add persistent search result sets with stable result/item IDs
4. decide whether `LabeledPostCollection` should remain the universal collection type or be generalized into a broader record/item collection type
5. decide whether source caches and curated collections should live in separate tables instead of both being flattened into `post_collections`
