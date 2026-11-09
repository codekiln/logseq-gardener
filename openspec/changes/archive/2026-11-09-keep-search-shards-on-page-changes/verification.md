# Keep search shards on page changes: verification

## One added page on the encode garden

Both builds published a copy of the encode garden at commit `66b81e9`, without its `.git` folder, with `SOURCE_DATE_EPOCH=1762500000`, `--filename-format triple-lowbar --journals --search` and every root namespace included. The second build added `pages/Logseq___Churn Added Page.md` holding one block, `An added page about zymurgy and quokkas for a churn test.`

| Build | Shards rewritten | Shard bytes rewritten | All pages files rewritten | Other files rewritten |
| --- | ---: | ---: | ---: | --- |
| main (`a25b3a1`, format 4) | 1,094 of 1,234 | 7.9 MB | 459 (7.8 MB) | the new page, `pages.html`, the `Logseq` page and its hierarchy fragment, two Unlinked References files, the graph data, `_pages/list.json`, `lsg-site.json`, the title index, the manifest, 1 chunk |
| this change (format 5) | 10 of 1,234 | 0.36 MB | 459 (7.8 MB) | the same |

The 10 shards are `a`, `ab`, `ad`, `an`, `ch`, `fo`, `pa`, `qu`, `te` and `zy`: each holds a word of the new page. The new page's slot held no other page. Every other changed file renders content that the new page changes; the All pages files are the non-goal in `design.md`.

`one_added_page_changes_only_the_shards_of_its_words` in `crates/logseq-gardener-sdk/tests/site_search.rs` checks the same on a 41-page fixture garden; with the format 4 numbering it fails on shard `34`, which holds none of the added words.

## Search files on the encode garden

| | Format 4 | Format 5 |
| --- | ---: | ---: |
| Shards | 1,234 | 1,234 |
| Shards, total gzip | 2.683 MB | 2.835 MB |
| Largest shard, gzip | 51.1 KB | 55.6 KB |
| Chunks, total gzip | 5.86 MB | 5.86 MB |
| Manifest, gzip | 52.6 KB | 52.5 KB |
| Title index, gzip | 147.3 KB | 147.3 KB |
| Engine, without comments and indentation | 16,026 bytes | 16,886 bytes |

9,186 of the 14,371 pages have block records, so there are 65,536 slots; 8,560 of them hold a page and 598 hold more than one.

### Shard size by numbering and encoding

`experiments/shard-numbering/shard_numbering.py` rewrote the format 4 shards of the same site, keeping each shard's key. Total and largest gzip:

| Block names | JSON pairs | Base-32 string |
| --- | ---: | ---: |
| Page number in title order (format 4) | 2.683 MB, 51.1 KB | 2.314 MB, 44.6 KB |
| Page number in a random order | 2.922 MB, 56.1 KB | 2.521 MB, 49.1 KB |
| Route digest slot, 16,384 slots | 2.944 MB, 56.6 KB | 2.541 MB, 49.7 KB |
| Route digest slot, 32,768 slots | 3.101 MB, 59.7 KB | 2.682 MB, 52.5 KB |
| Route digest slot, 65,536 slots | 3.259 MB, 62.8 KB | 2.826 MB, 55.4 KB |

Grouping each page's blocks (page step, count, block steps) was 3% to 7% larger than pairs in every row.

## Search latency

`parity:perf --search-site` on each build's site, lsg only. Phone values are the median of three runs, desktop values the mean of two.

| Profile | Build | Dialog (ms) | First page results (ms) | First open (KB, gzip) | Cold keystroke p95 (ms) | Warm keystroke p95 (ms) | Largest keystroke download (KB) | 95th percentile keystroke download (KB) | Heap growth (MB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| phone | format 4 | 236 | 2,017 | 164.5 | 486 | 66 | 97.2 | 47.8 | 5.8 |
| phone | format 5 | 246 | 2,034 | 165.4 | 493 | 80 | 103.6 | 47.8 | 6.3 |
| desktop | format 4 | 13 | 122 | 171.2 | 41 | 12 | 115.2 | 40.9 | 5.9 |
| desktop | format 5 | 12 | 136 | 172.1 | 33 | 12 | 121.5 | 42.5 | 6.4 |

Every value is within its `[search]`, `[desktop]` and `[phone]` budget. In Node on the same site, computing the slots took about 6 ms and parsing all 1,234 shards 115 ms, against 111 ms for format 4.

## Build time

Three warm builds of the same copy with `--timings`, alternating binaries: the whole build took 19.1–20.5 s with main and 20.2–21.6 s with this change; the search phase took 2.02–2.13 s and 1.73–1.95 s. Writing each word's string once, instead of again for every shard split, made the search phase faster.

## Checks

- `cargo test -p logseq-gardener-sdk` passes, including the number encoding, the static search contract's example entry, a page added before and after another in its slot, and the added-page test above.
- `node --test tests/search.test.cjs tests/search-loader.test.cjs` passes, including malformed number strings, unknown slots and format version 4 data. The shared fixture holds two slots with two pages each, and both engines return its expected results.
- `mise run ci` passes.
