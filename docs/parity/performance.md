# Parity performance

- measured: 2026-10-26
- garden commit: 0ce74d6
- gh-pages commit: 8132c95
- lsg: 0.0.0 (b490d68); the build time and Hierarchy: the Hierarchy change on c373d90 against garden commit 083106d, measured `lsg` only; reference sections: 84e9278; the All pages rows and section: the All pages change on 48fcc54 with garden commit 083106d, measured `lsg` only; the search sections: search format version 2 on a2284e0 with garden commit 75ef84c and `gh-pages` b491e00 on 2026-10-27, both builds, from `parity:perf --search-site` (see [Search performance](search-performance.md))
- profiles: desktop, phone
- loads per page, logseq: 1
- loads per page, lsg: 3

Medians of the page loads. **Bold** marks an `lsg` value over its budget in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).

## Page loads

| Page | Profile | Build | Title visible (ms) | First contentful paint (ms) | Data before title (KB, gzip) | Script before title (KB) | Peak JS heap (MB) |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| tmux | desktop | logseq | 21,952 | 22,000 | 22,643 | 8,087 | 2,013 |
| tmux | desktop | lsg | 35.9 | 48.0 | **101** | 14.6 | 2.0 |
| tmux | phone | logseq | 204,376 | 204,568 | 22,067 | 8,087 | 1,989 |
| tmux | phone | lsg | 234 | 200 | **101** | 14.6 | 1.9 |
| Foam | desktop | logseq | 23,614 | 23,628 | 22,643 | 8,087 | 1,988 |
| Foam | desktop | lsg | 21.6 | 32.0 | 13.3 | 14.6 | 1.8 |
| Foam | phone | logseq | 196,789 | 196,820 | 22,067 | 8,087 | 1,975 |
| Foam | phone | lsg | 244 | 208 | 13.3 | 14.6 | 1.9 |
| Logseq/Block/Id | desktop | logseq | 17,716 | 17,736 | 22,643 | 8,087 | 1,976 |
| Logseq/Block/Id | desktop | lsg | 22.9 | 32.0 | 11.8 | 14.6 | 1.8 |
| Logseq/Block/Id | phone | logseq | 184,889 | 184,916 | 22,067 | 8,087 | 1,949 |
| Logseq/Block/Id | phone | lsg | 239 | 248 | 11.8 | 14.6 | 1.8 |
| Logseq/Queries | desktop | logseq | 19,760 | 19,772 | 22,643 | 8,087 | 1,969 |
| Logseq/Queries | desktop | lsg | 21.8 | 28.0 | 11.5 | 14.6 | 1.8 |
| Logseq/Queries | phone | logseq | 185,344 | 185,372 | 22,067 | 8,087 | 1,974 |
| Logseq/Queries | phone | lsg | 243 | 244 | 11.5 | 14.6 | 1.8 |
| Lazygit | desktop | logseq | 23,806 | 23,824 | 22,643 | 8,087 | 1,997 |
| Lazygit | desktop | lsg | 27.2 | 40.0 | 30.9 | 14.6 | 2.1 |
| Lazygit | phone | logseq | 194,248 | 194,292 | 22,067 | 8,087 | 1,974 |
| Lazygit | phone | lsg | 226 | 240 | 30.9 | 14.6 | 2.2 |
| rw-cli | desktop | logseq | 21,022 | 21,036 | 22,958 | 9,096 | 2,008 |
| rw-cli | desktop | lsg | 27.7 | 40.0 | 16.0 | 14.6 | 1.8 |
| rw-cli | phone | logseq | 192,159 | 192,204 | 22,067 | 8,087 | 1,983 |
| rw-cli | phone | lsg | 252 | 212 | 16.0 | 14.6 | 1.9 |
| ff | desktop | logseq | 17,395 | 17,412 | 22,958 | 9,096 | 2,021 |
| ff | desktop | lsg | 27.9 | 40.0 | 14.5 | 14.6 | 1.8 |
| ff | phone | logseq | 196,794 | 196,844 | 22,067 | 8,087 | 1,981 |
| ff | phone | lsg | 238 | 200 | 14.5 | 14.6 | 1.9 |
| Contrando | desktop | logseq | 22,494 | 22,504 | 22,643 | 8,087 | 2,007 |
| Contrando | desktop | lsg | 22.0 | 28.0 | 10.7 | 14.6 | 1.8 |
| Contrando | phone | logseq | 191,461 | 191,484 | 22,067 | 8,087 | 1,959 |
| Contrando | phone | lsg | 253 | 212 | 10.7 | 14.6 | 1.8 |
| AI/E/WS/25/d1 | desktop | logseq | 17,595 | 17,612 | 22,643 | 8,087 | 2,002 |
| AI/E/WS/25/d1 | desktop | lsg | 25.4 | 36.0 | 12.3 | 14.6 | 1.8 |
| AI/E/WS/25/d1 | phone | logseq | 190,372 | 190,412 | 22,067 | 8,087 | 1,961 |
| AI/E/WS/25/d1 | phone | lsg | 240 | 204 | 12.3 | 14.6 | 1.8 |
| GitP/A/Session/24/11/19-Tue | desktop | logseq | 18,885 | 18,904 | 22,643 | 8,087 | 1,989 |
| GitP/A/Session/24/11/19-Tue | desktop | lsg | 26.7 | 36.0 | 12.7 | 14.6 | 1.8 |
| GitP/A/Session/24/11/19-Tue | phone | logseq | 186,104 | 186,160 | 22,067 | 8,087 | 1,978 |
| GitP/A/Session/24/11/19-Tue | phone | lsg | 252 | 256 | 12.7 | 14.6 | 1.9 |
| 2026-10-23 Fri | desktop | logseq | 22,266 | 22,280 | 22,643 | 8,087 | 1,972 |
| 2026-10-23 Fri | desktop | lsg | 21.6 | 32.0 | 11.8 | 14.6 | 1.8 |
| 2026-10-23 Fri | phone | logseq | 188,766 | 188,796 | 22,067 | 8,087 | 1,948 |
| 2026-10-23 Fri | phone | lsg | 234 | 200 | 11.8 | 14.6 | 1.8 |
| AI/Coding/History | desktop | logseq | 19,211 | 19,264 | 22,643 | 8,087 | 1,982 |
| AI/Coding/History | desktop | lsg | 23.7 | 36.0 | 15.4 | 14.6 | 1.8 |
| AI/Coding/History | phone | logseq | 188,363 | 188,568 | 22,067 | 8,087 | 1,960 |
| AI/Coding/History | phone | lsg | 231 | 240 | 15.4 | 14.6 | 1.8 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | desktop | logseq | 23,196 | 23,208 | 23,062 | 9,096 | 2,028 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | desktop | lsg | 35.3 | 48.0 | 32.1 | 14.6 | 2.0 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | phone | logseq | 188,662 | 188,684 | 22,172 | 8,087 | 1,948 |
| VSCode/Proj/24t3/LearnVSCode/03 Basic Python | phone | lsg | 235 | 248 | 32.1 | 14.6 | 1.8 |
| Home | desktop | logseq | 20,937 | 20,952 | 22,643 | 8,087 | 1,991 |
| Home | desktop | lsg | 21.0 | 32.0 | 12.3 | 14.6 | 1.7 |
| Home | phone | logseq | 204,698 | 204,736 | 22,067 | 8,087 | 1,838 |
| Home | phone | lsg | 242 | 256 | 12.3 | 14.6 | 1.8 |
| All pages | desktop | logseq | 19,820 | 19,828 | 22,643 | 8,087 | 2,019 |
| All pages | desktop | lsg | 23.1 | 36.0 | 13.2 | 14.4 | 1.8 |
| All pages | phone | logseq | 197,866 | 197,876 | 22,067 | 8,087 | 1,982 |
| All pages | phone | lsg | 235 | 248 | 13.2 | 14.4 | 1.9 |

