tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/All Pages]], [[lsg/Ref/Site Navigation]]

- # All Pages Compatibility
	- `lsg` publishes Logseq's All pages table: `Total N pages`, 40 rows at a time, a `Search page names` search on Enter with Logseq's fuzzy match, a journals toggle, and the `Page name`, `Backlinks`, `Created At` and `Updated At` columns, sorted as Logseq sorts them. Its dates are the first and last git commits of each page's file, where Logseq shows the moments its build parsed the file. It leaves out Logseq's editing controls, and it lists only published pages, so the pages that exist only because something references them are missing from the list.
	  id:: a0bca4bd-ecec-46f7-9b06-17e2a522c960
	- Measured on 2026-10-26 against the `gh-pages` build of garden commit `0ce74d6` (`gh-pages` `8132c95`, served locally, light theme, 1280×800, browser time zone America/New_York) and an `lsg` build of the whole garden from `lsg` `1e72969` (`mise run parity:build`, run `20261026T102638Z`). The OpenSpec change [match-logseq-all-pages](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-27-match-logseq-all-pages) then built the table, and [published-all-pages](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-all-pages/spec.md) holds its requirements.
	- ## What Logseq's published site shows
		- Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). The live build is newer than that checkout and adds a page-icon column; measured live values come first and win where they differ from the checkout or from [[lsg/Ref/Logseq Published Site/All Pages]]. The board's 15px cells, bordered table wrapper, `Total 16693 pages` and button-styled `Next` do not match the live build.
		- ### Which pages are listed
			- Every page in the database except names that are UUIDs, whiteboard drawings and Logseq's built-in pages ([`handler/page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/page.cljs#L738-L746)). `card`, `Favorites` and `contents` are listed; the garden creates them.
			- Journals are left out until a reader turns on the calendar toggle ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1080-L1087)). Whiteboards have their own toggle; the encode garden has none.
			- Pages that exist only because something references them are listed like any other page.
			- Measured: `Total 19206 pages`; with journals, `Total 19741 pages`.
		- ### Header and toolbar
			- `h1` `All pages`: 36px, weight 300, `rgb(0,0,0)`, 40px line height, margin `8px 0 24px`.
			- `Total N pages` below it: 14px, opacity 0.7, margin `0 0 16px 4px` ([`dicts/en.edn`](https://github.com/logseq/og/blob/6e7afa8eb/src/resources/dicts/en.edn#L627)). N counts the rows that match the current toggles and search.
			- A toolbar row that sticks to the top while the table scrolls ([`page.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.css#L95-L110)). Left: a red `Delete` button for checked rows and the search box. Right: the whiteboard toggle, the journal toggle (calendar icon, tooltip `Show journals`), the page buttons and a `⋯` menu with `Remove orphaned pages` and `All files`.
			- In the published build, `Delete` and `Remove orphaned pages` change only the reader's in-memory copy, and `All files` leads to a list of file paths.
		- ### Search
			- An input with a search icon, placeholder `Search page names`, 14px, white background, 28px tall, 173px wide ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1129-L1155)).
			- The search runs on Enter, not on each key. It is a fuzzy match on the lower-cased page name and returns at most 20 rows, ordered by match quality; Escape or the `×` link clears it ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1092-L1097)).
			- Measured: `mise` gives `Total 20 pages`, starting `mise` (598 backlinks), `mise/x`, `mise/g`, `mise/r`: shorter names first, not the most-linked pages.
		- ### Paging
			- 40 rows per page ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L978-L1006)). The control reads `1/481` followed by `Next ▸` and a `»` jump to the last page; from page 2 on, `«` and `◂ Prev` appear before the number. The same control repeats, right-aligned, below the table.
			- Changing page scrolls to the top. The page number is not in the URL, so reload and the back button return to page 1.
		- ### Table
			- Columns: a checkbox column (40px), the page icon (16px, from the `icon::` property; 2 garden pages have one), `Page name`, `Backlinks` (120px), `Created At` (150px), `Updated At` (150px) ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1205-L1250), [`page.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.css#L28-L93)).
			- Header cells: 14px, weight 400, `rgb(104,112,118)`, padding `10px 8px`, centred, `2px solid rgb(230,232,235)` bottom border ([`table.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/table.css#L1-L76)). Each column title is a link; clicking it sorts by that column and flips the direction, and the sorted column shows a caret.
			- Body cells: 16px, padding `6px 8px`, 49px rows. Rows alternate `rgb(251,252,253)` and `rgb(236,238,240)`. The page name is left-aligned, wraps, and is a page reference in `rgb(52,81,178)` with no brackets; shift-click opens it in the right sidebar. Backlinks and dates are centred and do not wrap.
			- Dates use `yyyy-MM-dd HH:mm` in the reader's time zone ([`date.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/date.cljs#L217-L221)).
			- On a phone (Logseq checks the user agent), only the checkbox, icon and `Page name` columns remain.
		- ### Sorting
			- The default sort is `Updated At`, newest first ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1014-L1015)).
			- All columns share one direction, and every click on a column title selects that column and flips the direction ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L916-L926)). From the default, a click on `Page name` sorts ascending, starting with symbols and digits (`)`, `.9999`, `.agents`); a click on `Backlinks` after that sorts descending, starting `Microfreak` 7,662, `Logseq` 6,429, `Logseq/Entity` 5,902; another click on `Backlinks` puts pages with 0 first.
			- Names sort by their lower-cased form ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L891-L897)).
		- ### What the columns contain
			- **Backlinks** counts the blocks whose references include the page ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L1076-L1079)). Namespace parents count, as in Linked References ([[lsg/Ref/Logseq Compatibility/References]]), and blocks on the page itself count too: `mise` shows 598 here and `589 Linked References` on its page, which leaves out its own blocks.
			- **Created At** and **Updated At** are the times the page entered the database. A published build has no file history, so the values are the moments the publish build parsed each file (2026-10-25 16:31–16:35 in the measured time zone, a few minutes before the `gh-pages` commit), or, for pages that have no timestamp, the moment the reader's browser loaded the graph ([`handler/common.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/common.cljs#L47-L65)). `card`, `Favorites` and `contents` show the reader's current time and so sort first by default.
			- The default order is therefore close to the reverse of the order in which the build parsed the files: the first rows are `card`, `Favorites`, `contents`, then `yt-dlp/GitHub`, `CLI/Tool/yt-dlp`, `yt-dlp/PyPI`, `ya`, `yazi/Plugin/spotter`.
		- ### Performance in Logseq
			- All pages is a route inside Logseq's single-page app. It downloads the whole graph (`static/js/db.gz`, 22.6 MB of transfer before the title) and about 8 MB of script, then counts every page's references in the browser. [[lsg/Ref/Logseq Compatibility/Performance]] measured 19,820 ms to the title on desktop and 197,866 ms on the phone profile, with a peak JavaScript heap of about 2 GB.
	- ## What lsg publishes
		- The page index is titled `All pages`. It opens at `index.html`, or at `pages.html` when a homepage or journals feed takes `index.html`; the left sidebar's All pages item and the `g a` shortcut lead there. [[lsg/Ref/Site Navigation]] describes it for readers.
		- The screen shows the 36px weight-300 `All pages` heading, `Total N pages` where N counts the rows of the current view, and a sticky toolbar with the `Search page names` input, the journals toggle and the paging control. The paging control repeats below the table.
		- The table has the columns `Page name`, `Backlinks`, `Created At` and `Updated At`, with the measured header and body cell styles, alternating row colours and page links without brackets. A page with an `icon::` property shows the icon before its name.
		- **Backlinks** counts the blocks that refer to the page, including references to its namespace children and blocks on the page itself: `mise` shows 598, `Microfreak` 7,662 and `Logseq` 6,429. The page's Linked References count leaves out its own blocks, as Logseq's does.
		- **Created At** and **Updated At** are the times of the first and last git commits that touched the page's file, shown as `yyyy-MM-dd HH:mm` in the reader's time zone, or in UTC without JavaScript. Generated namespace pages have no file and no dates. A garden that is not a git repository gets no date columns, and its rows are ordered by page name.
		- The default view hides journals and lists pages by `Updated At`, newest first, 40 rows at a time; generated namespace pages follow the dated pages, by name. `pages-2.html`, `pages-3.html` and onward hold the rest of the default view, so every page is reachable without JavaScript.
		- Clicking a column title sorts by that column and flips one direction shared by all columns, with a caret on the sorted column; names compare lower-cased. `Page name` and then `Backlinks` from the default view gives `Microfreak`, `Logseq`, `Logseq/Entity`.
		- Search runs on Enter, scores page names with Logseq's fuzzy `score`, and shows at most 20 rows; Escape clears it. `mise` gives `Total 20 pages` with `mise` first.
		- The journals toggle adds journals to the view, and the total grows by the number of published journals.
		- Sorting, search and the journals toggle load `_pages/list.json`, which holds every entry, the first time a reader uses one of them. Paging through the default view never loads it.
		- Below 640px wide, the table shows only the icon and `Page name` columns, and the page does not scroll sideways.
		- Generated namespace pages appear in the list with their dotted link style and accessible Namespace name, and without a visible Namespace badge.
		- The SDK answers the list: `ReferenceIndex::backlink_count` counts backlinks, `GitDates::read` reads each file's commit times in one `git log` pass, and `page_list` gives each entry its title, route, kind, icon, backlink count, sort key and dates. The publisher renders the table, the static paging pages, the list data file, `lsg-all-pages.js` and the styles.
		- On the whole encode garden, measured with the change at `48fcc54` on garden commit `083106d`, the title showed in 23.1 ms on desktop and 235 ms on the phone profile, with 13.2 KB of data before it. Sorting by Backlinks, which loads the 172.9 KB gzip page list, took 57 ms on desktop and 1,069 ms on the phone profile. [[lsg/Ref/Logseq Compatibility/Performance]] holds every row, and the `[all_pages]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) holds the budgets.
	- ## Differences from Logseq
		- Dates are git commit times. Logseq's published values are the moments one build parsed each file and change on every build; git dates keep the same columns and the newest-first default, and stay the same for one commit.
		- Row checkboxes, `Delete`, `Remove orphaned pages`, the whiteboard toggle and the `⋯` menu are left out. In Logseq's published site they change only the reader's in-memory copy.
		- `lsg` lists only the pages it publishes. Logseq also lists the pages that exist only because something references them, so its total is larger by the reference-only pages that `parity:compare` reports: 7,540 on 2026-10-26.
		- The phone layout starts below a viewport width of 640px. Logseq decides by the browser's user agent.
		- `lsg` has no `All files` list and no whiteboards.
		- Logseq's dark-theme values were not measured; the table uses `lsg`'s theme tokens.
	- ## Before the change
		- The `lsg` build measured on 2026-10-26, from `1e72969`, wrote the list from `crates/logseq-gardener-sdk/src/site/mod.rs` (the index list, lines 477–640).
		- **Route:** `pages.html` when the garden had a home page or a journals home, otherwise `index.html`. The shell's `All pages` sidebar item and the `g a` shortcut led there.
		- **Content:** title and heading `Garden`, then one `<ul>` of 10,776 links: every published page, every journal (dates sorted among the other titles) and 3,266 namespace pages that `lsg` generates for hierarchy nodes without a file, each marked `Namespace`. Rows sorted by title in byte order.
		- **Missing:** the total, backlink counts, dates, the table, sorting, search, the journal toggle and paging.
		- **Size:** 1,266,786 bytes, 267,848 gzipped, all before the title, the largest first paint in the build. [[lsg/Ref/Logseq Compatibility/Performance]] marked its 271 KB of data before the title as over the 100 KB budget on both profiles. Title visible: 85.6 ms on desktop, 233 ms on phone.
		- **Where the data lived:** `ReferenceIndex` in `crates/logseq-gardener-sdk/src/references.rs` already counted each page's linked references for the CLI, the language server and the publisher, but its `linked` answer leaves out blocks on the page itself, so its counts are Logseq's Linked References count, not its Backlinks count.
	- ## Fixture pages
		- | Page | Why | Logseq | `lsg` on 2026-10-26, before the change |
		  | --- | --- | --- | --- |
		  | `All pages` | The screen itself; in `scripts/parity/fixtures.txt` | `Total 19206 pages`, `1/481` | `Garden`, 10,776 links, 1.27 MB |
		  | `mise` | Search results; backlinks including its own blocks | 598 backlinks; first of 20 results for `mise` | listed, no count |
		  | `Microfreak` | Most backlinks, through namespace children | 7,662 | listed, no count |
		  | `card`, `Favorites`, `contents` | Pages with no timestamp that lead Logseq's default order | first three rows | listed, no count |
		  | a page with `icon::` | Icon column | icon before the name | no icon |
