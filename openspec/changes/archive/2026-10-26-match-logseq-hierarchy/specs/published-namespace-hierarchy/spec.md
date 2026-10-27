## MODIFIED Requirements

### Requirement: Navigate ancestors and children

Every published source and generated namespace page that has a hierarchy in the namespace index SHALL show a Hierarchy section in the position the published reference sections set; journal pages and pages without a hierarchy SHALL have no section. The rows SHALL be the namespace index's rows for the page with rows for prefixes without a selected stored page left out, so a generated namespace page lists its stored descendants and a leaf lists its parent's path. The current page SHALL have no row of its own. The publisher SHALL remove top namespace breadcrumb navigation. The section SHALL hold a flat list with one row per path. Each row SHALL show its full path with slash separators, each segment labelled with the row's own spelling and linking independently to its cumulative prefix when that prefix has a destination. When the graph's `:ui/show-brackets?` is on, each segment SHALL show the page-reference brackets that inline page references show. Unavailable private, withheld, or alias-conflicting prefix destinations SHALL remain plain segment text without recreating routes. The section header SHALL be the foldable row with the chevron and the `Hierarchy` heading that the reference sections use, at 16px, weight 700 and 30% opacity, expanded by default. The list SHALL have a `12px 24px` margin, no padding and circle markers, and each row SHALL have `8px 0` margins. Labels and attributes SHALL be escaped. Semantic disclosure controls and links SHALL support keyboard operation, visible focus, and wrapping of long paths on narrow screens.

#### Scenario: Source page with ancestors and children
- **WHEN** A/B/C, A/B/C/D, A/B/C/E, and A/B/C/E/F are selected stored source pages
- **THEN** A/B/C displays the three rows A/B/C/D, A/B/C/E and A/B/C/E/F at the bottom, with separate links for every available cumulative prefix segment and no top breadcrumb

#### Scenario: Missing ancestor destination creates a gap
- **WHEN** Root and Root/Private/PublicChild are visible but Root/Private is withheld
- **THEN** Root's bottom list includes Root/Private/PublicChild, Private remains plain text in its row, and no navigation points to a standalone Root/Private destination

#### Scenario: Namespaced leaf shows its parent
- **WHEN** `tmux/Config/local` has no descendants
- **THEN** its Hierarchy holds the single row `tmux/Config`, with `tmux` and `Config` linked

#### Scenario: Plain page
- **WHEN** a reader opens `Abbreviation`, which has no `/`, no descendants and no namespaced alias
- **THEN** the page has no Hierarchy section

#### Scenario: Page with a namespaced alias
- **WHEN** a reader opens `AGI`, whose alias is `AI/General/Intelligence`
- **THEN** its Hierarchy holds the single row `AI/General`

#### Scenario: Similar sibling namespace
- **WHEN** A/B/C and A/B/Cousin are published
- **THEN** Cousin is absent from C's rows

#### Scenario: Expanded disclosure
- **WHEN** a page with multiple descendant levels is opened
- **THEN** the in-page rows are initially visible and the Hierarchy control can collapse and expand the whole list by keyboard

#### Scenario: Long mobile path
- **WHEN** a deep namespace with long segments is viewed at a narrow mobile width
- **THEN** complete path rows wrap within the available width and every available segment link remains keyboard reachable with visible focus

#### Scenario: Generated intermediate rows
- **WHEN** a stored page has a stored descendant below several generated namespace levels
- **THEN** only the stored descendant appears as a row, with every available generated prefix still linked in its path

#### Scenario: Generated current page
- **WHEN** the current page is a generated namespace with deeper stored descendants
- **THEN** all stored descendant rows appear, while the current page and generated-only descendants have no row

#### Scenario: Look on tmux
- **WHEN** a reader opens `tmux` at 1280×800 in the light theme
- **THEN** the heading, list, row and separator computed styles match the `gh-pages` build's within 1px, and each segment shows brackets

### Requirement: Space hierarchy path separators

The default stylesheet SHALL give hierarchy slash separators an 8px horizontal margin on either side and 30% opacity. Separators SHALL remain unlinked text, and paths SHALL retain natural wrapping.

