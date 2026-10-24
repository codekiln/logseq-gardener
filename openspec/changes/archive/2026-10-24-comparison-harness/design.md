## Context

The [comparison harness analysis](../../../../docs/parity/comparison-harness.md) built the whole encode garden with `lsg` and decoded Logseq's `gh-pages` build by hand on 2026-10-24. Every page `lsg` published exists in Logseq's build. Logseq's extra pages fall into groups: 505 journals with content, about 6,600 reference-only pages, three titles containing `%2F`, and three other pages. Each later parity issue under #106 needs the same build and comparison, plus screenshots of the same page from both builds. Issue #106 also requires `lsg` to beat Logseq's load time, memory use and search speed by a wide margin, measured by this harness on the same pages and backed by benchmark rows in CI.

The harness is a contributor tool. It runs on a machine with the encode garden checked out through `ghq`, needs the `gh-pages` branch fetched, and takes about two minutes for the build. CI runs its unit tests and a size-budget check on a repository fixture garden.

## Goals / Non-Goals

**Goals:**

- One command per job: build, compare, screenshot, measure.
- A report whose groups separate the large known differences (journals, reference-only pages) from pages that need an explanation.
- Screenshots that a person can review page by page.
- Load, memory and search numbers for both builds on the same pages, checked against budgets.

**Non-Goals:**

- Automatic pixel or DOM diff scoring of screenshots. A score would let later parity PRs show progress as a number; it waits until the feature issues have closed the large gaps.

## Decisions

The harness is five mise file tasks under `.mise/tasks/parity/` (`build`, `compare`, `screenshots`, `perf`, `budgets`) backed by Python modules in `scripts/parity/`. Python is already pinned in `mise.toml` and used by `scripts/docs.py`, and keeping the harness outside the `lsg` binary leaves the CLI contract, its help text and the clilint checks unchanged.

`parity:build` runs a release build of `lsg` and passes one `--include` per top-level namespace. It finds the roots by decoding each file name in `pages/` with the `triple-lowbar` rules: split on `___`, URL-decode, take the first segment. A root that fails `lsg`'s selector rules is skipped and listed. Output goes to `target/parity/<UTC timestamp>/site/`, with `settings.json` beside it holding the garden commit, the `lsg` version and the arguments. `target/` is already ignored by git, and a timestamped directory satisfies `lsg`'s rule that output is new on each run. The build uses the garden's `logseq/config.edn` as it is, so the comparison shows what the garden would publish today; for example, page properties stay off because the encode garden does not set `:publishing/page-properties?`.

`parity:compare` reads Logseq's build with `git archive origin/gh-pages` into `target/parity/<timestamp>/logseq/`, so the garden checkout is never switched. The decoder reads `static/js/db.gz`, parses the outer JavaScript string as JSON, reverses publish-spa's `logseq____&…;` replacements in the order Logseq's `frontend/publishing.cljs` uses, and decodes Transit's cached keys to get the datoms. Page records come from `:block/name`, `:block/original-name`, `:block/journal?` and `:block/page`.

The `lsg` title-to-file list comes from the links in the built `index.html`, which lists every page with its full title. This avoids a new output file in `lsg`. A unit test on a small built index keeps the parser in step with the index markup.

`:hidden` matching follows Logseq's rule in `logseq.common.config/hidden?`: a pattern without a leading `/` gains one, and a file is hidden when `/` plus its graph-relative path starts with the pattern. With the encode garden's current `:hidden` list no file in `pages/` or `journals/` matches, so this group is empty today; it stays in the report so a future entry shows up.

`parity:screenshots` uses Playwright for Python with Chromium. Playwright 1.60.0 is pinned in the inline script metadata of `screenshots.py` and `perf.py`, which run through `uv run --script`; `uv` is pinned in `mise.toml`. It serves each build with Python's `http.server` bound to 127.0.0.1 on a free port, opens each fixture at 1280 by 800 pixels in the light colour scheme, waits for `.page-title` (Logseq) or the page heading (`lsg`), scrolls to the bottom in steps until the page height stops growing, and then captures the full page. The fixture list is `scripts/parity/fixtures.txt`, one title per line, plus two route names, `Home` and `All pages`, for Logseq's `#/` and `#/all-pages`. The run writes `screenshots/index.html` with the two captures of each fixture side by side.

