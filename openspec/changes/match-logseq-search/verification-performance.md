# Verification: performance tasks

Tasks 1.1, 1.2, 5.1, 5.2 and 5.3 of `match-logseq-search`. The results are in [Search performance](../../../docs/parity/search-performance.md).

## 1.1 Parity build and search rows

- `scripts/parity/build.py` passes `--search`; `mise run parity:build` published 8,696 pages from 1,044 roots of garden commit `75ef84c` with `lsg` `a2284e0`.
- `mise run parity:perf --search-site target/parity/20261027T165841Z/site --record` measured `lsg` and Logseq's `gh-pages` build `b491e00` on the desktop and phone profiles. The search sections of `docs/parity/performance.md` hold numbers for both builds.
- Measured: title index 243 KB gzip and 0.7 MB decoded; largest shard 63.5 KB (`re`); largest chunk 32.8 KB; first open 275 KB; largest keystroke download 75.6 KB; dialog 9 / 207 ms; first page results 108 / 2,343 ms; cold keystroke p95 29 / 653 ms; warm keystroke p95 19 / 541 ms; heap growth after the query list 4.2 MB; build 13.1 s with `--search`, 11.7 s without (+12%).

## 1.2 Pagefind

`experiments/pagefind/measure.py` indexed the same build with Pagefind 1.5.2 through `mise exec npm:pagefind@1.5.2`.

| Measure | Pagefind | `lsg` |
| --- | ---: | ---: |
| Index size | 18.9 MB in 13,713 files (index chunks 7.5 MB, largest 102.6 KB; fragments 10.6 MB) | 7.2 MB gzip in 1,599 files |
| First search, bytes | 299 KB desktop, 271 KB phone | 275 KB |
| First search, time | 316 ms desktop, 2,229 ms phone | 108 ms, 2,343 ms |
| Bytes per keystroke, largest / 95th percentile | 103.7 KB / 57.5 KB | 75.6 KB |
| Keystrokes that download, of 36 | 26 | 10 |
| Keystroke latency, median / 95th percentile, desktop | 30 / 75 ms | 6 / 20 ms |
| Keystroke latency, median / 95th percentile, phone | 206 / 739 ms | 25 / 653 ms |

The purpose-built index is within Pagefind's numbers except the phone first open, where both take about 2.3 s.

## 5.1 Budgets

`[search]` in `scripts/parity/budgets.toml` replaces `corpus_decoded_mb`, `corpus_gzip_kb` and the `first_open_*` keys with the design's budgets. `parity:budgets` checks the title index, every shard and every chunk on the fixture garden and names the largest offender. `test_oversized_shard_fails_naming_the_shard` in `scripts/parity/test_parity.py` builds a format version 2 site whose shard is over 64,000 bytes gzip and checks the failure names the shard. The SDK's default shard budget changed from 64 × 1,024 to 64,000 bytes so the writer and the budget agree.

## 5.2 Whole-garden values over budget

Two `lsg` values are over budget; the budgets were not changed.

- Largest record chunk 32.8 KB against 32 KB: chunk 402 holds 76 transcript blocks of about 940 characters each. Fix: size-limited chunks in the SDK writer.
- Phone warm keystroke p95 541 ms against 100 ms: 3 of 29 warm keystrokes fetched a record chunk, and the dialog waits for chunks before drawing. Keystrokes that fetched nothing: 26 ms. Fix: task 4.1's placeholder rows.

## 5.3 CI

`mise run ci` passes with these changes (format, lint, tests, `parity:budgets`, OpenSpec validation and documentation links).
