[execute_public_summary]
status: start
query: summarize the most recent 400 posts by this actor into 4 paragraphs
actor_anchor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
actor_anchor_source: selected_actor_fallback

[execute_public_summary]
status: actor_resolved
actor_handle: schizanon.bsky.social
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5

[execute_public_summary]
status: hydrate_start
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
hydrate_args: {
  "include_pinned_posts": true,
  "include_profile": true,
  "include_recent_posts": true,
  "recent_posts_feed_fetch_limit": 400,
  "recent_posts_min_top_level_posts": 400
}

[execute_public_summary]
status: hydrate_complete
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
collection_count: 7
collections:
actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=actor_profile | posts=1
clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=clearsky_lists | posts=100
pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=pinned_posts | posts=1
recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts | posts=400
recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts_unaddressed | posts=88
recent_replies_received:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_received | posts=100
recent_replies_sent:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_sent | posts=215

[execute_public_summary]
status: collection_selected
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5
collection_kind: recent_posts
post_count: 400
requested_scope: Count { requested_items: 400 }

[collection_summary_loop]
node: init_window
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_posts: 400
initial_offset: 0
max_pages: 8
requested_scope: Count { requested_items: 400 }

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 0
offset: 0
window_size: 50

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 0
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 50 item(s), but 400 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 0 (page 1 of at most 8)

[execute_public_summary]
status: loop_finished
outcome_count: 0
rendered:
status: failed
reason: no summary pages were processed

[execute_public_summary]
status: start
query: summarize the most recent 400 posts by did:plc:6lwfvmss45d7j7fot34v2kw5 into 4 paragraphs
actor_anchor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
actor_anchor_source: explicit_query_ref

[execute_public_summary]
status: actor_resolved
actor_handle: schizanon.bsky.social
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5

[execute_public_summary]
status: hydrate_start
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
hydrate_args: {
  "include_pinned_posts": true,
  "include_profile": true,
  "include_recent_posts": true,
  "recent_posts_feed_fetch_limit": 400,
  "recent_posts_min_top_level_posts": 400
}

[execute_public_summary]
status: hydrate_complete
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
collection_count: 7
collections:
actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=actor_profile | posts=1
clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=clearsky_lists | posts=100
pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=pinned_posts | posts=1
recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts | posts=400
recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts_unaddressed | posts=88
recent_replies_received:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_received | posts=100
recent_replies_sent:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_sent | posts=215

[execute_public_summary]
status: collection_selected
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5
collection_kind: recent_posts
post_count: 400
requested_scope: Count { requested_items: 400 }

[collection_summary_loop]
node: init_window
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_posts: 400
initial_offset: 0
max_pages: 8
requested_scope: Count { requested_items: 400 }

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 0
offset: 0
window_size: 50

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 0
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 50 item(s), but 400 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 0 (page 1 of at most 8)

[execute_public_summary]
status: loop_finished
outcome_count: 0
rendered:
status: failed
reason: no summary pages were processed

[execute_public_summary]
status: start
query: summarize the last 400 posts by schizanon.bsky.social into 4 paragraphs
actor_anchor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
actor_anchor_source: explicit_query_ref

[execute_public_summary]
status: actor_resolved
actor_handle: schizanon.bsky.social
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5

[execute_public_summary]
status: hydrate_start
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
hydrate_args: {
  "include_pinned_posts": true,
  "include_profile": true,
  "include_recent_posts": true,
  "recent_posts_feed_fetch_limit": 400,
  "recent_posts_min_top_level_posts": 400
}

[execute_public_summary]
status: hydrate_complete
actor_did: did:plc:6lwfvmss45d7j7fot34v2kw5
collection_count: 7
collections:
actor_profile:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=actor_profile | posts=1
clearsky_lists:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=clearsky_lists | posts=100
pinned_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=pinned_posts | posts=1
recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts | posts=400
recent_posts_unaddressed:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_posts_unaddressed | posts=88
recent_replies_received:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_received | posts=100
recent_replies_sent:did:plc:6lwfvmss45d7j7fot34v2kw5 | kind=recent_replies_sent | posts=215

[execute_public_summary]
status: collection_selected
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_label: Recent posts by did:plc:6lwfvmss45d7j7fot34v2kw5
collection_kind: recent_posts
post_count: 400
requested_scope: Count { requested_items: 400 }

[collection_summary_loop]
node: init_window
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
collection_posts: 400
initial_offset: 0
max_pages: 8
requested_scope: Count { requested_items: 400 }

[collection_summary_loop]
node: summarize_page
status: running
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
page_index: 0
offset: 0
window_size: 50

[collection_summary_loop]
node: summarize_page
status: page_outcome
collection_id: recent_posts:did:plc:6lwfvmss45d7j7fot34v2kw5
offset: 0
result_present: false
review_status: fail
review_reason: Grounded summary coverage currently reaches 50 item(s), but 400 item(s) are required before parent synthesis is sufficient.
diagnostic: summary cursor processed offset 0 (page 1 of at most 8)

[execute_public_summary]
status: loop_finished
outcome_count: 0
rendered:
status: failed
reason: no summary pages were processed