#### Scenario: Read namespace segments
- **WHEN** a generated page displays the hierarchy path `GitP/A/Session/26/10/22-Thu`
- **THEN** each slash has 8px of space on both sides at 30% opacity while the existing segment links remain intact

#### Scenario: Wrap a long path
- **WHEN** a hierarchy path exceeds the available width on a narrow screen
- **THEN** the path continues to wrap without a forced no-wrap separator container

### Requirement: Distinguish namespace destinations

The publisher SHALL visibly label generated namespace pages and their entries in the complete page index as Namespace. Links to generated namespace destinations, including hierarchy segments, SHALL have a distinction beyond color and an accessible Namespace name; hierarchy rows SHALL carry no Namespace badge. Selected stored source pages SHALL retain ordinary page labels even when empty. Classification SHALL use only selected visible source routes: missing and ordinarily unselected source ancestors SHALL have the same Namespace presentation. Withheld and alias-conflicting ancestors SHALL retain their existing unavailable behavior. Full logical anchor text in the page index SHALL remain unchanged, with the status label outside the anchor. Source-only name, alias, embed, homepage, route, and count contracts SHALL remain intact.

#### Scenario: Generated and stored destinations
- **WHEN** a selected stored page needs a generated ancestor, and another selected stored page has an empty body
- **THEN** only the generated ancestor has the Namespace page and index status and the distinct link style, including in hierarchy segments

#### Scenario: Unselected ancestor privacy
- **WHEN** one generated ancestor has an ordinary unselected source file and another has no source file
- **THEN** both have identical Namespace status without source file paths, content, or existence claims

#### Scenario: Safe index labels and private gaps
- **WHEN** the index and hierarchy include generated destinations while a private or alias-conflicting prefix is unavailable
- **THEN** generated index anchors keep their exact full logical names, generated links identify Namespace status, and unavailable prefixes receive no new route

#### Scenario: Rows without badges
- **WHEN** a reader opens `tmux`, whose rows include paths through the generated `tmux/Command`
- **THEN** no row shows a Namespace badge and the `Command` segment keeps the generated link style

## ADDED Requirements

### Requirement: Load long hierarchy lists as the reader scrolls

A page's HTML SHALL hold at most the first 100 hierarchy rows. The publisher SHALL write the remaining rows, in order, to fragment files in the site's `_refs/` directory, about 500 rows per file. When the reader scrolls near the end of the loaded rows, the references script SHALL fetch the next file and append its rows. Without scripts, the section SHALL end with a link to the first fragment, and each fragment SHALL be a readable page linking to the next. Fragments SHALL follow the same escaping, link and withholding rules as in-page rows.

#### Scenario: Large namespace root
- **WHEN** a reader opens `Person`
- **THEN** the page holds 100 rows, scrolling to the end of the list loads the rest in order, and the full list matches the namespace index's rows

#### Scenario: Scripts off
- **WHEN** a reader with scripts disabled opens `Person`
- **THEN** the section shows 100 rows and a link to the first fragment, which shows the next rows and links to the next fragment

#### Scenario: Short list
- **WHEN** a page has at most 100 rows
- **THEN** all rows are in the page and no hierarchy fragment is written for it

### Requirement: Keep the hierarchy section within budgets

The `[hierarchy]` table in `scripts/parity/budgets.toml` SHALL set: at most 100 inline rows, hierarchy styles at most 0.5 KB gzip on every page, `references.js` growth at most 1 KB minified, each hierarchy fragment at most 100 KB gzip, peak phone heap at most 20 MB after scrolling `Microfreak`'s list to the end, and whole-garden build time at most 10% above the build before this change. `parity:budgets` SHALL check the size budgets in CI, including that `Person` and `Microfreak` stay within the `[desktop]` and `[phone]` first-paint budgets and that a fixture namespace grown by 2,000 pages leaves its root page's size unchanged within 1 KB; `parity:perf` SHALL mark the others.

#### Scenario: Grown namespace
- **WHEN** 2,000 pages are added under one namespace in the `parity:budgets` fixture garden
- **THEN** the namespace root's page grows by at most 1 KB and every hierarchy fragment stays within 100 KB gzip
