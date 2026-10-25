## 1. SDK journal identity

- [x] 1.1 Add the public `journals` module with `JournalDate` (civil-date weekday) and `DateFormat` parse, format and read for `yyyy`, `MM`, `M`, `dd`, `d`, `do`, `MMM`, `MMMM`, `EEE`, `EEEE` and literals; unit-test both encode-garden formats, the default format, impossible dates, case-insensitive names and a contradicting weekday
- [x] 1.2 Read `:journal/page-title-format` and `:journal/file-name-format` in `GardenSettings`, with defaults and diagnostics for malformed values; unit-test absent, malformed and valid configs
- [x] 1.3 Add `journal_identity` (date from the stem, title from `title::` or the title format) and `journal_list` (newest first, errors for unparseable stems); unit-test `journals/2026_10_23.md`, a `title::` override and `journals/2025_02_30.md`
- [x] 1.4 Add `parse_journal_reference`, `PageNameIndex::with_journal_formats` and `PageNameIndex::insert_journal`, with date fallback in `candidates`; unit-test `Feb 1st, 2025`, `2026-10-23 fri`, `2026_10_23` and a contradicting weekday

## 2. Journal pages

- [x] 2.1 Add `SiteOptions.journals` and `lsg publish --journals`; select journals through `public:: false`, skip unparseable files with a diagnostic, and report published journals in the human and JSON output
- [x] 2.2 Render journal pages with journal titles, properties, aliases and Linked References and without the namespace hierarchy section; add them to the page index and search index
- [x] 2.3 Add publication tests for journal pages, date-format links, private journals, a page title that equals a journal title, and output without `--journals`

## 3. Journals feed

- [x] 3.1 Compute the build date from `SOURCE_DATE_EPOCH` or the system clock and write the feed batches (`index.html` or `journals.html`, then `journals-<n>.html`) with articles, collapsed Linked References summaries and Earlier links
- [x] 3.2 Move the page index to `pages.html` when the feed holds `index.html`, and write no feed when journals are disabled or none is dated on or before the build date
- [x] 3.3 Write `lsg-journals.js` once to the site root, loaded by `lsg-shell.js` after the feed page's `load` event: append batches near the bottom, load Linked References on first opening with working filters, and keep links usable after failed fetches
- [x] 3.4 Add publication tests for batch contents and order, Earlier targets, future journals, `--homepage` with journals, and disabled journals; add browser-controller tests for batch loading, reference loading and fetch failures

## 4. Shell

- [x] 4.1 Add Journals with Tabler's calendar icon as the first sidebar item when a feed is written, marked current on feed batches; point All pages at `pages.html` when the feed holds `index.html`
- [x] 4.2 Add `g j` to `lsg-shell.js`; extend the shell tests

## 5. Harness and budgets

- [x] 5.1 Pass `--journals` in `parity:build`; add journals (one private, one future-dated, one with `title::`) to `tests/fixtures/parity-garden`
- [x] 5.2 Add the `lsg-journals.js` size check and the 1,000-older-journals size check to `parity:budgets` and `scripts/parity/budgets.toml`
- [x] 5.3 Add feed rows to `parity:perf` (first paint on `index.html`, heap after ten batches on the phone profile) and the whole-garden build time with and without journals
- [x] 5.4 Update help text and `docs/local-site.md` for `--journals`, and run `mise run clilint:check`

## 6. Verification

- [x] 6.1 Build the encode garden with `parity:build` and compare the home page, `2026-10-23 Fri`, `2019-08-07 Wed` and `Docker/For Mac/How to Install into Zsh` with the `gh-pages` build at 1280×800 and 390×844; record screenshots and the feed's journal order in `verification.md`
- [x] 6.2 Run `parity:compare` and record that the journals group drops from 534 to the 28 link-only dates
- [ ] 6.3 Run `parity:perf --record` and commit the new rows in `docs/parity/performance.md`
- [x] 6.4 Check a Gitpa build without `--journals`: output unchanged apart from the shell's existing markup
- [x] 6.5 Run `mise run ci`
