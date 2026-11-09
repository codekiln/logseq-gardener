## MODIFIED Requirements

### Requirement: Present accessible sequence navigation

Pages with valid sequence targets SHALL display a semantic Page sequence navigation region after source content and before Hierarchy. Links SHALL use Previous and Next labels with escaped canonical titles, the corresponding rel attribute, visible keyboard focus, and natural wrapping on narrow screens. Pages without valid targets SHALL omit the region. A selected homepage SHALL retain the same canonical neighbor links. Publication SHALL preserve source files, media, transclusions, routes, and page counts. The graph SHALL be able to disable the sequence region. Valid neighbors already displayed as visible prev/next property rows SHALL be omitted from the bottom sequence region to avoid duplicate links. A neighbor whose key, `prev` or `next`, is in the graph's `:block-hidden-properties` SHALL be omitted from the region as well. Remaining valid neighbors SHALL retain normal navigation.

#### Scenario: One-sided sequence and homepage
- **WHEN** a homepage or sequence endpoint has one valid neighbor
- **THEN** only that neighbor appears with its canonical route and Previous or Next label

#### Scenario: Accessible long title
- **WHEN** a neighbor title contains markup characters or exceeds the narrow viewport width
- **THEN** the title is escaped, the link wraps and remains keyboard reachable, and navigation precedes Hierarchy without changing source media

#### Scenario: Hidden neighbor key
- **WHEN** the graph sets `:block-hidden-properties #{:next}` and a page has a valid `next::` neighbor and no `prev::`
- **THEN** the page shows no `next` property row and no Page sequence region
