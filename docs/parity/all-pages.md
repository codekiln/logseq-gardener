# All pages

Logseq's All pages screen is a table of every non-journal page in the graph, 40 rows at a time: a total count, a page-name search, a toggle that adds journals, page buttons, and sortable columns for page name, backlink count, and creation and update times. `lsg` publishes `pages.html`, titled `Garden`: one alphabetical list of 10,776 links, journals mixed in, with no counts, no sorting, no search box and no paging.

The gap is mostly the table and its controls. Two parts need decisions in the proposal. Logseq's creation and update times in a published build are the moments the build or the reader's browser parsed each page, so they carry no information about the garden. Logseq also lists 7,540 pages that exist only because something references them, and `lsg` does not publish those pages yet.

`pages.html` is already the largest first paint in the `lsg` build: 1.27 MB, 268 KB gzipped, against a 100 KB budget. Logseq's version needs the whole graph first: 19.8 s to show the title on the desktop profile and 198 s on the phone profile ([Parity performance](performance.md)). The `lsg` page should contain only the first 40 rows and load the rest of the list when a reader sorts, searches or changes page. Backlink counts come from the SDK's existing reference index; the publisher renders the table.

Measured on 2026-10-26 against the `gh-pages` build of garden commit `0ce74d6` (`gh-pages` `8132c95`, served locally, light theme, 1280×800, browser time zone America/New_York) and an `lsg` build of the whole garden from `lsg` `1e72969` (`mise run parity:build`, run `20261026T102638Z`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). The live build is newer than that checkout and adds a page-icon column; measured live values come first and win where they differ from the checkout or from [lsg/Ref/Logseq Published Site/All Pages](../pages/lsg___Ref___Logseq%20Published%20Site___All%20Pages.md). The board's 15px cells, bordered table wrapper, `Total 16693 pages` and button-styled `Next` do not match the live build.

### Which pages are listed

- Every page in the database except names that are UUIDs, whiteboard drawings and Logseq's built-in pages ([`handler/page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/page.cljs#L738-L746)). `card`, `Favorites` and `contents` are listed; the garden creates them.
- Journals are left out until a reader turns on the calendar toggle ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1080-L1087)). Whiteboards have their own toggle; the encode garden has none.
- Pages that exist only because something references them are listed like any other page.
- Measured: `Total 19206 pages`; with journals, `Total 19741 pages`.

### Header and toolbar

- `h1` `All pages`: 36px, weight 300, `rgb(0,0,0)`, 40px line height, margin `8px 0 24px`.
- `Total N pages` below it: 14px, opacity 0.7, margin `0 0 16px 4px` ([`dicts/en.edn`](https://github.com/logseq/og/blob/6e7afa8eb/src/resources/dicts/en.edn#L627)). N counts the rows that match the current toggles and search.
- A toolbar row that sticks to the top while the table scrolls ([`page.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.css#L95-L110)). Left: a red `Delete` button for checked rows and the search box. Right: the whiteboard toggle, the journal toggle (calendar icon, tooltip `Show journals`), the page buttons and a `⋯` menu with `Remove orphaned pages` and `All files`.
- In the published build, `Delete` and `Remove orphaned pages` change only the reader's in-memory copy, and `All files` leads to a list of file paths.

### Search

- An input with a search icon, placeholder `Search page names`, 14px, white background, 28px tall, 173px wide ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1129-L1155)).
- The search runs on Enter, not on each key. It is a fuzzy match on the lower-cased page name and returns at most 20 rows, ordered by match quality; Escape or the `×` link clears it ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1092-L1097)).
- Measured: `mise` gives `Total 20 pages`, starting `mise` (598 backlinks), `mise/x`, `mise/g`, `mise/r`: shorter names first, not the most-linked pages.

### Paging

- 40 rows per page ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L978-L1006)). The control reads `1/481` followed by `Next ▸` and a `»` jump to the last page; from page 2 on, `«` and `◂ Prev` appear before the number. The same control repeats, right-aligned, below the table.
- Changing page scrolls to the top. The page number is not in the URL, so reload and the back button return to page 1.

### Table

- Columns: a checkbox column (40px), the page icon (16px, from the `icon::` property; 2 garden pages have one), `Page name`, `Backlinks` (120px), `Created At` (150px), `Updated At` (150px) ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1205-L1250), [`page.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.css#L28-L93)).
- Header cells: 14px, weight 400, `rgb(104,112,118)`, padding `10px 8px`, centred, `2px solid rgb(230,232,235)` bottom border ([`table.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/table.css#L1-L76)). Each column title is a link; clicking it sorts by that column and flips the direction, and the sorted column shows a caret.
- Body cells: 16px, padding `6px 8px`, 49px rows. Rows alternate `rgb(251,252,253)` and `rgb(236,238,240)`. The page name is left-aligned, wraps, and is a page reference in `rgb(52,81,178)` with no brackets; shift-click opens it in the right sidebar. Backlinks and dates are centred and do not wrap.
- Dates use `yyyy-MM-dd HH:mm` in the reader's time zone ([`date.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/date.cljs#L217-L221)).
- On a phone (Logseq checks the user agent), only the checkbox, icon and `Page name` columns remain.

### Sorting

- The default sort is `Updated At`, newest first ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1014-L1015)).
- All columns share one direction, and every click on a column title selects that column and flips the direction ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L916-L926)). From the default, a click on `Page name` sorts ascending, starting with symbols and digits (`)`, `.9999`, `.agents`); a click on `Backlinks` after that sorts descending, starting `Microfreak` 7,662, `Logseq` 6,429, `Logseq/Entity` 5,902; another click on `Backlinks` puts pages with 0 first.
- Names sort by their lower-cased form ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L891-L897)).

