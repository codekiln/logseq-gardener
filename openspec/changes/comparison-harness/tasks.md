## 1. Logseq build reader

- [ ] 1.1 Add `scripts/parity/logseq_db.py`: decode `db.gz` (outer string, `logseq____&…;` replacements, Transit cache) into page records with name, title, journal flag and content flag.
- [ ] 1.2 Add a small `db.gz` fixture and unit tests for page records, journal and content flags, and escaped characters in titles.

## 2. Build and title mapping

- [ ] 2.1 Add `scripts/parity/build.py` and `.mise/tasks/parity/build`: compute top-level roots from page file names, run a release `lsg publish` into `target/parity/<UTC timestamp>/site/`, and write `settings.json` with the garden commit, `lsg` version, arguments and skipped roots.
- [ ] 2.2 Add the `lsg` index parser and the Logseq route builder with lower-case and NFC title matching; unit-test both on fixtures.

## 3. Comparison report

- [ ] 3.1 Add `scripts/parity/compare.py` and `.mise/tasks/parity/compare`: archive `origin/gh-pages`, read both page lists, apply Logseq's `:hidden` prefix rule, and write `report.md` and `report.json` with the groups in the specified order.
- [ ] 3.2 Unit-test the grouping, including one page per group and an `lsg`-only page.

## 4. Screenshots

- [ ] 4.1 Pin Playwright for Python in `mise.toml` and follow the dependency notice process if it applies to contributor tools.
- [ ] 4.2 Add `scripts/parity/fixtures.txt` with the fixture pages from the analysis, plus `Home` and `All pages`.
- [ ] 4.3 Add `scripts/parity/screenshots.py` and `.mise/tasks/parity/screenshots`: serve both builds on 127.0.0.1, wait for the title, scroll until the height settles, capture full pages at 1280 by 800 in light mode, and write the side-by-side `index.html`.

## 5. Wiring and verification

- [ ] 5.1 Run the harness unit tests from `mise run test` without the garden, network or a browser.
- [ ] 5.2 Add `docs/parity/README.md` explaining the three tasks and where their output goes.
- [ ] 5.3 Run all three tasks on the encode garden; record the group counts and the `tmux` screenshots in the implementation PR, and check the counts against the analysis.
- [ ] 5.4 Open an issue under #106 for publishing titles that contain `%2F`.
- [ ] 5.5 Run `mise run ci`, open the implementation PR and create its cleanup sub-issue under #108.
