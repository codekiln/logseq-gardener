## MODIFIED Requirements

### Requirement: Show Logseq's header on every page

Every generated page SHALL begin with a sticky header 48px tall holding, left to right, a Toggle left sidebar button, then Home, Graph and More at the right edge, followed at viewport widths of 640px and more by a Toggle right sidebar button with the `layout-sidebar-right` icon and `aria-expanded`. Home SHALL link to `index.html` and SHALL be absent from `index.html`. Graph SHALL be an `a.button` linking to `graph.html#/graph`, with the `hierarchy` icon and the accessible name Graph, inside a header carrying the class `cp__header`, so Logseq's selector `.cp__header a.button[href$="#/graph"]` matches it and a stylesheet written for Logseq's header can hide it. Each control SHALL have an accessible name and visible keyboard focus. The header SHALL contain only controls whose destination exists in the generated site. The Toggle right sidebar button SHALL be hidden when JavaScript is disabled.

#### Scenario: Header on an ordinary page
- **WHEN** a reader at 1280px opens the `tmux` page
- **THEN** the header shows Toggle left sidebar, Home, Graph, More and Toggle right sidebar in that order, each reachable by keyboard with an accessible name

#### Scenario: Header on the home page
- **WHEN** a reader opens `index.html`
- **THEN** the header shows Toggle left sidebar, Graph, More and Toggle right sidebar, and Home is absent

#### Scenario: No missing destinations
- **WHEN** a site is generated before search exists
- **THEN** no header control links to a file missing from the output, and Graph links to `graph.html#/graph`

### Requirement: Show the left sidebar with All pages and favorites

Every generated page SHALL include a left sidebar listing Journals when a journals feed is written, then Graph view, then All pages, then a FAVORITES heading followed by one link per resolved favorite in config order, labelled with the author's spelling of the title or alias the favorite names. Journals SHALL link to the feed's first batch and SHALL be marked current on every feed batch. Graph view SHALL link to `graph.html`, carry the `hierarchy` icon, the `g g` hint when that shortcut is on, and the class `graph-view-nav` on its item, as Logseq's does. All pages SHALL link to the complete page index (`pages.html` when a homepage is selected or a journals feed holds `index.html`, `index.html` otherwise). The link for the current page SHALL carry `aria-current="page"` and a highlighted style. The FAVORITES heading SHALL be absent when no favorite resolves.

#### Scenario: Encode garden favorites
- **WHEN** the encode garden is published with all its namespaces selected
- **THEN** the sidebar lists the 19 favorites in `:favorites` order and each link opens the matching page

#### Scenario: Current favorite
- **WHEN** a reader opens a page that is a favorite
- **THEN** that page's sidebar link is marked current and highlighted

#### Scenario: Journals item
- **WHEN** the encode garden is published with journals and a reader opens `tmux`
- **THEN** Journals is the sidebar's first item, links to `index.html`, Graph view follows it and links to `graph.html`, and All pages links to `pages.html`

#### Scenario: Journals item on the feed
- **WHEN** a reader opens `journals-2.html`
- **THEN** the Journals item is marked current

#### Scenario: Garden stylesheet hides the graph entry points
- **WHEN** the encode garden is published with `--stylesheet logseq/custom.css`
- **THEN** the garden's rules hide the header Graph link and the Graph view item, as they do on Logseq's build, and `graph.html` still opens when typed

### Requirement: Handle Logseq's shell shortcuts

Generated pages SHALL handle `t l` (toggle left sidebar), `t r` (toggle right sidebar, at 640px and wider), `t t` (toggle theme), `t w` (toggle wide mode), `g h` (open Home), `g a` (open All pages), `g g` (open the graph view) unless `config.edn` sets `:shortcuts {:go/graph-view false}`, `alt+shift+c` (toggle the Contents card, at 640px and wider) and, when a journals feed is written, `g j` (open the feed's first batch), with two-key shortcuts recognized when the second key follows the first within one second. Shortcuts SHALL be ignored while focus is in an input, textarea, select or editable element, or while Control or Meta is held, and the two-key shortcuts also while Alt is held.

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
