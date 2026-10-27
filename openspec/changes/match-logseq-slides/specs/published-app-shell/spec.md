## MODIFIED Requirements

### Requirement: Offer the More menu

The More button SHALL open a menu containing, on page routes at viewport widths of 640px and more, View as slides as its first item, then Toggle theme and, while a light or dark choice is saved, Use system theme. View as slides SHALL be absent from the home page, from All pages and below 640px. The menu SHALL open from the keyboard, move focus to its first item, support arrow-key movement between items, and close on Escape, on a click outside it, and after an item is chosen, returning focus to the More button. Every item shown SHALL work.

#### Scenario: Keyboard use
- **WHEN** a reader on the home page focuses More, presses Enter, then Escape
- **THEN** the menu opens with focus on Toggle theme and then closes with focus back on More

#### Scenario: Toggle theme from More
- **WHEN** a reader in light appearance chooses Toggle theme and reloads
- **THEN** the page is dark after the click and after the reload

#### Scenario: View as slides on a page
- **WHEN** a reader at 1280px opens More on `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`
- **THEN** the menu lists View as slides first, then Toggle theme

#### Scenario: No slides item on the home page or a phone
- **WHEN** a reader opens More on the home page at 1280px, or on `tmux` at 390×844
- **THEN** the menu has no View as slides item

### Requirement: Keep the shell's cost fixed per page

The shell's behaviour SHALL ship as one shared `lsg-shell.js` file at the site root, loaded with `defer` by every page and at most 6.6 KB uncompressed. The inline shell styles plus header and sidebar markup SHALL stay within 6 KB per page for the encode garden's favorites. A generated page's size SHALL stay the same when unrelated pages are added to the garden. The size checks SHALL run in CI through the parity budgets check.

#### Scenario: Shared script
- **WHEN** a site is generated
- **THEN** the output contains one `lsg-shell.js` and every page references it with `defer`

#### Scenario: Garden growth
- **WHEN** the parity fixture garden is built again with 2,000 unrelated pages added
- **THEN** every fixture page and `lsg-shell.js` keep the same size and stay within their budgets
