## ADDED Requirements

### Requirement: Space hierarchy path separators

The default stylesheet SHALL give hierarchy slash separators a logical horizontal margin of 0.2em on either side. Separators SHALL remain unlinked text, and paths SHALL retain natural wrapping.

#### Scenario: Read namespace segments
- **WHEN** a generated page displays the hierarchy path `GitP/A/Session/26/10/22-Thu`
- **THEN** each slash has visible space on both sides while the existing segment links remain intact

#### Scenario: Wrap a long path
- **WHEN** a hierarchy path exceeds the available width on a narrow screen
- **THEN** the path continues to wrap without a forced no-wrap separator container
