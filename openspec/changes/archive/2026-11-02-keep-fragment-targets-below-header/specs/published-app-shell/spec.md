## MODIFIED Requirements

### Requirement: Show Logseq's header on every page

Every generated page SHALL begin with a sticky header 48px tall holding, left to right, a Toggle left sidebar button, then Home, Graph and More at the right edge, followed at viewport widths of 640px and more by a Toggle right sidebar button with the `layout-sidebar-right` icon and `aria-expanded`. Home SHALL link to `index.html` and SHALL be absent from `index.html`. Graph SHALL be an `a.button` linking to `graph.html#/graph`, with the `hierarchy` icon and the accessible name Graph, inside a header carrying the class `cp__header`, so Logseq's selector `.cp__header a.button[href$="#/graph"]` matches it and a stylesheet written for Logseq's header can hide it. Each control SHALL have an accessible name and visible keyboard focus. The header SHALL contain only controls whose destination exists in the generated site. The Toggle right sidebar button SHALL be hidden when JavaScript is disabled. Every generated page SHALL set `scroll-padding-top` on `html` to 56px, so a fragment target that the browser or `scrollIntoView()` scrolls into view lands with its first line below the header at every viewport width.

#### Scenario: Header on an ordinary page
- **WHEN** a reader at 1280px opens the `tmux` page
- **THEN** the header shows Toggle left sidebar, Home, Graph, More and Toggle right sidebar in that order, each reachable by keyboard with an accessible name

#### Scenario: Header on the home page
- **WHEN** a reader opens `index.html`
- **THEN** the header shows Toggle left sidebar, Graph, More and Toggle right sidebar, and Home is absent

#### Scenario: No missing destinations
- **WHEN** a site is generated before search exists
- **THEN** no header control links to a file missing from the output, and Graph links to `graph.html#/graph`

#### Scenario: Fragment target below the header
- **WHEN** a reader at 1280px or 390px opens a page at a `#source-block-…` address far down the page, follows a link to another block of the same page, or chooses a note result in search
- **THEN** the block's first line is below the header's bottom edge
