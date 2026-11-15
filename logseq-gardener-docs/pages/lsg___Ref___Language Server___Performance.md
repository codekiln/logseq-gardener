tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Command Line]], [[lsg/How To/Edit a Garden in Neovim]], [[lsg/Ref/Logseq Compatibility/Performance]]

- # Language Server Performance
	- How long `lsg lsp` takes to load the encode garden, to answer completion, definition, hover and references, to reindex after an edit and to rename a page or a property key, on a release build. `mise run lsp:perf --record <garden>` writes this page.
	- measured: 2026-11-15
	- garden: logseq-encode-garden
	- garden commit: fcfefe0b524
	- lsg: 0.0.53 (1b59475)
	- machine: Apple M2 Pro, Darwin 25.6.0
	- ## Method
		- [`scripts/lsp/perf.py`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/lsp/perf.py) starts the release build of `lsg lsp` on the garden and exchanges LSP messages with it on its standard input and output, as an editor does, with `utf-8` positions. Every time is measured in the script, from sending a request to reading its answer, so it includes writing and reading the JSON.
		- Loading starts the server 5 times. The other measurements use the last of those servers.
		- Completion types each text into an unsaved buffer and asks 20 times at the cursor. `|` marks the cursor in the texts: `- [[|`, `- [[a|`, `- [[lo|`, `- [[the |`, `- [[20|`, `- see [[note|]]`, `- #|`, `- #c|`, `- #pro|`, `- ((|`, `- ((the|`, `- ((how to |))`, `- item ⏎   t|`, `- item ⏎   sta|`, `- item ⏎   tags:: |`, `- item ⏎   type:: b|`.
		- Definition, hover and references ask once at each of up to 200 page links, tags and block references, chosen from the garden's files with a fixed random seed. Answered counts the requests with an answer other than null or an empty list.
		- Each edit is repeated 20 times and each rename 5 times. The renamed pages are the three names with the most `[[links]]` and the name with the most namespace children among the page files, and the property key is the one most lines write.
		- The script writes no file of the garden: typed text reaches the server as buffer contents, and it applies no rename. It compares `git status` of the garden before and after the run.
	- ## Targets
		- The targets come from [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638).
		- | Target | Measured | Result |
		  | --- | ---: | --- |
		  | Loading under 5 s | 1.42 s | met |
		  | Completion within 100 ms at the 95th percentile | 22.3 ms | met |
		  | Definition within 100 ms at the 95th percentile | 0.06 ms | met |
		  | Reindexing after an edit under 50 ms | 4.21 ms | met |
		- Loading is the slowest of the measured starts, from starting the process to its answer to the first request after `initialize`. Reindexing is the highest 95th percentile among the edited pages below.
	- ## Loading
		- The server read 9,855 documents: 9,330 pages and 525 journals.
		- | Measurement | Starts | Median (ms) | Slowest (ms) |
		  | --- | ---: | ---: | ---: |
		  | Reading and indexing, as `lsg/status` reports `loadMilliseconds` | 5 | 1,145 | 1,163 |
		  | From starting the process to the answer to the first request after `initialize` | 5 | 1,159 | 1,420 |
	- ## Completion
		- | Typed | Requests | Median (ms) | 95th percentile (ms) | Slowest (ms) | Items per answer |
		  | --- | ---: | ---: | ---: | ---: | --- |
		  | `[[ page` | 120 | 21.3 | 23.0 | 66.5 | 100 to 100 |
		  | `# tag` | 60 | 20.6 | 22.3 | 22.9 | 100 to 100 |
		  | `(( block` | 60 | 3.15 | 3.56 | 3.67 | 23 to 100 |
		  | `key` | 40 | 0.35 | 0.56 | 0.58 | 24 to 95 |
		  | `key:: value` | 40 | 7.11 | 15.8 | 16.7 | 0 to 100 |
		  | All | 320 | 15.3 | 22.3 | 66.5 | |
	- ## Definition, hover and references
		- | Request | At | Requests | Answered | Median (ms) | 95th percentile (ms) | Slowest (ms) |
		  | --- | --- | ---: | ---: | ---: | ---: | ---: |
		  | definition | page link | 200 | 199 | 0.04 | 0.06 | 0.11 |
		  | hover | page link | 200 | 199 | 14.6 | 30.3 | 31.7 |
		  | references | page link | 200 | 200 | 14.8 | 32.5 | 37.1 |
		  | definition | tag | 200 | 175 | 0.03 | 0.06 | 0.10 |
		  | hover | tag | 200 | 175 | 14.0 | 17.8 | 19.2 |
		  | references | tag | 200 | 198 | 14.7 | 18.9 | 19.8 |
		  | definition | block reference | 200 | 195 | 0.03 | 0.06 | 0.10 |
		  | hover | block reference | 200 | 200 | 0.06 | 0.11 | 0.20 |
		  | references | block reference | 200 | 200 | 0.04 | 0.16 | 0.23 |
		  | definition | all three | 600 | | 0.04 | 0.06 | 0.11 |
	- ## Reindexing after an edit
		- The time from sending `textDocument/didChange` to receiving the answer to an `lsg/status` request sent right after it. The server handles messages in order, so the answer arrives after the index holds the edit.
		- | Edit | Edits | Median (ms) | 95th percentile (ms) | Slowest (ms) |
		  | --- | ---: | ---: | ---: | ---: |
		  | lsg/status alone, for comparison | 20 | 0.06 | 0.31 | 0.31 |
		  | a new unsaved page of two lines | 20 | 0.09 | 0.16 | 0.16 |
		  | a page of median size (0.7 kB), one character typed | 20 | 0.11 | 0.31 | 0.31 |
		  | the largest page (284.0 kB), one character typed | 20 | 3.56 | 4.21 | 4.21 |
	- ## Rename
		- Each row is one `textDocument/rename` whose answer was counted and dropped; no edit was applied.
		- | Renamed | Median (ms) | Slowest (ms) | Text edits | Files edited | File renames | Answer (MB) |
		  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
		  | the page `microfreak/ui/front/row/02 main/03 analog filter/knob/cutoff` | 27.6 | 73.4 | 1,006 | 367 | 2 | 0.19 |
		  | the page `logseq/entity/asset/b2` | 16.4 | 17.3 | 822 | 820 | 1 | 0.28 |
		  | the page `logseq/entity/book/section/level 3` | 17.1 | 17.2 | 682 | 681 | 1 | 0.24 |
		  | the page `microfreak` | 251 | 253 | 20,209 | 998 | 1,399 | 3.97 |
		  | the property key `logseq-entity` | 38.7 | 42.5 | 5,503 | 5,503 | 0 | 1.73 |