`parity:perf` measures both builds on the fixture pages with Playwright's Chromium and the Chrome DevTools Protocol. It serves each build on 127.0.0.1 and loads each page with an empty cache, once on a desktop profile (1280 by 800, no throttling) and once on a phone profile (412 by 823 at device pixel ratio 1.75, CPU slowed four times, 150 ms round trip, 1.6 Mbps down and 750 Kbps up; the mobile settings Lighthouse uses). For each load it records:

- time until the page title is visible (`.page-title` in Logseq, the page heading in `lsg`), and the browser's first contentful paint;
- data transferred before the title is visible, counted as the gzip size of each response whose request started by then, because GitHub Pages compresses text files and the local server does not; `db.gz` is counted at its stored size;
- script size before the title is visible, inline and fetched;
- peak JavaScript heap, sampled every 100 ms from `Performance.getMetrics` until the title is visible and two seconds have passed with no network request.

Search latency is measured on the whole garden with a fixed query list in `scripts/parity/search-queries.txt`. The harness opens each build's search, types each query one character at a time 300 ms apart, and records for each keystroke the time until the results list stops changing. Selectors for the search button, input and results come from a per-build table in `scripts/parity/perf.py`. `lsg` has no search yet (#94, #95), so its rows read "no search" until a build contains the search input.

The harness reads budgets from `scripts/parity/budgets.toml` and marks each `lsg` number that exceeds its budget. The starting budgets, for every fixture page:

| Measure | Desktop | Phone profile |
| --- | --- | --- |
| Data before the title is visible (gzip) | 100 KB | 100 KB |
| Script before the title is visible | 16 KB | 16 KB |
| Time until the title is visible | 300 ms | 1.5 s |
| Peak JavaScript heap | 10 MB | 10 MB |
| Search latency per keystroke, 95th percentile | 50 ms | 100 ms |

A whole-garden `lsg` build on 2026-10-24 had a median page of 5 KB gzip, `tmux` at 86 KB and the largest page (`Answer/AI`, mostly backlinks) at 369 KB, with about 8.5 KB of inline script per page. The budgets hold for every fixture page today and leave the backlink-heavy pages to #115.

`parity:perf --record` rewrites `docs/parity/performance.md` with one row per page, profile and build, plus the garden commit, `gh-pages` commit and `lsg` version, so each later parity PR can show its numbers next to the previous ones.

`parity:budgets` runs in `mise run ci`. It builds the repository fixture garden under `tests/fixtures/` with the release `lsg`, then builds it again with 2,000 generated filler pages that nothing references. It fails when a fixture page's first-paint size (the page's HTML with the styles and scripts it loads before rendering) or script size exceeds the budget, or when any fixture page grows between the two builds. Timings stay out of CI because shared runners make them unreliable; the size checks and the filler-page check catch the regressions that would make first paint depend on garden size.

The harness reads only the two builds' output. It adds nothing to the SDK or the publisher.

## Risks / Trade-offs

