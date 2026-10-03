## ADDED Requirements

### Requirement: Separate SDK and CLI packages

The project SHALL provide a reusable logseq-gardener-sdk library package and a logseq-gardener CLI package that consumes it and builds the lsg executable.

#### Scenario: Inspect build targets

- **WHEN** a contributor runs cargo metadata and the release build
- **THEN** the metadata identifies the SDK library and the CLI dependency on it, and the build produces lsg

#### Scenario: Use the SDK from Rust

- **WHEN** a Rust application depends on logseq-gardener-sdk without the CLI package
- **THEN** it can compile against the SDK and read its version without pulling in command-line parsing dependencies

### Requirement: Pinned shared workflow

The repository SHALL commit mise.toml, mise.lock, and Cargo.lock, with executable described file tasks using USAGE metadata. Local and CI checks SHALL run the same aggregate task for formatting, warnings-denied Clippy, tests, locked builds, strict OpenSpec validation, RuleSync checks, CLI conformance, and release checks.

#### Scenario: Reproduce CI

- **WHEN** a contributor runs mise install and mise run ci
- **THEN** the pinned tools run every required check and any failed check makes the aggregate fail

### Requirement: Generated agent instructions

The repository SHALL keep agent instructions and OpenSpec skills in RuleSync source, require skill loading and CLI instructions before artifact edits, and validate the generated output and skill versions.

#### Scenario: Stale skill

- **WHEN** the managed OpenSpec version differs from a skill generatedBy value
- **THEN** the skill version check fails and identifies the mismatch