## Journals feed after 10 later batches

The home page scrolled until it shows 73 journals or runs out. The phone heap budget applies to `lsg`.

| Profile | Build | Journals shown | Peak JS heap (MB) | Time to load the batches (ms) |
| --- | --- | ---: | ---: | ---: |
| desktop | logseq | 3 | 1,998.3 | 30,046 |
| desktop | lsg | 73 | 4.6 | 331 |
| phone | logseq | 3 | 1,988.8 | 30,045 |
| phone | lsg | 73 | 3.1 | 3,601 |

## Whole-garden build time

`lsg publish` with the run's options, once without and once with `--journals`.

| Build | Seconds |
| --- | ---: |
| without journals | 10.4 |
| with journals | 10.5 |

Against the baseline `lsg` with the same options: 10.8 seconds, this build 10.5 seconds (-3%).

## Block zoom on tmux

Time from clicking the bullet of a nested block halfway down the page to the zoomed view.

| Profile | Build | Zoom (ms) |
| --- | --- | ---: |
| desktop | logseq | 77 |
| desktop | lsg | 20 |
| phone | logseq | 612 |
| phone | lsg | 23 |

## Reference sections on mise

Time from clicking a section header to its first group, time from clicking a filter chip to the new count, and the peak JS heap after opening Linked and Unlinked References and scrolling both to the end.

