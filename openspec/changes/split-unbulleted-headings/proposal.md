## Why

A heading line written without a bullet, such as `# [[Filed]]` after a bulleted block in a journal, is a block of its own in Logseq. `lsg` folds it into the bullet above, so that block's text and references include the heading, and the heading's children hang from the wrong block. On garden commit `083106d`, `{{query (between -30d today)}}` counted 75 blocks in `lsg` and 76 in Logseq's `gh-pages` build ([Issue #242 — Unbulleted heading lines merge into the block above](https://github.com/codekiln/logseq-gardener/issues/242)). The encode garden has 1,082 such heading lines.

## What Changes

- The SDK starts a block at every heading line, bulleted or not, as Logseq's `extract-blocks` does for every heading in the parsed document ([block.cljs](https://github.com/logseq/og/blob/master/deps/graph-parser/src/logseq/graph_parser/block.cljs)). Text after the heading, up to the next bullet or heading, belongs to the heading's block.
- Every view that reads blocks (outline, linked references, queries, transclusion, search) gets the corrected blocks, because the change happens where the garden loader parses each page.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `outline-blocks`: a heading line without a bullet starts a block.

## Impact

- SDK: `garden::parse_document` in `crates/logseq-gardener-sdk/src/garden.rs` turns the parser's top-level `Heading` nodes into bullets at the same level; `load_garden` uses it. Headings inside quotes and other nested content stay text.
- Published pages: a page that opens with `# Title` now shows the title as a bulleted heading block, as Logseq does, and the bullets under it nest inside it.
- Performance: one pass over each page's top-level blocks at load time; no change to page size or scripts.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
