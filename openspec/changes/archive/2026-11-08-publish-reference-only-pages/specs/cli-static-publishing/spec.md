## MODIFIED Requirements

### Requirement: Publication reports and diagnostics

Successful human and JSON output SHALL report pages, published journals, reference-only pages, assets, withheld pages, skipped journals, diagnostic count, and an index location. JSON SHALL contain format_version 1 and command_path [publish]. Unsupported selected constructs SHALL produce local stderr diagnostics and success. Fatal SDK failures SHALL return one with empty stdout and an actionable stderr error; invalid selectors SHALL return two.

#### Scenario: Unsupported content

- **WHEN** generation completes with unresolved references
- **THEN** the success report remains on stdout, diagnostics stay on stderr, and the exit is zero

#### Scenario: Fatal generation errors

- **WHEN** the source is missing, output exists or lies inside the garden, or selected titles are ambiguous
- **THEN** publication returns one with empty stdout and explains the SDK failure on stderr

#### Scenario: Journal counts
- **WHEN** a caller publishes a garden with three journals using --journals, one of them private
- **THEN** the report shows two published journals and one withheld page

#### Scenario: Reference-only page count
- **WHEN** a caller publishes a garden whose published pages link two names that no file holds
- **THEN** the human report shows two reference-only pages, the JSON report holds `reference_only_pages` 2, and the page count is unchanged