### What the columns contain

- **Backlinks** counts the blocks whose references include the page ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1076-L1079)). Namespace parents count, as in Linked References ([references analysis](references.md)), and blocks on the page itself count too: `mise` shows 598 here and `589 Linked References` on its page, which leaves out its own blocks.
- **Created At** and **Updated At** are the times the page entered the database. A published build has no file history, so the values are the moments the publish build parsed each file (2026-10-25 16:31–16:35 in the measured time zone, a few minutes before the `gh-pages` commit), or, for pages that have no timestamp, the moment the reader's browser loaded the graph ([`handler/common.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/common.cljs#L47-L65)). `card`, `Favorites` and `contents` show the reader's current time and so sort first by default.
- The default order is therefore close to the reverse of the order in which the build parsed the files: the first rows are `card`, `Favorites`, `contents`, then `yt-dlp/GitHub`, `CLI/Tool/yt-dlp`, `yt-dlp/PyPI`, `ya`, `yazi/Plugin/spotter`.

### Performance in Logseq

All pages is a route inside Logseq's single-page app. It downloads the whole graph (`static/js/db.gz`, 22.6 MB of transfer before the title) and about 8 MB of script, then counts every page's references in the browser. [Parity performance](performance.md): 19,820 ms to the title on desktop and 197,866 ms on the phone profile, with a peak JavaScript heap of about 2 GB.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/mod.rs` (the index list, lines 477–640), checked against the whole-garden build:

- **Route:** `pages.html` when the garden has a home page or a journals home, otherwise `index.html`. The shell's `All pages` sidebar item and the `g a` shortcut lead there.
- **Content:** title and heading `Garden`, then one `<ul>` of 10,776 links: every published page, every journal (dates sort among the other titles) and 3,266 namespace pages that `lsg` generates for hierarchy nodes without a file, each marked `Namespace`. Rows sort by title in byte order.
- **Missing:** the total, backlink counts, dates, the table, sorting, search, the journal toggle and paging.
- **Size:** 1,266,786 bytes, 267,848 gzipped, all before the title. [Parity performance](performance.md) marks its 271 KB data-before-title as over the 100 KB budget on both profiles. Title visible: 85.6 ms on desktop, 233 ms on phone.
- **Reference-only pages:** `lsg` publishes no page for the 7,540 Logseq pages that only references create (`parity:compare`), so it has no destination to list for them.
- **Where the data lives:** `ReferenceIndex` in `crates/logseq-gardener-sdk/src/references.rs` already counts each page's linked references for the CLI, the language server and the publisher, but `linked` leaves out blocks on the page itself, so its counts are Logseq's Linked References count, not its Backlinks count.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Title and total.** The page is titled `All pages`, with the 36px weight-300 heading and `Total N pages` below it, where N counts the listed rows. Check: with journals hidden, N equals the number of non-journal pages `lsg` publishes.
- 2. **Table.** Columns `Page name`, `Backlinks`, `Created At`, `Updated At` with the measured header and cell styles, alternating row colours and page links without brackets. Check: computed styles on the `All pages` fixture match the values under [Table](#table).
- 3. **Backlink counts.** Each row shows the number of blocks that refer to the page, including namespace children and blocks on the page itself. Check: `mise` shows 598, `Microfreak` 7,662, `Logseq` 6,429.
- 4. **Dates.** Logseq's dates are build or load times. The proposal decides whether `lsg` shows the two date columns, what they contain (for example the build time, or a date from the garden such as git history) and how the default order is chosen without them. Check: whatever the proposal decides, the default first rows are stable across two builds of the same commit.
- 5. **Sorting.** Clicking a column title sorts by it and flips the shared direction, with a caret on the sorted column; names compare lower-cased. Check: `Backlinks` descending starts `Microfreak`, `Logseq`, `Logseq/Entity`.
- 6. **Paging.** 40 rows per page, `N/M` with `Prev`, `Next`, first and last controls, repeated below the table, scrolling to the top on change. Check: the `All pages` fixture reads `1/M` where M is the row count over 40, rounded up.
- 7. **Search.** A `Search page names` input that runs on Enter, fuzzy-matches lower-cased names, shows at most 20 rows and clears on Escape. Check: `mise` lists `mise` first and shows `Total 20 pages`.
- 8. **Journals toggle.** Journals are hidden by default and a calendar toggle adds them. Check: the total grows by the number of published journals.
- 9. **Phone layout.** At phone width only the page-name column remains. Check: at 390×844 the table has one data column and no horizontal scroll.
- 10. **Editing controls.** Logseq shows checkboxes, `Delete`, the whiteboard toggle and the `⋯` menu, none of which does anything lasting in a published site. The proposal decides whether `lsg` draws them. Check: the decided controls are present or absent on the fixture.
- 11. **Performance.** The `All pages` first paint fits the `[desktop]` and `[phone]` budgets (100 KB data, 16 KB script before the title) whatever the garden's size: the page contains the header, total and first 40 rows, and the full list loads only when a reader sorts, searches, toggles journals or changes page. Without scripts, the first rows link to further static pages so every page stays reachable. Check: `parity:budgets` adds an `All pages` row that stays within budget and does not grow when 2,000 pages are added; `parity:perf` records `All pages` under 100 KB.
- 12. **Reference-only pages.** Logseq lists pages that only references create. `lsg` can list them once it publishes them; until then the totals differ by the 7,540 pages `parity:compare` reports. Check: the report's reference-only group explains the whole difference between Logseq's total and `lsg`'s.

## Where the work belongs

- **SDK:** one page-list operation over the garden index: each page's title, route, kind (page, journal, generated namespace page), icon, and backlink count. The backlink count is a `ReferenceIndex` query that includes blocks on the page itself, beside the existing `linked` query that leaves them out. Sorting keys (lower-cased names) and any date the proposal chooses are derived there too, so the CLI and the language server can answer "list pages by backlinks" the same way.
- **Publisher:** the table markup and styles, the static first page, the list data file for the browser, the sort, search, toggle and paging script, and the budget rows.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `All pages` | The screen itself; already in `scripts/parity/fixtures.txt` | `Total 19206 pages`, `1/481` | `Garden`, 10,776 links, 1.27 MB |
| `mise` | Search results; backlinks including its own blocks | 598 backlinks; first of 20 results for `mise` | listed, no count |
| `Microfreak` | Most backlinks, through namespace children | 7,662 | listed, no count |
| `card`, `Favorites`, `contents` | Pages with no timestamp that lead Logseq's default order | first three rows | listed, no count |
| a page with `icon::` | Icon column | icon before the name | no icon |

## Out of scope

- Publishing reference-only pages, which belongs with [Issue #111](https://github.com/codekiln/logseq-gardener/issues/111) and [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115); this change lists whatever pages `lsg` publishes.
- Shift-click to open a page in the right sidebar, which belongs with [Issue #119](https://github.com/codekiln/logseq-gardener/issues/119).
- The `All files` list and whiteboards, which `lsg` does not publish.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
