## MODIFIED Requirements

### Requirement: Navigate ancestors and children

Every published source and generated namespace page SHALL place its Hierarchy view after page content. The publisher SHALL remove top namespace breadcrumb navigation. The Hierarchy view SHALL contain a flat list whose first row is the current complete logical path, followed by every navigable descendant destination at all depths in deterministic normalized logical-title order. Descendant matching SHALL respect slash-separated namespace boundaries. Each row SHALL show its full path with slash separators, and every segment with an available destination SHALL link independently to its cumulative prefix, including the current page's final segment. Unavailable private, withheld, or alias-conflicting prefix destinations SHALL remain plain segment text without recreating routes. The whole Hierarchy section SHALL be foldable and expanded by default; all rows SHALL be visible when expanded. Labels and attributes SHALL be escaped. Semantic disclosure controls and links SHALL support keyboard operation, visible focus, and wrapping of long paths on narrow screens.

#### Scenario: Source page with ancestors and children
- **WHEN** A/B/C, A/B/C/D, A/B/C/E, and A/B/C/E/F are navigable destinations
- **THEN** A/B/C displays those four full-path rows at the bottom, including E/F, with separate links for every available cumulative prefix segment and no top breadcrumb

#### Scenario: Missing ancestor destination creates a gap
- **WHEN** Root and Root/Private/PublicChild are visible but Root/Private is withheld
- **THEN** Root's bottom list includes Root/Private/PublicChild, Private remains plain text in its row, and no navigation points to a standalone Root/Private destination

#### Scenario: Namespaced leaf retains ancestor links
- **WHEN** A/B/C has no descendants
- **THEN** its bottom Hierarchy includes the current A/B/C row with available A, A/B, and A/B/C segment links

#### Scenario: Similar sibling namespace
- **WHEN** A/B/C and A/B/Cousin are published
- **THEN** Cousin is absent from C's descendant rows

#### Scenario: Expanded disclosure
- **WHEN** a page with multiple descendant levels is opened
- **THEN** every path row is initially visible and the Hierarchy control can collapse and expand the whole list by keyboard

#### Scenario: Long mobile path
- **WHEN** a deep namespace with long segments is viewed at a narrow mobile width
- **THEN** complete path rows wrap within the available width and every available segment link remains keyboard reachable with visible focus
