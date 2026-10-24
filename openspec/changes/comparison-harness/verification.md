# Verification

Run on 2026-10-24 against garden commit `0ce74d6` and `gh-pages` commit `1ec008c`, with `lsg` built from `4805e43`.

- `mise run parity:build` published 6,984 source pages and 3,266 generated namespace pages from 1,042 top-level namespaces in 26 seconds (release build). No file was skipped for its top-level namespace.
- `mise run parity:compare` read 17,396 Logseq pages, 533 journals and 7,492 pages with content, the counts the [analysis](../../../docs/parity/comparison-harness.md) measured by hand. Logseq-only pages: 533 journals, 6,607 reference-only pages, 0 hidden, 3 titles `lsg` cannot select (the `%2F` titles, now #129) and 3 other pages (`contents`, `PARAMETERS` and the page parsed from a Markdown file under `assets/`). No page is only in `lsg`.
- `mise run parity:screenshots` captured all 13 fixtures from Logseq and 12 from `lsg`; the journal `2026-10-23 Fri` is noted as missing from `lsg`. The `tmux` captures are in `verification/`; Logseq's includes its Linked References, drawn only after scrolling.
- `mise run parity:perf --loads 1` measured both builds on both profiles, then `--builds lsg --loads 3` re-measured `lsg`; the results are in [Parity performance](../../../docs/parity/performance.md). Every `lsg` fixture page is within budget except Home and All pages, whose 1.2 MB page list transfers 257 KB gzip before the title (#117 covers All pages).
- `mise run parity:budgets` passed: the four fixture-garden pages are 15–18 KB (about 5 KB gzip) with 8.4 KB of script, and none changed size when 2,000 pages were added.
- `mise run ci` passed, including 26 harness unit tests.
