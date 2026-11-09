tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility/Search]], [[lsg/Ref/Logseq Compatibility/Performance]], [[lsg/How To/Measure Performance]], [[lsg/Ref/Static Search Data]]

- # Search Performance
	- On the whole encode garden, every `lsg` value of search format version 2 is within its budget. The dialog opens in 23 ms on desktop and 245 ms on the phone profile, page results appear 110 ms and 2,378 ms after Cmd/Ctrl+K, a keystroke on loaded shards draws its results in 35 ms at the 95th percentile on the phone, and each keystroke downloads at most 95 KB. Logseq needs 3.5 s and 14.4 s for the same first results, after its whole 22.8 MB graph has loaded with the page.
	  id:: 3bfaeda3-b372-4c33-98e7-b9eeae8061ba
	- Measured on 2026-10-27 with `mise run parity:perf --search-site` on an `lsg` build of garden commit `75ef84c` (`lsg` `db371f2`, the `parity:build` options with `--search`), beside Logseq's `gh-pages` build `b491e00`, served locally. [[lsg/Ref/Logseq Compatibility/Performance]] holds the same rows, and [`docs/parity/search-performance.json`](https://github.com/codekiln/logseq-gardener/blob/main/docs/parity/search-performance.json) holds the raw values. [[lsg/How To/Measure Performance]] shows how to measure again.
	- A record chunk holds at most 256 block records and at most 32 KB gzip. The largest chunk measured is 32.0 KB (31,988 bytes); chunk 402, 76 blocks of one podcast transcript at about 940 characters each, was 32.8 KB before chunks closed by size as well as count.
	- The dialog draws block rows whose chunk is still loading with the page title and a placeholder line, and fills in the text when the chunk arrives. A keystroke ends when its result list is drawn, so the phone's 95th percentile on loaded shards is 35 ms; it was 541 ms while keystrokes waited for record chunks.
	- ## Budgets
		- The budgets are in the `[search]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml). Keystrokes on loaded shards use the `[desktop]` and `[phone]` `search_p95_ms`. A keystroke, and the first page results, end when `lsg` draws the result list; block rows may still show a placeholder line then.
		- | Measure | Budget | `lsg` desktop | `lsg` phone | Logseq desktop | Logseq phone |
		  | --- | --- | ---: | ---: | ---: | ---: |
		  | Title index, gzip | 300 KB | 243 KB | | | |
		  | Title index, decoded | 1.5 MB | 0.7 MB | | | |
		  | Largest shard, gzip (words starting `re`) | 64 KB | 63.5 KB | | | |
		  | Largest record chunk, gzip | 32 KB | 32.0 KB | | | |
		  | Dialog shown after Cmd/Ctrl+K | 100 ms desktop, 300 ms phone | 23 ms | 245 ms | 30 ms | 116 ms |
		  | First page results after Cmd/Ctrl+K | 500 ms desktop, 2,500 ms phone | 110 ms | 2,378 ms | 3,549 ms | 14,369 ms |
		  | Largest download by one keystroke, gzip | 200 KB | 95.1 KB | 78.0 KB | 0 | 0 |
		  | Keystroke that fetches a shard, 95th percentile | 150 ms desktop, 1,000 ms phone | 46 ms | 578 ms | – | – |
		  | Keystroke on loaded shards, 95th percentile | 50 ms desktop, 100 ms phone | 16 ms | 35 ms | 1,692 ms | 13,839 ms |
		  | Heap growth after the query list | 30 MB on phone | 6.7 MB | 6.5 MB | 59.1 MB | 59.2 MB |
		  | Whole-garden build, with `--search` against without | at most +25% | +17% (13.3 s against 11.4 s) | | | |
		- The first open downloads 281 KB gzip on the phone profile in five files: the dialog module, the search engine, the manifest, the title index and the first query's shard. On desktop two record chunks also start downloading before the list is drawn, 306 KB in seven files. Nothing loads before Cmd/Ctrl+K or a pointer or focus on the search button. Logseq downloads nothing more when its dialog opens, because the whole graph is already in memory. The query list in [`scripts/parity/search-queries.txt`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/search-queries.txt) types 36 keystrokes; 7 of them fetch a shard.
		- One phone keystroke that downloaded nothing took 262 ms, which is why the column for those keystrokes on [[lsg/Ref/Logseq Compatibility/Performance]] reads 262 ms; the 95th percentile of all 29 keystrokes on loaded shards is 35 ms. Build times are the fastest of three publishes, since other work on the machine changed single timings by more than the budget.
	- ## Comparison with Pagefind
		- These numbers come from the earlier measurement of `lsg` `a2284e0`, before the grouped dialog and size-limited chunks. [Pagefind](https://github.com/CloudCannon/pagefind) 1.5.2 indexed the same `lsg` build, the 13,468 page files without the header and left sidebar, with [`experiments/pagefind`](https://github.com/codekiln/logseq-gardener/tree/main/experiments/pagefind). Its keystrokes are timed one after another, each searching and loading the first ten results.
		- | Measure | `lsg` | Pagefind |
		  | --- | ---: | ---: |
		  | Whole index, gzip | 7.2 MB in 1,599 files | 18.9 MB in 13,713 files |
		  | First open or first search, gzip | 275 KB | 299 KB |
		  | First open or first search, desktop / phone | 108 / 2,343 ms | 316 / 2,229 ms |
		  | Keystrokes that download something, of 36 | 10 | 26 |
		  | Largest download by one keystroke, gzip | 75.6 KB | 103.7 KB |
		  | Keystroke 95th percentile, desktop / phone | 20 / 653 ms | 75 / 739 ms |
		- `lsg`'s own index is within Pagefind's numbers on every measure but the phone first open, where both take about 2.3 s. Pagefind's index is larger because it stores a fragment per page with its whole text, and it downloads on most keystrokes because its index chunks split words across many files. Pagefind also ranks whole pages by relevance, without Logseq's groups, aliases, breadcrumbs or block anchors, so it could not replace `lsg`'s index in any case. Pagefind's main-thread heap grew by 0.7 MB, but it searches in a Web Worker whose heap this measurement does not include.
