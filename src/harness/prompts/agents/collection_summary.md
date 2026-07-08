Inspect the provided collection window carefully.

Return exactly one JSON object with this shape:

```json
{
  "title": "short title",
  "summary": "grounded coverage-oriented paragraph",
  "covered_item_uris": ["real item uri 1", "real item uri 2"],
  "omitted_item_uris": ["real item uri 3"],
  "window_start": 0,
  "window_total_items": 25
}
```

- `title` is optional but preferred.
- `summary` is required.
- `covered_item_uris` is required and must list only real item URIs from this exact collection window.
- `omitted_item_uris` is required and must list only real item URIs from this exact collection window.
- `window_start` is required and must match the provided window start.
- `window_total_items` is required and must match the number of items in the provided window.
- Do not return markdown, code fences, or any text outside the JSON object.

This tool is coverage-oriented, not relevance-ranked.

Your summary must account for the whole requested window rather than silently picking only the strongest few items.

Rules:

- Every item in the window must appear in exactly one of `covered_item_uris` or `omitted_item_uris`.
- Keep `omitted_item_uris` empty unless you have a concrete grounded reason not to discuss those items directly.
- The `summary` field must be one grounded paragraph of roughly 120-220 words.
- Group recurring themes, contrasts, topic shifts, and unusual outliers that are directly supported by the collection text.
- Quote exact short snippets, list names, list descriptions, or other text taken from the collection when that helps ground the grouping.
- For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.
- Do not mention `item[...]`, `matched_item[...]`, or raw metadata labels inside the `summary`.
- Do not answer the user's overall question; just return a grounded summary of this collection window.
