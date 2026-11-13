## ADDED Requirements

### Requirement: Reproducible candidate and relationship comparison

The experiment SHALL compare lsdoc and mldoc on shared fixtures and available public gardens using pinned revisions. It SHALL record the normalization rules, corpus identities, reproducible disagreements, performance measurements, and source-preservation limits. It SHALL run a file-compatible Logseq graph parser and graph-validator on focused fixtures and record graph identities and relationships separately from syntax results.

#### Scenario: Reproduce candidate results

- **WHEN** a reviewer runs the documented comparison commands with the pinned sources
- **THEN** the results identify differences and failures, and the fixture check fails when a recorded deterministic result changes

#### Scenario: Assess parser adoption

- **WHEN** a reviewer reads the findings
- **THEN** the findings describe the tested scope, unresolved differences, repair work, and the SDK responsibilities suggested by the evidence
