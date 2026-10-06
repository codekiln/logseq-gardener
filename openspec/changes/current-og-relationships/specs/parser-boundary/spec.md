## ADDED Requirements

### Requirement: Compare current Logseq OG relationships

The experiment SHALL compare pinned current and historical Logseq file parsers on the same Markdown notes using the same locked mldoc runtime. It SHALL record page names, file paths, aliases, namespace parents, journal identities, properties, block hierarchy, and reference targets with generated UUIDs normalized. Results SHALL identify source revisions, runtime versions, input hashes, and differences from the saved historical evidence.

#### Scenario: Reproduce graph relationships

- **WHEN** a contributor runs the documented comparison check with the pinned revisions available in ghq
- **THEN** the check reproduces the saved default and configured-filename results and fails if inputs or projected results change

#### Scenario: Review parser changes

- **WHEN** a reviewer reads the comparison findings
- **THEN** the findings identify differences between current and historical parsers and explain the implications for SDK page lookup

#### Scenario: Keep source checkouts intact

- **WHEN** the experiment prepares a parser revision
- **THEN** it extracts committed source into an ignored experiment directory and leaves the registered upstream checkout unchanged
