## ADDED Requirements

### Requirement: Name a page without its boundary slashes

A page name that a block writes SHALL lose one slash at its start and one at its end, as Logseq does, before the SDK titles the page or finds its namespace parents. The page SHALL take the shortened name as its title, and its namespace parent SHALL be the text before the last remaining slash.

#### Scenario: Trailing slash

- **WHEN** a published page links `[[a/b/]]`, and neither `a/b` nor `a` has a file
- **THEN** `a/b` is a reference-only node titled `a/b`, it has an edge to its namespace parent `a`, and no node is titled `a/b/`

#### Scenario: Leading slash

- **WHEN** a published page links `[[/D♭)]]`, and `D♭)` has no file
- **THEN** `D♭)` is a reference-only node, and no node is titled `/D♭)`
