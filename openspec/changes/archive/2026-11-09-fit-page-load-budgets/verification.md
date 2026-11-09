# Verification

Measured on 2026-11-09 on an Apple M2 Pro shared with other work (load average 11 to 20), on encode garden commit `f1c1146`. Before is `lsg` at `e5b6479` (run `20261109T172729Z`), after is this change (run `20261109T180209Z`). Both runs used `mise run parity:perf --builds lsg --loads 3` with the fixtures `tmux`, `Foam`, `All pages`, `Contents` and `mise`; page-load values are medians of three loads.

## Values the issue lists

| Value | Budget | #590 | Before | After |
| --- | ---: | ---: | ---: | ---: |
| Script before the title, `All pages`, desktop (KB) | 16 | 17.0 | 17.0 | 11.7 |
| Script before the title, `All pages`, phone (KB) | 16 | 17.0 | 17.0 | 11.7 |
| Script before the title, `Contents`, desktop (KB) | 16 | 26.6 | 11.6 | 11.7 |
| Filter change on `mise`, desktop (ms) | 50 | 56 | 58.6 | 13.5 |
| Warm keystroke 95th percentile, phone (ms) | 100 | 158 | 152.6 | 74.3 |

- `Contents` loaded 15 times on desktop: before, 3 of 15 loads counted 26.6 KB of script before the title, because `lsg-outline.js` and `lsg-references.js` were requested before the title's frame; after, 0 of 15.
- The filter change in a separate seven-load probe: median 48.3 ms before and 12.5 ms after. On the phone profile the harness measured 95.7 ms before and 44.4 ms after.
- Every other script before the title grew by 118 bytes, the size `lsg-shell.js` added (11.6 to 11.7 KB). All pages' data before the title dropped from 17.9 to 15.7 KB gzip.

## Search with the new cold classification

| Profile | Cold keystrokes | Cold 95th percentile (ms) | Warm 95th percentile (ms) | Keystrokes that fetched nothing, 95th percentile (ms) |
| --- | ---: | ---: | ---: | ---: |
| phone, before | 9 | 467.7 | 152.6 | 61.9 |
| phone, after | 12 | 496.2 | 74.3 | 67.0 |
| desktop, before | 9 | 45.4 | 25.2 | 24.9 |
| desktop, after | 9 | 46.5 | 25.2 | 23.5 |

The three keystrokes that moved from warm to cold on the phone were typed while their word's shard was still loading. In a separate probe of the before build, the two slowest of them took 171 and 155 ms, each typed 300 ms after a cold keystroke that took 486 or 460 ms. The cold 95th percentile stays within the 1,000 ms phone budget.

## Other rows

| Row | Before | After | Budget |
| --- | ---: | ---: | ---: |
| Open Linked References on `mise`, desktop / phone (ms) | 20.2 / 367 | 23.2 / 363 | 300 / 1,500 |
| Peak heap after both sections on `mise`, desktop / phone (MB) | 5.4 / 5.3 | 6.3 / 6.8 | 20 |
| All pages sort by Backlinks, desktop / phone (ms) | 66.6 / 1,392 | 70.8 / 1,389 | – |
| Zoom on `tmux`, desktop / phone (ms) | 16.5 / 20.8 | 16.1 / 18.6 | 50 / 150 |
| Phone title, `tmux` (ms) | 397 | 396 | 1,500 |
| Whole-garden build with journals and search (s) | 18.8 | 19.0 | – |

## Phone title time

The phone title time stays at about 400 ms. The design's decision "Accept the phone title time" explains it: one render-blocking request for the shared `lsg-site.css` since #400.

## Build time

The design's decision "Explain the build time" holds the measurements of both `lsg` versions on both garden commits.

## Checks

- `mise run parity:budgets` passes; the references styles are 1,021 bytes gzip against the new 1.05 KB budget.
- On the fixture garden built with the recorder plugin, following the All pages link from a page adopted All pages in the same document, loaded `lsg-all-pages.js`, and a click on Backlinks sorted the table, with no page errors.
- `mise run ci` passes.
