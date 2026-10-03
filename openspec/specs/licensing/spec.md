## Purpose

Preserve project licensing and third-party distribution notices.

## Requirements

### Requirement: AGPL distribution

The repository SHALL contain the complete AGPLv3 license and declare AGPL-3.0-only in package metadata. Distribution SHALL include license and third-party notices.

#### Scenario: Inspect distribution

- **WHEN** a contributor inspects cargo-dist planned archives
- **THEN** the archive configuration includes LICENSE and THIRD_PARTY_NOTICES.md

### Requirement: Dependency notice process

The project SHALL track locked dependency versions, licenses and source origins, plus revision, attribution, modifications, and source obligations for adapted code. CI SHALL reject an outdated dependency inventory.

#### Scenario: Dependency update

- **WHEN** Cargo.lock changes without refreshing the dependency inventory
- **THEN** the notices check fails and tells the contributor how to regenerate it
