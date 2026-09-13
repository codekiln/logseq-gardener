## Purpose

Define the public offline help, version, output, and process contract for lsg.

## Requirements

### Requirement: Equivalent offline help

Every advertised command path SHALL support equivalent help, --help, and -h output, describing purpose, behavior, usage, immediate children, side effects, permissions, and an explained example.

#### Scenario: Discover all commands

- **WHEN** a caller recursively visits children returned by lsg help --format json
- **THEN** each discovered path returns matching help aliases offline without reading stdin

### Requirement: Navigable sections

Help SHALL provide outline and section operations in text and JSON, stable section identifiers, level and max-level filtering, recursive descendants, and programmatic guidance.

#### Scenario: Retrieve headings

- **WHEN** a caller retrieves every section returned by an outline, with and without --recursive
- **THEN** the response contains the matching heading and body, and recursive retrieval includes descendants in document order

### Requirement: Versioned output and exits

Successful JSON SHALL be one document with format_version 1. Success SHALL return 0, invalid invocations 2 with empty stdout and a useful stderr diagnostic, and output failure 1; broken pipes SHALL return 0. Redirected output SHALL contain no terminal control sequences.

#### Scenario: Reject invalid input

- **WHEN** a caller supplies an unknown option or nonexistent section with --format json
- **THEN** lsg returns 2, keeps stdout empty, and writes an actionable diagnostic to stderr

### Requirement: Version reporting

lsg SHALL report its Cargo version through version, --version, and -V and support version --format json.

#### Scenario: Identify installation

- **WHEN** a caller runs lsg version --format json
- **THEN** the response includes format_version 1, command_path [version], program lsg, and the package version
