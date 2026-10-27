# Verification

`lsg` now publishes Logseq's All pages table for the whole encode garden: 40 rows of the default view in `pages.html`, 318 static paging pages, and a 173 KB (gzip) page list that loads only when a reader sorts, searches or shows journals. Measured values match the analysis's Logseq values, and every size budget holds.

Builds compared on 2026-10-26:

- Logseq: the `gh-pages` build `8132c95` of garden commit `0ce74d6`, served locally, light theme, 1280×800.
- `lsg`: this change on `48fcc54` (`origin/main`), whole garden at commit `083106d` through `mise run parity:build` (run `20261026T203919Z`, `--journals`, 1,042 roots).
- Baseline: `lsg` built from `48fcc54` without this change.

## Screen

| Check | Logseq `gh-pages` | `lsg` before | `lsg` with this change |
| --- | --- | --- | --- |
| Title and heading | `All pages` | `Garden` | `All pages` |
| Heading | 36px, weight 300, 40px line height, margin `8px 0 24px`, `rgb(0,0,0)` | 2em default | same as Logseq |
| Total | `Total 19206 pages`, 14px, opacity 0.7, margin `0 0 16px 4px` | none | `Total 12752 pages`, same style |
| Paging | `1/481 Next ▸ »` | none | `1/319 Next ▸ »`, repeated below |
| Search box | 14px, 28px tall, 173px wide | none | same |
| Header cells | 14px, weight 400, `rgb(104,112,118)`, `10px 8px`, `2px solid rgb(230,232,235)` | none | same |
| Body cells | 16px, `6px 8px`, rows `rgb(251,252,253)` / `rgb(236,238,240)`, 49px rows | list items | same values; 37px rows (no checkbox column) |
| Page link | `rgb(52,81,178)`, no brackets | default link | same |
| Default order | `Updated At` newest first (build parse times: `card`, `Favorites`, `contents`, …) | byte order | `Updated At` newest first (git: `GitP/A/Session/26/10/22-Thu`, then the MicroFreak presets changed on 2026-10-25) |
| Sort `Page name`, then `Backlinks` | `Microfreak` 7,662, `Logseq` 6,429, `Logseq/Entity` 5,902 | none | `Microfreak` 7,662, `Logseq` 6,439, `Logseq/Entity` 5,912, `Microfreak/UI` 5,778 |
| Search `mise` + Enter | `Total 20 pages`, `mise` (598) first | none | `Total 20 pages`, `mise` (598) first |
| Journals toggle | 19,206 → 19,741 | journals always listed | 12,752 → 13,259 |
| 390×844 | checkbox, icon and name | list | icon and name only; page width 390, no sideways scroll |
| Editing controls | checkboxes, `Delete`, whiteboard toggle, `⋯` | none | none (Resolved Question 3) |

Differences that remain, each expected:

- Totals: Logseq also lists pages that only references create; `lsg` does not publish those yet ([Issue #111](https://github.com/codekiln/logseq-gardener/issues/111), [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115)).
- `Logseq` and `Logseq/Entity` have 10 more backlinks in `lsg` because the garden gained blocks between `0ce74d6` and `083106d`. `Microfreak` and `mise` match exactly.
- Search results after the first can differ in order: Logseq breaks equal fuzzy scores by its current sort, which is its parse order; `lsg` breaks them by its current sort, which is git order. `mise/x` and `mise/Q` score the same.
- Dates are git commit times, not parse times (Resolved Question 1).
- Rows are 37px rather than 49px because the checkbox column, which sets Logseq's row height, is not drawn.
- The table keeps the page's 810px width; `t w` widens it to 1280px.

Dark theme at 390×844 uses the existing theme tokens: stripes `--lsg-background` and `--lsg-rule`, links `--lsg-link`.

## Sizes

| File | Before | After |
| --- | ---: | ---: |
| `pages.html`, first paint | 1,266,786 bytes, 267,848 gzip | 41,719 bytes, 9,071 gzip |
| Largest `pages-<n>.html` | – | 44,535 bytes, about 9.1 KB gzip (budget 10 KB) |
| `_pages/list.json` | – | 959,249 bytes, 172,920 gzip (budget 300 KB) |
| `lsg-all-pages.js` | – | 5,383 bytes, 2,264 gzip (budget 3 KB) |

The first version of the list carried each row's route hash and was 325 KB gzip, over budget. Routes are `p-<FNV-1a digest of the title>.html`, so the script now computes them for the rows it shows and the file carries a route only when it differs.

`mise run parity:budgets` on the fixture garden: `pages.html` first paint 32,678 bytes (7,872 gzip), script 3,393 bytes; growth 0 bytes when a garden with a full first screen gains 2,000 pages; `lsg-all-pages.js` 2,157 gzip at the time; no other page loads the script.

## Browser checks

In the built-in browser on the whole-garden `lsg` build:

- Opening `pages.html` and following `Next ▸` to `pages-2.html` (`2/319`, first row `Microfreak/Preset/040 Disrespectful`) requested no `_pages/` file.
- Searching 13,259 names with the ported `score`, sorting and taking 20: median 6 ms, p95 12 ms on desktop.
- JS heap after loading the list, sorting by `Backlinks`, searching and showing journals: 8.8 MB.

## Performance

`parity:perf --builds lsg --fixtures <All pages> --baseline-lsg <baseline>` on run `20261026T203919Z`. No saved run holds Logseq's rows, so a full `--record` would have dropped them; the All pages rows and a new `All pages` section were written into [Parity performance](../../../../docs/parity/performance.md) by hand from the run's `perf.json`.

| Measure | Budget | Desktop | Phone |
| --- | --- | ---: | ---: |
| All pages data before title (KB gzip) | 100 | 13.2 (was 271) | 13.2 (was 271) |
| All pages script before title (KB) | 16 | 14.4 | 14.4 |
| Title visible (ms) | 300 / 1,500 | 23.1 (was 85.6) | 235 (was 233) |
| `Page name`, then `Backlinks`, including the list fetch (ms) | – | 57 | 1,069 |
| Search on Enter, p95 (ms) | 50 / 150 | 11 | 41 |
| Peak JS heap after loading the list and sorting (MB) | 20 on phone | 8.6 | 13.9 |
| List requested before the first click | never | no | no |

Whole-garden build with `--journals`: 10.5 s with this change against 11.0 s for the baseline; a second pair measured 10.6 s against 10.5 s (+1.7%). Both are within the 10% budget; the git history read runs on its own thread while pages render.

## Tests

- SDK: `ReferenceIndex::backlink_count`, `GitDates` (repository, no repository, shallow clone, log parsing) and `page_list` (kinds, icon, backlinks, ordering) unit tests; `tests/site_all_pages.rs` builds a git garden and checks order, counts, dates, icon, the list file and identical output from two builds of one commit, and a garden without git for paging and name order.
- Publisher: `site::all_pages` unit tests for paging links, the no-dates layout, generated rows without badges and list escaping.
- Script: `tests/all-pages.test.cjs` covers the `score` port, the route digest against the publisher's, no list request before interaction, shared sort direction, undated rows last, search and Escape, the journals toggle, row markup and paging.
- Existing index, homepage, journals, hierarchy and navigation tests updated for the table.
