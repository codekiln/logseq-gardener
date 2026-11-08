# Stable search addresses verification

## One added block on the encode garden

Both builds published a copy of the encode garden at commit `a7d948e`, without its `.git` folder, with `SOURCE_DATE_EPOCH=1762500000`, `--filename-format triple-lowbar --journals --search` and every root namespace included. The second build appended `- An added block for a churn test.` to `pages/Deno.md`.

| Build | Files changed or added | Size | Files removed | What changed |
| --- | ---: | ---: | ---: | --- |
| main (`a432076`) | 16,056 | 227.8 MB | 1,453 | 14,602 pages, 1,127 shards, 324 chunks, the title index, the manifest, `lsg-site.json` |
| this change | 11 | 2.7 MB | 8 | the `Deno` page, 7 shards (`an`, `ad`, `bl`, `fo`, `a`, `ch`, `te`), 1 chunk, the manifest, `lsg-site.json` |

`lsg-site.json` (2.1 MB) accounts for most of the remaining size; its `revision` follows every generated file, as the published-site-manifest spec requires. The test `one_added_block_rewrites_only_its_page_and_the_search_files_that_hold_it` in `crates/logseq-gardener-sdk/tests/site_search.rs` checks the same edit on a 41-page fixture garden; it failed on main with 47 changed files outside search data.

## Search files on the encode garden

| File | main: count | main: gzip total | main: largest gzip | this change: count | this change: gzip total | this change: largest gzip |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Shards | 1,194 | 2.16 MB | 56.6 KB | 1,226 | 2.58 MB | 48.9 KB |
| Chunks | 475 | 5.14 MB | 32.0 KB | 1,906 | 5.75 MB | 21.0 KB |
| Title index | 1 | 254 KB | | 1 | 244 KB | |
| Manifest | 1 | 23 KB | | 1 | 50 KB | |

The site has 50,308 files instead of 48,845 because of the smaller chunks.

## Search latency

`parity:perf --search-site` on each build's site, lsg only. Phone values are the median of three runs, desktop values the mean of two.

| Profile | Build | First page results (ms) | First open (KB, gzip) | Cold keystroke p95 (ms) | Warm keystroke p95 (ms) | Largest keystroke download (KB) | 95th percentile keystroke download (KB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| phone | main | 2,473 | 293 | 498 | 38 | 78 | 76 |
| phone | this change | 2,372 | 261 | 443 | 42 | 93 | 38 |
| desktop | main | 124 | 319 | 31 | 9 | 112 | 76 |
| desktop | this change | 101 | 281 | 38 | 8 | 111 | 40 |

Before the loader fetched the title index in parallel with the engine, the phone's first page results were 2,534–2,556 ms, over the 2,500 ms budget. Every value is within its `[search]` budget with the parallel fetch.

## Build time

Three warm builds of the same copy with `--timings`: the whole build took 20.4–21.1 s with main and 22.0–22.7 s with this change; the search phase took 1.76–1.88 s and 2.14–2.28 s. Both stay inside the 20% publication budget and the 25% search build budget.

## Checks

- `mise run test` passes, including the regenerated shared search fixture, the engine's pair decoding and count checks, and the loader's recovery after a missing shard.
- `mise run search:browser-test` passes. It now serves the nested page's `lsg-search-boot.js` from the site root, as it already did for the other search files; on main the proof stopped at the nested page because the start-up script 404ed there. A new step removes a shard after search loads and checks that Retry fetches the manifest again and answers.
