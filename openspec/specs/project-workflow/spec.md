## Purpose

Define reproducible development tasks and generated agent instructions.

## Requirements

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

### Requirement: Post-merge OpenSpec cleanup

Contributors SHALL create or reuse a GitHub cleanup sub-issue under each implementation issue, using GitHub's parent-child relationship. The child SHALL identify the implementation PR and exact OpenSpec change, require the implementation PR to be merged before cleanup, and remain open until its separate cleanup PR merges.

#### Scenario: Prepare implementation review

- **WHEN** a contributor submits an implementation PR for an OpenSpec change
- **THEN** its implementation issue has a cleanup child linked to the PR and change, with acceptance criteria for verification, synchronization, archival, validation, and a follow-up PR

#### Scenario: Clean up a merged implementation

- **WHEN** the implementation PR is merged and its change is complete
- **THEN** a contributor verifies the implementation from current main in a separate worktree, syncs every delta capability inline, verifies the resulting requirements before archiving, validates the result, and submits a separate follow-up PR against main

#### Scenario: Discover eligible cleanup

- **WHEN** a contributor or monitor checks for post-merge work
- **THEN** it compares open cleanup children and active OpenSpec changes with live implementation PR merge state, reuses existing cleanup work, and leaves unrelated open implementations active

#### Scenario: Main changes during cleanup

- **WHEN** main advances or a later merged change overlaps a requirement
- **THEN** the contributor preserves later requirements, reconciles the deltas in implementation merge order, repeats synchronization verification after updating the cleanup branch, and stops archival for unresolved contradictions

#### Scenario: Cleanup already exists

- **WHEN** a cleanup PR or archive already exists for the exact change
- **THEN** the contributor reuses the PR or verifies the completed archive and resulting specs before closing the cleanup child, without creating another archive
