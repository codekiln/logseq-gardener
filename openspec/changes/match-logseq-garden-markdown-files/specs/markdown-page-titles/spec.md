## MODIFIED Requirements

### Requirement: Return explicit errors for unsupported inputs

The SDK SHALL return an error carrying the relative path for journal documents, non-normal or non-relative page paths, paths beneath `journals/`, non-Markdown paths, non-UTF-8 stems, and empty or whitespace-only resulting titles. A page path in any other folder of the garden, or at its root, SHALL take its title from the file name as a path beneath `pages/` does. Title extraction SHALL leave source and syntax unchanged and SHALL perform no filesystem writes.

#### Scenario: Journal requires date configuration
- **WHEN** a journal document is supplied
- **THEN** extraction returns a journal-specific unsupported-input error

#### Scenario: Page outside pages/
- **WHEN** a page document has the path `mise-tasks/microfreak/PARAMETERS.md`
- **THEN** extraction returns `PARAMETERS`

#### Scenario: Empty namespace title is unusable
- **WHEN** TripleLowbar receives `pages/______.md`
- **THEN** extraction returns an empty-title error

#### Scenario: Loaded documents remain unchanged
- **WHEN** page titles are extracted successfully or with errors
- **THEN** the document source, syntax, and relative path retain their original values
