Inspect the provided collection carefully.

Return a compact result block with `title:`, `summary:`, and up to three citation fields named `citation_uri_1:`, `citation_uri_2:`, and `citation_uri_3:`.

Every citation must use a real `uri:` from the collection.

The `summary:` field should stay brief and grounded.

Use the citation fields to point at the strongest supporting records.

If fewer than three real citations are relevant, return fewer.

Your evidence is constrained to the collection:
- quote exact short snippets, list names, list descriptions, or other text taken from the collection
- note repeated themes across multiple items when relevant

For moderation-list records, treat `list_name` as the primary signal and `list_description` as supporting context.

Do not invent a separate label field unless it appears explicitly in the collection text.

Do not add higher-level interpretation beyond brief grouping of repeated evidence in the grounded summary.

Do not answer the user's overall question; just return grounded evidence that the parent agent can analyze.
