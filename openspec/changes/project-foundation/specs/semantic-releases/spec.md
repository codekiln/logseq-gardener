## ADDED Requirements

### Requirement: Conventional Emoji titles

Pull request checks SHALL validate emoji/type pairing and support feature, fix, breaking, and non-release titles.

#### Scenario: Invalid title

- **WHEN** a pull request title omits its emoji or pairs an emoji with the wrong type
- **THEN** the title check fails with a correction message

### Requirement: Release preparation

release-plz SHALL prepare version and changelog changes and tags in Git-only mode with tested pre-1.0 and post-1.0 release impact.

#### Scenario: Version impact

- **WHEN** release tests apply feature, fix, breaking, and documentation commits to fixture repositories
- **THEN** the resulting versions match the documented release policy

### Requirement: Checksummed distribution

A tag-driven cargo-dist workflow SHALL run repository checks before publishing configured native archives and SHA-256 checksums.

#### Scenario: Validate workflow

- **WHEN** a contributor runs the distribution check
- **THEN** cargo-dist validates the generated workflow and plans the configured native archives and checksums
