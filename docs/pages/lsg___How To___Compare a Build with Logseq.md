tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Parity Tasks]], [[lsg/Ref/Logseq Compatibility/Comparison Harness]], [[lsg/How To/Measure Performance]], [[lsg/Ref/Logseq Compatibility]]

- # How To Compare a Build with Logseq
	- ## Overview
		- The `parity:` tasks build the whole encode garden with `lsg` and compare the result with Logseq's own build of it, the `gh-pages` branch of `codekiln/logseq-encode-garden`. Use them to find where an `lsg` page differs from Logseq's and to show a change's progress. [[lsg/Ref/Logseq Compatibility/Comparison Harness]] describes what the two builds contain, and [[lsg/Ref/Parity Tasks]] lists every task and what it writes.
	- ## Prerequisites
		- A checkout of [codekiln/logseq-gardener](https://github.com/codekiln/logseq-gardener) with its tools installed by `mise install`.
		- The encode garden checked out with `ghq`. To publish another garden instead, pass `parity:build --garden <path>`.
		- Network access to fetch the garden's `gh-pages` branch. The tasks read it with `git archive`, so your garden checkout stays on its branch.
		- For screenshots and measurements, Playwright's Chromium. The tasks install it through `uv` on first use, into Playwright's shared browser cache.
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
	- ## Troubleshooting
		- If a page you need is missing from the screenshots, add its title to `scripts/parity/fixtures.txt`; [[lsg/Ref/Parity Tasks]] explains the fixture list.
