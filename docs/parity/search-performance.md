# Search performance

On the whole encode garden, `lsg`'s search format version 2 meets every budget but two. The dialog opens in 9 ms on desktop and 207 ms on the phone profile, page results appear 108 ms and 2,343 ms after Cmd/Ctrl+K, and each keystroke downloads at most 76 KB. Logseq needs 3.5 s and 14.2 s for the same first results, after its whole 22.8 MB graph has loaded with the page.

The two values over budget:

- **One record chunk is 32.8 KB gzip, over the 32 KB budget.** Chunks hold a fixed 256 block records. Chunk 402 holds 76 blocks of one podcast transcript page, each around 940 characters, where the median chunk is 10.8 KB. A size limit per chunk, in place of a fixed record count, would keep every chunk within budget; it changes the manifest, so it belongs in the SDK's file writer.
- **Warm keystrokes on the phone profile take 541 ms at the 95th percentile, over the 100 ms budget.** A warm keystroke fetches no word shard, but 3 of the 29 warm keystrokes fetched a record chunk, and the dialog waits for chunks before it draws any row. The keystrokes that fetched nothing take 26 ms at the 95th percentile. The design draws rows whose chunk is still loading with the page title and a placeholder line (task 4.1 of `match-logseq-search`); with that change these keystrokes no longer wait on the network.

Measured on 2026-10-27 with `mise run parity:perf --search-site` on an `lsg` build of garden commit `75ef84c` (`lsg` `a2284e0`, the `parity:build` options with `--search`), beside Logseq's `gh-pages` build `b491e00`, served locally. [Parity performance](performance.md) holds the same rows; [`search-performance.json`](search-performance.json) holds the raw values.

## Budgets

The budgets are in the `[search]` table of [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml). Warm keystrokes use the `[desktop]` and `[phone]` `search_p95_ms`. **Bold** marks an `lsg` value over its budget.

| Measure | Budget | `lsg` desktop | `lsg` phone | Logseq desktop | Logseq phone |
| --- | --- | ---: | ---: | ---: | ---: |
| Title index, gzip | 300 KB | 243 KB | | | |
| Title index, decoded | 1.5 MB | 0.7 MB | | | |
| Largest shard, gzip (words starting `re`) | 64 KB | 63.5 KB | | | |
| Largest record chunk, gzip | 32 KB | **32.8 KB** | | | |
| Dialog shown after Cmd/Ctrl+K | 100 ms desktop, 300 ms phone | 9 ms | 207 ms | 32 ms | 127 ms |
| First page results after Cmd/Ctrl+K | 500 ms desktop, 2,500 ms phone | 108 ms | 2,343 ms | 3,532 ms | 14,230 ms |
| Largest download by one keystroke, gzip | 200 KB | 75.6 KB | 75.6 KB | 0 | 0 |
| Keystroke that fetches a shard, 95th percentile | 150 ms desktop, 1,000 ms phone | 29 ms | 653 ms | – | – |
| Keystroke on loaded shards, 95th percentile | 50 ms desktop, 100 ms phone | 19 ms | **541 ms** | 1,698 ms | 13,361 ms |
| Heap growth after the query list | 30 MB on phone | 4.2 MB | 4.2 MB | 59.0 MB | 59.2 MB |
| Whole-garden build, with `--search` against without | at most +25% | +12% (13.1 s against 11.7 s) | | | |

The first open downloads 275 KB gzip: the dialog module, the search engine, the manifest and the title index. Nothing loads before Cmd/Ctrl+K or a click on the search button. Logseq downloads nothing more when its dialog opens, because the whole graph is already in memory. The query list in [`scripts/parity/search-queries.txt`](../../scripts/parity/search-queries.txt) types 36 keystrokes; 7 of them fetch a shard.

## Comparison with Pagefind

[Pagefind](https://github.com/CloudCannon/pagefind) 1.5.2 indexed the same `lsg` build, the 13,468 page files without the header and left sidebar, with [`experiments/pagefind`](../../experiments/pagefind/README.md). Its keystrokes are timed one after another, each searching and loading the first ten results.

| Measure | `lsg` | Pagefind |
| --- | ---: | ---: |
| Whole index, gzip | 7.2 MB in 1,599 files | 18.9 MB in 13,713 files |
| First open or first search, gzip | 275 KB | 299 KB |
| First open or first search, desktop / phone | 108 / 2,343 ms | 316 / 2,229 ms |
| Keystrokes that download something, of 36 | 10 | 26 |
| Largest download by one keystroke, gzip | 75.6 KB | 103.7 KB |
| Keystroke 95th percentile, desktop / phone | 20 / 653 ms | 75 / 739 ms |

`lsg`'s own index is within Pagefind's numbers on every measure but the phone first open, where both take about 2.3 s. Pagefind's index is larger because it stores a fragment per page with its whole text, and it downloads on most keystrokes because its index chunks split words across many files. Pagefind also ranks whole pages by relevance, without Logseq's groups, aliases, breadcrumbs or block anchors, so it could not replace `lsg`'s index in any case. Pagefind's main-thread heap grew by 0.7 MB, but it searches in a Web Worker whose heap this measurement does not include.

## Reproduce

```sh
mise run parity:build
mise run parity:compare   # extracts Logseq's gh-pages build beside the lsg build
mise run parity:perf --search-site target/parity/<run>/site --record
uv run --script experiments/pagefind/measure.py
```

`--record` rewrites the search sections of [Parity performance](performance.md) and `search-performance.json`. `parity:perf --search-site` fails while any `lsg` value is over its budget and names it. `mise run parity:budgets` checks the title index, shard and chunk budgets on the fixture garden in CI.
