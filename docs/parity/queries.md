# Queries

Logseq renders a `{{query …}}` macro or a `#+BEGIN_QUERY` block as a live result box: a header with a title, a result count and view controls, then the matching blocks grouped by page, or a table. It evaluates every query in the reader's browser, against the whole graph, after the whole graph has loaded. `lsg` evaluates nothing: it prints a simple query's source as inline code and an advanced query's source as a plain `<aside>`.

The encode garden barely uses queries. Its pages and journals hold one `{{query}}` macro, and that one is blank (`Logseq/Queries`), and no `#+BEGIN_QUERY` block. Its `logseq/config.edn` defines two advanced queries that Logseq adds under today's journal (`🔨 NOW` and `📅 NEXT`). Logseq shows them only when the reader's date has a journal page, so they were not visible when measured. The query language matters more through flashcards: 37 `{{cards …}}` macros select their cards with the same simple-query language, using page references, `and`, `or` and `not`. Flashcards belong to [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123), but both features need one query evaluator.

So the gap is the whole feature, but little of the garden depends on it. The proposal has to choose how much of the language to support (gap 9 and gap 10), and what "today" means in a static build (gap 11).

Performance is the reason to do this at build time. Logseq shows nothing on `Logseq/Queries` until the whole graph has loaded: 19.8 s on the desktop profile and 185 s on the phone profile ([Parity performance](performance.md)). `lsg` can compute each result during the build and write it into the page's HTML, so the browser runs no query and loads no graph. Large results (a query for `[[tmux]]` returns 603 blocks) need the same treatment as large reference sections: the first part in the page and the rest in a file that loads when the reader asks for it.

