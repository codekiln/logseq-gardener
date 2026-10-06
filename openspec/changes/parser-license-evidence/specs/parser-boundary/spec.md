## ADDED Requirements

### Requirement: Reproducible Rust candidate license evidence

The parser comparison SHALL record the tested Rust candidate's exact revision, locked dependency identities and checksums, declared licenses, and actual license and attribution files. A local check SHALL detect changes to the saved inventory or license texts and SHALL leave the upstream source checkout unchanged. Adoption guidance SHALL distinguish candidate evidence from the SDK's resulting dependency inventory and name the remaining compatibility work.

#### Scenario: Reproduce license evidence

- **WHEN** a contributor runs the documented audit with the pinned source and locked dependencies available
- **THEN** it compares the inventory and license texts with saved evidence and fails on differences

#### Scenario: Prepare SDK adoption

- **WHEN** a contributor reads the adoption guide
- **THEN** it identifies the candidate revision, notice and source requirements, the need to audit the SDK's actual lockfile, and the outstanding corpus comparison
