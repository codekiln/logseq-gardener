## 1. SDK namespace index

- [x] 1.1 Add the `namespaces` module with `NamespaceIndex::build`, `replace` and `remove`; unit-test that replacing one entry rebuilds nothing else
- [x] 1.2 Add `hierarchy(page, RowFilter)` with the subject choice, descendants, the parent row for leaves, lower-cased ordering and segments; unit-test every scenario in the namespace-index spec, including withheld titles and both row filters

## 2. Markup and fragments

- [x] 2.1 Make `bottom_view` render `hierarchy(page, StoredOnly)`: no own row, no section on plain pages, alias subjects, row spelling, brackets from `settings.show_brackets`, generated link styles without the badge, the reference-section header
- [x] 2.2 Write rows after the first 100 to `_refs/<page>-hierarchy-<n>.html`, about 500 per file, with the no-script link and next links; test paging, escaping and withholding
- [x] 2.3 Update the existing hierarchy tests and the property tests that assert the own row or the badge

## 3. Script and styles

- [x] 3.1 Load hierarchy fragments on scroll in `references.js` and initialize them again after a content swap; add tests in `tests/`
- [x] 3.2 Move the hierarchy styles from `site/mod.rs` to `site/references.css` with the light values from the analysis and the dark tokens

## 4. Harness and budgets

- [x] 4.1 Add the `[hierarchy]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check inline rows, styles, script growth, fragment sizes and the grown-namespace fixture, and `parity:perf` mark the phone heap and build time

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build` and compare `tmux`, `tmux/Config`, `tmux/Config/local`, `tmux/Command`, `AGI`, `Abbreviation`, `GitP/A/Session/26/10/20-Tue`, `Person` and `Microfreak` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record rows, computed styles, screenshots and page sizes before and after in `verification.md`
- [x] 5.2 Run `parity:perf --record` and commit the updated rows in `docs/parity/performance.md`
- [x] 5.3 Run `mise run ci`