Query evaluation is relationship logic, so it belongs in the SDK, where the CLI, the language server and the publisher share it; the publisher only renders the results ([Where the work belongs](#where-the-work-belongs)).

Measured on 2026-10-27 against the `gh-pages` build of garden commit `083106d` (`gh-pages` `8132c95`, served locally, light theme, 1280×800, browser date 2026-10-27, time zone America/New_York) and an `lsg` build of the whole garden from `lsg` `48fcc54` (`mise run parity:build`, run `20261027T002614Z`). Because the garden has no query that returns results, the result, table, collapse and error states were measured on a copy of the `gh-pages` build in which the text of seven blocks on `Logseq/Queries` and `Logseq/Flavored Markdown` was replaced with test queries in `static/js/db.gz`. The blocks' references, properties and the rest of the graph were unchanged, so the results come from the garden's real data. [Fixture pages](#fixture-pages) lists the test queries and their measured results.

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`, `QC` = `frontend/components/query.cljs`). Measured live values come first and win where they differ from the checkout or from `docs/logseq-reference/Prototype.md`.

The prototype board differs from the live build in these places:

- Its header is a 14px muted row with `2px 0 4px` padding; the live header is a 12px box with padding, a background and a ring.
- It shows a collapsed query's count in parentheses; Logseq does not run a collapsed query, so it has no count to show.
- It hides the tools of a collapsed advanced query; the live build shows them.
- Its tables have only `block` and `page` columns, with `—` for dates; the live build adds property columns and fills the dates.
- It has one error form, `Query failed:`; the live build has three.
- It omits the query builder row that the live build shows under every simple query.

### Where queries come from

- **`{{query …}}`** in a block, anywhere in its text ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1248-L1256)). The macro's arguments are joined with `, ` and read as the simple query language.
- **`#+BEGIN_QUERY` … `#+END_QUERY`**, read as an EDN map ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3278-L3284)). Its `:query` is either the simple language or Datalog (a vector with `:find`) ([`query_custom.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_custom.cljs#L71-L73)).
- **`:default-queries :journals`** in `config.edn`, rendered below today's journal on the Journals home and on today's journal page ([`journal.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/journal.cljs#L66), [`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L211-L226)). "Today" is the reader's local date. The Journals home only shows journals that exist, so a published site shows these queries only to a reader who visits on a date that has a journal page. Measured: on 2026-10-27 the home shows `2026-10-25 Sun` first and no default queries. A default query with no results is hidden ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L188)).
- Page references inside a query are not references: Logseq's parser skips query blocks when it collects a block's references ([`graph_parser/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/block.cljs#L343-L348)).

### Header

- The header is the `.custom-query > .th` row ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L631-L635)). Measured: 12px, opacity 0.9, margin `4px 2px`, padding `16px 24px`, background `rgb(248,249,250)`, radius 8px, ring `0 0 0 2px rgb(233,236,239)`, 52px tall with one line. The title is on the left; the count and buttons are on the right.
- **Simple query title:** a 12px `ti-search` icon and `Live query`, or `Live query for pages` when the query returns pages ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L190-L196)). The query text is not shown in the header.
- **Advanced query title:** the `:title` value in a `font-size: initial` div, so a string is 16px text and a vector is rendered as markup. Measured: `[:h3 "…"]` renders an `h3`. There is no icon, and there is no default title.
- **Count:** `N results`, or `1 result`, in a `.results-count` span, only when N is above 0 ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L200-L205)). For a grouped list, N counts blocks, not pages. Measured: `18 results`, `603 results`, `1 result`.
- **Buttons**, 20px icons at opacity 0.8, 4px apart ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L207-L233)):
  - `Switch to table view` or `Switch to list view` (`ti-table` or `ti-list`). It is left out when the results are pages, when the query has a `:view`, or when it sets `:table-view?`. A click sets `query-table:: true` or `false` on the block, in memory only in a published site. Measured: the toggle works and switches the view.
  - `Setting properties` (`ti-settings`), always shown. It opens a dialog headed `Properties settings for this query:` with a switch for each column ([`events.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/events.cljs#L290-L331)). Measured on the tmux query: `block`, `page`, `logseq-entity`, `see-also`, `alias`, `tags`, `date-created`, `created-by`, `published`, `isbn`, `via`.
  - A refresh button, only when the query took more than 50 ms. Its tooltip begins "This query takes Nms to finish, it's a bit slow so that auto refresh is disabled" ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L25-L42)). None of the measured queries showed it.
- **Query builder:** every simple query also gets a row under the header that shows the parsed query as clauses, for example `( AND [[yazi]] [[card]] )` or `between: -30d ~ today`, followed by a `+` (`Add clause`) button ([`query/builder.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query/builder.cljs#L421-L477)). Its edits rewrite the macro in memory. Measured: a blank `{{query }}` shows only the header and a `+`.
- None of these controls checks for a published site. Toggling or editing changes only the reader's in-memory copy.

### Collapsed queries

- An advanced query with `:collapsed? true` starts collapsed. So does a block with `collapsed:: true` ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L135-L158)).
- A collapsed query does not run, so it shows no count and no results ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L171-L172)). An advanced query keeps its title, table toggle and `Setting properties` buttons; a simple query keeps only its title. The block's bullet shows the closed style (`bullet-closed`) and its arrow points right.
- Measured: `Blocks that mention tmux` with `:collapsed? true` shows the title and two buttons. Opening it runs the query, which then shows `603 results`.
- Default journal queries are the exception: they run while collapsed and show their count ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L237-L247)).

### List view

- The query's own block is left out of its results ([`query/result.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query/result.cljs#L65-L66)). A result whose parent is also a result is dropped, unless `:remove-block-children? false` ([`result.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query/result.cljs#L67-L73)).
- Block results are grouped by page unless the query has a `:result-transform`, sets `:group-by-page? false`, or contains `(by-page false)` ([`result.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query/result.cljs#L51-L57)). Groups are sorted by journal date, newest first, then the other pages ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3495-L3519)). Measured: the tmux query starts `2026-09-19 Sat`, `2026-09-15 Tue`, `2026-09-11 Fri`.
- A group (`.custom-query-page-result`): margin `8px 0`, padding 8px, background `rgb(248,249,250)`, radius 4px, ring `0 0 0 2px rgb(233,236,239)`. It starts with a fold arrow and the page title link (16px, `rgb(16,107,163)`), then each result block. Above each block is a breadcrumb of its ancestors (`.breadcrumb`, margin `8px 0`, `ti-chevron-right` separators at opacity 0.5); below it are all its children, like an embed ([`reference.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.css#L22-L24)).
- Ungrouped results (`:group-by-page? false`) are a plain list of blocks with breadcrumbs. Results that are not blocks or pages, such as `[:find ?name …]`, show as a list of values ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L102-L112)).
- Groups mount as they scroll into view. Measured: the 603-block tmux query had 5 groups in the document at first and 49 after scrolling about 22,000px.
- A `{{query}}` inside a result block is not run again; it shows as the text `{{query …}}` ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L184-L187)).

### Table view

- Logseq uses the table when the block has `query-table:: true`, when the query sets `:table-view? true`, or when a simple query ends with the word `table`. Page results always use the table ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L160-L175)).
- **Columns** ([`query_table.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query_table.cljs#L93-L110)): for blocks, `block`, `page` and then every property found on the results, except built-in ones. For pages, `page`, the page properties, `created-at` and `updated-at`. `query-properties:: [:block :page]` chooses and orders the columns.
  - Measured on the tmux query: `block`, `page`, `logseq-entity`, `see-also`, `alias`, `tags`, `date-created`, `created-by`, `published`, `isbn`, `via`.
  - Measured on `(page-tags [[CLI/Tool]])`: `page`, `logseq-entity`, `created-by`, `alias`, `tags`, `see-also`, `via`, `date-created`, `readwise-link`, `filters`, `created-at`, `updated-at`.
- **Sorting** ([`query_table.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query_table.cljs#L44-L91)): by `query-sort-by` (default `updated-at`), descending unless `query-sort-desc:: false`, comparing with `localeCompare` and numeric order. Clicking a column title sorts by it and flips the direction, in memory.
- **Cells:**
  - `block` shows the block's first line.
  - `page` shows the page's original name as a link without brackets.
  - Property values show as links when they name pages.
  - `created-at` and `updated-at` use `yyyy-MM-dd HH:mm` in the reader's time zone ([`date.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/date.cljs#L217-L221)). In a published build they are the times the build parsed each file; measured `2026-10-25 16:35`, as on [All pages](all-pages.md).
  - Clicking a cell opens the block in the right sidebar.
- **Styles** (measured; the same table styles as All pages):
  - The table is `table-auto` inside an `overflow-x-auto` wrapper.
  - Header cells: 14px, weight 400, `rgb(104,112,118)`, padding `10px 8px`, bottom border `2px solid rgb(230,232,235)`, no wrapping.
  - Body cells: 16px, padding `6px 8px`. Rows alternate `rgb(251,252,253)` and `rgb(236,238,240)`.
- The table has no paging: the tmux query drew all 603 rows.

### Empty results and errors

- **No results:** `No matched result` in a `div.text-sm.mt-2.opacity-90` (14px, margin-top 8px, opacity 0.9) under the header, with no count ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L114-L119), [`dicts/en.edn`](https://github.com/logseq/og/blob/6e7afa8eb/src/resources/dicts/en.edn#L121)).
- **Blank query:** `{{query }}` and `(and)` show the header and builder and no text at all. Measured on `Logseq/Queries`.
- **A simple query that does not parse**, such as `{{query (and [[tmux]]}}`, replaces the whole box with Logseq's block error. The block error is a bordered box with an `h5` `Query Error:` (13.28px, weight 600, `rgb(235,144,145)`), a `report issue` link on the right, and the query text in a code box ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L249-L256)). Measured: the code box shows `(and [[tmux]]`.
- **An advanced query whose map does not read**, such as an unbalanced `:query`, shows the same box headed `Invalid query:` with the block text ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L3278-L3284)).
- **A query that reads but fails to run** keeps the header and buttons and shows a `.warning.my-1` line in the body: `Query failed:` followed by the message in a paragraph ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L63-L67)). Measured: `Query failed: Unknown predicate 'no-such-fn in [(no-such-fn ?m)]`, 16px, `rgb(245,217,10)`. A failing `:view` shows `Custom view failed:` instead. A failing `:result-transform` is logged, and Logseq shows the untransformed results ([`query_react.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_react.cljs#L50-L55)).

### The simple query language

[`query_dsl.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_dsl.cljs#L377-L446) translates each form to Datalog over the graph database; [Queries](https://github.com/logseq/docs/blob/628cdde/pages/Queries.md) in `logseq/docs` (`628cdde`) describes it for users. A query returns blocks, unless every form in it is about pages. Then it returns pages, and the header says `Live query for pages`.

| Form | Matches | Used by the garden |
| --- | --- | ---: |
| `[[page]]`, `#tag` | blocks whose own or ancestors' references include the page, through aliases ([`rules.cljc`](https://github.com/logseq/og/blob/6e7afa8eb/deps/db/src/logseq/db/rules.cljc#L140-L143)) | 33 `{{cards}}` |
| `and`, `or`, `not` | combinations; `not` excludes | 4, 4 and 5 `{{cards}}` |
| `"text"` | blocks whose source text contains the string, case-sensitive ([`query_dsl.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_dsl.cljs#L373-L375)) | 0 |
| a bare word | full-text search in the desktop app; no results on the web or in a published site ([`result.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query/result.cljs#L28-L37)) | 1 (`{{cards query}}`) |
| `(task TODO DOING)`, `(todo …)` | task markers | 0 |
| `(priority A)` | priorities | 0 |
| `(between -7d today)` | blocks on journals in a date range; `today`, `yesterday`, `tomorrow`, `[[journal]]`, ±N `d`/`w`/`m`/`y` ([`query_dsl.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_dsl.cljs#L52-L79)) | 0 |
| `(between created-at …)` | the `created_at` and `last_modified_at` block properties | 0 |
| `(property k v)`, `(property k)` | block properties | 0 |
| `(page-property k v)`, `(page-tags …)`, `(all-page-tags)`, `(namespace ns)` | pages | 0 |
| `(page name)` | blocks on a page | 0 |
| `(sort-by k asc)`, `(sample n)` | ordering and a random sample of the results | 0 |

Simple queries also accept `<% today %>`-style templates before parsing ([`query_dsl.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_dsl.cljs#L566-L569)).

### Advanced queries

- Keys: `:title`, `:query`, `:inputs`, `:rules`, `:result-transform`, `:view`, `:collapsed?`, `:group-by-page?`, `:remove-block-children?`, `:breadcrumb-show?`, `:table-view?` ([Advanced Queries](https://github.com/logseq/docs/blob/628cdde/pages/Advanced%20Queries.md)).
- A Datalog `:query` runs against the whole Datascript database. Logseq rewrites `pull ?b [*]` and adds any simple-query rules the `:where` uses ([`query_custom.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_custom.cljs#L12-L58)).
- `:inputs` resolve as follows ([`graph_parser/util/db.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/util/db.cljs#L77-L168)):
  - `:current-page` is the page in the route, else the home page, else today's journal ([`query_react.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_react.cljs#L19-L30)).
  - `:query-page` is the page that holds the query.
  - `:current-block` and `:parent-block` are the query's block and its parent.
  - `:today`, `:yesterday`, `:tomorrow`, `:-7d`, `:+1w` and similar are journal dates relative to the reader's clock. Their `-start` and `-end` forms are timestamps, and `:right-now-ms` is the current time.
  - A `"[[page]]"` string is that page's name.
- `:result-transform` and `:view` are ClojureScript functions, or keywords naming functions in `:query/result-transforms` and `:query/views` in `config.edn`, evaluated in the browser with SCI ([QC](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L163-L164), [`query_react.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_react.cljs#L46-L57)). The garden defines `:pprint` and `:sort-by-priority`. Measured: `:view :pprint` on a `[:find ?name …]` query shows `("Logseq/Flashcard/Review" "Logseq/Flashcard/Bug")` in a `pre`, with `2 results` and no table toggle.
- The garden's two default queries use Datalog over `:block/marker`, `:block/page` and `:block/journal-day`, with the inputs `[:14d :today]` and `[:today :7d-after]`, a `:result-transform` that sorts by priority, and `:group-by-page? false`.

### Performance in Logseq

- The published app loads the whole database into the browser before it renders a page ([`publishing.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/publishing.cljs#L50-L53)). On `Logseq/Queries` that is 22.6 MB of data and 8.1 MB of script before the title: 19,760 ms to the title on the desktop profile and 185,344 ms on the phone profile, with a peak JS heap near 1,970 MB ([Parity performance](performance.md)).
- After that, each query is fast. None of the measured queries took more than 50 ms, the point at which Logseq shows its refresh button. That includes the 603-block tmux query and a case-sensitive text search over every block.
- Logseq caches each result and re-runs every query after each change to the database, unless the query's last run took more than 80 ms ([`react.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/react.cljs#L298-L361)). In a published site, nothing changes the database except the reader's in-memory edits.
- Result lists mount as they scroll into view, but tables draw every row.

## What lsg does today

From the worktree's `crates/logseq-gardener-sdk/src/site/render.rs`, checked against the whole-garden build and a three-page test garden built with the same `lsg`:

- **`{{query …}}`** goes to `macro_fallback` (lines 1158–1182), which prints the macro source as `<code>{{query …}}</code>` and records the diagnostic `Unsupported macro (including embeds and queries); preserved as literal text`. `Logseq/Queries` shows `<code>{{query}}</code>`. The unit test `unknown_macros_show_warnings_and_owned_macros_keep_fallbacks` (line 2068) and `tests/site.rs` (line 208) expect this fallback.
- **`#+BEGIN_QUERY`** is a `Block::Custom` named `query` (lines 566–574): `<aside>query<p>{:title "…"<br> :query …}</p></aside>`, with the diagnostic `Unsupported custom block query; rendered children`. Its `[[page]]` text renders as links.
- **References:** unlike Logseq, `lsg` counts a `[[page]]` inside `#+BEGIN_QUERY` as a reference. In the test garden, the query block appears under the page's `1 Linked Reference`. A `[[page]]` inside `{{query …}}` is not counted, which matches Logseq.
- **Properties:** `query-table`, `query-properties`, `query-sort-by` and `query-sort-desc` are already hidden (`site/properties.rs`, lines 41–44), as in Logseq.
- **Default journal queries:** nothing in the SDK reads `:default-queries`.
- **`{{cards …}}`** has the same literal-code fallback; [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123) owns it.
- **SDK data a query evaluator can use:** `ReferenceIndex` in `crates/logseq-gardener-sdk/src/references.rs` already gives each block's page, ancestors and path references (`path_refs`, the page plus the references of the block and its ancestors), which is the set `[[page]]` matches. `outline.rs` parses task markers and priorities; `journals.rs` gives journal dates; the garden settings read `config.edn`.
- **Performance:** `Logseq/Queries` is 11.5 KB of data and 14.6 KB of script before the title; its title is visible at 21.8 ms on desktop and 243 ms on phone ([Parity performance](performance.md)).

## Gaps

Each gap names a check the Apply step can run against the fixture pages or the parity fixture garden.

- 1. **Simple query header.** A box with the search icon and `Live query` (or `Live query for pages`), and on the right `N results` (only when N > 0) and the buttons the proposal keeps (gap 12), with the measured header styles. Check: `{{query (and [[yazi]] [[card]])}}` on the encode garden shows `Live query` and `18 results`; computed styles match [Header](#header).
- 2. **Advanced query header.** The `:title` string as 16px text, or a vector title rendered as markup, with no icon. Check: `:title [:h3 "…"]` renders an `h3`; a string title renders as text in the header.
- 3. **List view grouped by page.** Results grouped into `.custom-query-page-result` boxes, journals newest first. Each block has its breadcrumb and children. The query's own block is left out, and so is any result whose parent is also a result. The groups use the measured styles. Check: the tmux test query lists `2026-09-19 Sat`, `2026-09-15 Tue`, `2026-09-11 Fri` first; `(and [[yazi]] [[card]])` gives 18 blocks in 13 groups.
- 4. **Ungrouped and value results.** `:group-by-page? false` and `:result-transform` give a flat list; non-block results give a list of values. Check: a `[:find ?name …]` query with `:view :pprint` shows the two names; the default-query form with `:group-by-page? false` lists blocks without page boxes.
- 5. **Table view.** The table is used for `query-table:: true`, `:table-view? true`, a simple query that ends in `table`, and every page result. The table has the measured columns, `query-properties` order, `query-sort-by` and `query-sort-desc` (default `updated-at` descending), the shared table styles, and page links without brackets. Check: `(page-tags [[CLI/Tool]])` gives 60 rows with the twelve columns listed under [Table view](#table-view); `query-properties:: [:block :page]` gives exactly those two columns.
- 6. **Collapsed queries.** `:collapsed? true` and `collapsed:: true` start closed, with no count and no results. Opening the query shows its results and count. The publisher can hide results that are already in the page, so the browser runs nothing on open. Check: `Blocks that mention tmux` starts closed without a count and shows `603 results` when opened.
- 7. **Empty and blank queries.** `No matched result` (14px, margin-top 8px, opacity 0.9) for no results; header only for a blank query. Check: a fixture-garden query that matches nothing shows the text and no count; `Logseq/Queries` shows the `Live query` header and no text.
- 8. **Errors.** Three forms, each with the measured strings and styles:
  - a simple query that does not parse shows the `Query Error:` box with the query text;
  - an advanced map that does not read shows the `Invalid query:` box with the block text;
  - a query that fails to run shows the `Query failed:` warning under the header.
  
  `lsg` reports each one as a build diagnostic with its source location as well. Check: the three broken test queries render these three forms, and `lsg publish` lists three diagnostics.
- 9. **Simple query language.** The SDK evaluates the forms in [The simple query language](#the-simple-query-language) with Logseq's meaning: path references through aliases, case-sensitive text, page versus block results, `between` relative dates. The proposal decides which forms to support first. The 37 `{{cards}}` need page references, `and`, `or` and `not`; nothing else in the garden uses any form. A form `lsg` does not support renders as a `Query failed:` warning naming the form, not as wrong results. Check: on the encode garden, each test query in [Fixture pages](#fixture-pages) returns Logseq's count.
- 10. **Advanced queries.** The SDK evaluates an advanced query whose `:query` is in the simple language. Datalog `:find` queries and SCI functions (`:view`, `:result-transform`) cannot run as written without a Datalog engine and a ClojureScript interpreter. The proposal decides between a Datalog subset over the SDK's index that covers the garden's two default queries and the documented `:inputs`, named Rust equivalents for the garden's `:pprint` and `:sort-by-priority`, and a `Query failed:` warning that says the build does not support the query. Check: whatever is decided, the two default queries either render their results or show the decided message, and no query is silently dropped.
- 11. **"Today" in a static build.** Relative dates (`between -7d today`, `:today`, `:-14d`, `<% today %>`) and the default journal queries depend on the reader's clock in Logseq and on the build's clock in `lsg`. Results go stale between builds. The proposal decides whether `lsg` fixes them at the build date (as the journals feed already does, using `SOURCE_DATE_EPOCH` when set), and whether default queries render under the build date's journal, under the newest journal, or not at all. Check: two builds of the same commit with the same `SOURCE_DATE_EPOCH` give the same results; `(between -30d today)` on 2026-10-27 gives 76 results, as Logseq did.
- 12. **Controls in a read-only site.** In Logseq, the table toggle, `Setting properties`, column sorting and the query builder change only the reader's in-memory copy. The proposal decides which of them `lsg` draws and whether they work. The table toggle and column sorting can work in the browser over results already in the page; the builder and the properties dialog cannot keep their edits. Check: the decided controls are present on the fixture, and the ones that work do so without loading any other file.
- 13. **No references from query text.** `[[page]]` inside `#+BEGIN_QUERY` stops counting as a reference, as in Logseq. Check: in the parity fixture garden, a page named only inside an advanced query has no Linked Reference from it.
- 14. **Performance.** Results are computed at build time and written into the page, so a page with a query runs no query script. A query's results count toward the page's first paint like any other blocks, within the `[desktop]` and `[phone]` budgets (100 KB data before the title). Large results put the first part in the page and the rest in a fragment that loads on scroll or open. That is the pattern `[references]` already uses: 100 KB gzip per fragment, opening within 300 ms on desktop and 1,500 ms on phone. Collapsed queries keep their results out of the page until opened. A whole-garden build stays within a `build_slowdown_pct` against the `--baseline-lsg` binary. Check: `parity:budgets` adds a page with the 603-block tmux query that stays within the first-paint budget and does not grow when 2,000 pages are added; `parity:perf` records it.

## Where the work belongs

- **SDK (`garden-core`):** one query operation shared by the CLI, the language server and the publisher, per [Issue #106](https://github.com/codekiln/logseq-gardener/issues/106): "query results and card decks are SDK operations".
  - It parses the simple language and the advanced-query map, with errors that carry source locations.
  - It evaluates queries over `ReferenceIndex` (path references, aliases, pages, ancestors), task markers, priorities, properties and journal dates.
  - It returns ordered results: blocks with their page and ancestors, pages with their properties, or values. It also applies the own-block exclusion and the top-level-only rule.
  - It resolves "today" from one build-time clock.
  - The same evaluator selects card decks for [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123).
  - The parser also gives `ReferenceIndex` the query spans to skip (gap 13).
  - The CLI could then answer `lsg query '(and [[tmux]] (task TODO))'` with the same results the site shows.
- **Publisher:** the header, list, group, table, empty and error markup and styles; the static first part and fragment files for large results; the decided controls and their small script; the default journal queries' placement; and the budget rows.

## Fixture pages

The garden has too few queries to exercise the feature, so the proposal needs both real pages and test queries.

Real garden pages, added to [`scripts/parity/fixtures.txt`](../../scripts/parity/fixtures.txt) where they were missing (`Logseq/Queries` and `Home` were already there):

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `Logseq/Queries` | The garden's only `{{query}}` macro, which is blank | `Live query` header, table and settings buttons, builder `+`, no results, no text | `<code>{{query}}</code>` |
| `Home` | Today's journal and the two `:default-queries` | none on 2026-10-27: no journal for that date | no default queries |
| `Logseq/Flavored Markdown` | `` `{{query}}` `` in inline code must stay code | inline code | inline code |
| `LangChain/Academy/LangSmith/M5 Production Observability/L2 Online Evaluation` | `{{query}}` inside a fenced code block must stay literal | code block | code block |
| `Logseq/Flashcard/Review/Favorite` | `{{cards}}` with `and`, `or`, `not`, and a hidden `query-properties:: [:block]`; shares the evaluator with #123 | card decks | `<code>{{cards …}}</code>`; property hidden |

Test queries measured on the patched copy of the `gh-pages` build. They should become pages in the parity fixture garden (`tests/fixtures/parity-garden/`), in the shape of `Prototype/Fixture/Queries`, with results checked against the encode garden where they depend on its data:

| Query | Logseq on garden `083106d` |
| --- | --- |
| `{{query (and [[yazi]] [[card]])}}` | `18 results`, 13 groups |
| `{{query (page-tags [[CLI/Tool]])}}` | `Live query for pages`, `60 results`, table, no toggle |
| `{{query (and [[yazi]] [[Rust]])}}` | `5 results` (the prototype expects none) |
| `{{query (and (task TODO DOING) [[tmux]])}}` | `2 results` |
| `{{query (between -30d today)}}` on 2026-10-27 | `76 results` |
| `{{query "lightweight database"}}` | `1 result` |
| `{{query (page-property logseq-entity [[Logseq/Entity/Proxy/Page]])}}` | `Live query for pages`, `6 results` |
| `#+BEGIN_QUERY` `:query [[tmux]]`, `:collapsed? true` | closed, no count; `603 results` when opened |
| `#+BEGIN_QUERY` Datalog with `:inputs [:current-page]`, on `Logseq/Flavored Markdown` | `5 results` |
| `#+BEGIN_QUERY` `[:find ?name …]` with `:view :pprint` | `2 results`, a `pre` with two names |
| `{{query (and [[tmux]]}}` | `Query Error:` box |
| `#+BEGIN_QUERY` with unbalanced `:query` | `Invalid query:` box |
| `#+BEGIN_QUERY` with an unknown Datalog predicate | `Query failed: Unknown predicate 'no-such-fn …` |

## Out of scope

- Flashcard decks and the review dialog, which belong to [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123); this change provides the evaluator they share.
- Opening a result in the right sidebar by clicking a table cell, which belongs to [Issue #119](https://github.com/codekiln/logseq-gardener/issues/119).
- Full-text search for bare words, which a published Logseq site does not run either.
- Scheduled and deadline lists under today's journal, which are a separate journal feature.
- Editing a query through the query builder in a way that persists, which a published site cannot do.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
