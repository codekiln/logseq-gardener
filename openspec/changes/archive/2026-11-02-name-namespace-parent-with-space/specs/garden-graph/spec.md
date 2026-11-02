## MODIFIED Requirements

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
