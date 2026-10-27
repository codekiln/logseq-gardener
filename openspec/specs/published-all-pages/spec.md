## Purpose

Publish Logseq's All pages table: static first view, static paging pages, a lazily loaded page list, and the script behaviour and budgets that keep it fast.

## Requirements

### Requirement: Show the All pages screen

The complete page index SHALL be titled `All pages` and SHALL show the `All pages` heading, `Total N pages` where N counts the rows of the current view, a toolbar with a `Search page names` input, a journals toggle and the paging control, a table with the columns `Page name`, `Backlinks`, `Created At` and `Updated At`, and the paging control again below the table. Each row SHALL show the page's icon when it has one and link the page name to its route without brackets. Dates SHALL show as `yyyy-MM-dd HH:mm`. When the page list has no dates, the date columns SHALL be absent. Row checkboxes, `Delete`, the whiteboard toggle and the `⋯` menu SHALL be absent.

#### Scenario: Default view

- **WHEN** a reader opens All pages on the encode garden
- **THEN** the screen shows `Total N pages` with N the number of non-journal entries, `1/M` with M equal to N over 40 rounded up, and 40 rows ordered by `Updated At`, newest first

#### Scenario: No git history

- **WHEN** the garden is not a git repository
- **THEN** the table has no date columns and rows are ordered by page name

#### Scenario: Generated namespace rows

- **WHEN** the list includes the generated namespace page `tmux/Command`
- **THEN** its row keeps the generated link style and accessible Namespace name and shows no Namespace badge

### Requirement: Page through the default view without scripts

The index page SHALL contain the first 40 rows of the default view. Static pages `pages-2.html`, `pages-3.html` and onward SHALL contain the following rows of that view, 40 per page, inside the page shell. The paging control SHALL read `n/M` with `Prev` and `«` from the second page on and `Next ▸` and `»` before the last page, as links between these pages.

#### Scenario: Next page

- **WHEN** a reader without scripts follows `Next ▸` on the first page
- **THEN** `pages-2.html` shows rows 41 to 80 of the default view and reads `2/M`

### Requirement: Sort, search and show journals with the list data file

The publisher SHALL write every entry, journals included, to one list data file. The script SHALL fetch it only on the reader's first sort, search or journals toggle. Clicking a column title SHALL sort by that column, flip one direction shared by all columns and show a caret on the sorted column. Search SHALL run on Enter, score names with Logseq's fuzzy `score`, keep positive scores, order them by score and show at most 20 rows; Escape SHALL clear it. The journals toggle SHALL add journals to the view. Paging in a script-made view SHALL show 40 rows per page and scroll to the top. Dates SHALL show in the reader's time zone when the script runs.

#### Scenario: Sort by backlinks

- **WHEN** a reader clicks `Page name` and then `Backlinks` from the default view
- **THEN** the rows start `Microfreak`, `Logseq`, `Logseq/Entity`, and the `Backlinks` header shows the caret

#### Scenario: Search

- **WHEN** a reader types `mise` and presses Enter
- **THEN** the screen shows `Total 20 pages` and `mise` is the first row

#### Scenario: Journals toggle

- **WHEN** a reader turns on the journals toggle
- **THEN** the total grows by the number of published journals

#### Scenario: No data before interaction

- **WHEN** a reader opens All pages and only follows `Next ▸`
- **THEN** the list data file is never requested

### Requirement: Fit the table to a phone

Below a viewport width of 640px, the table SHALL show only the icon and `Page name` columns and the page SHALL not scroll sideways.

#### Scenario: Phone width

- **WHEN** a reader opens All pages at 390×844
- **THEN** the table has one data column and no horizontal scroll

### Requirement: Keep All pages within budgets

The comparison harness SHALL check, on the whole encode garden: the All pages first paint within the `[desktop]` and `[phone]` data and script budgets; each static paging page at most 10 KB gzip; first-page growth of at most 1 KB when the garden gains 2,000 pages; the All pages script at most 3 KB minified gzip; the list data file at most 300 KB gzip; a peak phone heap of at most 20 MB after loading the list and sorting by Backlinks; search on Enter at p95 within 50 ms desktop and 150 ms phone; and a whole-garden build time at most 10% above the build without this change. `parity:budgets` SHALL check the size budgets in CI.

#### Scenario: Grown garden

- **WHEN** the budget fixture garden gains 2,000 pages
- **THEN** the All pages first page grows by at most 1 KB and stays within the first-paint budget
