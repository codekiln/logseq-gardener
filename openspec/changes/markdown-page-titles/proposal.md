## Why

Rust publishers need page titles to select garden namespaces and assign pages to output URLs. The Markdown loader retains filenames and syntax; this change supplies the title that Logseq derives from those inputs for [#16 — Markdown page titles](https://github.com/codekiln/logseq-gardener/issues/16).

## What Changes

- Derive a page title through the SDK with an explicit legacy or triple-lowbar filename format.
- Follow leading page-property title overrides, filename decoding, and the Contents special case from the Logseq OG source covered by the saved relationship experiment.
- Return explicit errors for journals, invalid page paths, and empty titles.
- Test recorded fixture titles and decoding edge cases, and demonstrate the API on the public garden.

## Capabilities

### New Capabilities

- `markdown-page-titles`: Derive display titles for loaded Markdown page documents.

### Modified Capabilities

None.

## Impact

Adds an SDK module and example on top of [PR #15 — Markdown loader](https://github.com/codekiln/logseq-gardener/pull/15). The change preserves capitalization and supplies titles, while [#4 — Page lookup](https://github.com/codekiln/logseq-gardener/issues/4) continues with canonical keys, aliases, journal identities, and configuration loading. [#11 — Static publishing](https://github.com/codekiln/logseq-gardener/issues/11) remains the product priority.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md): verify title extraction before adding lookup and rendering.
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md): require an explicit filename format and distinguish titles from lookup keys.
