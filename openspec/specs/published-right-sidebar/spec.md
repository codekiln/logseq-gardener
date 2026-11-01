## Purpose

Let readers of a published garden keep other pages and blocks open beside the current page, as Logseq's right sidebar does, while loading nothing for it until a reader uses it.

## Requirements

### Requirement: Open and close the right sidebar

At viewport widths of 640px and more, the Toggle right sidebar button and the `t r` shortcut SHALL open and close a right column of cards, updating the button's `aria-expanded`. The sidebar SHALL start closed on a first visit. It SHALL close when its last card is closed, after Close all, or by the toggle or `t r`. While open, the column SHALL be 40% of the window wide unless the reader resized it, the header and main column SHALL use the remaining width, and the card list SHALL scroll on its own.

#### Scenario: Toggle on tmux
- **WHEN** a reader at 1280px on `tmux` presses `t r`
- **THEN** the sidebar opens 512px wide with the main column in the remaining 768px and no sideways scroll, and pressing `t r` again closes it

#### Scenario: Empty sidebar
- **WHEN** a reader opens the sidebar with no cards
- **THEN** it shows one Contents card

### Requirement: Add cards with Shift-click

At 640px and wider, Shift-click on a page reference, tag, page-valued property, Pages tagged with row, reference group title, page embed header, query result, All pages row or plugin-derived page link SHALL add a card for that page; on a bullet, a card for that block; on a block reference, a Block references card for the referenced block. In Linked References and query results, Shift-click on a bullet SHALL add a card for that block on its source page, and on a breadcrumb crumb a Block references card for the crumb's block, as Logseq does. A link SHALL count as a page link when it sits inside `.page-reference`, has the class `page-ref` or `tag`, or its address is a `p-*.html` file, a link with the class `block-ref-link` SHALL count as a block reference at any address, and a link whose address holds `#source-block-` SHALL count as a link to that block of its page at any address, so the gesture keeps working when a host rewrites page addresses after the build. Gardener SHALL render each of those page links inside `.page-reference` or with the class `page-ref` or `tag`, as Logseq renders every page link as `a.page-ref`; a plugin-derived link to a block of a page SHALL stay a plain link. A card's route SHALL be the address the browser resolves for the link, or for the current page when the target is one of its bullets, written relative to the site root that serves `lsg-sidebar.js`. The main page and its URL SHALL stay the same. The new card SHALL go to the top, expanded, and the sidebar SHALL open. A target already in the sidebar SHALL move to the top and expand. The placeholder Contents card SHALL be replaced by the first added card. Links and bullets inside cards SHALL behave the same way, and a plain click on a link inside a card SHALL open the page in the main column.

#### Scenario: Page and block cards on tmux
- **WHEN** a reader on `tmux` Shift-clicks the link to `tmux/Idea/Pomodoro with status line and silent flash` and then the second bullet
- **THEN** the sidebar holds a block card whose header is the breadcrumb starting `tmux`, above a page card for `tmux/Idea/Pomodoro with status line and silent flash`, and the URL is unchanged

#### Scenario: Block reference
- **WHEN** a reader on `VSCode/Proj/24t3/LearnVSCode/01 Terminal` Shift-clicks its block reference
- **THEN** a `Block references` card shows the referenced block and its children

#### Scenario: Card already present
- **WHEN** a reader Shift-clicks a page that already has a collapsed card lower in the sidebar
- **THEN** that card moves to the top, expanded, and no second card for the page appears

#### Scenario: Page links at rewritten addresses
- **WHEN** a host serves the site root at `/gitpa/` and pages at readable routes, and a reader on `/gitpa/GitP/A/Session/26/10/29-Thu/` Shift-clicks an inline page reference to `/gitpa/GitP/A/Session/26/10/28-Wed/`, a query result `a.page-ref` and a tag `a.tag`
- **THEN** each adds a page card that shows its page, the first is saved with the route `GitP/A/Session/26/10/28-Wed/`, and a plain click on the same links still navigates

