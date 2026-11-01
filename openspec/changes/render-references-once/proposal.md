## Why

Publishing the encode garden spends about 45% of its render phase drawing Linked References ([Issue #290 · Render each Linked References context once](https://github.com/codekiln/logseq-gardener/issues/290)). The publisher copies each referring block and renders the copy again for every page the block refers to, so a block that links to five pages renders five copies that differ only in a few attributes.

## What Changes

- The publisher renders each referring block's copy once per build and reuses it on every page that lists the block: in Linked References, Unlinked References and query results.
- The parts of a copy that depend on the listing page, each bullet's row number in that page's reference data and each bullet's link to its block, are filled in for each page.
- Each referring block's breadcrumb also renders once per build.
- The published files stay byte for byte the same. A whole encode garden build before and after the change is compared to show it.
- New rows in `docs/parity/publication-speed.json` record the result, and `docs/parity/publication-speed.md` reports it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-linked-references`: add a requirement that a block listed on several pages renders its copy once per build, with output identical to rendering it for each page.

## Impact

Only `crates/logseq-gardener-sdk/src/site/` changes: `render.rs` returns a rendered copy with gaps that the listing page fills, and `references.rs` keeps each block's copy and breadcrumb for the rest of the build. The SDK's public API, the CLI and the published files are unchanged. No dependency changes.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