- [Logseq changes its data format] → The decoder checks for the `window.logseq_db=` prefix and the `~#datascript/DB` tag and stops with a message naming the missing piece.
- [Logseq's app renders slowly on a cold start] → Each capture waits up to 60 seconds for the page title and reports a timeout per page instead of failing the run.
- [The index markup changes] → The index parser's unit test fails in CI before a harness run gives a wrong mapping.
- [Timings vary between runs] → `parity:perf` loads each page three times per profile and reports the median; budgets for timings are checked only on contributor machines.
- [Logseq's search answers from memory once the graph is loaded] → Its per-keystroke latency is measured after the graph has loaded, and the report shows the load time beside it, so a fast keystroke time does not hide the cost of loading the graph.
- [Playwright downloads a browser on first use] → The task installs Chromium through `playwright install chromium` and says so before it starts; later runs reuse the cached browser.

## Resolved Questions

### 1 - Should the harness be an `lsg` subcommand or separate scripts?

Decided by the agent without codekiln: separate Python scripts behind mise tasks. The harness compares `lsg` with another program's output for one garden, which is contributor work rather than something `lsg` users need, and a subcommand would add help text, clilint checks and a public contract to maintain.

### 2 - How does the build select the whole garden?

Decided by the agent without codekiln: one `--include` per top-level namespace, computed from the page file names. An `--all` flag on `lsg publish` would be simpler to call, but it changes the CLI; it can be proposed separately if users ask for whole-garden publishing.

### 3 - Where does the title-to-file list come from?

Decided by the agent without codekiln: from the links in the built `index.html`. The index already lists every page with its full title, so `lsg` needs no new output file; a unit test guards the parser.

### 4 - Which `lsg` options does the build use?

Decided by the agent without codekiln: `--filename-format triple-lowbar` (the garden's `:file/name-format`) and no stylesheet or homepage, with the garden's config unchanged. The comparison should show what the garden publishes today; turning on page properties or a homepage is a decision for the matching feature issues (#109, #110, #111).

### 5 - In what order are Logseq-only pages grouped?

Decided by the agent without codekiln: journals, reference-only pages, hidden files, titles `lsg` cannot select, then other pages. The large groups belong to other feature issues (#110, #111, #115), so checking them first keeps the other-pages group short enough to read.

### 6 - Should the harness also fix the `%2F` titles and apply `:hidden` in `lsg`?

Decided by the agent without codekiln: the harness only reports them. Fixing them changes page selection in `lsg`, which needs its own analysis and spec; the apply step opens an issue under #106 for the `%2F` titles. No `:hidden` entry affects the encode garden's pages today, so that fix waits until one does.

### 7 - Which browser tool takes the screenshots?

Decided by the agent without codekiln: Playwright for Python, pinned in `mise.toml`. It runs in the same language as the rest of the harness, can scroll a page before a full-page capture, and its Chromium build is already cached on codekiln's machine.

### 8 - Does CI run the harness?

Decided by the agent without codekiln: CI runs only the unit tests. A full run needs the encode garden, a fetched `gh-pages` branch and a browser, and takes minutes.

### 9 - Which numbers does the harness measure?

Decided by the agent without codekiln: time until the page title is visible, first contentful paint, data and script transferred before the title is visible, peak JavaScript heap, and search latency per keystroke, on a desktop and a phone profile. These are the measures codekiln named on #108. The title time is the main load measure because Logseq paints a loading screen first, which makes its first contentful paint look earlier than the page a reader waits for.

### 10 - Which phone profile does the harness use?

Decided by the agent without codekiln: the mobile settings Lighthouse uses (412 by 823, four times CPU slowdown, 150 ms round trip, 1.6 Mbps down). They are a widely used model of a mid-range phone on a slow network, so numbers can be compared with published Lighthouse results.

### 11 - How is transferred data counted on a local server that sends uncompressed files?

Decided by the agent without codekiln: as the gzip size of each response, with `db.gz` at its stored size. GitHub Pages serves both sites compressed, so raw sizes from the local server would overstate both builds.

### 12 - How is search measured while `lsg` has no search?

Decided by the agent without codekiln: the harness measures Logseq's search now and reports "no search" for `lsg` until a build contains the search input. #118 waits on #94, #95 and #96, and the harness should not delay their work.

### 13 - What are the starting budgets?

Decided by the agent without codekiln: the table under Decisions. Each budget is at least 100 times smaller than Logseq's first-paint download and holds for every fixture page on today's `lsg` build. Feature proposals tighten or add budgets for their own scripts and data.

### 14 - What runs in CI?

Decided by the agent without codekiln: the unit tests and `parity:budgets`, a size check on the repository fixture garden that also adds 2,000 unrelated pages and requires fixture pages to stay the same size. This adds to question 8: the full harness still needs the encode garden and a browser, but #106 asks for benchmark rows that run in CI, and sizes are stable enough on shared runners where timings are not.

### 15 - Where do the whole-garden numbers live?

Decided by the agent without codekiln: in `docs/parity/performance.md`, rewritten by `parity:perf --record`. A committed table lets each parity PR show its change in a diff.

## Open Questions

None.
