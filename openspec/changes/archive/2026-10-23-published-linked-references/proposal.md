## Why

Published pages do not show which other pages refer to them. Readers need bottom-of-page linked references with visible linking-page counts and related-page filters, including when a long reference list starts collapsed.

## What Changes

- Index authored references only after publication selection and privacy checks.
- Group referring context by source page, with separate distinct-page and reference counts, including zero.
- Provide related-page include/exclude filters, removal, reset, and accessible responsive controls.
- Read linked-reference collapse threshold and open-block depth from `logseq/config.edn`.
- Preserve ordinary source rendering, hierarchy, authored collapse, and fragment navigation.

## Capabilities

### New Capabilities

- `published-linked-references`: Safe backlink indexing, context rendering, visible counts, filters, and graph-configured initial disclosure state.

### Modified Capabilities

None.

## Impact

The SDK gains a selected-source reference index and bounded config reader. The site renderer adds a bottom section and a trusted filter script; host pages can initialize replaced content through a small browser API. Rust and JavaScript fixtures cover identity, privacy, counts, filters, and disclosure behavior. No new dependency is planned.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing.md)
- [Logseq 0.10.6 references](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/components/reference.cljs)
- [Logseq filter predicates](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/handler/block.cljs#L240-L283)
