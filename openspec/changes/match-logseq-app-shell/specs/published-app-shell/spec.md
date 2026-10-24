## ADDED Requirements

### Requirement: Supply shell settings from the SDK

The SDK SHALL expose an operation that reads `:favorites`, `:feature/enable-journals?`, `:feature/enable-flashcards?` and `:feature/enable-whiteboards?` from the garden's `logseq/config.edn` with the existing bounded EDN reader. Favorites SHALL keep config order. Each feature flag SHALL default to true when absent or malformed. A second operation SHALL resolve favorites through the selected visible title and alias lookup used by page links, returning in order each resolved page's route and original title, keeping only the first entry for a page named twice, and returning a diagnostic for each entry that resolves to no selected visible page.

#### Scenario: Favorites through an alias
- **WHEN** `:favorites` lists `["tmux" "github/cli/extension/gh-dash"]` and `gh-dash` has the alias `GitHub/CLI/Extension/gh-dash`
- **THEN** the operation returns `tmux` and `gh-dash`, in that order, with their routes and original titles

#### Scenario: Unpublished favorite
- **WHEN** a favorite names an excluded, private or missing page
- **THEN** the resolved list omits it and the publish report contains a diagnostic naming the entry

#### Scenario: Missing or malformed config
- **WHEN** `config.edn` is absent or `:favorites` is not a vector of strings
- **THEN** the favorites list is empty, every feature flag is true, and publication continues

### Requirement: Show Logseq's header on every page

Every generated page SHALL begin with a sticky header 48px tall holding, left to right, a Toggle left sidebar button, then Home and More at the right edge. Home SHALL link to `index.html` and SHALL be absent from `index.html`. Each control SHALL have an accessible name and visible keyboard focus. The header SHALL contain only controls whose destination exists in the generated site.

#### Scenario: Header on an ordinary page
- **WHEN** a reader opens the `tmux` page
- **THEN** the header shows Toggle left sidebar, Home and More in that order, each reachable by keyboard with an accessible name

#### Scenario: Header on the home page
- **WHEN** a reader opens `index.html`
- **THEN** the header shows Toggle left sidebar and More, and Home is absent

#### Scenario: No missing destinations
- **WHEN** a site is generated before search, graph view or the right sidebar exist
- **THEN** no header control links to a file missing from the output

### Requirement: Show the left sidebar with All pages and favorites

Every generated page SHALL include a left sidebar listing All pages, then a FAVORITES heading followed by one link per resolved favorite in config order, labelled with the page's original title. All pages SHALL link to the complete page index (`pages.html` when a homepage is selected, `index.html` otherwise). The link for the current page SHALL carry `aria-current="page"` and a highlighted style. The FAVORITES heading SHALL be absent when no favorite resolves.

#### Scenario: Encode garden favorites
- **WHEN** the encode garden is published with all its namespaces selected
- **THEN** the sidebar lists the 19 favorites in `:favorites` order and each link opens the matching page

#### Scenario: Current favorite
- **WHEN** a reader opens a page that is a favorite
- **THEN** that page's sidebar link is marked current and highlighted

### Requirement: Open and close the left sidebar

The left sidebar SHALL start closed. The header button and the `t l` shortcut SHALL toggle it, update `aria-expanded`, and save the state in browser storage so it persists across pages and reloads. At viewport widths of 640px and more the open sidebar SHALL be 246px wide and push the main column right. Below 640px it SHALL open as an overlay 74vw wide over a shade; choosing the shade, a sidebar link or Escape SHALL close it, and each page load SHALL start with the overlay closed. A saved open state SHALL apply before first paint at 640px and wider. Unavailable storage SHALL keep the state for the current page only.

#### Scenario: State survives navigation
- **WHEN** a reader at 1280px opens the sidebar on `tmux` and follows a link
- **THEN** the next page opens with the sidebar open and the main column already shifted right

#### Scenario: Phone overlay
- **WHEN** a reader at 390px opens the sidebar and taps the shade
- **THEN** the sidebar closes, focus returns to the toggle button, and the page never scrolls sideways

#### Scenario: Without JavaScript
- **WHEN** JavaScript is disabled
- **THEN** the sidebar stays closed, its toggle button and More are hidden, and Home and the page content remain usable

### Requirement: Offer the More menu

The More button SHALL open a menu containing Toggle theme and, while a light or dark choice is saved, Use system theme. The menu SHALL open from the keyboard, move focus to its first item, support arrow-key movement between items, and close on Escape, on a click outside it, and after an item is chosen, returning focus to the More button. Every item shown SHALL work.

#### Scenario: Keyboard use
- **WHEN** a reader focuses More, presses Enter, then Escape
- **THEN** the menu opens with focus on Toggle theme and then closes with focus back on More

#### Scenario: Toggle theme from More
- **WHEN** a reader in light appearance chooses Toggle theme and reloads
- **THEN** the page is dark after the click and after the reload

### Requirement: Match Logseq's widths and type

The main column SHALL be at most 810px wide, or 1280px in wide mode, with 16px side padding below 640px and 32px from 640px. Body text SHALL use the font stack `"Source Sans 3", "Source Sans Pro", system-ui, sans-serif` at 16px. The `t w` shortcut SHALL toggle wide mode and save it in browser storage. At 390px the header SHALL stay on one line and no page SHALL scroll sideways.

#### Scenario: Desktop widths
- **WHEN** the `tmux` page is shown at 1280px with the sidebar closed
- **THEN** the main column's content box is at most 810px wide with 32px side padding

#### Scenario: Wide mode
- **WHEN** a reader presses `t w` and reloads
- **THEN** the main column is at most 1280px wide after the key press and after the reload

### Requirement: Handle Logseq's shell shortcuts

Generated pages SHALL handle `t l` (toggle left sidebar), `t t` (toggle theme), `t w` (toggle wide mode), `g h` (open Home) and `g a` (open All pages) when the second key follows the first within one second. Shortcuts SHALL be ignored while focus is in an input, textarea, select or editable element, or while Control, Alt or Meta is held.

#### Scenario: Shortcut outside a field
- **WHEN** a reader on `tmux` presses `g` then `a`
- **THEN** the browser opens the complete page index

#### Scenario: Shortcut inside a field
- **WHEN** focus is in a backlink filter input and the reader types `t t`
- **THEN** the input receives the text and the appearance stays the same

### Requirement: Keep the shell's cost fixed per page

The shell's behaviour SHALL ship as one shared `lsg-shell.js` file at the site root, loaded with `defer` by every page and at most 6 KB uncompressed. The inline shell styles plus header and sidebar markup SHALL stay within 6 KB per page for the encode garden's favorites. A generated page's size SHALL stay the same when unrelated pages are added to the garden. The size checks SHALL run in CI through the parity budgets check.

#### Scenario: Shared script
- **WHEN** a site is generated
- **THEN** the output contains one `lsg-shell.js` and every page references it with `defer`

#### Scenario: Garden growth
- **WHEN** the parity fixture garden is built again with 2,000 unrelated pages added
- **THEN** every fixture page and `lsg-shell.js` keep the same size and stay within their budgets
