## Why

Every parity analysis under issue #106 compares an `lsg` page with the same page in Logseq's published build of the encode garden. Today each comparison starts with a hand-made run: 1,042 `--include` flags to cover the whole garden, a script to decode Logseq's `db.gz`, and manual lookup of each page's file in both builds ([analysis](../../../../docs/parity/comparison-harness.md)).

The parity work also has to show that `lsg` loads much faster than Logseq. Logseq's build downloads a 15.8 MB `db.gz`, which inflates to 174 MB of graph data, plus about 35 MB of JavaScript before it shows any page. Nothing measures either build's load time, memory or search speed today.

## What Changes

- Add a mise task that builds every public page of a garden with `lsg` into a new directory and records the options it used.
- Add a mise task that reads the page list from Logseq's `gh-pages` build, compares it with the `lsg` build, and writes a report of pages present in one build and missing from the other, in named groups: journals, reference-only pages, hidden files, titles that `lsg` cannot select, and other pages.
- Add a mise task that saves full-page screenshots of a fixed list of pages from both builds, side by side, after drawing lazily rendered sections.
- Add a mise task that measures both builds on the same pages, on a desktop profile and a mid-range phone profile: time until the page title appears, first contentful paint, data transferred before the title appears, peak JavaScript memory, and search latency per keystroke on the whole garden. It checks the `lsg` numbers against stated budgets and can record them as rows in `docs/parity/performance.md`.
- Add a CI check that builds a repository fixture garden with `lsg` and fails when a page's first-paint size or script size exceeds its budget, or grows when unrelated pages are added to the garden.
- Add a title-to-file list that both comparison tasks use to open the same page in both builds.
- Unit-test the Logseq data decoding and the grouping in `mise run test`.

## Capabilities

### New Capabilities

- `parity-comparison-harness`: building the whole encode garden with `lsg`, reading Logseq's published build, reporting missing pages by group, saving side-by-side screenshots, and measuring both builds' load performance and search latency against budgets.

### Modified Capabilities

None.

## Impact

New scripts under `scripts/parity/`, new mise tasks under `.mise/tasks/parity/`, a pinned browser-automation tool in `mise.toml`, a contributor guide at `docs/parity/README.md`, benchmark rows in `docs/parity/performance.md`, and a size-budget check in `mise run ci`. The `lsg` command, the SDK and the published output stay as they are.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
