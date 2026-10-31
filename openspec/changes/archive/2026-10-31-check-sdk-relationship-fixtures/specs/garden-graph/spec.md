## ADDED Requirements

### Requirement: Report a page's namespace parent

The graph index SHALL return, for a node title matched without regard to case, the title of the node's namespace parent: the name before the title's last `/`, whether or not either page has a file. It SHALL return nothing for a title without `/` or a title the graph does not hold.

#### Scenario: Parent of a page without a file

- **WHEN** a page links `[[Names/Nested]]` and no file holds either `Names/Nested` or `Names`
- **THEN** the namespace parent of `names/nested` is `Names`

#### Scenario: Top-level page

- **WHEN** the graph holds `Names`
- **THEN** `Names` has no namespace parent
