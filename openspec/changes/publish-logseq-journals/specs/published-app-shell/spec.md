## MODIFIED Requirements

### Requirement: Show the left sidebar with All pages and favorites

Every generated page SHALL include a left sidebar listing Journals when a journals feed is written, then All pages, then a FAVORITES heading followed by one link per resolved favorite in config order, labelled with the author's spelling of the title or alias the favorite names. Journals SHALL link to the feed's first batch and SHALL be marked current on every feed batch. All pages SHALL link to the complete page index (`pages.html` when a homepage is selected or a journals feed holds `index.html`, `index.html` otherwise). The link for the current page SHALL carry `aria-current="page"` and a highlighted style. The FAVORITES heading SHALL be absent when no favorite resolves.

#### Scenario: Encode garden favorites
- **WHEN** the encode garden is published with all its namespaces selected
- **THEN** the sidebar lists the 19 favorites in `:favorites` order and each link opens the matching page

#### Scenario: Current favorite
- **WHEN** a reader opens a page that is a favorite
- **THEN** that page's sidebar link is marked current and highlighted

#### Scenario: Journals item
- **WHEN** the encode garden is published with journals and a reader opens `tmux`
- **THEN** Journals is the sidebar's first item, links to `index.html`, and All pages links to `pages.html`

#### Scenario: Journals item on the feed
- **WHEN** a reader opens `journals-2.html`
- **THEN** the Journals item is marked current

### Requirement: Handle Logseq's shell shortcuts

Generated pages SHALL handle `t l` (toggle left sidebar), `t t` (toggle theme), `t w` (toggle wide mode), `g h` (open Home), `g a` (open All pages) and, when a journals feed is written, `g j` (open the feed's first batch) when the second key follows the first within one second. Shortcuts SHALL be ignored while focus is in an input, textarea, select or editable element, or while Control, Alt or Meta is held.

#### Scenario: Shortcut outside a field
- **WHEN** a reader on `tmux` presses `g` then `a`
- **THEN** the browser opens the complete page index

#### Scenario: Shortcut inside a field
- **WHEN** focus is in a backlink filter input and the reader types `t t`
- **THEN** the input receives the text and the appearance stays the same

#### Scenario: Open journals
- **WHEN** the encode garden is published with journals and a reader on `tmux` presses `g` then `j`
- **THEN** the browser opens `index.html` showing the feed
