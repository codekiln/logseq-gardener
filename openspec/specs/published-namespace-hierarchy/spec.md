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

### Requirement: Space hierarchy path separators

The default stylesheet SHALL give hierarchy slash separators a logical horizontal margin of 0.2em on either side. Separators SHALL remain unlinked text, and paths SHALL retain natural wrapping.

#### Scenario: Read namespace segments
- **WHEN** a generated page displays the hierarchy path `GitP/A/Session/26/10/22-Thu`
- **THEN** each slash has visible space on both sides while the existing segment links remain intact

#### Scenario: Wrap a long path
- **WHEN** a hierarchy path exceeds the available width on a narrow screen
- **THEN** the path continues to wrap without a forced no-wrap separator container

### Requirement: Distinguish namespace destinations

The publisher SHALL visibly label generated namespace pages and their entries in the complete page index and Hierarchy as Namespace. Links to generated namespace destinations SHALL have a distinction beyond color and an accessible Namespace name. Selected stored source pages SHALL retain ordinary page labels even when empty. Classification SHALL use only selected visible source routes: missing and ordinarily unselected source ancestors SHALL have the same Namespace presentation. Withheld and alias-conflicting ancestors SHALL retain their existing unavailable behavior. Full logical anchor text in the page index SHALL remain unchanged, with the status label outside the anchor. Source-only name, alias, embed, homepage, route, and count contracts SHALL remain intact.

#### Scenario: Generated and stored destinations
- **WHEN** a selected stored page needs a generated ancestor, and another selected stored page has an empty body
- **THEN** only the generated ancestor has the Namespace page/index/hierarchy status and distinct link style

#### Scenario: Unselected ancestor privacy
- **WHEN** one generated ancestor has an ordinary unselected source file and another has no source file
- **THEN** both have identical Namespace status without source file paths, content, or existence claims

#### Scenario: Safe index labels and private gaps
- **WHEN** the index and hierarchy include generated destinations while a private or alias-conflicting prefix is unavailable
- **THEN** generated index anchors keep their exact full logical names, generated links identify Namespace status, and unavailable prefixes receive no new route
