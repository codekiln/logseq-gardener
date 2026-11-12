tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/How To/Measure Performance]], [[lsg/Ref/Logseq Compatibility/Search Performance]], [[lsg/Ref/Publication Speed]]

- # Performance
	- How long the fixture pages of the encode garden take to show their titles in `lsg`'s build and in Logseq's, what they download and how much memory they use, on a desktop profile and a throttled phone profile, with build times and the timings of features that load after the page. `mise run parity:perf --record` writes this page.
	  id:: f45eb369-6fff-46e9-a147-cd0ca9c03fc9
	- measured: 2026-11-09
	- garden commit: 66b81e9
	- gh-pages commit: 8132c95
	- lsg: 0.0.46 (c4108d6)
	- profiles: desktop, phone
	- loads per page, logseq: 1
	- loads per page, lsg: 3
	- Medians of the page loads. **Bold** marks an `lsg` value over its budget in [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml).
	- ## Page loads
		- | Page | Profile | Build | Title visible (ms) | First contentful paint (ms) | Data before title (KB, gzip) | Script before title (KB) | Peak JS heap (MB) |
		  | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
		  | tmux | desktop | logseq | 22,145 | 22,200 | 22,639 | 8,087 | 2,009 |
		  | tmux | desktop | lsg | 17.3 | 28.0 | 18.0 | 11.6 | 1.8 |
		  | tmux | phone | logseq | 182,574 | 182,744 | 22,064 | 8,087 | 1,968 |
		  | tmux | phone | lsg | 395 | 420 | 18.0 | 11.6 | 1.9 |
		  | Foam | desktop | logseq | 19,673 | 19,684 | 22,639 | 8,087 | 1,985 |
		  | Foam | desktop | lsg | 17.1 | 28.0 | 14.1 | 11.6 | 1.8 |
		  | Foam | phone | logseq | 182,003 | 182,032 | 22,064 | 8,087 | 1,947 |
		  | Foam | phone | lsg | 394 | 416 | 14.1 | 11.6 | 1.9 |
		  | Logseq/Block/Id | desktop | logseq | 18,743 | 18,756 | 22,639 | 8,087 | 1,977 |
		  | Logseq/Block/Id | desktop | lsg | 16.4 | 28.0 | 14.0 | 11.6 | 1.8 |
		  | Logseq/Block/Id | phone | logseq | 179,243 | 179,268 | 22,064 | 8,087 | 1,978 |
		  | Logseq/Block/Id | phone | lsg | 395 | 416 | 14.0 | 11.6 | 1.9 |
		  | Logseq/Queries | desktop | logseq | 17,846 | 17,856 | 22,639 | 8,087 | 1,970 |
		  | Logseq/Queries | desktop | lsg | 16.9 | 28.0 | 15.2 | 13.6 | 1.8 |
		  | Logseq/Queries | phone | logseq | 180,141 | 180,164 | 22,064 | 8,087 | 1,950 |
		  | Logseq/Queries | phone | lsg | 402 | 424 | 15.2 | 13.6 | 1.9 |
		  | Lazygit | desktop | logseq | 18,370 | 18,388 | 22,639 | 8,087 | 2,001 |
		  | Lazygit | desktop | lsg | 20.2 | 28.0 | 15.2 | 11.6 | 1.8 |
		  | Lazygit | phone | logseq | 197,945 | 198,004 | 22,064 | 8,087 | 1,952 |
		  | Lazygit | phone | lsg | 412 | 432 | 15.2 | 11.6 | 1.9 |
		  | rw-cli | desktop | logseq | 23,292 | 23,304 | 22,954 | 9,096 | 2,009 |
		  | rw-cli | desktop | lsg | 21.5 | 48.0 | 17.6 | 11.6 | 1.8 |
		  | rw-cli | phone | logseq | 186,479 | 186,512 | 22,064 | 8,087 | 1,952 |
		  | rw-cli | phone | lsg | 410 | 476 | 17.6 | 11.6 | 1.9 |
		  | ff | desktop | logseq | 20,365 | 20,380 | 22,954 | 9,096 | 2,026 |
		  | ff | desktop | lsg | 17.6 | 28.0 | 14.7 | 11.6 | 1.8 |
		  | ff | phone | logseq | 185,596 | 185,636 | 22,064 | 8,087 | 1,942 |
		  | ff | phone | lsg | 410 | 432 | 14.7 | 11.6 | 1.9 |
		  | Contrando | desktop | logseq | 19,463 | 19,472 | 22,639 | 8,087 | 2,000 |
		  | Contrando | desktop | lsg | 18.6 | 32.0 | 15.2 | 14.0 | 1.8 |
		  | Contrando | phone | logseq | 176,641 | 176,664 | 22,639 | 8,087 | 1,960 |
		  | Contrando | phone | lsg | 418 | 448 | 15.2 | 14.0 | 1.9 |
		  | Deno Deploy | desktop | logseq | 20,724 | 20,732 | 22,639 | 8,087 | 1,980 |
		  | Deno Deploy | desktop | lsg | 18.8 | 32.0 | 13.0 | 11.6 | 1.8 |
		  | Deno Deploy | phone | logseq | 186,162 | 186,180 | 22,064 | 8,087 | 1,946 |
		  | Deno Deploy | phone | lsg | 410 | 436 | 13.0 | 11.6 | 1.8 |
		  | AI/E/WS/25/d1 | desktop | logseq | 18,924 | 18,936 | 22,639 | 8,087 | 2,005 |
		  | AI/E/WS/25/d1 | desktop | lsg | 19.2 | 32.0 | 17.2 | 14.0 | 1.8 |
		  | AI/E/WS/25/d1 | phone | logseq | 190,123 | 190,164 | 22,639 | 8,087 | 1,961 |
		  | AI/E/WS/25/d1 | phone | lsg | 418 | 440 | 17.2 | 14.0 | 1.9 |
		  | GitP/A/Session/24/11/19-Tue | desktop | logseq | 20,017 | 20,032 | 22,639 | 8,087 | 1,999 |
		  | GitP/A/Session/24/11/19-Tue | desktop | lsg | 16.8 | 28.0 | 14.3 | 11.6 | 1.8 |
		  | GitP/A/Session/24/11/19-Tue | phone | logseq | 188,438 | 188,496 | 22,064 | 8,087 | 1,964 |
		  | GitP/A/Session/24/11/19-Tue | phone | lsg | 402 | 428 | 14.3 | 11.6 | 1.9 |
		  | 2026-10-23 Fri | desktop | logseq | 21,809 | 21,820 | 22,639 | 8,087 | 1,992 |
		  | 2026-10-23 Fri | desktop | lsg | 17.1 | 32.0 | 14.0 | 11.6 | 1.8 |
		  | 2026-10-23 Fri | phone | logseq | 189,555 | 189,588 | 22,064 | 8,087 | 1,955 |
		  | 2026-10-23 Fri | phone | lsg | 409 | 444 | 14.0 | 11.6 | 1.9 |
		  | AI/Coding/History | desktop | logseq | 20,578 | 20,632 | 22,639 | 8,087 | 1,982 |
		  | AI/Coding/History | desktop | lsg | 21.7 | 44.0 | 17.9 | 11.6 | 1.8 |
		  | AI/Coding/History | phone | logseq | 188,644 | 188,840 | 22,064 | 8,087 | 1,960 |
		  | AI/Coding/History | phone | lsg | 411 | 464 | 17.9 | 11.6 | 1.9 |
		  | VSCode/Proj/24t3/LearnVSCode/03 Basic Python | desktop | logseq | 21,937 | 21,948 | 23,059 | 9,096 | 2,030 |
		  | VSCode/Proj/24t3/LearnVSCode/03 Basic Python | desktop | lsg | 17.9 | 28.0 | 23.7 | 11.6 | 2.0 |
		  | VSCode/Proj/24t3/LearnVSCode/03 Basic Python | phone | logseq | 203,787 | 203,808 | 22,064 | 8,087 | 1,980 |
		  | VSCode/Proj/24t3/LearnVSCode/03 Basic Python | phone | lsg | 410 | 432 | 23.7 | 11.6 | 2.0 |
		  | Home | desktop | logseq | 23,007 | 23,020 | 22,639 | 8,087 | 1,981 |
		  | Home | desktop | lsg | 21.6 | 40.0 | 16.6 | 11.6 | 1.9 |
		  | Home | phone | logseq | 190,900 | 190,924 | 22,064 | 8,087 | 1,983 |
		  | Home | phone | lsg | 418 | 472 | 16.6 | 11.6 | 1.9 |
		  | All pages | desktop | logseq | 22,713 | 22,720 | 22,639 | 8,087 | 2,023 |
		  | All pages | desktop | lsg | 31.5 | 48.0 | 17.5 | **17.0** | 1.9 |
		  | All pages | phone | logseq | 193,323 | 193,332 | 22,064 | 8,087 | 1,982 |
		  | All pages | phone | lsg | 426 | 460 | 17.5 | **17.0** | 1.9 |
		  | AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025 | desktop | logseq | 20,559 | 20,588 | 22,954 | 9,096 | 2,047 |
		  | AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025 | desktop | lsg | 18.1 | 28.0 | 28.1 | 14.0 | 1.6 |
		  | AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025 | phone | logseq | 195,653 | 195,744 | 22,954 | 9,096 | 1,966 |
		  | AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025 | phone | lsg | 418 | 440 | 28.1 | 14.0 | 2.0 |
		  | Logseq/Doc/slide | desktop | logseq | 20,159 | 20,172 | 22,639 | 8,087 | 1,968 |
		  | Logseq/Doc/slide | desktop | lsg | 20.2 | 40.0 | 13.7 | 11.6 | 1.8 |
		  | Logseq/Doc/slide | phone | logseq | 185,937 | 185,960 | 22,064 | 8,087 | 1,952 |
		  | Logseq/Doc/slide | phone | lsg | 412 | 452 | 13.7 | 11.6 | 1.9 |
		  | Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs | desktop | logseq | 20,500 | 20,516 | 22,639 | 8,087 | 2,010 |
		  | Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs | desktop | lsg | 18.8 | 32.0 | 20.1 | 14.0 | 1.9 |
		  | Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs | phone | logseq | 186,681 | 186,740 | 22,639 | 8,087 | 1,957 |
		  | Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs | phone | lsg | 418 | 440 | 20.1 | 14.0 | 1.9 |
		  | Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton | desktop | logseq | 21,385 | 21,404 | 22,639 | 8,087 | 2,009 |
		  | Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton | desktop | lsg | 20.1 | 32.0 | 94.4 | 14.0 | 1.8 |
		  | Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton | phone | logseq | 191,170 | 191,224 | 22,639 | 8,087 | 1,966 |
		  | Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton | phone | lsg | 424 | 448 | 94.4 | 14.0 | 2.0 |
		  | Logseq/Flashcard/Review/Favorite/yazi | desktop | logseq | 21,901 | 21,912 | 22,639 | 8,087 | 1,972 |
		  | Logseq/Flashcard/Review/Favorite/yazi | desktop | lsg | 21.0 | 40.0 | 14.4 | 11.6 | 1.8 |
		  | Logseq/Flashcard/Review/Favorite/yazi | phone | logseq | 191,711 | 191,744 | 22,064 | 8,087 | 1,977 |
		  | Logseq/Flashcard/Review/Favorite/yazi | phone | lsg | 418 | 464 | 14.4 | 11.6 | 1.9 |
		  | Logseq/Flashcard/Review/Favorite/Zoxide | desktop | logseq | 20,678 | 20,688 | 22,639 | 8,087 | 1,986 |
		  | Logseq/Flashcard/Review/Favorite/Zoxide | desktop | lsg | 18.1 | 36.0 | 13.9 | 11.6 | 1.8 |
		  | Logseq/Flashcard/Review/Favorite/Zoxide | phone | logseq | 188,207 | 188,232 | 22,064 | 8,087 | 1,961 |
		  | Logseq/Flashcard/Review/Favorite/Zoxide | phone | lsg | 411 | 452 | 13.9 | 11.6 | 1.8 |
		  | Logseq/Flashcard/Review/Favorite | desktop | logseq | 26,105 | 26,124 | 22,639 | 8,087 | 1,995 |
		  | Logseq/Flashcard/Review/Favorite | desktop | lsg | 17.2 | 28.0 | 15.6 | 11.6 | 1.8 |
		  | Logseq/Flashcard/Review/Favorite | phone | logseq | 206,111 | 206,160 | 22,064 | 8,087 | 959 |
		  | Logseq/Flashcard/Review/Favorite | phone | lsg | 410 | 436 | 15.6 | 11.6 | 1.9 |
		  | Logseq/Flashcard | desktop | logseq | 26,310 | 26,328 | 22,639 | 8,087 | 972 |
		  | Logseq/Flashcard | desktop | lsg | 27.9 | 44.0 | 16.1 | 11.6 | 1.8 |
		  | Logseq/Flashcard | phone | logseq | 217,337 | 217,404 | 22,064 | 8,087 | 960 |
		  | Logseq/Flashcard | phone | lsg | 417 | 440 | 16.1 | 11.6 | 1.9 |
		  | tmux/Keyshort/Pane/Zoom Pane | desktop | logseq | 17,291 | 17,304 | 22,639 | 8,087 | 1,993 |
		  | tmux/Keyshort/Pane/Zoom Pane | desktop | lsg | 20.1 | 40.0 | 13.6 | 11.6 | 1.8 |
		  | tmux/Keyshort/Pane/Zoom Pane | phone | logseq | 196,482 | 196,504 | 22,064 | 8,087 | 1,972 |
		  | tmux/Keyshort/Pane/Zoom Pane | phone | lsg | 410 | 452 | 13.6 | 11.6 | 1.9 |
		  | tmux/Keyshort/Copy Mode | desktop | logseq | 21,539 | 21,552 | 22,639 | 8,087 | 1,986 |
		  | tmux/Keyshort/Copy Mode | desktop | lsg | 16.8 | 28.0 | 14.4 | 11.6 | 1.8 |
		  | tmux/Keyshort/Copy Mode | phone | logseq | 191,285 | 191,320 | 22,064 | 8,087 | 1,950 |
		  | tmux/Keyshort/Copy Mode | phone | lsg | 412 | 436 | 14.4 | 11.6 | 1.9 |
		  | Logseq/Flavored Markdown | desktop | logseq | 20,114 | 20,128 | 22,639 | 8,087 | 1,976 |
		  | Logseq/Flavored Markdown | desktop | lsg | 16.8 | 28.0 | 13.8 | 11.6 | 1.8 |
		  | Logseq/Flavored Markdown | phone | logseq | 176,294 | 176,324 | 22,064 | 8,087 | 1,973 |
		  | Logseq/Flavored Markdown | phone | lsg | 403 | 428 | 13.8 | 11.6 | 1.9 |
		  | LangChain/Academy/LangSmith/M5 Production Observability/L2 Online Evaluation | desktop | logseq | 17,503 | 17,520 | 22,954 | 9,096 | 2,024 |
		  | LangChain/Academy/LangSmith/M5 Production Observability/L2 Online Evaluation | desktop | lsg | 21.8 | 48.0 | 15.6 | 11.6 | 1.8 |
		  | LangChain/Academy/LangSmith/M5 Production Observability/L2 Online Evaluation | phone | logseq | 186,125 | 186,180 | 22,064 | 8,087 | 1,945 |
		  | LangChain/Academy/LangSmith/M5 Production Observability/L2 Online Evaluation | phone | lsg | 410 | 472 | 15.6 | 11.6 | 1.9 |
		  | VSCode/Proj/24t3/LearnVSCode/01 Terminal | desktop | logseq | 21,118 | 21,132 | 22,639 | 8,087 | 1,991 |
		  | VSCode/Proj/24t3/LearnVSCode/01 Terminal | desktop | lsg | 17.5 | 28.0 | 14.1 | 11.6 | 1.8 |
		  | VSCode/Proj/24t3/LearnVSCode/01 Terminal | phone | logseq | 187,949 | 187,984 | 22,064 | 8,087 | 1,969 |
		  | VSCode/Proj/24t3/LearnVSCode/01 Terminal | phone | lsg | 410 | 436 | 14.1 | 11.6 | 1.9 |
		  | Contents | desktop | logseq | 20,829 | 20,836 | 22,639 | 8,087 | 1,980 |
		  | Contents | desktop | lsg | 22.9 | 28.0 | 18.4 | **26.6** | 1.8 |
		  | Contents | phone | logseq | 182,376 | 182,396 | 22,064 | 8,087 | 1,964 |
		  | Contents | phone | lsg | 409 | 432 | 12.9 | 11.6 | 1.8 |
		  | Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture | desktop | logseq | 20,974 | 20,984 | 22,639 | 8,087 | 2,000 |
		  | Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture | desktop | lsg | 19.6 | 32.0 | 16.3 | 14.0 | 1.8 |
		  | Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture | phone | logseq | 189,934 | 189,964 | 22,064 | 8,087 | 1,943 |
		  | Art/Sound/Idea/Vocal/Tract Simulation with Motion Capture | phone | lsg | 417 | 440 | 16.3 | 14.0 | 1.9 |
	- ## Site size
		- Files and sizes as written, before any server compresses them. GitHub Pages serves at most 1 GB. **Bold** marks a size over its `[site]` budget. Measured 2026-11-09, `lsg` 0.0.46 (c4108d6) on garden commit 66b81e9.
		- | Build | Part | Files | Size (MB) |
		  | --- | --- | ---: | ---: |
		  | logseq | Whole site | 506 | 227.0 |
		  | lsg | Whole site | 65,778 | 576.2 |
		  | lsg | Page HTML at the site root | 19,501 | 252.2 |
		  | lsg | `_refs/` | 23,938 | 103.2 |
		  | lsg | `assets/` | 112 | 165.2 |
		  | lsg | Search files | 3,188 | 30.2 |
		  | lsg | `_graph/` | 18,959 | 20.7 |
		  | lsg | Everything else | 80 | 4.8 |
	- ## Journals feed after 10 later batches
		- The home page scrolled until it shows 73 journals or runs out. The phone heap budget applies to `lsg`.
		- | Profile | Build | Journals shown | Peak JS heap (MB) | Time to load the batches (ms) |
		  | --- | --- | ---: | ---: | ---: |
		  | desktop | logseq | 3 | 2,002.7 | 30,137 |
		  | desktop | lsg | 73 | 4.3 | 451 |
		  | phone | logseq | 10 | 2,005.8 | 31,680 |
		  | phone | lsg | 73 | 4.4 | 3,822 |
	- ## Whole-garden build time
		- `lsg publish` with the run's options, then without `--journals` and without `--search`, each the fastest of 3 runs. **Bold** marks a build more than the `[journals]` or `[search]` `build_slowdown_pct` above the build without that feature.
		- | Build | Seconds |
		  | --- | ---: |
		  | with journals, with search | 19.0 |
		  | with journals, without search | 18.6 |
		  | without journals, with search | 18.6 |
	- ## Block zoom on tmux
		- Time from clicking the bullet of a nested block halfway down the page to the zoomed view.
		- | Profile | Build | Zoom (ms) |
		  | --- | --- | ---: |
		  | desktop | logseq | 164 |
		  | desktop | lsg | 19 |
		  | phone | logseq | 283 |
		  | phone | lsg | 19 |
	- ## Reference sections on mise
		- Time from clicking a section header to its first group, time from clicking a filter chip to the new count, and the peak JS heap after opening Linked and Unlinked References and scrolling both to the end.
		- | Profile | Open linked (ms) | Filter change (ms) | Open unlinked (ms) | Groups loaded | Peak JS heap (MB) |
		  | --- | ---: | ---: | ---: | ---: | ---: |
		  | desktop | 19 | **56** | 9 | 1,204 | 5.4 |
		  | phone | 374 | 99 | 246 | 1,204 | 5.2 |
	- ## All pages
		- Time from clicking `Page name` and then `Backlinks` to the table sorted by Backlinks, descending, which loads the page list; the first row; search on Enter for each query once the list has loaded, and the peak JS heap. The page list is 232.7 KB gzip.
		- | Profile | Sort by Backlinks (ms) | First row | Search median (ms) | Search 95th percentile (ms) | Peak JS heap (MB) |
		  | --- | ---: | --- | ---: | ---: | ---: |
		  | desktop | 71 | Microfreak | 12 | 13 | 15.6 |
		  | phone | 1,384 | Microfreak | 48 | 55 | 15.8 |
	- ## Hierarchy on Microfreak
		- Rows in the page before scrolling, rows after scrolling the list to its end, and the peak JS heap.
		- | Profile | Rows in the page | Rows after scrolling | Peak JS heap (MB) |
		  | --- | ---: | ---: | ---: |
		  | desktop | 100 | 1,396 | 2.2 |
		  | phone | 100 | 1,396 | 2.1 |
	- ## Right sidebar on tmux
		- On `tmux` at 1280 by 800, with a cleared sidebar: Shift-click a page link, the first bullet, a block reference and then the next 10 distinct page links, one after another, each timed from the click to the card's outline drawn; the first click includes loading `lsg-sidebar.js`. `tmux` has no block reference, so `parity:perf` adds a link to the first one it finds in the build. Transfer is the gzip size of the page files a card fetched; the heap is measured after the last card and garbage collection. Below 640px there is no sidebar, so there is no phone row. **Bold** marks a value over its `[sidebar]` budget.
		- | Profile | Cards | Shift-click to card, p95 (ms) | JS heap after the cards (MB) | `lsg-sidebar.js` (KB, gzip) | Largest card transfer (KB, gzip) |
		  | --- | ---: | ---: | ---: | ---: | ---: |
		  | desktop | 13 | 58 | 2.2 | 6.2 | 13.9 |
		- | Card | Type | Shift-click to card (ms) | Transfer (KB, gzip) |
		  | --- | --- | ---: | ---: |
		  | GitHub/Star | page | 58 | 11.3 |
		  | tmux | block | 20 | 13.9 |
		  | Block references | refs | 29 | 13.7 |
		  | 2026-07-04 Sat | page | 9 | 11.2 |
		  | tmux/Idea/Pomodoro with status line and silent flash | page | 19 | 10.3 |
		  | tmux/Idea/Toggl Track status in status line | page | 10 | 10.2 |
		  | tmux/Config | page | 9 | 9.6 |
		  | Catppuccin | page | 8 | 11.1 |
		  | oh-my-tmux | page | 19 | 11.4 |
		  | Person/Grégory Pakosz | page | 18 | 9.9 |
		  | tmux/Docs/Getting Started | page | 9 | 9.6 |
		  | Person/Brian P. Hogan/Book/25/tmux 3 | page | 18 | 9.9 |
		  | tmux/man | page | 8 | 9.5 |
		- 2 published pages exceed the 60 KB card transfer budget as cards and as pages: `Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton` (88.1 KB gzip), `Ableton/Live/UG/29 Audio Effect Reference` (62.3 KB gzip). Their content, not the shell, makes up the size.
	- ## View as slides
		- On `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025` at 1280 by 800: the time from choosing View as slides to the first slide drawn, 95th percentile of 5 fresh windows; the 95th-percentile time from a key to the next slide drawn while stepping through every slide; and the JS heap after that walk, less the heap before opening the deck, both after garbage collection. Phones have no right sidebar and so no Slides card. **Bold** marks a value over its `[slides]` budget.
		- | Profile | Slides | Open (ms) | Step, 95th percentile (ms) | Heap growth (MB) | Slides in the document | `lsg-slides.js` (KB, transfer) |
		  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
		  | desktop | 106 | 83 | 13.9 | 1.5 | 2 | 4.5 |
	- ## Graph view
		- From navigating to `graph.html` (Logseq: `#/graph`) to the first frame with every node of the default view drawn; the 95th-percentile frame time while dragging the background; and the peak JS heap after the first frame, the drag and five wheel zooms. Logseq's desktop row comes from the [[lsg/Ref/Logseq Compatibility/Graph View]], which measured 38 minutes to its first frame; `parity:perf` does not wait that long. **Bold** marks an `lsg` value over its `[graph]` budget.
		- | Profile | Build | First frame (ms) | Pan frame time, 95th percentile (ms) | Peak JS heap (MB) | Graph data (KB, gzip) | Worker loaded |
		  | --- | --- | ---: | ---: | ---: | ---: | --- |
		  | desktop | logseq | 2,291,000 | – | 2,340 | 19,800 (`db.gz`) | – |
		  | desktop | lsg | 76 | 9 | 9.9 | 409 | no |
		  | phone | lsg | 2,871 | 10 | 10.2 | 409 | no |
		- WebGL renderer: `ANGLE (Apple, ANGLE Metal Renderer: Apple M2 Pro, Unspecified Version)`.
		- `lsg`'s graph files: `_graph/default.bin` 409 KB as stored (gzip), `lsg-graph.js` with its styles 11.3 KB gzip, the worker 2.7 KB gzip, 13,467 neighbourhood files, the largest 85.0 KB gzip.
	- ## Page graph
		- Time from clicking `Page graph` in the right sidebar to the card's first frame, and the file it fetched. Below 640px there is no right sidebar, so the phone rows use a 1280px window with the phone profile's CPU and network throttling.
		- | Page | Profile | Open (ms) | File |
		  | --- | --- | ---: | --- |
		  | Foam | desktop | 221 | `p/077e10850d78a1dc.json` |
		  | tmux | desktop | 153 | `p/424036ef4353ec8f.json` |
		  | Person | desktop | 164 | `p/40d76f1f51639ec0.json` |
		  | Foam | phone | 435 | `p/077e10850d78a1dc.json` |
		  | tmux | phone | 578 | `p/424036ef4353ec8f.json` |
		  | Person | phone | 821 | `p/40d76f1f51639ec0.json` |
		- Whole-garden build with the run's options, the fastest of 3 runs: 15.3 seconds with the graph, 12.9 seconds with the baseline `lsg` (+19%; budget +25%).
	- ## Search index
		- `lsg`'s search files for 14,379 pages and 123,855 blocks: one title index, 1,236 word shards and 1,946 record chunks. **Bold** marks a value over its `[search]` budget.
		- | File | Size | Budget |
		  | --- | ---: | ---: |
		  | Title index (KB, gzip) | 147 | 300 |
		  | Title index (MB, decoded) | 0.6 | 1.5 |
		  | Largest shard, words starting `pr` (KB, gzip) | 55.6 | 64 |
		  | Largest record chunk (KB, gzip) | 21.1 | 32 |
		  | Search starter (KB) | 1.9 | 2 |
	- ## Search
		- Cmd/Ctrl+K on the home page, then the first query typed at once: the time from the key to the dialog and to its page results, and what that first open downloaded. Then each query in [`search-queries.txt`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/search-queries.txt) typed a key every 300 ms. A cold keystroke fetches a word shard; a warm one does not. The heap growth is measured after the whole query list, after garbage collection. **Bold** marks an `lsg` value over its budget.
		- | Profile | Build | Dialog shown (ms) | First page results (ms) | First open (KB, gzip) | Cold keystrokes | Cold 95th percentile (ms) | Warm 95th percentile (ms) | Largest keystroke download (KB, gzip) | Heap growth (MB) |
		  | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
		  | desktop | logseq | 32 | 3,621 | 0.0 | 0 | – | 1,622 | 0.0 | 60.0 |
		  | desktop | lsg | 23 | 139 | 172.2 | 9 | 48 | 31 | 121.7 | 6.9 |
		  | phone | logseq | 112 | 14,821 | 72.2 | 0 | – | 12,908 | 0.0 | 60.4 |
		  | phone | lsg | 246 | 2,042 | 165.5 | 9 | 504 | **158** | 103.8 | 6.8 |
	- ## Search latency per keystroke
		- Every keystroke of the query list. An unanswered keystroke is one whose results never showed. An `lsg` keystroke ends when its result list is drawn; block rows whose record chunk is still loading show a placeholder line until it arrives. The keystrokes that fetched nothing show the cost of the query alone. `lsg` reopens the dialog from memory after the first open.
		- | Profile | Build | Keystrokes | Unanswered | Median (ms) | 95th percentile (ms) | Warm keystrokes that fetched a chunk | 95th percentile of keystrokes that fetched nothing (ms) | Reopen (ms) | Reopen requests |
		  | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
		  | desktop | logseq | 47 | 0 | 790 | 1,622 | 0 | 1,622 | – | – |
		  | desktop | lsg | 47 | 0 | 24 | 39 | 16 | 31 | 16 | 0 |
		  | phone | logseq | 47 | 0 | 3,425 | 12,908 | 1 | 12,908 | – | – |
		  | phone | lsg | 47 | 0 | 43 | 352 | 17 | 62 | 21 | 0 |
	- ## Search per query
		- Each query of the list, typed a key every 300 ms: its keystrokes, the 95th percentile and the last keystroke's time to the drawn results, what the query downloaded, and the first page result `lsg` showed. **Bold** marks an `lsg` 95th percentile over the profile's cold keystroke budget in `[search]`.
		- | Query | Profile | Build | Keystrokes | Unanswered | 95th percentile (ms) | Last keystroke (ms) | Cold keystrokes | Downloaded (KB, gzip) | First result |
		  | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
		  | `claude code` | desktop | logseq | 11 | 0 | 1,639 | 1,639 | 0 | 0.0 | – |
		  | `claude code` | desktop | lsg | 11 | 0 | 45 | 24 | 3 | 238.5 | `Claude Code` |
		  | `claude code` | phone | logseq | 11 | 0 | 16,281 | 16,281 | 0 | 0.0 | – |
		  | `claude code` | phone | lsg | 11 | 0 | 504 | 44 | 3 | 220.5 | `Claude Code` |
		  | `deno deploy` | desktop | logseq | 11 | 0 | 1,584 | 1,584 | 0 | 0.0 | – |
		  | `deno deploy` | desktop | lsg | 11 | 0 | 48 | 20 | 2 | 114.4 | `Deno Deploy` |
		  | `deno deploy` | phone | logseq | 11 | 0 | 12,908 | 12,908 | 0 | 0.0 | – |
		  | `deno deploy` | phone | lsg | 11 | 0 | 465 | 40 | 2 | 109.1 | `Deno Deploy` |
		  | `flashcard` | desktop | logseq | 9 | 0 | 1,392 | 1,392 | 0 | 0.0 | – |
		  | `flashcard` | desktop | lsg | 9 | 0 | 25 | 19 | 1 | 71.8 | `Flashcard/Cloze` |
		  | `flashcard` | phone | logseq | 9 | 0 | 10,849 | 10,849 | 0 | 0.0 | – |
		  | `flashcard` | phone | lsg | 9 | 0 | 239 | 38 | 1 | 71.8 | `Flashcard/Cloze` |
		  | `logseq query` | desktop | logseq | 12 | 0 | 1,872 | 1,872 | 0 | 0.0 | – |
		  | `logseq query` | desktop | lsg | 12 | 0 | 39 | 24 | 3 | 138.5 | `Logseq/Q/When using Logseq GUI is there a keyshort that will take me to the parent of a page?` |
		  | `logseq query` | phone | logseq | 12 | 0 | 15,014 | 15,014 | 0 | 0.0 | – |
		  | `logseq query` | phone | lsg | 12 | 0 | 352 | 42 | 3 | 138.5 | `Logseq/Q/When using Logseq GUI is there a keyshort that will take me to the parent of a page?` |
		  | `tmux` | desktop | logseq | 4 | 0 | 790 | 790 | 0 | 0.0 | – |
		  | `tmux` | desktop | lsg | 4 | 0 | 32 | 19 | 0 | 0.0 | `tmux` |
		  | `tmux` | phone | logseq | 4 | 0 | 4,138 | 4,138 | 0 | 0.0 | – |
		  | `tmux` | phone | lsg | 4 | 0 | 56 | 43 | 0 | 6.7 | `tmux` |
	- ## Video embeds
		- Each page loaded in a fresh window and left for 40 seconds, then its first bound timestamp clicked. Player requests count every request to a video provider or Google host before that click, where Logseq makes about 1.2 MB of them per page; other hosts are the page's own remote images. **Bold** marks a value over its `[media]` budget.
		- | Page | Profile | Videos | Timestamps | Player area (px) | Player requests before a click | Other hosts | Player after the click |
		  | --- | --- | ---: | ---: | --- | ---: | --- | --- |
		  | Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs | desktop | 1 | 13 | 728x315 | 0 | none | `https://www.youtube.com/embed/6cqVzgVQJfE` with `start=0` |
		  | Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs | phone | 1 | 13 | 298x178 | 0 | none | `https://www.youtube.com/embed/6cqVzgVQJfE` with `start=0` |
		  | AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025 | desktop | 2 | 0 | 728x315 | 0 | `do3z7e6uuakno.cloudfront.net`, `pbs.twimg.com`, `swyx.io`, `www.swyx.io` | none |
		  | AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025 | phone | 2 | 0 | 298x178 | 0 | `swyx.io`, `www.swyx.io` | none |
	- ## Queries
		- `Parity/Queries` holds the queries analysis's test queries, added to a copy of the whole garden. Open is the time from opening the collapsed `Blocks that mention tmux` query (603 results in Logseq) to its first group drawn; the heap is read after every query fragment on the page has loaded, after garbage collection. **Bold** marks a value over its `[queries]` budget.
		- | Profile | Title (ms) | Data before title | Script before title | Open (ms) | Groups after scrolling | Heap after all fragments (MB) |
		  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
		  | desktop | 121 | 57,390 | 12,392 | 20 | 568 | 1.2 |
		  | phone | 374 | 57,390 | 12,392 | 618 | 568 | 1.2 |
		- Whole-garden build with `Parity/Queries`: 14.48 s against 14.25 s for the baseline binary (+1.6%, budget 10%).
	- ## Advanced queries
		- Measured 2026-11-08 on encode garden commit `a7d948e` with `lsg` `154e0ef`.
		- `Parity/AdvancedQueries` holds a query in the form of gitpa's homepage query (a `:result-transform` that sorts and a `:view` that draws links) over the encode garden's proxy pages, and `Parity/AdvancedQueries/All Pages` the worst case: every page, sorted by a property, with a view of the first 1,000. Both pages are added to a copy of the whole garden, which `lsg` and Logseq's own export (`parity:logseq-export`) each build. Rows is the time to the first row of the view; Logseq runs the query in the browser after it loads the graph, which its export writes into a script element of `index.html`, so Logseq's script before the title is mostly the graph. Values are medians of 3 loads; heap is read after garbage collection once the rows show. **Bold** marks an `lsg` value over its `[desktop]` or `[phone]` budget.
		- | Page | Profile | Build | Title (ms) | Rows (ms) | Rows shown | Data before title (KB) | Script before title (KB) | Peak heap (MB) | Heap with rows (MB) |
		  | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
		  | `Parity/AdvancedQueries` | desktop | logseq | 28,881 | 29,090 | 6 | 24,185.5 | 245,832.8 | 2,106.5 | 727.7 |
		  | `Parity/AdvancedQueries` | desktop | lsg | 19 | 13 | 6 | 15.0 | 13.6 | 1.8 | 1.2 |
		  | `Parity/AdvancedQueries` | phone | logseq | 191,193 | 191,782 | 6 | 23,610.2 | 245,832.8 | 2,085.4 | 727.1 |
		  | `Parity/AdvancedQueries` | phone | lsg | 414 | 227 | 6 | 15.0 | 13.6 | 1.9 | 1.2 |
		  | `Parity/AdvancedQueries/All Pages` | desktop | logseq | 22,455 | 23,250 | 1,000 | 24,185.5 | 245,832.8 | 2,144.5 | 754.3 |
		  | `Parity/AdvancedQueries/All Pages` | desktop | lsg | 30 | 24 | 1,000 | 34.4 | 13.6 | 2.1 | 1.2 |
		  | `Parity/AdvancedQueries/All Pages` | phone | logseq | 198,196 | 200,656 | 1,000 | 23,610.2 | 245,832.8 | 2,127.8 | 753.6 |
		  | `Parity/AdvancedQueries/All Pages` | phone | lsg | 402 | 217 | 1,000 | 34.4 | 13.6 | 2.1 | 1.3 |
		- Search on the copy, from the home page, typing each query in `scripts/parity/search-queries.txt` a key at a time. A cold keystroke fetched a search shard; **bold** marks an `lsg` value over `cold_keystroke_ms` in `[search]` or, for warm keystrokes, over `search_p95_ms` in `[desktop]` or `[phone]`:
		- | Profile | Build | Keystrokes | p50 (ms) | p95 (ms) | Cold p95 (ms) | Warm p95 (ms) |
		  | --- | --- | ---: | ---: | ---: | ---: | ---: |
		  | desktop | logseq | 36 | 793 | 1,727 |  | 1,727 |
		  | desktop | lsg | 36 | 16 | 34 | 45 | 23 |
		  | phone | logseq | 36 | 2,854 | 13,692 |  | 13,692 |
		  | phone | lsg | 36 | 28 | 325 | 416 | 47 |
		- Builds of the copy with the run's options and `--include Parity`, fastest of three. A query's cost is how much it grows the fastest render phase, where queries run, measured with the query 10 times on one page and divided by 10, so that the cost stands out from the variation between builds:
		- With `Parity/AdvancedQueries` only: 16.79 s, render 5,736 ms. With `Parity/AdvancedQueries/All Pages` as well: 16.69 s, render 5,860 ms.
		- The all-pages query, 10 times: render 6,689 ms, so one costs 95 ms (budget 500 ms).
		- `Parity/AdvancedQueries/Runaway`, a transform that stops at the step limit, 10 times: render 6,583 ms, so stopping one costs 0.08 s (budget 2 s). Logseq does not build this page.
		- Logseq's export of the copy with both pages: 159 s.
		- The `--baseline-lsg` binary built the copy with both pages in 16.67 s; this build is +0.1% against it (budget 10%).
	- ## Plugins
		- The whole encode garden built with the recorder fixture plugin (`tests/fixtures/plugins/recorder`), same-document navigations: one to every fixture page, then 20 more. Navigation is the time from the fetched page's response end to the adopted page: `<main>` swapped, lsg's page initializers run, plugins notified and the fragment revealed. Listeners are the document's and window's event listeners after every page has been visited once, when scripts that only some pages load are in, and after the 20 more navigations. **Bold** marks a value over its `[plugins]` budget.
		- | Profile | Navigations | Median (ms) | Slowest (ms) | Slowest page | Listeners after one visit each | Listeners 20 navigations later |
		  | --- | --- | --- | --- | --- | --- | --- |
		  | desktop | 47 | 2.6 | 16.3 | Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton | 30 | 30 |
		  | phone | 47 | 11.4 | **80.8** | Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton | 30 | 30 |
		- Host module `lsg-host-739a578f944fb016.mjs`: 7,759 bytes, 2,924 gzip (budget 4,000). Largest fixture page first paint with the plugin: 81,815 gzip (Person/Gergely Orosz/Pod/26/09 Design Engineering with Maggie Appleton, 545,147 bytes of HTML).
	- ## First-party plugins
		- Measured 2026-11-08 on a copy of encode garden commit `66b81e9` whose `logseq/config.edn` holds `:lsg/plugins [:podcast-episode :sequence-header]`, built by `lsg` `f64db09` and by Logseq's own export of the same copy, which ignores the key. Each value is the median of 3 loads. **Bold** marks an `lsg` value over its `[desktop]` or `[phone]` budget.
		- | Page | Profile | Build | Title (ms) | Data before title (KB) | Script before title (KB) | Peak heap (MB) |
		  | --- | --- | --- | --- | --- | --- | --- |
		  | GitP/A/Session/26/11/01-Sun | desktop | logseq | 31759 | 24,266.3 | 246,852.4 | 2,126.8 |
		  | GitP/A/Session/26/11/01-Sun | desktop | lsg | 19 | 22.2 | 11.7 | 1.7 |
		  | GitP/A/Session/26/11/01-Sun | phone | logseq | 205017 | 23,690.9 | 246,852.4 | 2,102.9 |
		  | GitP/A/Session/26/11/01-Sun | phone | lsg | 396 | 22.2 | 11.7 | 2.2 |
		  | GitP/A/Session/26/10/29-Thu | desktop | logseq | 29754 | 24,266.3 | 246,852.4 | 2,123.8 |
		  | GitP/A/Session/26/10/29-Thu | desktop | lsg | 32 | 18.9 | 11.7 | 2.1 |
		  | GitP/A/Session/26/10/29-Thu | phone | logseq | 205235 | 23,690.9 | 246,852.4 | 2,114.7 |
		  | GitP/A/Session/26/10/29-Thu | phone | lsg | 395 | 18.9 | 11.7 | 2.1 |
		  | GitP/A/Session/26/10/24-Sat | desktop | logseq | 29000 | 24,266.3 | 246,852.4 | 2,129.4 |
		  | GitP/A/Session/26/10/24-Sat | desktop | lsg | 26 | 19.8 | 12.2 | 2.0 |
		  | GitP/A/Session/26/10/24-Sat | phone | logseq | 208858 | 23,690.9 | 246,852.4 | 2,113.9 |
		  | GitP/A/Session/26/10/24-Sat | phone | lsg | 413 | 19.8 | 12.2 | 2.0 |
		  | GitP/A/Session/26/10/20-Tue | desktop | logseq | 29549 | 24,266.3 | 246,852.4 | 2,117.2 |
		  | GitP/A/Session/26/10/20-Tue | desktop | lsg | 29 | 32.9 | 12.2 | 1.6 |
		  | GitP/A/Session/26/10/20-Tue | phone | logseq | 211239 | 23,690.9 | 246,852.4 | 2,121.1 |
		  | GitP/A/Session/26/10/20-Tue | phone | lsg | 406 | 32.9 | 12.2 | 2.3 |
		  | GitP/A/Session/26/10/18-Sun | desktop | logseq | 28700 | 24,266.3 | 246,852.4 | 2,123.1 |
		  | GitP/A/Session/26/10/18-Sun | desktop | lsg | 17 | 31.2 | 11.7 | 2.4 |
		  | GitP/A/Session/26/10/18-Sun | phone | logseq | 196465 | 23,690.9 | 246,852.4 | 2,107.2 |
		  | GitP/A/Session/26/10/18-Sun | phone | lsg | 402 | 31.2 | 11.7 | 2.4 |
		  | GitP/A/Session/26/09/24-Thu | desktop | logseq | 29359 | 24,266.3 | 246,852.4 | 2,100.7 |
		  | GitP/A/Session/26/09/24-Thu | desktop | lsg | 17 | 14.4 | 11.7 | 1.9 |
		  | GitP/A/Session/26/09/24-Thu | phone | logseq | 205996 | 23,690.9 | 246,852.4 | 2,120.4 |
		  | GitP/A/Session/26/09/24-Thu | phone | lsg | 411 | 14.4 | 11.7 | 2.0 |
		  | GitP/A | desktop | logseq | 31060 | 24,266.3 | 246,852.4 | 2,161.0 |
		  | GitP/A | desktop | lsg | 20 | 19.1 | 11.7 | 1.6 |
		  | GitP/A | phone | logseq | 203803 | 24,266.3 | 246,852.4 | 2,124.9 |
		  | GitP/A | phone | lsg | 404 | 19.1 | 11.7 | 1.8 |
		  | GitP/A/Session | desktop | logseq | 28670 | 24,266.3 | 246,852.4 | 2,159.2 |
		  | GitP/A/Session | desktop | lsg | 19 | 19.0 | 11.7 | 1.6 |
		  | GitP/A/Session | phone | logseq | 208819 | 24,266.3 | 246,852.4 | 2,136.9 |
		  | GitP/A/Session | phone | lsg | 403 | 19.0 | 11.7 | 1.8 |
		  | GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3 | desktop | logseq | 27158 | 24,266.3 | 246,852.4 | 2,109.7 |
		  | GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3 | desktop | lsg | 18 | 13.9 | 11.7 | 1.8 |
		  | GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3 | phone | logseq | 204224 | 23,690.9 | 246,852.4 | 2,110.5 |
		  | GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3 | phone | lsg | 409 | 13.9 | 11.7 | 1.9 |
		  | Home | desktop | logseq | 27632 | 24,266.3 | 246,852.4 | 2,123.3 |
		  | Home | desktop | lsg | 21 | 16.8 | 11.7 | 1.9 |
		  | Home | phone | logseq | 201995 | 23,690.9 | 246,852.4 | 2,099.7 |
		  | Home | phone | lsg | 417 | 16.8 | 11.7 | 1.9 |
		- Phone heap after starting `GitP/A/Session/26/10/24-Sat` and following 20 links in place, the median of 3 visits each. A plugin's growth is the heap with its module against the heap with its module blocked. Recordings were served as a local tone. **Bold** marks a value over its budget.
		- Heap: 1.55 MB with every plugin on and the recording playing, 1.41 MB with `podcast-episode` blocked and nothing playing, 1.39 MB with every plugin blocked.
		- | Plugin | Phone heap growth (MB) | Budget (MB) | Module and stylesheet (gzip bytes) | Budget (bytes) |
		  | --- | --- | --- | --- | --- |
		  | `podcast-episode` | 0.14 | 1 | 4,906 | 8,000 |
		  | `sequence-header` | 0.03 | 0.2 | 1,465 | 1,500 |
		- Pages that load plugin code before the title: 0.
	- ## Installable gardens
		- Measured 2026-11-11 with `mise run parity:budgets` on the fixture garden and `mise run parity:build-bench --install-only --runs 5` on a copy of encode garden commit `d9722ef`. A copy of the garden gains `:lsg/install` and an icon; a second copy has neither. A garden without the key publishes the same files as before: the whole encode garden, 65,774 files, matched byte for byte against the previous release.
		- | Row | Measured | Budget |
		  | --- | --- | --- |
		  | Largest page growth with the key, 53 fixture pages (bytes) | 380 | 500 |
		  | Largest page growth with the key, gzip (bytes) | 90 | 300 |
		  | `lsg-install.js` (bytes, gzip bytes) | 694, 380 | 1,500, 1,000 |
		  | `lsg.webmanifest` (bytes) | 240 | 1,000 |
		  | Icon in the fixture build (bytes) | 1,497 | 256,000 |
		  | Page growth when the garden gains 2,000 pages | 0 | 0 |
		  | Search files compared with the build without the key | Identical | Identical |
		  | Added build time on the encode garden, fastest of 5 builds (ms) | 252 | 50 |
		  | Added build time on the encode garden, median of 5 paired builds (ms) | -82 | 50 |
		- First paint of every fixture page stays inside the `[desktop]` and `[phone]` budgets, because the manifest, the icon and `lsg-install.js` load outside first paint.
		- The five builds of each copy took 19.2 to 20.4 s and varied by about 1 s between runs, more than the 50 ms budget. The fastest-build difference of 252 ms and the paired median of -82 ms both fall inside that spread; the key's real cost is not measurable on this machine. `parity:build-bench` fails on the paired median.
