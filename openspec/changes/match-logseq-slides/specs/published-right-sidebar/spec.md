## MODIFIED Requirements

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

### Requirement: Keep the right sidebar within budgets

The sidebar's behaviour beyond the shell hook SHALL ship as one `lsg-sidebar.js` at the site root, loaded on the reader's first use or, with a saved open sidebar, after the page's `load` event. The comparison harness SHALL check, on the whole encode garden: every fixture page's first paint unchanged apart from the toggle button and within the `[desktop]` and `[phone]` budgets; `lsg-sidebar.js` at most 6.5 KB gzip; Shift-click to a card's outline at p95 within 300 ms desktop and 1,500 ms phone; one card's transfer at most 60 KB gzip; and a peak phone heap of at most 20 MB after ten cards. `parity:budgets` SHALL check the size budgets in CI.

#### Scenario: No script before use
- **WHEN** a reader with no saved sidebar opens `tmux` and does not use the sidebar
- **THEN** `lsg-sidebar.js` and no other page's file are requested

#### Scenario: Largest card
- **WHEN** a reader adds a card for `Person`
- **THEN** the card's transfer is at most 60 KB gzip
