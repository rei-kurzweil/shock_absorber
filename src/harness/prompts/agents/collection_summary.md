Inspect the provided collection window carefully.

Return exactly one flat tagged result block with this shape:

```text
SUMMARY_RESULT_START
title: short title
summary: grounded coverage-oriented paragraph
covered_item_uri: real item uri 1
covered_item_uri: real item uri 2
omitted_item_uri: real item uri 3
window_offset: 0
window_size: 25
page_index: 0
page_size: 25
collection_total_items: 73
has_more: true
SUMMARY_RESULT_END
```

- `title` is optional but preferred.
- `summary` is required.
- Repeat `covered_item_uri:` once per covered item from this exact collection window.
- Repeat `omitted_item_uri:` once per omitted item from this exact collection window.
- `window_offset` is required and must match the provided window start.
- `window_size` is required and must match the number of items in the provided window.
- `page_index` is required and is zero-based.
- `page_size` is required and must match the provided collection page size.
- `collection_total_items` is required and must report the full available collection size, not just this page.
- `has_more` is required and must match whether more items exist after this window.
- Do not return markdown, code fences, YAML, JSON, or any text outside the tagged block.

This tool is coverage-oriented, not relevance-ranked.

Your summary must account for the whole requested window rather than silently picking only the strongest few items.

Rules:

- Every item in the window must appear in exactly one of `covered_item_uris` or `omitted_item_uris`.
- Keep `omitted_item_uris` empty unless you have a concrete grounded reason not to discuss those items directly.
- The `summary` field must be one grounded paragraph of roughly 120-220 words.
- The user may ask for `page 1` to mean the first human-facing page, but this block must still report `page_index: 0` for that first page.
- Group recurring themes, contrasts, topic shifts, and unusual outliers that are directly supported by the collection text.
- Quote exact short snippets, list names, list descriptions, or other text taken from the collection when that helps ground the grouping.
- For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.
- Do not mention `item[...]`, `matched_item[...]`, or raw metadata labels inside the `summary`.
- Do not answer the user's overall question; just return a grounded summary of this collection window.
