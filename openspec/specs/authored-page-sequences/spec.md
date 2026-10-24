## Purpose

Define explicit page sequence links that resolve only to published source pages.

## Requirements

### Requirement: Navigate explicit page sequences

The publisher SHALL read only leading page-level `prev` and `next` properties containing one direct page reference. Property keys SHALL be case-insensitive and the last occurrence SHALL supply the value. Each target SHALL resolve uniquely through selected visible source titles and aliases to its canonical source route and title. The publisher SHALL omit malformed, unavailable, ambiguous, and self targets. Nested properties, code, generated namespace destinations, and source chronology SHALL supply no sequence links.

#### Scenario: Authored neighbors and aliases
- **WHEN** a selected page names a visible stored neighbor by title or a unique alias in a leading prev or next property
- **THEN** navigation links to the neighbor's canonical route and displays its canonical title

#### Scenario: Unsafe or ambiguous neighbor
- **WHEN** a property names a private, excluded, missing, generated-only, ambiguous, or self target
- **THEN** that side has no navigation link and exposes no hidden source title or route

#### Scenario: Literal and duplicate properties
- **WHEN** nested properties or code contain prev/next text, or leading properties repeat a key
- **THEN** only the last leading value for that key can supply a neighbor, and only when it contains one direct page reference

### Requirement: Present accessible sequence navigation

Pages with valid sequence targets SHALL display a semantic Page sequence navigation region after source content and before Hierarchy. Links SHALL use Previous and Next labels with escaped canonical titles, the corresponding rel attribute, visible keyboard focus, and natural wrapping on narrow screens. Pages without valid targets SHALL omit the region. A selected homepage SHALL retain the same canonical neighbor links. Publication SHALL preserve source files, media, transclusions, routes, and page counts.

#### Scenario: One-sided sequence and homepage
- **WHEN** a homepage or sequence endpoint has one valid neighbor
- **THEN** only that neighbor appears with its canonical route and Previous or Next label

#### Scenario: Accessible long title
- **WHEN** a neighbor title contains markup characters or exceeds the narrow viewport width
- **THEN** the title is escaped, the link wraps and remains keyboard reachable, and navigation precedes Hierarchy without changing source media
