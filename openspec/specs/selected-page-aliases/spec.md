## Purpose

Define reusable page alias lookup within publication visibility.

## Requirements

### Requirement: Extract leading page aliases

The SDK SHALL extract aliases from the last case-insensitive `alias` property in the first parsed properties block. It SHALL combine comma-separated plain names and complete direct page references, splitting plain text on ASCII or full-width commas. Quoted values SHALL remain one literal name including quotes. Code and formatted spans SHALL not contribute their inner text as names; a nonempty value with no parsed names SHALL remain a complete literal name. Nested properties and `aliases` SHALL not provide page aliases. Blank and self aliases SHALL be omitted.

#### Scenario: Mixed alias syntax
- **WHEN** a leading alias value contains `Plain, [[Comma, Name]]，Other` and a later outline contains another alias property
- **THEN** the SDK returns `Plain`, `Comma, Name` as one reference name, and `Other`, without the nested alias

#### Scenario: Literal and repeated properties
- **WHEN** a page contains duplicate alias properties and the last value is a quoted reference-like string
- **THEN** the last value remains one literal alias including its quotes and contributes no inner reference

### Requirement: Preserve page name candidates

The SDK SHALL provide reusable deterministic candidate lookup for caller-supplied page identifiers. Titles, aliases and lookup queries SHALL share trimmed lowercase NFC keys with one leading and trailing slash removed. Repeated claims from one page SHALL be deduplicated. Claims from different pages, including title and alias collisions, SHALL remain distinguishable and ambiguous.

#### Scenario: Duplicate alias claims
- **WHEN** Alias Source and Chosen Name both claim Other Name
- **THEN** lookup retains both page identifiers and reports an ambiguous result

#### Scenario: Unicode and title collision
- **WHEN** a title and another page's alias have equivalent normalized spelling
- **THEN** lookup retains both candidates for either spelling

### Requirement: Resolve within publication visibility

Publishing SHALL populate lookup only from selected pages that pass visibility checks. A unique visible alias SHALL link to the page's existing local route and retain the source label. Ambiguous references SHALL retain a visible fallback and local diagnostic without listing candidate names. Unavailable destinations SHALL retain the existing unavailable fallback. Namespace selection SHALL continue to use logical page titles.

#### Scenario: Hidden claims cannot expose destinations
- **WHEN** an included page and excluded or private pages claim the same alias
- **THEN** lookup links to the included page and generated files contain no withheld page name, content, URL or asset introduced by alias lookup

#### Scenario: Aliases cannot select pages
- **WHEN** an excluded page declares an alias under an included namespace
- **THEN** the page remains excluded and the alias stays unavailable
