## Purpose

Give the SDK, the CLI, the language server and the publisher one source for a garden's graph: Logseq's nodes and edges under given settings, and one page's neighbourhood.

## Requirements

### Requirement: Return the garden graph with Logseq's rules

The SDK SHALL return a graph of the garden's pages for given settings (journals, orphan pages, built-in pages, excluded pages). Each node SHALL carry its title, its route when the page is published, its degree, and flags for journal, tag, namespace parent, built-in, excluded, orphan and reference-only. Edges SHALL be distinct unordered page pairs from three relations: a reference from a block on one page to another page, a page's `tags::`, and a namespaced page's parent. The operation SHALL be available to the CLI, the language server and the publisher, SHALL derive from the same reference index that answers linked references, and SHALL leave garden files unchanged.

#### Scenario: Three relations

- **WHEN** a block on `Foam` links `Obsidian`, `Foam` has `tags:: Tool`, and `Foam/Plugin` exists
- **THEN** the graph holds the edges `Foam`–`Obsidian`, `Foam`–`Tool` and `Foam/Plugin`–`Foam`

#### Scenario: Distinct edges

- **WHEN** `mise` both links `tmux` in a block and lists `tmux` in its `tags::`
- **THEN** the graph holds one `mise`–`tmux` edge and counts two links toward `N links`

#### Scenario: Read-only

- **WHEN** the graph is computed for a garden
- **THEN** no garden file changes

### Requirement: Drop nested namespace edges

When a block on page A refers to a namespaced page, the SDK SHALL leave out the edge from A to each namespace ancestor that the reference index added for that reference, keeping an ancestor edge only when A links the ancestor itself.

#### Scenario: Child link only

- **WHEN** a block on `tmux` links only `tmux/Command/new-session`
- **THEN** the graph holds `tmux`–`tmux/Command/new-session` and no edge from `tmux` to `tmux/Command`

#### Scenario: Ancestor linked too

- **WHEN** another block on `tmux` links `tmux/Command` directly
- **THEN** the graph also holds `tmux`–`tmux/Command`

### Requirement: Count property names as references

The graph SHALL treat each property name in a block or page's properties as a reference to the page of that name, as Logseq does for file graphs. Other SDK answers, such as linked references, SHALL stay as they are.

#### Scenario: Navigation properties

- **WHEN** `2026/t4/w43` has the properties `prev::` and `next::`
- **THEN** the graph holds edges from `2026/t4/w43` to the reference-only pages `prev` and `next`

### Requirement: Filter nodes by Logseq's settings

The SDK SHALL leave out journal pages and the references they make unless journals are on, Logseq's built-in pages (`NOW`, `LATER`, `TODO`, `DOING`, `DONE`, `WAIT`, `WAITING`, `CANCELED`, `CANCELLED`, `STARTED`, `IN-PROGRESS`, `A`, `B`, `C`, `Favorites`, `Contents`, `card`) unless built-in pages are on, pages with `exclude-from-graph-view:: true` unless excluded pages are on, pages with no edge unless orphan pages are on, and pages whose names look like UUIDs or asset paths always. Edges with an end outside the node set SHALL be dropped. Pages that are private or outside the selected namespaces SHALL never be nodes, and references made only from such pages SHALL create no node or edge.

#### Scenario: Defaults

- **WHEN** the graph is computed with journals, orphan pages, built-in pages and excluded pages off
- **THEN** it holds no journal page, no built-in page, no excluded page and no page without an edge

#### Scenario: Excluded page

- **WHEN** a fixture page sets `exclude-from-graph-view:: true` and excluded pages are off
- **THEN** that page is absent and so are its edges

#### Scenario: Private references

- **WHEN** the only page that links `Secret Project` is private
- **THEN** `Secret Project` is absent from the graph

### Requirement: Include reference-only pages as nodes

A page that exists only because a published page refers to it SHALL be a node flagged reference-only, without a route. A name a block writes SHALL bring in its namespace parents as reference-only nodes whether or not the name itself opens a page, including when the name is an alias of a page. A reference-only node that names one of the pages Logseq creates in every graph (`NOW`, `LATER`, `DOING`, `DONE`, `CANCELED`, `CANCELLED`, `IN-PROGRESS`, `TODO`, `WAIT`, `WAITING`, `A`, `B`, `C`, `Favorites`, `Contents`, `card`) SHALL take that spelling as its title, whatever the case of the references that name it.

#### Scenario: Reference-only node

- **WHEN** a published page links `Contrando`, which has no file
- **THEN** `Contrando` is a node flagged reference-only with no route

#### Scenario: Built-in page spelled as in Logseq

- **WHEN** a published page writes `#c` and a later page writes `TODO [#C] write`, and neither `C` nor `TODO` has a file
- **THEN** the reference-only nodes are titled `C` and `TODO`

#### Scenario: Namespace parents of an alias

