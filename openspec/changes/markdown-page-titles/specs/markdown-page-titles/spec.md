## ADDED Requirements

### Requirement: Derive page titles with an explicit filename format

The SDK SHALL derive a title from a loaded Markdown page document using a caller-supplied Legacy or TripleLowbar format. It SHALL use only the final filename stem and preserve capitalization. Legacy SHALL replace dots with slashes and atomically decode valid percent-encoded UTF-8, preserving the replaced string if decoding fails. TripleLowbar SHALL replace non-overlapping triple underscores with slashes, decode each ASCII percent escape once, preserve malformed and non-ASCII escapes, and remove empty slash segments.

#### Scenario: Configuration changes namespace decoding
- **WHEN** the document path is `pages/Names___Nested.md`
- **THEN** Legacy yields `Names___Nested` and TripleLowbar yields `Names/Nested`

#### Scenario: Percent decoding follows the format
- **WHEN** the stem is `%C3%A9.%2F%252F`
- **THEN** Legacy yields `é//%2F` and TripleLowbar yields `%C3%A9./%2F`

#### Scenario: Malformed escape preserves legacy input
- **WHEN** the stem is `Bad%2X.%2F`
- **THEN** Legacy yields `Bad%2X/%2F` and TripleLowbar yields `Bad%2X.`

### Requirement: Respect leading page-title overrides and Contents

The SDK SHALL use the last case-insensitively named title property in the first root parsed Properties block when present. Later properties and bullet properties SHALL retain the filename title. A normal page path whose relative spelling starts with `pages/contents.` SHALL yield `Contents` before considering properties.

#### Scenario: Title overrides the filename
- **WHEN** `pages/Title Override.md` has leading `title:: Chosen Name`
- **THEN** both formats yield `Chosen Name`

#### Scenario: Block title retains the filename
- **WHEN** `pages/Example.md` has a bullet containing a title property
- **THEN** the derived title is `Example`

#### Scenario: Contents overrides its title property
- **WHEN** `pages/contents.extra.md` has leading `title:: Different`
- **THEN** the derived title is `Contents`

### Requirement: Return explicit errors for unsupported inputs

The SDK SHALL return an error carrying the relative path for journal documents, non-normal or non-relative page paths, paths outside `pages/`, non-Markdown paths, non-UTF-8 stems, and empty or whitespace-only resulting titles. Title extraction SHALL leave source and syntax unchanged and SHALL perform no filesystem writes.

#### Scenario: Journal requires date configuration
- **WHEN** a journal document is supplied
- **THEN** extraction returns a journal-specific unsupported-input error

#### Scenario: Empty namespace title is unusable
- **WHEN** TripleLowbar receives `pages/______.md`
- **THEN** extraction returns an empty-title error

#### Scenario: Loaded documents remain unchanged
- **WHEN** page titles are extracted successfully or with errors
- **THEN** the document source, syntax, and relative path retain their original values
