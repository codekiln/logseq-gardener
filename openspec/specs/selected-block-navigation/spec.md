## Purpose

Define navigation to explicit outline IDs within published pages.

## Requirements

### Requirement: Navigate unique selected outline IDs

The publisher SHALL emit stable local anchors for valid explicit hyphenated UUID IDs attached to rendered Markdown outline items and SHALL link references to unique published targets. UUID case SHALL be normalized. Root page properties SHALL remain distinct from outline IDs.

#### Scenario: Reference a nested selected note
- **WHEN** a selected page references a valid ID on a nested outline item in another selected visible page
- **THEN** the generated link resolves to that item's page and anchor
- **AND** an unlabeled reference displays bounded escaped plain target text

#### Scenario: Preserve an explicit source label
- **WHEN** a reference to a published target has an explicit source label
- **THEN** that label is displayed without expanding another target or copying assets

### Requirement: Preserve unavailable reference fallbacks

The publisher SHALL preserve literal source references and emit selected-source diagnostics when the target is missing, invalid, excluded, private, unsupported, or ambiguous. The target index SHALL contain only rendered selected pages. Generated output SHALL omit excluded and private target text and assets.

#### Scenario: Excluded or private target
- **WHEN** a selected page references an ID owned only by an excluded or private page
- **THEN** the reference remains literal with a diagnostic
- **AND** the target's text, link destination, and assets are absent from generated output

#### Scenario: Duplicate published IDs
- **WHEN** published outline items share a normalized UUID
- **THEN** references to that UUID remain literal with a diagnostic
- **AND** the generated pages do not emit duplicate target anchors for that UUID

#### Scenario: Invalid outline ID or root page ID
- **WHEN** an ID is malformed or belongs to root page properties
- **THEN** it supplies no outline anchor or reference destination
