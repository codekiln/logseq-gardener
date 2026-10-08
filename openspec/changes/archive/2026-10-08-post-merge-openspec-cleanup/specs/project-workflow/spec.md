## ADDED Requirements

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
