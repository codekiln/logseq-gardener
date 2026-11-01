tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Measure Performance]], [[lsg/Ref/Logseq Compatibility/Performance]], [[lsg/Ref/Parity Tasks]]

- # Publication Speed
	- A warm build of the whole encode garden took 21 to 30 seconds on GitHub's 4-processor runners, depending on the processor, and 18 seconds on an Apple M2 Pro laptop. A generated garden the size of Gitpa's took 4 to 5 seconds on the runners and 3.5 seconds on the laptop. Rendering pages took most of the time, and about half of rendering went to the Linked References sections; rendering each reference copy once cut the laptop's render phase by 32% for the encode garden and 27% for the Gitpa-sized garden.
	  id:: 31339be0-cd13-4a22-b8f1-a9d73a1128d5
	- [[lsg/How To/Measure Performance]] shows how to run the benchmark and how `--timings` shows where a build spends its time.
	- ## Measured results
		- Each measurement is the fastest of five warm builds, after one cold build, with `--journals`, `--search` and every top-level namespace included. Times are in seconds. [`docs/parity/publication-speed.json`](https://github.com/codekiln/logseq-gardener/blob/main/docs/parity/publication-speed.json) holds every recorded row with its `lsg` commit, corpus identity and machine; the GitHub rows come from five runs of the `publication-speed` job on the change that added the benchmark.
		- | Machine | Runs | Gitpa-sized warm | Encode garden warm | Encode garden render |
		  | --- | --- | --- | --- | --- |
		  | GitHub runner, AMD EPYC 7763 | 3 | 5.1 to 5.3 | 28.1 to 29.8 | 19.1 to 20.4 |
		  | GitHub runner, Intel Xeon Platinum 8370C | 1 | 5.1 | 28.2 | 19.8 |
		  | GitHub runner, Intel Xeon 6975P-C | 1 | 3.8 | 21.1 | 14.6 |
		  | Apple M2 Pro, 12 processors | 1 | 3.5 | 17.7 | 8.7 |
		- The generated Gitpa-sized garden has 2,410 Markdown files and 3 MB of text, and its site has 9,900 files and 236 MB. The encode garden, at commit `5095bdf` on GitHub and `4d37b9f` on the laptop, has 9,246 files and 16 MB of text, and its site has 46,767 files and 1,212 MB. Peak memory for the encode garden was 2.5 GB on the runners and 3.5 GB on the laptop.
		- The cold build is the first build after the garden is written or checked out. It was 1% to 13% slower than the fastest warm build. The benchmark does not clear the operating system's file cache, which needs administrator rights, so the difference shows the cost of a fresh process rather than of reading from disk.
		- Phase times for one encode garden build, in seconds:
		- | Phase | GitHub runner, AMD EPYC 7763 | Apple M2 Pro | What it covers |
		  | --- | --- | --- | --- |
		  | `read` | 0.1 | 1.3 | Finding and reading source files, checking options |
		  | `parse` | 0.5 | 0.3 | Parsing Markdown |
		  | `index` | 3.4 | 1.9 | Selection, page names, references, queries and search records |
		  | `render` | 20.4 | 8.7 | Pages, lists, feeds and fragments |
		  | `search` | 4.2 | 1.7 | Search files |
		  | `write` | 0.8 | 3.6 | Writing the site |
		- The `plugins`, `history` and `graph` phases measure only how long the main thread waited for work running on other threads: build plugins, the git history read and the graph layouts. All three were under 0.1 seconds, so that work finishes while pages render.
	- ## Site size
		- Each recorded row also holds the file count and size of each part of the site. `parity:publish-bench --check` fails when the encode garden's site exceeds 550 MB, its page HTML at the site root 200 MB, or `_refs/` 100 MB, the `[site]` budgets in [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml). GitHub Pages serves at most 1 GB.
		- Sharing the page styles in `lsg-site.css`, moving the theme controller and search start-up into deferred files, and storing Linked and Unlinked References fragments gzip-compressed ([issue 348](https://github.com/codekiln/logseq-gardener/issues/348)) shrank the encode garden's site at commit `5095bdf`, on the Apple M2 Pro:
		- | Part | Files | Before (MB) | After (MB) |
		  | --- | ---: | ---: | ---: |
		  | Page HTML at the site root | 13,929 | 552.4 | 171.1 |
		  | `_refs/` | 17,486 | 447.9 | 82.5 |
		  | `assets/` | 111 | 164.5 | 164.5 |
		  | Search files | 1,640 before, 1,641 after | 29.0 | 29.0 |
		  | `_graph/` | 13,528 | 19.1 | 19.1 |
		  | Everything else | 73 before, 79 after | 3.9 | 3.9 |
		  | Whole site | 46,767 before, 46,774 after | 1,216.8 | 470.2 |
		- The fastest of five interleaved warm builds took 15.7 seconds before and 16.6 seconds after, 6% slower, within the `[site]` budget of 10%. Compression adds about 5 seconds of processor time, spread across the writing threads.
	- ## Rendering each reference copy once
		- The publisher renders each copy of a referring block and its breadcrumb once per build, and fills in the parts that depend on the listing page: the bullet's row number in that page's reference data and the bullet's link. Before [issue 290](https://github.com/codekiln/logseq-gardener/issues/290), it copied each referring block and rendered the copy again for every page that listed it. The published files did not change: the encode garden at `4d37b9f` and the generated Gitpa-sized garden, published with the binaries from before and after the change, gave the same 46,767 and 9,900 files byte for byte and the same diagnostics.
		- On the Apple M2 Pro, fastest of five warm builds, in seconds:
		- | Corpus | Warm before | Warm after | Render before | Render after | Graph wait after | Peak memory before | Peak memory after |
		  | --- | --- | --- | --- | --- | --- | --- | --- |
		  | Gitpa-sized | 3.50 | 2.98 | 1.65 | 1.21 | 0.00 | 563 MB | 575 MB |
		  | Encode garden | 17.61 | 16.29 | 8.67 | 5.89 | 1.29 | 3,490 MB | 3,600 MB |
		- For the encode garden, the whole build gained less than rendering did, because the graph layouts, which run on another thread during rendering, now finish after it: the main thread waits 1.3 seconds for them in the `graph` phase.
	- ## The bottleneck before issue 290
		- Rendering runs on one thread and took 68% to 74% of a warm build on the GitHub runners and 49% on the laptop. A 7-second profile of the laptop build during rendering put 45% of the main thread's samples in `References::render`, which renders Linked References. It copied each referencing block, with its children, and rendered it again for every page the block references. [Issue 290, Render each Linked References context once](https://github.com/codekiln/logseq-gardener/issues/290), removed that repeated work without changing the output.
		- Writing the site takes 3.6 seconds on macOS and 0.8 seconds on the Linux runner for the same 46,767 files, and reading the garden 1.3 and 0.1 seconds, so file access costs far more on the laptop than on the runner.
		- Gitpa's [hosted run 41195021354](https://github.com/codekiln/gitpa/actions/runs/41195021354/job/124519359328) logged Gardener's result about two seconds after its own query step. That run is not directly comparable with these rows: Gitpa publishes without `--journals` and with its own homepage and stylesheet. Gitpa's homepage query runs in Gitpa's tooling before Gardener starts; [gitpa issue 102, Reduce repeated work in site publication](https://github.com/codekiln/gitpa/issues/102), tracks that work. Passing `--timings` in Gitpa's build would show Gardener's phases in that workflow.
	- ## The CI check
		- The `publication-speed` job in [`.github/workflows/ci.yml`](https://github.com/codekiln/logseq-gardener/blob/main/.github/workflows/ci.yml) runs `parity:publish-bench --corpus all --check` on the generated garden and on the encode garden commit pinned in that file. It compares each fastest warm build with the GitHub rows for the same corpus and fails when the build exceeds its limit:
			- With rows on the same processor model, the limit is the slowest of them plus 20%.
			- Without such a row, the limit is the slowest GitHub row for the corpus plus 100%.
		- GitHub assigns runners with different processors. In October 2026 the `build-plugins` job's encode garden build took between 15.5 and 28.8 seconds for nearly the same code; within a group of similar times, runs differed by about 6%. The runs recorded for the benchmark show the processor model explains the groups: three runs on the AMD EPYC 7763 differed by 6%, and the Intel Xeon 6975P-C took about 25% less time than either other model. The 20% limit is more than three times the spread on one model, and the 100% limit covers the gap between the fastest and slowest groups. The tolerances live in the `[publication]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml).
		- The job uploads its rows as the `publication-speed` artifact. Record new GitHub rows by adding them to `docs/parity/publication-speed.json` when:
			- the job reports a processor model with no row;
			- the pinned encode garden commit changes, which leaves the encode garden unchecked until new rows exist; or
			- `sample_garden.py` changes, which changes the digest and leaves the generated garden unchecked.
		- A change that makes publication intentionally slower records new rows in the same pull request and explains the cost.
