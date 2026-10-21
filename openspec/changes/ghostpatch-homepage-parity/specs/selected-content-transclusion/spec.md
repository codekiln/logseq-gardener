## ADDED Requirements

### Requirement: Render selected page and block embeds

The publisher SHALL render page embed macros using uniquely resolved selected visible titles or aliases. It SHALL render block embed macros using unique valid explicit outline UUIDs and include the target outline's descendants. Expanded content SHALL retain the original source document for asset resolution and diagnostics. Embedded content SHALL follow the existing escaping and supported-syntax rules.

#### Scenario: Embed a selected alias
- **WHEN** a selected page embeds a unique alias of another selected visible page
- **THEN** the target page's content appears inline with its supported media and links

#### Scenario: Embed an outline subtree
- **WHEN** a selected page embeds a valid UUID belonging to a unique selected visible outline item with descendants
- **THEN** the item and its descendants appear inline in source order

#### Scenario: Resolve embedded assets from their source
- **WHEN** embedded selected content references a supported local asset relative to its own source document
- **THEN** the asset is copied and rendered using that source location

### Requirement: Preserve publication visibility during expansion

Transclusion lookup SHALL contain only selected visible pages and their eligible outline targets. Missing, invalid, ambiguous, excluded, or private targets SHALL retain an escaped visible source fallback and a local diagnostic. Expansion SHALL introduce no withheld target text, asset, URL, or destination into generated output.

#### Scenario: Withheld target sentinel
- **WHEN** a selected page embeds an excluded or private page or block containing unique text and media sentinels
- **THEN** every sentinel is absent from generated output and the source embed receives a fallback and diagnostic

#### Scenario: Ambiguous embed
- **WHEN** a page alias or normalized outline UUID has multiple selected visible owners
- **THEN** the embed remains a visible fallback with a diagnostic and exposes no candidate content

### Requirement: Bound expansion and preserve document identity

The publisher SHALL bound transclusion depth and total expansion work per output document. Recursive or budget-exhausting expansion SHALL produce a visible fallback and local diagnostic. Repeated embeds SHALL produce valid HTML without duplicate IDs. Ordinary block references SHALL retain links to the original page and anchor.

#### Scenario: Recursive page embeds
- **WHEN** selected pages embed each other directly or through block descendants
- **THEN** generation terminates with readable retained content and a fallback diagnostic at the recursion boundary

#### Scenario: Repeated block embeds
- **WHEN** a selected page embeds the same outline UUID more than once
- **THEN** each supported occurrence renders within the expansion budget, HTML IDs remain unique, and block navigation reaches the original target

#### Scenario: Expansion budget
- **WHEN** repeated nested embeds exceed the publisher's fixed work budget
- **THEN** generation terminates with a visible fallback and diagnostic
