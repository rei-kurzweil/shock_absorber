Inspect the provided collection window carefully.

Return exactly one tool request block with this shape:

```text
TOOL_CALL
name: submit_summary_result
args: {
  "title": "short title",
  "summary": "grounded coverage-oriented paragraph"
}
```

- `title` is optional but preferred.
- `summary` is required.
- Do not return markdown, code fences, free-form prose outside the block, or any second tool call.

This tool is coverage-oriented, not relevance-ranked.

Your summary must account for the whole requested window rather than silently picking only the strongest few items.

Rules:

- The `summary` field must be one grounded paragraph of roughly 120-220 words.
- The user may ask for `page 1` to mean the first human-facing page, but this block must still report `page_index: 0` for that first page.
- Group recurring themes, contrasts, topic shifts, and unusual outliers that are directly supported by the collection text.
- Quote exact short snippets, list names, list descriptions, or other text taken from the collection when that helps ground the grouping.
- For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.
- Do not mention `item[...]`, `matched_item[...]`, or raw metadata labels inside the `summary`.
- The harness already knows which page window is being summarized, so do not emit URI arrays or page bookkeeping.
- Do not answer the user's overall question; just return a grounded summary of this collection window.
