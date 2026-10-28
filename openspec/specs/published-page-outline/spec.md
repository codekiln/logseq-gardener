## Purpose

Define how the published page shows its title and outline like Logseq: styling, task markers, collapse and expand, block zoom and the outline's performance budgets.

## Requirements

### Requirement: Style the title and outline like Logseq

The publisher SHALL mark the root outline of each page and each journal-feed article with the `outline` class and style it as Logseq's published site does: the page title at 36px, weight 300, 24px above the content; no browser list markers; a 6px round bullet centred in a 14px column at every depth, with a hover disc; children indented 29px with a 2px guide line; 2px block padding and a 1.6 line height. Headings `h1`–`h6` SHALL use Logseq's sizes and weights (32px/300, 24px/300, 19.2px/600, 16px/600, 13.28px/600, 12px/600), with a 1px bottom rule under h1 and h2, and keep their bullet. Colours SHALL come from theme tokens in both themes. The outline SHALL follow page properties directly, with no empty paragraph between them. Lists inside a block's content and Linked References contexts SHALL keep their current style.

#### Scenario: Headings and bullets on a fixture page
- **WHEN** a reader opens `1Password/Account/Service` at 1280px wide
- **THEN** the title's computed size and weight are 36px and 300, the h1, h2 and h3 blocks match Logseq's sizes, weights and rules, no outline item shows a list marker, and each level's bullet sits within 2px of its x position in Logseq's build

#### Scenario: No empty paragraph
- **WHEN** a page has page properties followed by a blank line and an outline
- **THEN** the published page has no empty paragraph between the properties and the outline

### Requirement: Render task markers

The publisher SHALL render a task block's marker before its text as Logseq does: a disabled unchecked checkbox and the marker word as a label for TODO, DOING, NOW, LATER, WAIT, WAITING and IN-PROGRESS; a disabled checked checkbox and the text struck through for DONE; the text struck through without checkbox or label for CANCELED. A priority SHALL render with its brackets, `[#A]`, linked to page `A` when that page is published.

#### Scenario: Mixed tasks
- **WHEN** a page has DOING, TODO and DONE blocks
- **THEN** the DOING and TODO blocks show unchecked disabled checkboxes and their labels, and the DONE block shows a checked disabled checkbox and struck-through text

#### Scenario: Priority
- **WHEN** a block reads `TODO [#A] ship` and page `A` is not published
- **THEN** the block shows `[#A]` as text after the TODO label

### Requirement: Collapse and expand any block

On a page with an outline, readers SHALL be able to collapse and expand any block that has children, by an arrow shown left of the bullet while the block's row is hovered or by the strip over its children's guide line. A collapsed block SHALL show its bullet on a filled disc. The state SHALL last only in the open tab. Navigating to a block anchor inside a reader-collapsed block SHALL expand its collapsed ancestors.

#### Scenario: Collapse an ordinary block
- **WHEN** a reader clicks the arrow on `Important limitations` on `1Password/Account/Service`, then clicks it again
- **THEN** its children are hidden and its bullet shows the filled disc, and the second click shows the children again

#### Scenario: Linked block inside a folded block
- **WHEN** a reader has collapsed a block and then follows a link to a block anchor inside it
- **THEN** the collapsed block expands and the target is visible

### Requirement: Zoom into a block

Clicking a bullet, or pressing Enter on a focused bullet, SHALL open the zoomed view of that block at `<page>#zoom/<identifier>`, using the SDK's identifier for the block. The zoomed view SHALL show a breadcrumb and then the block as the outline's root, with its bullet and all its descendants expanded, and SHALL hide the page title, page properties, the namespace hierarchy section, Linked References and every other block. The breadcrumb SHALL hold a link to the page, then up to three nearest ancestors as links to their own zoomed views, with `⋯` after the page when more ancestors are hidden, and SHALL omit the zoomed block. Leaving the zoom address SHALL restore the page view with the reader's earlier collapse state. In a journal-feed article, a bullet SHALL open the journal's page at the block's zoom address. An unknown identifier SHALL leave the page view.

#### Scenario: Zoom into a nested collapsed block
- **WHEN** a reader opens `rw-cli` at `#zoom/1.1.1.2`, the collapsed block below `rw-cli list - list items in Readwise/Reader`
- **THEN** the breadcrumb reads `rw-cli › rw-cli - An open-source… › Commands › rw-cli list - list items in Readwise/Reader`, the block and its children are shown expanded, and the title and Linked References are hidden

#### Scenario: Stable address
- **WHEN** a reader saves a zoom address for a block without `id::` and the unchanged garden is published again
- **THEN** the address opens the same block

#### Scenario: Back to the page
- **WHEN** a reader in a zoomed view presses the browser's Back button
- **THEN** the page view returns with the blocks the reader had collapsed still collapsed

### Requirement: Keep the outline within performance budgets

The outline SHALL add no file per block and no graph data to the browser. `lsg-outline.js` SHALL be at most 8 KB uncompressed and load after the page's `load` event. The outline CSS SHALL add at most 1.5 KB gzip to each page. The `tmux` fixture page SHALL stay under 100 KB gzip and grow by at most 2 KB gzip. A bullet click on `tmux` SHALL paint the zoomed view within 50 ms on the desktop profile and 150 ms on the phone profile. The existing budgets for data and script before the title and for peak heap SHALL hold. CI SHALL check the script and page sizes, and the comparison harness SHALL record the zoom time.

#### Scenario: Budgets in CI
- **WHEN** `parity:budgets` runs on the fixture garden
- **THEN** it fails if `lsg-outline.js` exceeds 8 KB, the outline styles exceed 1.5 KB gzip, or the largest fixture page exceeds its size budget

#### Scenario: Zoom timing recorded
- **WHEN** `parity:perf --record` runs on the encode garden
- **THEN** `docs/parity/performance.md` gains the zoom time on `tmux` for both profiles, marked when over budget

### Requirement: Break long words to fit the screen

The publisher SHALL break a word in an outline block that does not fit the block's width, as Logseq's block content does, so no outline block makes the page wider than the screen. A word that fits SHALL NOT break. Code blocks SHALL keep their own horizontal scrolling.

#### Scenario: Long page reference on a phone
- **WHEN** a reader opens `GitP/A/Session/26/10/24-Sat` at 390×844 with every block expanded
- **THEN** the page is 390px wide and each `Ableton/Live/Mode/Arrangement` link ends within the screen
