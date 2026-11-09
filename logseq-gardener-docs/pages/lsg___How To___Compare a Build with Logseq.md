tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Parity Tasks]], [[lsg/Ref/Logseq Compatibility/Comparison Harness]], [[lsg/How To/Measure Performance]], [[lsg/Ref/Logseq Compatibility]]

- # How To Compare a Build with Logseq
	- ## Overview
		- The `parity:` tasks build the whole encode garden with `lsg` and compare the result with Logseq's own build of it, the `gh-pages` branch of `codekiln/logseq-encode-garden`. Use them to find where an `lsg` page differs from Logseq's and to show a change's progress. [[lsg/Ref/Logseq Compatibility/Comparison Harness]] describes what the two builds contain, and [[lsg/Ref/Parity Tasks]] lists every task and what it writes.
		- `parity:logseq-export` builds Logseq's own export of a garden, the docs graph by default, and compares the text of every page with an `lsg` site of the same garden. It needs no `gh-pages` build; its steps are in the section Compare page text with Logseq's own export.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`.
		- The encode garden checked out with `ghq`. To publish another garden instead, pass `parity:build --garden <path>`.
		- Network access to fetch the garden's `gh-pages` branch. The tasks read it with `git archive`, so your garden checkout stays on its branch.
		- For screenshots, measurements and `parity:logseq-export`, Playwright's Chromium. The tasks install it through `uv` on first use, into Playwright's shared browser cache.
	- ## Steps
		- ### 1. Build the garden with lsg
			- Run `mise run parity:build` from the repository root. It builds a release `lsg` and publishes every page whose top-level namespace `lsg` can select into a new directory, `target/parity/<UTC time>/`.
		- ### 2. List the pages each build lacks
			- Run `mise run parity:compare`. It extracts Logseq's build beside the `lsg` build and writes `report.md`, which groups the pages present in one build and missing from the other.
			- Read the other-pages group first: each page there needs an explanation or a fix.
		- ### 3. Look at the same pages side by side
			- Run `mise run parity:screenshots`, then open `screenshots/index.html` in the run directory. It shows each fixture page from both builds side by side, at desktop and phone sizes.
			- Pass `--viewport desktop` or `--viewport phone` to capture one size.
		- ### 4. Measure both builds
			- Run `mise run parity:perf` for load times, transfer sizes, memory and search latency of both builds. A full run takes hours; [[lsg/How To/Measure Performance]] shows quicker runs and how to record the results.
		- `parity:compare`, `parity:screenshots` and `parity:perf` use the latest run directory. Pass `--run <directory>` to use another one.
	- ## Compare page text with Logseq's own export
		- Run `mise run parity:logseq-export` from the repository root. It builds Logseq 0.10.15's export of the docs graph and an `lsg` site of it with the options `docs:site` uses, then reads every page of both in headless Chromium. It writes into a new directory, `target/parity/<UTC time>/`, and takes a few minutes.
		- Logseq's export uses the publishing frontend inside `/Applications/Logseq.app` when that is Logseq 0.10.15. Otherwise the first run downloads the official 0.10.15 release, 202 MB, and keeps the frontend in `~/.cache/logseq-gardener/`. Pass `--static-dir <folder>` to use another Logseq 0.10.15 `app` folder.
		- Open `logseq-export.md` in the run directory. For each page that differs, it names the words only one build shows and the property rows only one build shows. The task exits 1 when any page differs.
		- To compare another garden, pass its folder and the `lsg publish` options that select its pages after `--`:
		- ```sh
		  mise run parity:logseq-export --garden ~/notes -- --include Notes --journals
		  ```
		- [[lsg/Ref/Parity Tasks]] describes how Logseq's export is built and what the comparison leaves out.
	- ## Troubleshooting
		- If a page you need is missing from the screenshots, add its title to `scripts/parity/fixtures.txt`; [[lsg/Ref/Parity Tasks]] explains the fixture list.
		- If `parity:logseq-export` stops because the `--static-dir` folder has another digest, that folder holds another build than the Logseq 0.10.15 release. Leave out `--static-dir`, and the task uses Logseq.app or the release.
