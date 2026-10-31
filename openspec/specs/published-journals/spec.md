## Purpose

Define how published sites show journal pages and the journals feed.

## Requirements

### Requirement: Publish journal pages

When the caller requests journals, the SDK SHALL publish every journal whose identity derives, that is public under the public-page rule of the local-static-publishing capability, and that contains no parsed `public:: false` property, as a page at its deterministic route, titled with its journal title. A journal that is not public SHALL be withheld and counted as withheld. A journal page SHALL show the journal's blocks, page properties and Linked References as other pages do, SHALL accept its aliases as link targets, and SHALL omit the namespace hierarchy section. Journal titles SHALL share the selected title lookup with pages, so a page whose title equals a journal title SHALL fail with the existing ambiguity error before output is created. A journal whose identity does not derive SHALL be skipped with a diagnostic and counted as a skipped journal. Journal pages SHALL appear in the complete page index and the search index.

#### Scenario: Ordinary journal page
- **WHEN** the encode garden is published with journals
- **THEN** `2026-10-23 Fri` shows its Filed and Updated blocks and 1 Linked Reference from the LangSmith question page

#### Scenario: Journal with properties and an alias
- **WHEN** the encode garden is published with journals
- **THEN** `2019-08-07 Wed` shows its page properties and a link to `1987` opens it

#### Scenario: Private journal
- **WHEN** a journal contains a nested `public:: false` property
- **THEN** the journal, its text and its referenced assets are absent from the output and the report counts it as withheld

#### Scenario: Unmarked journal in a garden that publishes marked pages only
- **WHEN** journals are requested, the garden's all-pages-public setting is off, and one journal is marked `public:: true` while another has no `public::` property
- **THEN** only the marked journal is published and the report counts the other as withheld

### Requirement: Show the journals feed

When journals are published, at least one published journal is dated on or before the build date, and `:feature/enable-journals?` is not `false`, the SDK SHALL write a journals feed. The build date SHALL be the UTC date of `SOURCE_DATE_EPOCH` when set, otherwise of the system clock. The feed SHALL list those journals newest first: three in the first batch file and seven in each later batch file `journals-<n>.html`. The first batch SHALL be `index.html`, or `journals.html` when a homepage is selected. Each journal SHALL appear as an article with an `h1` linking to its page, its block tree with authored collapse state, and, when it has Linked References, a collapsed section summarising their count with a link to the journal page's Linked References. Each batch with a later batch SHALL end with an Earlier link to it. Journals dated after the build date SHALL keep their pages and stay out of the feed.

#### Scenario: First batch
- **WHEN** the encode garden at commit `09cf7fb` is published with journals and a build date of 2026-10-25
- **THEN** `index.html` lists `2026-10-24 Sat`, `2026-10-23 Fri` and `2026-10-22 Thu` in that order, and its Earlier link opens `journals-2.html`

#### Scenario: Reading without JavaScript
- **WHEN** a reader with JavaScript off follows Earlier from `index.html`
- **THEN** `journals-2.html` lists `2026-10-21 Wed` through `2026-10-15 Thu`

#### Scenario: Future journal
- **WHEN** a journal is dated after the build date
- **THEN** its page is published and no feed batch lists it

#### Scenario: Journals disabled
- **WHEN** `:feature/enable-journals?` is `false`
- **THEN** journal pages are published, no feed is written, and `index.html` is the complete page index

### Requirement: Load later journals and references on demand

Feed pages SHALL load one shared `lsg-journals.js` file from the site root after the page's `load` event, outside the script loaded before the title. The script SHALL fetch the next batch file when the Earlier link comes within 500px of the viewport, append the batch's journal articles, and point Earlier at the following batch or remove it after the last. On the first opening of a journal's Linked References section, the script SHALL fetch the journal page and show its Linked References content with working filters. A failed fetch SHALL leave the Earlier link and the reference link usable.

#### Scenario: Scrolling the feed
- **WHEN** a reader scrolls `index.html` to the bottom once
- **THEN** the page shows ten journals, `2026-10-24 Sat` through `2026-10-15 Thu`, and Earlier points at `journals-3.html`

#### Scenario: Opening references
- **WHEN** a reader opens Linked References under `2026-10-23 Fri` in the feed
- **THEN** the section shows the reference from the LangSmith question page

#### Scenario: Offline batch
- **WHEN** fetching the next batch fails
- **THEN** the Earlier link stays in place and opens the batch file when followed

### Requirement: Keep the feed's cost independent of the journal count

`lsg-journals.js` SHALL be at most 4 KB uncompressed. The first batch SHALL meet the existing data, script and heap budgets before the title is visible. Adding older journals SHALL leave the first batch's size unchanged. Loading ten batches on the phone profile SHALL keep peak JavaScript heap at or below 20 MB. The `lsg-journals.js` size check and the older-journals size check SHALL run in CI through the parity budgets check, and `parity:perf --record` SHALL record the feed rows and the whole-garden build time with journals.

#### Scenario: Older journals added
- **WHEN** the parity fixture garden is built again with 1,000 older journals added
- **THEN** `index.html` and `lsg-journals.js` keep the same size and stay within their budgets
