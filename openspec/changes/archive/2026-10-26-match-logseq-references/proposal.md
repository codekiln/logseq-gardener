## Why

Below each page's outline, Logseq's published site shows Pages tagged with, Linked References, Hierarchy and Unlinked References. Its linked references count referring blocks, include references to the page's namespace children, put journals first, and filter through a funnel icon and a dialog of chips. `lsg` publishes linked references with a different count, order, look and filter, and has no Pages tagged with or Unlinked References section ([references analysis](../../../../docs/parity/references.md), [Issue #115 — Match Logseq's linked references, filters, unlinked references and tagged pages](https://github.com/codekiln/logseq-gardener/issues/115)).

Matching Logseq's counting makes the sections larger, and they are already 99% of the largest `lsg` pages. The section bodies therefore move out of the page and load when a reader opens them, so a page's first paint stays the same size however often other pages mention it.

## What Changes

- The SDK gains one reference index shared by the CLI, the language server and the publisher. It answers: the blocks that refer to a page, including references to its namespace children and aliases; each block's path references for filtering; related-page frequencies for a set of blocks; a page's starting filter from `filters::`; the pages tagged with a page; and the unlinked mentions of a page. The publisher's `Backlinks::new` moves behind these operations.
- A reference to `A/B` also counts as a reference to `A`, as Logseq's parser does. `mise` goes from 151 referring contexts to Logseq's 589 referring blocks.
- The Linked References header reads `N Linked References`, `1 Linked Reference` or `F of N Linked References`, counting blocks. A page with no linked references has no section.
- Sections appear in Logseq's order: Pages tagged with, Linked References, Hierarchy, Unlinked References.
- Groups put journals first, newest first, then other pages by title. Each group is a foldable shaded card with a plain page link, and breadcrumbs show at most three ancestors with their inline markup.
- The inline `Related pages` fieldset becomes Logseq's funnel icon and filter dialog: include and exclude rows, a search box, and chips sorted by frequency whose counts follow the active filter. Click includes, shift-click excludes, and the keyboard has the same choices.
- A page's `filters::` property sets its starting filter, so `mise` opens at `4 of 589 Linked References` with the green icon.
- A new collapsed Unlinked References section on every page lists blocks that mention the page name as plain text, found at build time and loaded when opened.
- A new Pages tagged with section lists the pages whose `tags::` name the page.
- Linked and unlinked reference bodies are written as separate fragment files and fetched when a section opens. Without scripts, the header links to the fragment.
- New budgets cap the references script and styles, the opened-section memory on a phone and the build-time cost.
- **BREAKING** for readers of `lsg` output only: the page count, the referring-context explanation and the zero-reference heading are gone, and the filter controls change.

## Capabilities

### New Capabilities

- `reference-index`: the SDK operations for linked references with namespace-parent and alias expansion, path references, related-page frequencies, starting filters, tagged pages and unlinked mentions.
- `published-reference-sections`: the order of the sections below the outline, deferred loading of their bodies, the no-script fallback and the budgets.
- `published-unlinked-references`: the Unlinked References section.
- `published-tagged-pages`: the Pages tagged with section.

### Modified Capabilities

- `published-linked-references`: what counts as a reference, the count text, zero-reference pages, group order and look, breadcrumbs, the filter dialog, filter options that follow the filter, starting filters and the collapse rule.

## Impact

- SDK: a new `references` module beside `page_names` and `outline`; the reference index moves out of `site/backlinks.rs`; a new `aho-corasick` dependency for the unlinked-mention pass, which follows the [dependency notice process](../../../../docs/licensing.md).
- Publisher: `site/backlinks.rs` and `site/backlinks.js` are rewritten for the new markup, dialog and fragment loading; new `site/references.css`; new fragment files under each page's output; the section order in `site/render.rs` and `site/mod.rs`; `site/properties.rs` keeps hiding `filters::`.
- Parity harness: `scripts/parity/budgets.toml` gains a `[references]` table checked by `parity:budgets` and marked by `parity:perf`.
- Gitpa receives the same sections in swapped `<main>` content; the references script initializes again after a swap, as the current one does.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
