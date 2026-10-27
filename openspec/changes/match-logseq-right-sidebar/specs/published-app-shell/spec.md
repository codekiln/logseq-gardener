## MODIFIED Requirements

### Requirement: Supply shell settings from the SDK

The SDK SHALL expose an operation that reads `:favorites`, `:default-home {:sidebar …}`, `:feature/enable-journals?`, `:feature/enable-flashcards?` and `:feature/enable-whiteboards?` from the garden's `logseq/config.edn` with the existing bounded EDN reader. Favorites SHALL keep config order. `:sidebar` SHALL accept one string or a vector of strings, keeping config order. Each feature flag SHALL default to true when absent or malformed. A second operation SHALL resolve favorites through the selected visible title and alias lookup used by page links, returning in order each resolved page's route and a label with the author's spelling of the matched title or alias, keeping only the first entry for a name listed twice, and returning a diagnostic for each entry that resolves to no selected visible page. The same operation SHALL resolve the `:sidebar` names, treating `"Contents"` as the Contents card.

#### Scenario: Favorites through an alias
- **WHEN** `:favorites` lists `["tmux" "github/cli/extension/gh-dash"]` and `gh-dash` has the alias `GitHub/CLI/Extension/gh-dash`
- **THEN** the operation returns `tmux`, then the `gh-dash` page labelled `GitHub/CLI/Extension/gh-dash`, with their routes

#### Scenario: Unpublished favorite
- **WHEN** a favorite names an excluded, private or missing page
- **THEN** the resolved list omits it and the publish report contains a diagnostic naming the entry

#### Scenario: Missing or malformed config
- **WHEN** `config.edn` is absent or `:favorites` is not a vector of strings
- **THEN** the favorites list is empty, every feature flag is true, and publication continues

#### Scenario: Sidebar start cards
- **WHEN** `:default-home` is `{:sidebar ["Contents" "tmux" "missing"]}`
- **THEN** the operation returns the Contents card and the `tmux` route in that order, and a diagnostic names `missing`

### Requirement: Show Logseq's header on every page

Every generated page SHALL begin with a sticky header 48px tall holding, left to right, a Toggle left sidebar button, then Home and More at the right edge, followed at viewport widths of 640px and more by a Toggle right sidebar button with the `layout-sidebar-right` icon and `aria-expanded`. Home SHALL link to `index.html` and SHALL be absent from `index.html`. Each control SHALL have an accessible name and visible keyboard focus. The header SHALL contain only controls whose destination exists in the generated site. The Toggle right sidebar button SHALL be hidden when JavaScript is disabled.

#### Scenario: Header on an ordinary page
- **WHEN** a reader at 1280px opens the `tmux` page
- **THEN** the header shows Toggle left sidebar, Home, More and Toggle right sidebar in that order, each reachable by keyboard with an accessible name

#### Scenario: Header on the home page
- **WHEN** a reader opens `index.html`
- **THEN** the header shows Toggle left sidebar, More and Toggle right sidebar, and Home is absent

#### Scenario: No missing destinations
- **WHEN** a site is generated before search or graph view exist
- **THEN** no header control links to a file missing from the output

### Requirement: Handle Logseq's shell shortcuts

Generated pages SHALL handle `t l` (toggle left sidebar), `t r` (toggle right sidebar, at 640px and wider), `t t` (toggle theme), `t w` (toggle wide mode), `g h` (open Home), `g a` (open All pages), `alt+shift+c` (toggle the Contents card, at 640px and wider) and, when a journals feed is written, `g j` (open the feed's first batch), with two-key shortcuts recognized when the second key follows the first within one second. Shortcuts SHALL be ignored while focus is in an input, textarea, select or editable element, or while Control or Meta is held, and the two-key shortcuts also while Alt is held.

#### Scenario: Shortcut outside a field
- **WHEN** a reader on `tmux` presses `g` then `a`
- **THEN** the browser opens the complete page index

#### Scenario: Shortcut inside a field
- **WHEN** focus is in a backlink filter input and the reader types `t t`
- **THEN** the input receives the text and the appearance stays the same

#### Scenario: Open journals
- **WHEN** the encode garden is published with journals and a reader on `tmux` presses `g` then `j`
- **THEN** the browser opens `index.html` showing the feed

#### Scenario: Toggle the right sidebar
- **WHEN** a reader at 1280px on `tmux` presses `t` then `r`
- **THEN** the right sidebar opens

### Requirement: Keep the shell's cost fixed per page

The shell's behaviour SHALL ship as one shared `lsg-shell.js` file at the site root, loaded with `defer` by every page and at most 6.5 KB uncompressed. The inline shell styles plus header and sidebar markup SHALL stay within 6 KB per page for the encode garden's favorites. A generated page's size SHALL stay the same when unrelated pages are added to the garden. The size checks SHALL run in CI through the parity budgets check.

#### Scenario: Shared script
- **WHEN** a site is generated
- **THEN** the output contains one `lsg-shell.js` and every page references it with `defer`

#### Scenario: Garden growth
- **WHEN** the parity fixture garden is built again with 2,000 unrelated pages added
- **THEN** every fixture page and `lsg-shell.js` keep the same size and stay within their budgets
