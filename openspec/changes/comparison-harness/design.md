## Context

The [comparison harness analysis](../../../docs/parity/comparison-harness.md) built the whole encode garden with `lsg` and decoded Logseq's `gh-pages` build by hand on 2026-10-24. Every page `lsg` published exists in Logseq's build. Logseq's extra pages fall into groups: 505 journals with content, about 6,600 reference-only pages, three titles containing `%2F`, and three other pages. Each later parity issue under #106 needs the same build and comparison, plus screenshots of the same page from both builds.

The harness is a contributor tool. It runs on a machine with the encode garden checked out through `ghq`, needs the `gh-pages` branch fetched, and takes about two minutes for the build. CI runs only its unit tests.

## Goals / Non-Goals

**Goals:**

- One command per job: build, compare, screenshot.
- A report whose groups separate the large known differences (journals, reference-only pages) from pages that need an explanation.
- Screenshots that a person can review page by page.

**Non-Goals:**

- Automatic pixel or DOM diff scoring of screenshots. A score would let later parity PRs show progress as a number; it waits until the feature issues have closed the large gaps.

## Decisions

The harness is three mise file tasks under `.mise/tasks/parity/` (`build`, `compare`, `screenshots`) backed by Python modules in `scripts/parity/`. Python is already pinned in `mise.toml` and used by `scripts/docs.py`, and keeping the harness outside the `lsg` binary leaves the CLI contract, its help text and the clilint checks unchanged.

`parity:build` runs a release build of `lsg` and passes one `--include` per top-level namespace. It finds the roots by decoding each file name in `pages/` with the `triple-lowbar` rules: split on `___`, URL-decode, take the first segment. A root that fails `lsg`'s selector rules is skipped and listed. Output goes to `target/parity/<UTC timestamp>/site/`, with `settings.json` beside it holding the garden commit, the `lsg` version and the arguments. `target/` is already ignored by git, and a timestamped directory satisfies `lsg`'s rule that output is new on each run. The build uses the garden's `logseq/config.edn` as it is, so the comparison shows what the garden would publish today; for example, page properties stay off because the encode garden does not set `:publishing/page-properties?`.

`parity:compare` reads Logseq's build with `git archive origin/gh-pages` into `target/parity/<timestamp>/logseq/`, so the garden checkout is never switched. The decoder reads `static/js/db.gz`, parses the outer JavaScript string as JSON, reverses publish-spa's `logseq____&…;` replacements in the order Logseq's `frontend/publishing.cljs` uses, and decodes Transit's cached keys to get the datoms. Page records come from `:block/name`, `:block/original-name`, `:block/journal?` and `:block/page`.

The `lsg` title-to-file list comes from the links in the built `index.html`, which lists every page with its full title. This avoids a new output file in `lsg`. A unit test on a small built index keeps the parser in step with the index markup.

`:hidden` matching follows Logseq's rule in `logseq.common.config/hidden?`: a pattern without a leading `/` gains one, and a file is hidden when `/` plus its graph-relative path starts with the pattern. With the encode garden's current `:hidden` list no file in `pages/` or `journals/` matches, so this group is empty today; it stays in the report so a future entry shows up.

`parity:screenshots` uses Playwright for Python with Chromium, pinned in `mise.toml`. It serves each build with Python's `http.server` bound to 127.0.0.1 on a free port, opens each fixture at 1280 by 800 pixels in the light colour scheme, waits for `.page-title` (Logseq) or the page heading (`lsg`), scrolls to the bottom in steps until the page height stops growing, and then captures the full page. The fixture list is `scripts/parity/fixtures.txt`, one title per line, plus two route names, `Home` and `All pages`, for Logseq's `#/` and `#/all-pages`. The run writes `screenshots/index.html` with the two captures of each fixture side by side.

## Risks / Trade-offs

- [Logseq changes its data format] → The decoder checks for the `window.logseq_db=` prefix and the `~#datascript/DB` tag and stops with a message naming the missing piece.
- [Logseq's app renders slowly on a cold start] → Each capture waits up to 60 seconds for the page title and reports a timeout per page instead of failing the run.
- [The index markup changes] → The index parser's unit test fails in CI before a harness run gives a wrong mapping.
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

## Open Questions

None.
