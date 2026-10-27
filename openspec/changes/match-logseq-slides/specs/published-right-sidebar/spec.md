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
