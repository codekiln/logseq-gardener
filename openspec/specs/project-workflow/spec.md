## Purpose

Define reproducible development tasks and generated agent instructions.

## Requirements

### Requirement: One Rust executable

The project SHALL build one binary crate named logseq-gardener with executable lsg and internal modules.

#### Scenario: Inspect build targets

- **WHEN** a contributor runs cargo metadata and the release build
- **THEN** the metadata identifies one binary target named lsg and the build produces that executable

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
