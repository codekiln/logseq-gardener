## Purpose

Let readers of a published garden open a Page graph card in the right sidebar that shows the current page's neighbourhood, loading its data only when the card opens.

## Requirements

### Requirement: Add a Page graph card

The right sidebar's top bar SHALL hold a `Page graph` button with the `hierarchy` icon after Contents and before Help. It SHALL add a Page graph card at the top, or move the existing one to the top; the card SHALL have the right sidebar's card header, menu and close controls, and SHALL be kept in the sidebar's saved state like other cards.

#### Scenario: Open from the top bar

- **WHEN** a reader on `tmux` opens the right sidebar and presses `Page graph`
- **THEN** a Page graph card titled `Page graph` with the `hierarchy` icon appears at the top

#### Scenario: Saved card

- **WHEN** a reader with a Page graph card open reloads the page
- **THEN** the card is still open

### Requirement: Show the current page's neighbourhood

The card SHALL draw the SDK's neighbourhood of the page in the current route, with the global graph's node and edge look, labels, theme colours, hover, pan, zoom and click to open, without the settings panel. Its body SHALL be a `Show Journals` row with a switch at the right, off at first and kept for the tab's session, then a canvas 500px high at the card's width. The card SHALL follow navigation to another page. On a page with no page in the route, the card SHALL show today's journal's neighbourhood when that journal is published, and nothing otherwise. On a zoomed block route it SHALL show the block's page.

#### Scenario: Foam

- **WHEN** a reader opens the Page graph card on `Foam`
- **THEN** it draws the SDK's neighbourhood of `Foam` with journals off

#### Scenario: Show Journals

- **WHEN** a reader on `tmux` turns on Show Journals
- **THEN** the card adds the journal pages that refer to `tmux` and their edges

#### Scenario: Follow navigation

- **WHEN** a reader with the card open clicks a link to `mise`
- **THEN** the card draws `mise`'s neighbourhood

### Requirement: Load neighbourhoods per page

The publisher SHALL write each published page's neighbourhood, with journal nodes flagged, to `_graph/p/<hash>.json`, matching the page's route, when its gzip size is at most 100 KB. The card SHALL fetch that file only when it opens or the route changes, and lay it out in the graph worker with Logseq's default forces before its first frame. For a page whose neighbourhood exceeds 100 KB gzip, the publisher SHALL write no file, and the card SHALL fetch `_graph/default.bin` (or `journals.bin`) and draw the neighbourhood from it at the global positions.

#### Scenario: Small page

- **WHEN** a reader opens the card on `Foam`
- **THEN** the browser fetches `Foam`'s neighbourhood file and no global graph file

#### Scenario: Very large page

- **WHEN** a reader opens the card on `Logseq`, whose neighbourhood holds 60,000 edges
- **THEN** the browser fetches the global graph file and draws `Logseq`'s neighbourhood from it

### Requirement: Keep the page graph within budgets

The Page graph SHALL meet the `[graph]` budgets: every neighbourhood file at most 100 KB gzip; from pressing `Page graph` to the first frame within 300 ms on the desktop profile and 1,500 ms on the phone profile for the fixture pages below the file limit. No page's first paint SHALL change except for the `Page graph` button inside the sidebar's script-drawn top bar. `parity:budgets` SHALL check the file sizes in CI and `parity:perf` SHALL mark the open times for `Foam`, `tmux` and `Person`.

#### Scenario: Open time

- **WHEN** `parity:perf` opens the card on `tmux`
- **THEN** the first frame appears within the desktop and phone budgets