| Profile | Open linked (ms) | Filter change (ms) | Open unlinked (ms) | Groups loaded | Peak JS heap (MB) |
| --- | ---: | ---: | ---: | ---: | ---: |
| desktop | 22 | 46 | 12 | 449 | 4.1 |
| phone | 346 | 96 | 243 | 449 | 3.4 |

## All pages

Time from clicking `Page name` and then `Backlinks` to the table sorted by Backlinks, descending, which loads the page list; the first row; search on Enter for 20 queries once the list has loaded; and the peak JS heap. The page list is 172.9 KB gzip and was not requested before the first click. Logseq's screen needs the whole graph before its title (Page loads above).

| Profile | Sort by Backlinks (ms) | First row | Search median (ms) | Search 95th percentile (ms) | Peak JS heap (MB) |
| --- | ---: | --- | ---: | ---: | ---: |
| desktop | 57 | Microfreak | 9 | 11 | 8.6 |
| phone | 1,069 | Microfreak | 36 | 41 | 13.9 |

Whole-garden build with `--journals`: 10.5 seconds with this change, 11.0 seconds with the baseline built from 48fcc54; an earlier pair measured 10.6 and 10.5 seconds (+1.7%), within the 10% budget.

## Hierarchy on Microfreak

Rows in the page before scrolling, rows after scrolling the list to its end, and the peak JS heap.

| Profile | Rows in the page | Rows after scrolling | Peak JS heap (MB) |
| --- | ---: | ---: | ---: |
| desktop | 100 | 1,394 | 3.0 |
| phone | 100 | 1,394 | 3.1 |

## Search index

`lsg`'s search files for 9,204 pages and 114,926 blocks: one title index, 1,148 word shards and 450 record chunks. **Bold** marks a value over its `[search]` budget.

| File | Size | Budget |
| --- | ---: | ---: |
| Title index (KB, gzip) | 243 | 300 |
| Title index (MB, decoded) | 0.7 | 1.5 |
| Largest shard, words starting `re` (KB, gzip) | 63.5 | 64 |
| Largest record chunk (KB, gzip) | 32.0 | 32 |
| Search starter (KB) | 2.0 | 2 |

## Search

Cmd/Ctrl+K on the home page, then the first query typed at once: the time from the key to the dialog and to its page results, and what that first open downloaded. Then each query in [`search-queries.txt`](../../scripts/parity/search-queries.txt) typed a key every 300 ms. A cold keystroke fetches a word shard; a warm one does not. The heap growth is measured after the whole query list, after garbage collection. **Bold** marks an `lsg` value over its budget.

| Profile | Build | Dialog shown (ms) | First page results (ms) | First open (KB, gzip) | Cold keystrokes | Cold 95th percentile (ms) | Warm 95th percentile (ms) | Largest keystroke download (KB, gzip) | Heap growth (MB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| desktop | logseq | 30 | 3,549 | 0.0 | 0 | – | 1,692 | 0.0 | 59.1 |
| desktop | lsg | 23 | 110 | 306.1 | 7 | 46 | 16 | 95.1 | 6.7 |
| phone | logseq | 116 | 14,369 | 160.2 | 0 | – | 13,839 | 0.0 | 59.2 |
| phone | lsg | 245 | 2,378 | 280.6 | 7 | 578 | 35 | 78.0 | 6.5 |

## Search latency per keystroke

Every keystroke of the query list. An unanswered keystroke is one whose results never showed. An `lsg` keystroke ends when its result list is drawn; block rows whose record chunk is still loading show a placeholder line until it arrives. The keystrokes that fetched nothing show the cost of the query alone. `lsg` reopens the dialog from memory after the first open.

| Profile | Build | Keystrokes | Unanswered | Median (ms) | 95th percentile (ms) | Warm keystrokes that fetched a chunk | 95th percentile of keystrokes that fetched nothing (ms) | Reopen (ms) | Reopen requests |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| desktop | logseq | 36 | 0 | 803 | 1,692 | 0 | 1,692 | – | – |
| desktop | lsg | 36 | 0 | 14 | 30 | 11 | 20 | 18 | 0 |
| phone | logseq | 36 | 0 | 3,026 | 13,839 | 1 | 13,839 | – | – |
| phone | lsg | 36 | 0 | 26 | 286 | 12 | 262 | 26 | 0 |

Whole-garden build with journals, the fastest of 3 runs: 13.3 seconds with `--search`, 11.4 seconds without (+17%; budget +25%).
