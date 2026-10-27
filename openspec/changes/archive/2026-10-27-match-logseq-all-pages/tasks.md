## 1. SDK

- [x] 1.1 Add `ReferenceIndex::backlink_count` counting the page's own blocks as well as linked references; unit-test the `mise` own-block case and namespace children
- [x] 1.2 Add `git_dates` reading first and last commit times per file from one `git log --format=%ct --name-only` pass; unit-test a temporary repository, a non-repository and a shallow clone diagnostic
- [x] 1.3 Add the `page_list` module with `PageListEntry`, kinds, icons, sort keys and `cmp_by`; unit-test every scenario in the page-list spec

## 2. Markup and static paging

- [x] 2.1 Move the index from `site/mod.rs` to `site/all_pages.rs`: heading, total, toolbar, table, icon, `<time>` dates, paging control above and below; no editing controls; generated rows without the badge
- [x] 2.2 Write `pages-<n>.html` for the default view, 40 rows each, with `Prev`, `Next ▸`, `«` and `»` links; test the first, a middle and the last page and the no-dates layout
- [x] 2.3 Write `_pages/list.json` with compact rows for every entry, journals included; test its shape and escaping
- [x] 2.4 Update the existing index, homepage, journals and hierarchy tests that assert the old list or the badge

## 3. Script and styles

- [x] 3.1 Add `site/all-pages.js`: fetch the list on the first sort, search or toggle; sort with a shared direction and caret; Logseq's fuzzy `score` on Enter with 20 results and Escape to clear; journals toggle; paging from memory; local-time dates; initialize again after a content swap. Add tests in `tests/`, including the `score` port
- [x] 3.2 Add `site/all-pages.css` with the analysis's light values, the shared table tokens, the sticky toolbar and the under-640px layout

## 4. Harness and budgets

- [x] 4.1 Add the `[all_pages]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check first paint, paging page size, grown-garden growth, script and list sizes, and `parity:perf` mark the heap, search and build values

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build` and compare All pages with the `gh-pages` build at 1280×800 and 390×844 in both themes: heading, total, first rows, column styles, sort by `Backlinks`, search for `mise`, journals toggle; record values, screenshots and page sizes before and after in `verification.md`
- [x] 5.2 Run `parity:perf` and commit the updated rows in `docs/parity/performance.md` (written by hand; see `verification.md`)
- [x] 5.3 Sync this change's `published-namespace-hierarchy` delta only after `match-logseq-hierarchy` is archived (done in the cleanup pull request)
- [x] 5.4 Run `mise run ci`
