## MODIFIED Requirements

### Requirement: Render selected page and block embeds

The publisher SHALL render page embed macros using uniquely resolved selected visible titles or aliases. It SHALL render block embed macros using unique valid explicit outline UUIDs and include the target outline's descendants. Expanded content SHALL retain the original source document for asset resolution and diagnostics: a diagnostic from embedded content SHALL be reported once, under the embedded file, and SHALL be left out when that file's own page reports the same diagnostic. Embedded content SHALL follow the existing escaping and supported-syntax rules.

#### Scenario: Embed a selected alias
- **WHEN** a selected page embeds a unique alias of another selected visible page
- **THEN** the target page's content appears inline with its supported media and links

#### Scenario: Embed an outline subtree
- **WHEN** a selected page embeds a valid UUID belonging to a unique selected visible outline item with descendants
- **THEN** the item and its descendants appear inline in source order

#### Scenario: Resolve embedded assets from their source
- **WHEN** embedded selected content references a supported local asset relative to its own source document
- **THEN** the asset is copied and rendered using that source location

#### Scenario: Embedded content with a missing asset
- **WHEN** `Notes/Start` embeds `Notes/Nested/Other`, whose file `pages/nested/Other.md` links a missing image
- **THEN** the report has one diagnostic for the image, with source `pages/nested/Other.md` and no file name in its message