- **WHEN** `pages/AGI.md` holds `alias:: [[AI/General/Intelligence]]`, and neither `AI/General` nor `AI` has a file
- **THEN** `AI/General` and `AI` are reference-only nodes, `AI/General` has an edge to `AI`, and the page graph of a page that links `AI/General/Intelligence` holds both

### Requirement: Return a page's neighbourhood

The SDK SHALL return a page's page graph as Logseq builds it: the page, the pages it refers to through itself and its aliases, the pages that refer to it, and its tags, with edges from the page to each page it refers to and each tag, from each referring page to the page, between any two of the other nodes, and the namespace edges among them. Referring journal pages SHALL be included only when journals are on; a journal page's own neighbourhood SHALL be the same either way.

#### Scenario: Small neighbourhood

- **WHEN** the neighbourhood of `Foam` is computed on the encode garden with journals off
- **THEN** it holds the nodes and edges Logseq's Page graph shows for `Foam`, minus pages `lsg` does not publish and their references

#### Scenario: Journals toggle

- **WHEN** a journal page refers to `tmux` and the neighbourhood of `tmux` is computed with journals on
- **THEN** that journal page is a node with an edge to `tmux`, and with journals off it is absent

### Requirement: Update the graph with the index

When one file changes, the SDK SHALL update the graph and the affected neighbourhoods from the reference index's replaced contribution, without rebuilding the index from every file.

#### Scenario: One file changes

- **WHEN** a block linking `Obsidian` is removed from `Foam` and the index replaces `Foam`'s contribution
- **THEN** the graph no longer holds `Foam`–`Obsidian` unless another block on `Foam` still links `Obsidian`

### Requirement: Report a page's namespace parent

The graph index SHALL return, for a node title matched without regard to case, the title of the node's namespace parent: the name before the title's last `/`, whether or not either page has a file. A space before that slash SHALL stay in the parent's title, and that parent SHALL be a node of its own, apart from the page without the space. It SHALL return nothing for a title without `/` or a title the graph does not hold.

#### Scenario: Parent of a page without a file

- **WHEN** a page links `[[Names/Nested]]` and no file holds either `Names/Nested` or `Names`
- **THEN** the namespace parent of `names/nested` is `Names`

#### Scenario: Top-level page

- **WHEN** the graph holds `Names`
- **THEN** `Names` has no namespace parent

#### Scenario: Space before the slash

- **WHEN** a page links `[[MacOS /Applications Folder]]` and a file holds `MacOS`
- **THEN** the namespace parent of `MacOS /Applications Folder` is `MacOS `, a reference-only node, and the graph has no edge from either page to `MacOS`

### Requirement: Name a page without its boundary slashes

A page name that a block writes SHALL lose one slash at its start and one at its end, as Logseq does, before the SDK titles the page or finds its namespace parents. The page SHALL take the shortened name as its title, and its namespace parent SHALL be the text before the last remaining slash.

#### Scenario: Trailing slash

- **WHEN** a published page links `[[a/b/]]`, and neither `a/b` nor `a` has a file
- **THEN** `a/b` is a reference-only node titled `a/b`, it has an edge to its namespace parent `a`, and no node is titled `a/b/`

#### Scenario: Leading slash

- **WHEN** a published page links `[[/D♭)]]`, and `D♭)` has no file
- **THEN** `D♭)` is a reference-only node, and no node is titled `/D♭)`

### Requirement: Title a page that names a date with the date's own weekday

A page name that spells a date in the journal title format, or in `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd`, SHALL name the journal with that date, as Logseq does. When no page with a file answers to the name, the reference-only page SHALL take the title the journal title format gives the date, whatever weekday or spelling the block wrote, and the page SHALL count as a journal. The graph SHALL hold one node for every spelling of the date.

#### Scenario: Wrong weekday
- **WHEN** the title format is `yyyy-MM-dd EEE`, a published page links `[[1967-09-03 Fri]]`, and no file answers to that date
- **THEN** the graph holds a reference-only journal node titled `1967-09-03 Sun` and no node titled `1967-09-03 Fri`

#### Scenario: Other spelling of the date
- **WHEN** the title format is `yyyy-MM-dd EEE`, one page links `[[Sep 3rd, 1967]]`, another links `[[1967-09-03 Fri]]`, and no file answers to that date
- **THEN** both pages have an edge to the one node titled `1967-09-03 Sun`

### Requirement: Spell a repeated alias name with its last spelling

When one `alias::` value writes the same name more than once in different letter case, the SDK SHALL spell the alias with the last of those spellings, as Logseq does. The alias SHALL keep the position of its first occurrence.

#### Scenario: Alias written in two cases
- **WHEN** a page has `alias:: [[homebrew]], [[Homebrew]]`
- **THEN** the alias name reads `Homebrew`, and the page's alias list holds it once

#### Scenario: Position of a repeated alias
- **WHEN** a page has `alias:: [[b]], [[a]], [[B]]`
- **THEN** the aliases read `B`, `a` in that order
