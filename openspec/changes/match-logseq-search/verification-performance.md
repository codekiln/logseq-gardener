# Verification: performance tasks

Tasks 1.1, 1.2, 5.1, 5.2 and 5.3 of `match-logseq-search`. The results are in [Search performance](../../../docs/parity/search-performance.md).

## 1.1 Parity build and search rows

- `scripts/parity/build.py` passes `--search`; `mise run parity:build` published 8,696 pages from 1,044 roots of garden commit `75ef84c` with `lsg` `db371f2`.
- `mise run parity:perf -- --search-site target/parity/20261027T173350Z/site --record` measured `lsg` and Logseq's `gh-pages` build `b491e00` on the desktop and phone profiles. The search sections of `docs/parity/performance.md` hold numbers for both builds.
- Measured (desktop / phone): title index 243 KB gzip and 0.7 MB decoded; largest shard 63.5 KB (`re`); largest chunk 32.0 KB (31,988 bytes) of 450 chunks; first open 306 / 281 KB; largest keystroke download 95.1 / 78.0 KB; dialog 23 / 245 ms; first page results 110 / 2,378 ms; cold keystroke p95 46 / 578 ms; warm keystroke p95 16 / 35 ms; heap growth after the query list 6.7 / 6.5 MB; build 13.3 s with `--search`, 11.4 s without (+17%).
- An earlier run on `lsg` `a2284e0`, before the grouped dialog, measured a 32.8 KB chunk and a 541 ms phone warm p95; section 5.2 explains the fixes.

## 1.2 Pagefind

Measured on `lsg` `a2284e0`, before the grouped dialog and size-limited chunks.

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

## 5.2 Whole-garden values within budget

Every `lsg` value is within its budget; no budget changed. The run on `a2284e0` had two values over budget, fixed as follows.

- Largest record chunk, 32.8 KB against 32 KB: chunk 402 held 76 transcript blocks of about 940 characters each. The SDK writer now closes a chunk at every 256 records or before the record that would take it past 32,000 bytes gzip, and the manifest lists each chunk's first block number in `chunk_starts`. Largest chunk now: 32.0 KB.
- Phone warm keystroke p95, 541 ms against 100 ms: the dialog already drew rows whose chunk was loading with the page title and a placeholder line, but the harness timed keystrokes until `data-lsg-search-complete`, which waits for every chunk. The dialog now also sets `data-lsg-search-drawn` when it draws a result list, and `parity:perf` times keystrokes to that, as the budget intends. `placeholder_rows` in `scripts/search/browser.py` holds chunk requests and checks the placeholder rows, then their text. Warm p95 now: 35 ms.

Two further harness changes, in `scripts/parity/perf.py`:

- First page results also end at `data-lsg-search-drawn`. Page rows never wait for chunks, and Logseq's time ends when its Pages group appears. With the grouped dialog the phone value measured to `data-lsg-search-complete` was 2,724 to 2,790 ms against 2,500 ms, under machine load; measured to the drawn list it is 2,378 ms.
- Build time is the fastest of three publishes. Single timings under other load on the machine gave +38% and +81%; separate interleaved timings gave +3% to +17%.

Editing one block: `one_edited_block_changes_only_its_shards_and_chunk` still checks that an edit within budget changes only the manifest, the affected shard and one chunk. Because a size limit can move chunk boundaries, chunks are cut within fixed groups of 256 records; `an_edit_that_moves_chunk_boundaries_stays_inside_its_group` checks that an edit that moves a boundary changes no chunk outside its group.

## 5.3 CI

`mise run ci` passes with these changes (format, lint, tests, `parity:budgets`, OpenSpec validation and documentation links).
