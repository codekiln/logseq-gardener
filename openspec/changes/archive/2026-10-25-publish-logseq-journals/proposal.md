## Why

Logseq's published site opens on the encode garden's journals, newest first, and every journal is a page that other pages link to. `lsg` skips all 505 journal files, so its home page is a 1.2 MB list of every page and 778 links to journal titles render as unresolved text ([journals analysis](../../../docs/parity/journals.md), [Issue #110 — Publish journals and the Journals home page](https://github.com/codekiln/logseq-gardener/issues/110)).

## What Changes

- The SDK gains journal identity: it derives a journal's date from its file name (`yyyy_MM_dd`, or `:journal/file-name-format`) and its title from `:journal/page-title-format` (default `MMM do, yyyy`), so `journals/2026_10_23.md` is `2026-10-23 Fri` in the encode garden. It also recognises a page reference written in the configured format, `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd` as a link to that date's journal, and lists journals newest first.
- `lsg publish --journals` publishes every journal file as a page with its blocks, page properties, aliases and Linked References, and without a namespace hierarchy section. `public:: false` withholds a journal as it withholds a page. Without `--journals`, journals are counted and skipped as today.
- Links to a published journal resolve in any of the four date formats and keep the author's text: `[[Feb 1st, 2025]]` opens `2025-02-01 Sat`.
- When journals are published and enabled, the home page is a journals feed: journals dated on or before the build date, newest first, three in `index.html`, then seven per batch file (`journals-2.html`, `journals-3.html`, …). A script appends the next batch when the reader comes within 500px of the bottom; without JavaScript, an Earlier link opens the next batch file. Each journal shows an `h1` linking to its page, its expanded blocks and a collapsed Linked References section that loads its content when opened.
- With a feed on `index.html`, the complete page index moves to `pages.html`, as it does with `--homepage`. With `--homepage`, the feed's first batch is `journals.html`.
- The left sidebar gains Journals as its first item, linking to the feed and marked current on feed pages, hidden when `:feature/enable-journals?` is `false` or no journal is published. `g j` opens the feed.
- Dates that only links mention, and today's NOW/NEXT and SCHEDULED AND DEADLINE sections, stay out of this change (see design.md).

## Capabilities

### New Capabilities

- `journal-identity`: the SDK operations for date formats, journal dates and titles, journal references in any accepted format, and the newest-first journal list.
- `published-journals`: journal pages, the journals feed and its batch files, and the feed's performance budgets.

### Modified Capabilities

- `local-static-publishing`: journals are published on request instead of always skipped, and the home page becomes the feed when journals are published.
- `cli-static-publishing`: `lsg publish` accepts `--journals` and reports published journals.
- `published-app-shell`: the sidebar gains Journals, All pages follows the page index to `pages.html` when the feed holds `index.html`, and `g j` joins the shortcuts.

## Impact

- SDK: a new `journals` module (date formats, journal identity, reference parsing, journal list); `GardenSettings` reads `:journal/page-title-format` and `:journal/file-name-format`; `PageNameIndex` resolves date references to journal routes.
- Publisher: `publish_site_with_options` in `crates/logseq-gardener-sdk/src/site/mod.rs` selects journals, renders journal pages and the feed, and writes batch files; a new `lsg-journals.js` feed loader; `shell.rs` and `shell.js` gain Journals and `g j`.
- CLI: `--journals` on `lsg publish`, help text and JSON report.
- Parity harness: `parity:build` passes `--journals`; the fixture garden gains journals; `budgets.toml` gains feed budgets; `parity:perf` measures the feed.
- Gitpa, which calls `lsg publish` without `--journals`, keeps its current output.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
