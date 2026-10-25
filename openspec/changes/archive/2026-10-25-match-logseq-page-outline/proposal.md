## Why

Logseq's published site reads as an outline: a light 36px title, small round bullets with a guide line down each level, headings that keep their bullets, task checkboxes, and bullets that open one block on its own with a breadcrumb back to the page. `lsg` publishes the same blocks as plain HTML lists with the browser's default markers and headings, task markers as plain words, and no way to collapse an ordinary block or zoom into one ([page outline analysis](../../../docs/parity/page-outline.md), [Issue #111 — Match Logseq's page title, outline, headings, collapse and block zoom](https://github.com/codekiln/logseq-gardener/issues/111)).

## What Changes

- The SDK gains a public outline model for a page: each block's depth, parent, task marker, priority, heading level (from `#` or `heading:: true`), authored collapsed state and a block identifier that stays the same across builds of unchanged content (the block's `id::` UUID, otherwise its position among its siblings). It also returns any block's ancestors and finds a block by identifier. The publisher reads collapse, headings and markers from it instead of working them out while rendering.
- Page and journal-feed outlines get Logseq's look through CSS on the existing markup: the title at 36px weight 300, 6px bullets in a 14px column at every depth, a 2px guide line on each level of children, 2px block padding and a 1.6 line height, and Logseq's heading sizes, weights and h1/h2 rules. The empty paragraph after page properties goes away.
- An authored collapsed block shows its bullet on a filled disc instead of the browser's disclosure triangle, and still opens without JavaScript.
- A task block renders a read-only checkbox and Logseq's marker label; DONE is checked and struck through, CANCELED is struck through without a checkbox, and `[#A]` keeps its brackets.
- A new deferred script, `lsg-outline.js`, loaded after the page's `load` event, adds the hover arrow and lets readers collapse and expand any block with children, and opens a zoomed view when a reader clicks a bullet: a breadcrumb (page, then up to three ancestors, with `⋯` when more are hidden), the block expanded as the root, and nothing else. Zoom works on the page already loaded, so it adds no files and no graph data. The address is `<page>#zoom/<identifier>`, and the browser's Back button returns to the page.
- New budgets: the script's size, the outline markup's share of the largest fixture page, and the time to open a zoomed view on that page, all checked or recorded by the comparison harness.

## Capabilities

### New Capabilities

- `outline-blocks`: the SDK operations that describe a page's blocks (depth, parent, marker, priority, heading level, collapsed state, stable identifier) and answer ancestor and identifier lookups.
- `published-page-outline`: the published title, bullets, guide lines, headings, task markers, reader collapse, block zoom and their performance budgets.

### Modified Capabilities

- `authored-outline-collapse`: an authored collapsed block shows Logseq's closed bullet instead of the browser's disclosure triangle.

## Impact

- SDK: a new `outline` module; `site/render.rs` takes collapse state, heading level, marker and priority from it.
- Publisher: the page stylesheet in `site/mod.rs`, bullet and task markup in `site/render.rs`, a new `site/outline.js` written once as `lsg-outline.js` and loaded by `lsg-shell.js` after `load`; `fragments.js` opens folded blocks around a linked block as it opens authored disclosures.
- Parity harness: `scripts/parity/budgets.toml` gains an `[outline]` table; `parity:budgets` checks the script size and the `tmux` page size; `parity:perf` adds a zoom row.
- Gitpa replaces `<main>` content after navigation; the outline script uses event delegation so swapped content needs no setup.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
