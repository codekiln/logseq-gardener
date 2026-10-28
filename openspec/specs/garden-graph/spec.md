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

A page that exists only because a published page refers to it SHALL be a node flagged reference-only, without a route.

#### Scenario: Reference-only node

- **WHEN** a published page links `Contrando`, which has no file
- **THEN** `Contrando` is a node flagged reference-only with no route

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