#### Scenario: Bullet on a page at a rewritten address
- **WHEN** a reader on `/gitpa/GitP/A/Session/26/10/29-Thu/` Shift-clicks the page's first bullet
- **THEN** the block card is saved with the route `GitP/A/Session/26/10/29-Thu/` and block `1`, and shows that page's block

#### Scenario: Links inside a card for a rewritten route
- **WHEN** a reader opens the card menu of the page card for `GitP/A/Session/26/10/28-Wed/` and chooses Open as page, or follows a crumb in a block card from that page
- **THEN** the browser opens `/gitpa/GitP/A/Session/26/10/28-Wed/`, with `#zoom/` and the block's identifier for a crumb

#### Scenario: Link inside a flashcard block
- **WHEN** a reader Shift-clicks a page reference inside a flashcard block (`li.lsg-card`)
- **THEN** a card for that page is added

#### Scenario: Every kind of page link at rewritten addresses
- **WHEN** a host serves pages at readable routes under `/gitpa/`, and a reader Shift-clicks a page-valued `prev::` property, a Pages tagged with row, a Linked References group title and a page embed header
- **THEN** each adds a card for its page, and a plain click on the same links still navigates

#### Scenario: Linked References crumb and bullet
- **WHEN** a host serves pages at readable routes under `/gitpa/`, and a reader on `/gitpa/Microfreak/` Shift-clicks a crumb in Linked References and then a bullet there
- **THEN** the crumb adds a Block references card for its block and the bullet a card for its block, each read from the source page, both are saved with the source page's route, another page restores them, and the URL is unchanged

### Requirement: Show page and block content in cards

A page card SHALL show the page as the main column shows it, without its title and without Unlinked References: properties, outline, Pages tagged with, Linked References closed until opened, and Hierarchy. A block card SHALL show the block and its children under a header holding the block's breadcrumb. A Block references card SHALL show `Block references`, a breadcrumb row and the block with its children. Card content SHALL come from the published page file and the existing reference fragments, fetched when the card is added or expanded. A card whose page is not published SHALL say so and offer Close.

#### Scenario: Page card content
- **WHEN** a reader adds a card for `tmux/Idea/Pomodoro with status line and silent flash`
- **THEN** the card has no `h1`, its outline starts with the heading `Pomodoro in tmux: status-line countdown and silent visual cue`, and Linked References is a closed header

#### Scenario: Reference bodies on demand
- **WHEN** a reader opens Linked References inside a page card
- **THEN** the references load from the page's `_refs/` fragment files, as they do on the page itself

### Requirement: Collapse, close and manage cards

