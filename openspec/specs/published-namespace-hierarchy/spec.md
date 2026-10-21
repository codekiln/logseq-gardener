## Purpose

Define safe published namespace destinations, ancestor navigation, and child discovery.

## Requirements

### Requirement: Derive safe namespace destinations

The publisher SHALL derive namespace navigation only from slash-separated prefixes of selected visible original page titles. Selected source pages SHALL retain their existing routes and content. Missing or ordinarily unselected intermediate namespaces SHALL receive navigation-only destinations derived from visible prefixes, without importing unselected source content, aliases, properties, or assets. The publisher SHALL classify stored source-title visibility before selection and SHALL suppress generated destinations matching any private or visibility-withheld source title, including ancestors outside include roots. Aliases SHALL create no hierarchy nodes. A generated prefix colliding with a selected alias SHALL remain without a generated destination. Normalized title sharing and display spelling SHALL be deterministic; route or unresolved name collisions SHALL fail before output creation.

#### Scenario: Missing intermediate namespaces
- **WHEN** only Instruments/Synth/Preset is selected and visible
- **THEN** Instruments and Instruments/Synth receive navigation-only destinations linked to the visible descendant

#### Scenario: Unselected ordinary ancestor
- **WHEN** Instruments has an unselected source body and Instruments/Synth is selected and visible
- **THEN** the generated Instruments destination contains navigation derived from visible descendants and no source body, alias, property, or asset from the unselected document

#### Scenario: Private ancestor outside include roots
- **WHEN** Secret is private and only Secret/PublicChild is included and visible
- **THEN** Secret receives no generated route or linked standalone destination and none of its source content or assets appears in output

#### Scenario: Hidden-only branch
- **WHEN** excluded or private pages are the only source of a namespace prefix
- **THEN** that prefix and its descendants contribute no generated navigation title or destination

#### Scenario: Alias collision
- **WHEN** a selected source page declares an alias equal to a potential generated namespace prefix
- **THEN** the prefix receives no synthetic route and existing source alias, embed, and homepage resolution retains its behavior

### Requirement: Navigate ancestors and children

Every published source and generated namespace page SHALL provide labelled breadcrumb navigation with linked available ancestors and an identified current page. Unavailable ancestor destinations SHALL appear as plain prefix text. A page with navigable namespace children SHALL include a Hierarchy section with deterministic child links. Descendants separated by unavailable intermediate destinations SHALL attach to their nearest available ancestor using an unambiguous relative path label. Labels and attributes SHALL be escaped. Navigation SHALL use semantic keyboard-accessible links and wrap long names on narrow screens while retaining visible focus indicators.

#### Scenario: Source page with ancestors and children
- **WHEN** Instruments, Instruments/Synth, and Instruments/Synth/Preset are published
- **THEN** the preset links both ancestors, Synth links its Preset child in Hierarchy, and source content remains present

#### Scenario: Missing ancestor destination creates a gap
- **WHEN** Root and Root/Private/PublicChild are visible but Root/Private is withheld
- **THEN** Root links Private/PublicChild in Hierarchy, the descendant breadcrumb shows Private as plain text, and no navigation points to a standalone Root/Private destination

#### Scenario: Long mobile path
- **WHEN** a deep namespace with long segments is viewed at a narrow mobile width
- **THEN** breadcrumbs and child labels wrap within the available width and every navigation link remains keyboard reachable with visible focus

### Requirement: Preserve index and source publication contracts

The complete page index SHALL list each selected source page and generated namespace destination once with its full logical title and local route. SiteReport.pages SHALL continue counting selected source documents only. Generated namespaces SHALL remain separate from source page-name, alias, block, transclusion, and homepage lookup. Homepage output, source routes, media rendering, and transclusion behavior SHALL retain their existing contracts.

#### Scenario: Indexed synthetic ancestors
- **WHEN** one selected source page requires two generated namespace ancestors
- **THEN** the complete index lists all three destinations and the source page count remains one

#### Scenario: Generated namespace is selected as homepage or embed
- **WHEN** a caller requests a source-absent generated namespace as homepage or a source page embeds it
- **THEN** homepage validation fails or the embed keeps its existing unavailable fallback, respectively

#### Scenario: Retained rendering
- **WHEN** a selected homepage includes supported media and selected-content transclusions
- **THEN** hierarchy navigation is added while its existing rendered content, media destinations, block links, and transclusions retain their behavior
