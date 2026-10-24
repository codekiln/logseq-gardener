## 1. Logseq build reader

- [x] 1.1 Add `scripts/parity/logseq_db.py`: decode `db.gz` (outer string, `logseq____&…;` replacements, Transit cache) into page records with name, title, journal flag and content flag.
- [x] 1.2 Add a small `db.gz` fixture and unit tests for page records, journal and content flags, and escaped characters in titles.

## 2. Build and title mapping

- [x] 2.1 Add `scripts/parity/build.py` and `.mise/tasks/parity/build`: compute top-level roots from page file names, run a release `lsg publish` into `target/parity/<UTC timestamp>/site/`, and write `settings.json` with the garden commit, `lsg` version, arguments and skipped roots.
- [x] 2.2 Add the `lsg` index parser and the Logseq route builder with lower-case and NFC title matching; unit-test both on fixtures.

## 3. Comparison report

- [x] 3.1 Add `scripts/parity/compare.py` and `.mise/tasks/parity/compare`: archive `origin/gh-pages`, read both page lists, apply Logseq's `:hidden` prefix rule, and write `report.md` and `report.json` with the groups in the specified order.
- [x] 3.2 Unit-test the grouping, including one page per group and an `lsg`-only page.

## 4. Screenshots

- [x] 4.1 Pin Playwright for Python in `mise.toml` and follow the dependency notice process if it applies to contributor tools. Playwright and uv run only on contributor machines and ship with no `lsg` release, so the Cargo notice process does not apply; `uv` is pinned in `mise.toml` and Playwright 1.60.0 in the scripts' inline metadata.
- [x] 4.2 Add `scripts/parity/fixtures.txt` with the fixture pages from the analysis, plus `Home` and `All pages`.
- [x] 4.3 Add `scripts/parity/screenshots.py` and `.mise/tasks/parity/screenshots`: serve both builds on 127.0.0.1, wait for the title, scroll until the height settles, capture full pages at 1280 by 800 in light mode, and write the side-by-side `index.html`.

## 5. Performance

- [x] 5.1 Add `scripts/parity/budgets.toml` with the starting budgets and `scripts/parity/search-queries.txt`.
- [x] 5.2 Add `scripts/parity/perf.py` and `.mise/tasks/parity/perf`: desktop and phone profiles, three loads per page, title time, first contentful paint, gzip data and script before the title, peak JavaScript heap, per-keystroke search latency, budget marks, and `--record` to rewrite `docs/parity/performance.md`.
- [x] 5.3 Unit-test the gzip accounting, the median and percentile summary, and the budget marks on stored samples.
- [x] 5.4 Add `scripts/parity/budgets.py` and `.mise/tasks/parity/budgets`: build the fixture garden twice, the second time with 2,000 unreferenced pages, and check sizes against the budgets; run it from `mise run ci`.

## 6. Wiring and verification

- [x] 6.1 Run the harness unit tests from `mise run test` without the garden, network or a browser.
- [x] 6.2 Add `docs/parity/README.md` explaining the five tasks and where their output goes.
- [x] 6.3 Run the four contributor tasks on the encode garden; record the group counts, the `tmux` screenshots and `docs/parity/performance.md` in the implementation PR, and check the counts against the analysis.
- [x] 6.4 Open an issue under #106 for publishing titles that contain `%2F`. Opened #129.
- [ ] 6.5 Run `mise run ci`, open the implementation PR and create its cleanup sub-issue under #108.
