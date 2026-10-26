## Why

Three encode-garden pages are on Logseq's published site and missing from `lsg`'s. Their file names encode `/` as `%2F`, so their titles contain ` / `, as in `Obsidian/Q/… colored Info / Warning / Danger blocks called?`. Splitting such a title at `/` gives segments that begin or end with a space. The SDK's namespace selection treats a page name with such a segment as malformed and leaves the page out, while Logseq keeps the segment as written ([Issue #129 — Publish pages whose file names encode a slash as %2F](https://github.com/codekiln/logseq-gardener/issues/129), part of [Issue #106 — Match Logseq's published site](https://github.com/codekiln/logseq-gardener/issues/106)).

## What Changes

- `NamespaceSelection::includes` accepts a page name whose segments begin or end with spaces, as Logseq's [`split-namespace-pages`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/util.cljs#L116-L125) does. A page name with an empty or whitespace-only segment, or a control character, is still unselected.
- Selector validation is unchanged: an include or exclude root with surrounding segment whitespace is still a configuration error.
- The parity harness reads `lsg` page titles exactly as written and applies the same page-name rule, so the comparison matches Logseq's names, including the generated namespace parents that end in a space.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `publishing-namespace-selection`: page names may keep surrounding segment whitespace; selectors may not.

## Impact

- SDK: `crates/logseq-gardener-sdk/src/publishing.rs`. Routes are hashes of the title, so no file name changes.
- Parity harness: `scripts/parity/common.py` and `scripts/parity/compare.py`.
- Performance: no change; the check is a scan of the name either way.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
