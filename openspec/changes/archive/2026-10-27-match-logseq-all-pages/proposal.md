## Why

Logseq's All pages screen is a sortable, searchable table of the garden's pages, 40 rows at a time, with each page's backlink count and dates. `lsg` publishes `pages.html` titled `Garden`: one alphabetical list of 10,776 links with journals mixed in, no counts, no sorting, no search and no paging. At 1.27 MB (268 KB gzipped) before its title, it is the largest first paint in the `lsg` build and over the 100 KB budget ([All pages analysis](../../../../docs/parity/all-pages.md), [Issue #117 — Publish Logseq's All pages table](https://github.com/codekiln/logseq-gardener/issues/117)).

Logseq's own screen needs the whole graph first: 19.8 s to its title on desktop and 198 s on the phone profile. `lsg` can show the same table with the first 40 rows in static HTML and load the full list only when a reader asks for another view of it.

## What Changes

- The SDK gains a page-list operation shared by the CLI, the language server and the publisher. For each published page, journal and generated namespace page, it returns the title, route, kind, icon, backlink count, lower-cased sort key, and created and updated dates. The backlink count is Logseq's: every block whose references include the page, counting namespace children and the page's own blocks.
- Created and updated dates come from the garden's git history: the first and the last commit that touched the page's file. Logseq's dates in a published build are the moments the build parsed each file, so they say nothing about the garden; git dates keep Logseq's columns and default order and make them meaningful and stable across builds. A garden that is not a git repository gets no date columns and sorts by page name.
- `pages.html` (or `index.html` when there is no homepage or journals feed) becomes the All pages screen: the `All pages` heading, `Total N pages`, a toolbar with the `Search page names` input, the journals toggle and the paging control, and a table with `Page name`, `Backlinks`, `Created At` and `Updated At` in Logseq's look, striped rows, and the paging control repeated below.
- The page holds the first 40 rows of the default view: journals hidden, `Updated At` newest first. Static pages `pages-2.html`, `pages-3.html`, … hold the following rows of that view, so every page stays reachable without scripts, and `Next`/`Prev` move between them.
- A small script loads one list data file the first time a reader sorts, searches or turns on journals. Sorting by a column title flips one shared direction and shows a caret. Search runs on Enter, ranks page names with Logseq's fuzzy score, shows at most 20 rows and clears on Escape. Paging stays at 40 rows.
- At phone width only the `Page name` column remains.
- Logseq's editing controls (row checkboxes, `Delete`, the whiteboard toggle and the `⋯` menu) are not drawn; in a published site they change nothing that lasts.
- Generated namespace pages appear in the table like any page, with their distinct link style and accessible Namespace name and without the visible `Namespace` badge, as Logseq lists them.
- New budgets cap the All pages first paint, the static paging pages, the script, the list data file, the phone heap after loading the list, search time, and the build-time cost of reading git history.
- **BREAKING** for readers of `lsg` output only: the page index is titled `All pages`, hides journals by default and is split into pages of 40 rows.

## Capabilities

### New Capabilities

- `page-list`: the SDK operation that lists published pages with their kind, icon, backlink count, sort key and git dates.
- `published-all-pages`: the All pages markup, styles, static paging, list data file, script behaviour and budgets.

### Modified Capabilities

- `published-namespace-hierarchy`: the complete page index shows generated namespace pages without the visible `Namespace` badge, keeping their link style and accessible name.

## Impact

- SDK: a new `page_list` module beside `references` and `page_names`; `ReferenceIndex` gains a backlink count that includes the page's own blocks; a new `git_dates` reader runs one `git log` over the garden.
- Publisher: the index in `site/mod.rs` moves to a new `site/all_pages.rs` with `site/all-pages.css` and `site/all-pages.js`; the static paging pages and the list data file are new outputs.
- Parity harness: `scripts/parity/budgets.toml` gains an `[all_pages]` table checked by `parity:budgets` and marked by `parity:perf`.
- Gitpa receives the same table in swapped `<main>` content; the script initializes it again after a swap.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
