[execute_public_summary]
status: start
query: summarize the last 100 posts by rei-cast.xyz
actor_anchor_did: did:plc:frudpt5kpurby7s7qdaz7zyw
actor_anchor_source: explicit_query_ref

[execute_public_summary]
status: actor_resolved
actor_handle: rei-cast.xyz
actor_did: did:plc:frudpt5kpurby7s7qdaz7zyw

[execute_public_summary]
status: hydrate_start
actor_did: did:plc:frudpt5kpurby7s7qdaz7zyw
hydrate_args: {
  "include_pinned_posts": true,
  "include_profile": true,
  "include_recent_posts": true,
  "recent_posts_feed_fetch_limit": 200,
  "recent_posts_min_top_level_posts": 100
}

[execute_public_summary]
status: hydrate_complete
actor_did: did:plc:frudpt5kpurby7s7qdaz7zyw
collection_count: 7
collections:
actor_profile:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=actor_profile | posts=1
clearsky_lists:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=clearsky_lists | posts=30
pinned_posts:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=pinned_posts | posts=1
recent_posts:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=recent_posts | posts=200
recent_posts_unaddressed:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=recent_posts_unaddressed | posts=14
recent_replies_received:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=recent_replies_received | posts=13
recent_replies_sent:did:plc:frudpt5kpurby7s7qdaz7zyw | kind=recent_replies_sent | posts=186

[execute_public_summary]
status: collection_selected
collection_id: recent_posts:did:plc:frudpt5kpurby7s7qdaz7zyw
collection_label: Recent posts by did:plc:frudpt5kpurby7s7qdaz7zyw
collection_kind: recent_posts
post_count: 200
requested_scope: Count { requested_items: 100 }

[collection_summary_loop]
node: init_window
collection_id: recent_posts:did:plc:frudpt5kpurby7s7qdaz7zyw
collection_posts: 200
initial_offset: 0
max_pages: 4
requested_scope: Count { requested_items: 100 }

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:frudpt5kpurby7s7qdaz7zyw
page_index: 0
offset: 0
window_size: 25

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:frudpt5kpurby7s7qdaz7zyw
offset: 0
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 25 item(s), but 100 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 0 (page 1 of at most 4)

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:frudpt5kpurby7s7qdaz7zyw
page_index: 1
offset: 25
window_size: 25

