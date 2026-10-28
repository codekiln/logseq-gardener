## MODIFIED Requirements

### Requirement: Show the left sidebar with All pages and favorites

Every generated page SHALL include a left sidebar listing Journals when a journals feed is written, then Flashcards unless `config.edn` sets `:feature/enable-flashcards? false`, then Graph view, then All pages, then a FAVORITES heading followed by one link per resolved favorite in config order, labelled with the author's spelling of the title or alias the favorite names. Journals SHALL link to the feed's first batch and SHALL be marked current on every feed batch. Graph view SHALL link to `graph.html`, carry the `hierarchy` icon, the `g g` hint when that shortcut is on, and the class `graph-view-nav` on its item, as Logseq's does. Flashcards SHALL carry the `infinity` icon, the label `Flashcards`, the `g f` hint and a rounded badge holding the number of cards due at the build time, left out when that number is 0, with Logseq's measured styles, and SHALL open the review of all cards. All pages SHALL link to the complete page index (`pages.html` when a homepage is selected or a journals feed holds `index.html`, `index.html` otherwise). The link for the current page SHALL carry `aria-current="page"` and a highlighted style. The FAVORITES heading SHALL be absent when no favorite resolves.

#### Scenario: Encode garden favorites
- **WHEN** the encode garden is published with all its namespaces selected
- **THEN** the sidebar lists the 19 favorites in `:favorites` order and each link opens the matching page

#### Scenario: Current favorite
- **WHEN** a reader opens a page that is a favorite
- **THEN** that page's sidebar link is marked current and highlighted

#### Scenario: Journals item
- **WHEN** the encode garden is published with journals and a reader opens `tmux`
- **THEN** Journals is the sidebar's first item and links to `index.html`, Flashcards follows it with a due badge, Graph view follows that and links to `graph.html`, and All pages links to `pages.html`

#### Scenario: Journals item on the feed
- **WHEN** a reader opens `journals-2.html`
- **THEN** the Journals item is marked current

#### Scenario: Garden stylesheet hides the graph entry points
- **WHEN** the encode garden is published with `--stylesheet logseq/custom.css`
- **THEN** the garden's rules hide the header Graph link and the Graph view item, as they do on Logseq's build, and `graph.html` still opens when typed

#### Scenario: Flashcards turned off
- **WHEN** a garden whose `config.edn` sets `:feature/enable-flashcards? false` is published
- **THEN** the left sidebar has no Flashcards item

### Requirement: Handle Logseq's shell shortcuts

Generated pages SHALL handle `t l` (toggle left sidebar), `t r` (toggle right sidebar, at 640px and wider), `t t` (toggle theme), `t w` (toggle wide mode), `g h` (open Home), `g a` (open All pages), `g g` (open the graph view) unless `config.edn` sets `:shortcuts {:go/graph-view false}`, `alt+shift+c` (toggle the Contents card, at 640px and wider), `g f` and `t c` (open or close the flashcard review) unless flashcards are turned off, and, when a journals feed is written, `g j` (open the feed's first batch), with two-key shortcuts recognized when the second key follows the first within one second. Shortcuts SHALL be ignored while focus is in an input, textarea, select or editable element, or while Control or Meta is held, and the two-key shortcuts also while Alt is held.

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

#### Scenario: Open the graph view
- **WHEN** a garden whose `config.edn` leaves `:go/graph-view` unset is published and a reader on `tmux` presses `g` then `g`
- **THEN** the browser opens `graph.html`

#### Scenario: Graph shortcut turned off
- **WHEN** the encode garden, whose `config.edn` sets `:shortcuts {:go/graph-view false}`, is published and a reader presses `g` then `g`
- **THEN** nothing happens and the Graph view item shows no `g g` hint

#### Scenario: Open the flashcard review
- **WHEN** a reader on `tmux` presses `g` then `f`
- **THEN** the review of all cards opens, and pressing `g` then `f` again closes it

### Requirement: Keep the shell's cost fixed per page

The shell's behaviour SHALL ship as one shared `lsg-shell.js` file at the site root, loaded with `defer` by every page and at most 6.9 KB uncompressed, including the loader that imports the flashcard review on first use. The inline shell styles plus header and sidebar markup SHALL stay within 6 KB per page for the encode garden's favorites. A generated page's size SHALL stay the same when unrelated pages are added to the garden. The size checks SHALL run in CI through the parity budgets check.

#### Scenario: Shared script
- **WHEN** a site is generated
- **THEN** the output contains one `lsg-shell.js` and every page references it with `defer`

#### Scenario: Garden growth
- **WHEN** the parity fixture garden is built again with 2,000 unrelated pages added
- **THEN** every fixture page and `lsg-shell.js` keep the same size and stay within their budgets