Each card header SHALL be a button with `aria-expanded` and `aria-controls` holding a chevron, an icon (the page's `icon::` when it has one) and the title truncated with an ellipsis; activating it SHALL collapse or expand the card, and a collapsed card SHALL show only its header. A Close button and a Card menu button SHALL follow, each with an accessible name. A middle click on the header SHALL close the card. The card menu SHALL offer Close; Close others and Close all when there are two or more cards; Collapse or Expand; Collapse others and Collapse all when there are two or more cards; Expand all when there are two or more cards; and Open as page for page cards and Slides cards, separated into groups as Logseq separates them. The menu SHALL open from the keyboard with focus on its first item, move between items with arrow keys, and close on Escape, on a click outside it and after an item is chosen, returning focus to the Card menu button.

#### Scenario: Menu with three cards
- **WHEN** the sidebar holds three expanded cards and a reader opens the first card's menu
- **THEN** the menu lists Close, Close others, Close all, Collapse, Collapse others, Collapse all and Expand all, and choosing Close all empties and closes the sidebar

#### Scenario: Collapse from the header
- **WHEN** a reader activates a card's header
- **THEN** the card shows only its header with `aria-expanded="false"`, and activating it again shows the content

#### Scenario: Slides card menu
- **WHEN** the sidebar holds only a Slides card for `tmux` and a reader opens its menu
- **THEN** the menu lists Close, Collapse and Open as page, and Open as page opens `tmux` in the main column

### Requirement: Offer Contents and Help

A top bar above the cards SHALL hold Contents and Help buttons, each adding its card at the top. The Contents card SHALL show the outline of the published `contents` page without its title, or one empty bullet when the garden publishes no `contents` page. `alt+shift+c` SHALL add the Contents card and open the sidebar, or remove it and close the sidebar when it is the only card. The Help card SHALL list the shortcuts and gestures the published site supports and SHALL link to no site outside the garden. A page whose file is one empty bullet SHALL be published like any selected page.

#### Scenario: Contents on the encode garden
- **WHEN** a reader on `tmux` presses `alt+shift+c`
- **THEN** the sidebar opens with a Contents card showing one empty bullet from `pages/contents.md`

#### Scenario: Help shortcuts work
- **WHEN** a reader opens the Help card
- **THEN** every shortcut it lists works on the published site and it contains no external link

### Requirement: Resize the right sidebar

The column's left edge SHALL be a focusable separator with an accessible name that resizes the column by dragging and by Left and Right arrow keys in 5px steps, between the larger of 320px and 10% of the window and 70% of the window. The width SHALL be saved with the sidebar state.

#### Scenario: Width survives a reload
- **WHEN** a reader at 1280px drags the separator to make the sidebar 600px wide and reloads
- **THEN** the sidebar is 600px wide

### Requirement: Keep the sidebar across pages and reloads

The open state, width, cards in order and each card's collapsed flag SHALL be saved in browser storage by route and block identifier and restored on every page. Restoring SHALL apply the column's width and card frames before first paint and fetch card content only after the page's `load` event. When there is no saved state, `:default-home {:sidebar …}` in `logseq/config.edn` SHALL supply the starting cards: each name that resolves to a selected visible page or alias adds a page card, `"Contents"` adds the Contents card, and any other name produces a publish diagnostic. Unavailable storage SHALL keep the state for the current page only.

#### Scenario: Cards survive navigation
- **WHEN** a reader opens three cards on `tmux`, follows a link and reloads
- **THEN** the same three cards appear in the same order with the same collapsed flags

#### Scenario: Configured start
- **WHEN** a fixture garden sets `:default-home {:sidebar ["Contents" "tmux"]}` and a reader with no saved state opens a page
- **THEN** the sidebar opens with a Contents card and a `tmux` card

#### Scenario: Rewritten route restored on another page
- **WHEN** a reader adds a card for `/gitpa/GitP/A/Session/26/10/28-Wed/` on one page of a host that serves readable routes, then opens `/gitpa/Microfreak/`
- **THEN** the restored card fetches `/gitpa/GitP/A/Session/26/10/28-Wed/` and shows that page

### Requirement: Leave phones to the page

Below a viewport width of 640px, the Toggle right sidebar button SHALL be absent, `t r` and `alt+shift+c` SHALL do nothing, and Shift-click SHALL follow the link as the browser does.

#### Scenario: Phone width
- **WHEN** a reader at 390×844 on `tmux` Shift-clicks a page link
- **THEN** no sidebar opens, the header has no Toggle right sidebar button, and the page never scrolls sideways

### Requirement: Keep the right sidebar within budgets

The sidebar's behaviour beyond the shell hook SHALL ship as one `lsg-sidebar.js` at the site root, loaded on the reader's first use or, with a saved open sidebar, after the page's `load` event. The comparison harness SHALL check, on the whole encode garden: every fixture page's first paint unchanged apart from the toggle button and within the `[desktop]` and `[phone]` budgets; `lsg-sidebar.js` at most 6.5 KB gzip; Shift-click to a card's outline at p95 within 300 ms desktop and 1,500 ms phone; one card's transfer at most 60 KB gzip; and a peak phone heap of at most 20 MB after ten cards. `parity:budgets` SHALL check the size budgets in CI.

#### Scenario: No script before use
- **WHEN** a reader with no saved sidebar opens `tmux` and does not use the sidebar
- **THEN** `lsg-sidebar.js` and no other page's file are requested

#### Scenario: Largest card
- **WHEN** a reader adds a card for `Person`
- **THEN** the card's transfer is at most 60 KB gzip
