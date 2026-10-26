## 1. SDK reference index

- [x] 1.1 Add the `references` module with `ReferenceIndex::build` from per-document contributions, moving the reference collection out of `site/backlinks.rs`; add namespace-parent expansion and drop property names; unit-test aliases, namespace parents, withheld sources and replacing one document's contribution
- [x] 1.2 Add `path_refs`, `related` and `starting_filter`; unit-test ordering, ties, `filters::` with includes, excludes, unknown names and `{}`
- [x] 1.3 Add `tagged_with`; unit-test alias tags and lower-cased ordering
- [x] 1.4 Add the `aho-corasick` dependency through the [dependency notice process](../../../../docs/licensing.md) and implement `unlinked` with Logseq's match rule in one pass; unit-test the boundary cases from the reference-index spec

## 2. Markup and fragments

- [x] 2.1 Render section headers in Logseq's order with the new count text and the starting-filter count, and omit Linked References on pages with none
- [x] 2.2 Write `<page>-linked-<n>.html` and `<page>-unlinked-<n>.html` in `_refs/`, the first file carrying the section's data, with groups ordered, breadcrumbs shortened and the no-script links; test anchors, withholding and fragment paging
- [x] 2.3 Render the Pages tagged with section
- [x] 2.4 Keep hiding `filters::` in page properties

## 3. Script and styles

- [x] 3.1 Replace `backlinks.js` with `references.js`: fragment loading on open and scroll, the filter dialog with click, shift-click, Enter and Shift+Enter, options recomputed from kept blocks, `sessionStorage` with fallback, and re-initialization after a content swap; add tests in `tests/`
- [x] 3.2 Add `site/references.css` and its theme tokens for both themes, light values from the analysis

## 4. Harness and budgets

- [x] 4.1 Add the `[references]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check script, style and fragment sizes and `parity:perf` mark open time, filter time, phone heap and build time

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build` and compare `mise`, `tmux`, `Claude/Code`, `Term`, `Microfreak/UG/07 Filter/Asset/01-The-Analogue-Filter` and `McKinsey` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record counts, group order, computed styles, screenshots, the ten largest unlinked totals, total fragment size and page sizes before and after in `verification.md`
- [x] 5.2 Run `parity:perf --record` and commit the updated rows in `docs/parity/performance.md`
- [x] 5.3 Run `mise run ci`
