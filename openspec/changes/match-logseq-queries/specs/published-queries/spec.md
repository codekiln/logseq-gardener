## ADDED Requirements

### Requirement: Show a query box with Logseq's header

Each `{{query …}}` and `#+BEGIN_QUERY` block SHALL render `div.custom-query` whose `.th` header uses Logseq's measured styles: 12px text, opacity 0.9, margin `4px 2px`, padding `16px 24px`, radius 8px, and the theme's background and 2px ring. A simple query's title SHALL be a search icon and `Live query`, or `Live query for pages` when it returns pages. An advanced query's `:title` SHALL render as 16px text when it is a string and as the named element with escaped text when it is a hiccup vector, with no icon. The header SHALL show `N results`, or `1 result`, only when the count is above zero, counting blocks for grouped results. A simple query SHALL show its parsed clauses in a read-only `.query-builder` row under the header, without an add button. The header SHALL carry no `Setting properties` button and no refresh button.

#### Scenario: Simple query header
- **WHEN** a fixture page built over the encode garden at commit `083106d` contains `{{query (and [[yazi]] [[card]])}}`
- **THEN** its header reads `Live query` and `18 results`, and the clause row reads `( AND [[yazi]] [[card]] )`

#### Scenario: Hiccup title
- **WHEN** a `#+BEGIN_QUERY` map sets `:title [:h3 "Blocks that mention tmux"]`
- **THEN** the header holds an `h3` with that text and no icon

#### Scenario: Blank query
- **WHEN** a reader opens `Logseq/Queries`, whose query is `{{query }}`
- **THEN** the page shows the `Live query` header, no count, no results and no literal `{{query`

### Requirement: List block results grouped by page

Block results SHALL be grouped by page unless the map sets `:group-by-page? false`, the query contains `(by-page false)`, or the map has a `:result-transform`. Each group SHALL be `.custom-query-page-result` with Logseq's measured margin, padding, radius and ring, a fold control and the page title link, then each result's breadcrumb and the block with its children, rendered as embedded blocks are. A `{{query}}` inside a result SHALL render as its literal text. Ungrouped results SHALL be a flat list of blocks with breadcrumbs. A query with no results SHALL show `No matched result` at 14px, margin-top 8px and opacity 0.9.

#### Scenario: Grouped list
- **WHEN** a reader opens the page with `{{query (and [[yazi]] [[card]])}}`
- **THEN** it lists 18 blocks in 13 page groups, each with its breadcrumb

#### Scenario: No results
- **WHEN** a fixture-garden query matches nothing
- **THEN** the box shows `No matched result` and no count

### Requirement: Show tables with Logseq's columns

A query SHALL render a table when its block has `query-table:: true`, its map sets `:table-view? true`, its simple text ends with the word `table`, or it returns pages. Block tables SHALL have the columns `block`, `page`, then each non-built-in property found on the results; page tables `page`, the page properties, `created-at` and `updated-at`. `query-properties` SHALL choose and order the columns. Rows SHALL be sorted by `query-sort-by` (default `updated-at`), descending unless `query-sort-desc:: false`. Page cells SHALL link to the page without brackets; property values that name pages SHALL be links; dates SHALL read `yyyy-MM-dd HH:mm` in UTC. The table SHALL use the All pages table styles. Clicking a column title SHALL sort by that column and a second click SHALL reverse it, without loading another file. A view toggle SHALL be drawn when Logseq draws it (block results without `:table-view?`) and SHALL load the other view from a fragment on click.

#### Scenario: Page table
- **WHEN** a reader opens the fixture page with `{{query (page-tags [[CLI/Tool]])}}`
- **THEN** the header reads `Live query for pages` and `60 results`, the table has 60 rows with the twelve columns Logseq showed, and no view toggle is drawn

#### Scenario: Chosen columns
- **WHEN** a query block sets `query-properties:: [:block :page]`
- **THEN** its table has exactly the `block` and `page` columns

#### Scenario: Toggle view
- **WHEN** a reader clicks `Switch to table view` on the yazi query
- **THEN** the results show as a table with 18 rows, fetched once from a fragment

### Requirement: Keep large and collapsed results out of first paint

Each query's in-page part SHALL hold whole groups, or table rows, up to 30 KB gzip, and the remaining results SHALL be written to fragments of at most 100 KB gzip that load as the reader scrolls near the end. A query whose map sets `:collapsed? true`, or whose block has `collapsed:: true`, SHALL render closed with its title and buttons and no count, and SHALL load its results and show its count when opened. The browser SHALL evaluate no query.

#### Scenario: Collapsed query
- **WHEN** a reader opens the fixture page with the collapsed `Blocks that mention tmux` query
- **THEN** the box shows its title and no count, and opening it shows `603 results` and the first groups

#### Scenario: Large result
- **WHEN** a reader opens the fixture page with the open `[[tmux]]` query
- **THEN** the page's data before the title stays within 100 KB and later groups load while scrolling

### Requirement: Show Logseq's three query errors

A simple query that does not parse SHALL replace the box with a bordered block error headed `Query Error:` above the query text in a code box. An advanced map that does not read SHALL show the same box headed `Invalid query:` above the block text. A query that parses but cannot run, including an unsupported form, a Datalog `:query`, `:inputs` or `:rules`, SHALL keep the header and show `.warning` `Query failed:` followed by the message. A map with a `:view` SHALL show `Custom view failed:` in the same way. A map with a `:result-transform` SHALL show its untransformed results. Each case SHALL also be a build diagnostic with the page and line.

#### Scenario: Parse error
- **WHEN** a page contains `{{query (and [[tmux]]}}`
- **THEN** it shows `Query Error:` with `(and [[tmux]]` in a code box, and the build lists one diagnostic

#### Scenario: Datalog query
- **WHEN** a page contains a `#+BEGIN_QUERY` whose `:query` is Datalog
- **THEN** the box shows its header and a `Query failed:` line saying the build evaluates simple queries only

### Requirement: Show default journal queries under the build date's journal

For each `:default-queries :journals` entry whose `:query` is in the simple language, the publisher SHALL render the query box, run while collapsed with its count, under the build date's journal on the journals feed and on that journal's page, when that journal is published and the query has results. Entries with a Datalog `:query` SHALL be skipped with one build diagnostic each.

#### Scenario: Encode garden default queries
- **WHEN** the encode garden, whose two default queries are Datalog, is published
- **THEN** no default query box appears and the build lists two diagnostics

### Requirement: Keep queries within budgets

`queries.js` and `queries.css` SHALL be linked only from pages with a query, and other pages SHALL be unchanged. `queries.js` SHALL be at most 2 KB and `queries.css` at most 1 KB gzip. Opening the collapsed tmux query SHALL show its first group within 300 ms on desktop and 1,500 ms on the phone profile, and the phone JS heap after all its fragments load SHALL stay within 20 MB. The fixture page with the tmux query SHALL stay within the `[desktop]` and `[phone]` first-paint budgets and SHALL not grow when the fixture garden gains 2,000 pages. A whole-garden build SHALL be at most 10% slower than the `--baseline-lsg` binary. CI SHALL check the size budgets through `parity:budgets`.

#### Scenario: Budgets in CI
- **WHEN** `parity:budgets` runs on the fixture garden
- **THEN** it checks the `[queries]` sizes and fails when one is exceeded

#### Scenario: Page without a query
- **WHEN** the encode garden is published before and after this change
- **THEN** `tmux` links to neither `queries.js` nor `queries.css`
