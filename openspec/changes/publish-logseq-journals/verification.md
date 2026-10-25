# Verification

The encode garden publishes 505 journals, and its home page is Logseq's journals feed at 9.8 KB of data before the title, down from 261 KB. Measured on 2026-10-25 with the garden at `0ce74d6` and Logseq's `gh-pages` build at `8bc3391`, using parity run `20261025T025833Z`.

## Feed and journal pages

- `index.html` lists `2026-10-23 Fri`, `2026-10-22 Thu` and `2026-10-21 Wed`. Logseq's build starts at `2026-10-24 Sat` because it was built from a newer garden commit than the local checkout.
- One scroll to the bottom appends `2026-10-20 Tue` through `2026-10-14 Wed` from `journals-2.html`; the Earlier link then points at `journals-3.html`. The encode garden has 72 batch files.
- Opening Linked References under `2026-10-23 Fri` in the feed loads the section from the journal page: one page (the LangSmith question), with the filter controls showing.
- The journal page `2026-10-23 Fri` has the title as its `h1`, its Filed and Updated blocks, one Linked Reference and no Hierarchy section.
- `[[Feb 1st, 2025]]` on `Docker/For Mac/How to Install into Zsh` links to the `2025-02-01 Sat` page and keeps its text.
- `g j` on `tmux` opens `index.html`. Journals is the first sidebar item and is marked current on the feed (screenshots `sidebar-phone-*.jpg`).

## Comparison report

`parity:compare` lists 29 Logseq journals missing from `lsg`, down from 534: the 28 dates that only links mention (left to #115) and `2026-10-24 Sat`, which the local checkout does not have yet. Unresolved-link diagnostics that name a `yyyy-MM-dd EEE` title fell from 778 to 18, all link-only dates.

## Gitpa and builds without journals

A build without `--journals` was compared file by file with `origin/main`'s `lsg` on the whole encode garden. Every file is identical apart from `lsg-shell.js` (`g j` and the feed loader) and the new `id="linked-references"` attribute on each Linked References section.

## Performance

See [Parity performance](../../../docs/parity/performance.md).

| Measure | Budget | Result |
| --- | --- | --- |
| `lsg-journals.js` | 4 KB | 2.7 KB |
| Home data before the title, phone | 100 KB gzip | 9.8 KB |
| Home script before the title | 16 KB | 14.0 KB |
| Peak heap after ten batches (73 journals), phone | 20 MB | 3.0 MB |
| `index.html` with 1,000 older journals added | unchanged | unchanged (`parity:budgets`) |
| Whole-garden build with journals | at most 20% over without | 4.1 s with, 4.9 s without |

Logseq's feed stopped loading new batches under automated scrolling: it reached 10 journals on desktop and 3 on the phone profile before a 30-second wait passed without a new batch. Its heap was 2.6 GB on desktop and 1.6 GB on the phone profile from loading the whole graph.

The backlinks index change made the whole-garden build about five times faster (25.6 s on `main`, 4.5 s on this branch, without journals) with identical output.

## Differences left to other issues

- Journal title size and weight: Logseq uses 36px at weight 300; `lsg` uses its page `h1` (#111).
- Outline bullets, heading blocks and link styling (#111, #112).
- `Filed` and other reference-only pages render as unresolved text (#115).
- Page properties on `2019-08-07 Wed`: `lsg` shows page properties only when `:publishing/page-properties?` is true, which the encode garden does not set.
