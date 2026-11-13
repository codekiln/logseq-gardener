## Purpose

Set the evidence needed to choose a parser before Logseq Gardener adds commands that read garden notes.

## Requirements

### Requirement: Compare parsers before adding garden commands

The OpenSpec plan SHALL require a named comparison experiment before parser adoption or garden operations, with lsdoc as the Rust candidate, official mldoc as syntax reference, and Logseq OG graph-parser and graph-validator as graph compatibility references.

#### Scenario: Review next parser change

- **WHEN** a contributor opens the architecture guide and its OpenSpec plan
- **THEN** the guide explains the parser roles and pending choice, while the plan identifies sample gardens, compatibility checks, text-preservation checks, and the named experiment directory for results

### Requirement: Reproducible syntax reference baseline

The parser comparison experiment SHALL include a checked-in Markdown fixture garden and a repeatable command that checks the recorded parse and reference output of the Logseq-used `mldoc` version. The experiment SHALL identify the source revision and SHALL state that the syntax baseline does not establish graph compatibility or text-preserving writes.

#### Scenario: Reproduce the baseline

- **WHEN** a contributor runs the documented mise check
- **THEN** the command compares each fixture and parser result with the saved version and fails when either changes

#### Scenario: Review the comparison scope

- **WHEN** a reviewer reads the experiment guide
- **THEN** it identifies the fixtures, parser version, source revision, and the remaining graph, candidate-parser, and text-preservation checks

### Requirement: Check SDK relationships against recorded Logseq results

The SDK tests SHALL include the fixture notes from the current Logseq OG relationship comparison and expected results derived from that comparison's recorded output. The expected results SHALL name the Logseq source revision, the runtime versions and each note's SHA-256 hash, and a guide beside them SHALL state what the derivation leaves out. For the default and triple-lowbar filename formats, an SDK test SHALL compare each note's page name, title, file, aliases, namespace parent and journal date; the files each alias name finds; and the pages without a file that notes link to, together with their namespace parents. `mise run ci` SHALL run the test and a check that the copied notes and expected results match the recorded evidence, without a Logseq checkout or a whole garden. An update command SHALL regenerate the copies from the recorded evidence.

#### Scenario: A relationship regression fails CI

- **WHEN** an SDK change stops reading a page's `alias::` names, keeps one page per shared alias, or loses a namespace parent
- **THEN** the relationship test fails and names the differing pages

#### Scenario: Copies drift from the evidence

- **WHEN** a copied fixture note or the expected results differ from what the recorded evidence implies
- **THEN** the check fails, names the file and points to the update command

#### Scenario: Regenerate after an intended change

- **WHEN** a contributor updates the recorded Logseq evidence and runs the update command
- **THEN** the copied notes and expected results are rewritten from the evidence for review, with no network access or Logseq checkout

### Requirement: Reproducible candidate and relationship comparison

The experiment SHALL compare lsdoc and mldoc on shared fixtures and available public gardens using pinned revisions. It SHALL record the normalization rules, corpus identities, reproducible disagreements, performance measurements, and source-preservation limits. It SHALL run a file-compatible Logseq graph parser and graph-validator on focused fixtures and record graph identities and relationships separately from syntax results.

#### Scenario: Reproduce candidate results

- **WHEN** a reviewer runs the documented comparison commands with the pinned sources
- **THEN** the results identify differences and failures, and the fixture check fails when a recorded deterministic result changes

#### Scenario: Assess parser adoption

- **WHEN** a reviewer reads the findings
- **THEN** the findings describe the tested scope, unresolved differences, repair work, and the SDK responsibilities suggested by the evidence

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

### Requirement: Reproducible Rust candidate license evidence

The local license audit SHALL save the tested Rust candidate's revision, locked dependency identities and checksums, declared licenses, and supplied license and attribution files. A local check SHALL detect changes to the saved inventory or license texts and SHALL leave the upstream source checkout unchanged. Adoption guidance SHALL distinguish candidate evidence from the SDK's resulting dependency inventory and name the remaining compatibility work.

#### Scenario: Reproduce license evidence

- **WHEN** a contributor runs the documented audit with the pinned source and locked dependencies available
- **THEN** it compares the inventory and license texts with saved evidence and fails on differences

#### Scenario: Prepare SDK adoption

- **WHEN** a contributor reads the adoption guide
- **THEN** it identifies the candidate revision, notice and source requirements, the need to audit the SDK's actual lockfile, and the outstanding corpus comparison

### Requirement: Public-garden comparisons respect the note format

The public-garden comparison SHALL pass each note's Markdown or Org format to the reference extractor. Any repair to pinned upstream tooling SHALL be documented, applied only to the extracted experiment source, and checked against the expected source before modification.

#### Scenario: Compare Org page links

- **WHEN** the public-garden runner compares an Org note containing `[[Guide/Page][Guide page]]`
- **THEN** the reference parser recognizes `Guide/Page` as a page reference and compares it with the candidate parser's result

#### Scenario: Detect a broken comparison runner

- **WHEN** the format argument is removed from the reference worker
- **THEN** the experiment's runner regression check fails on the Org reference fixture

#### Scenario: Review corrected corpus results

- **WHEN** a reviewer reads the saved Logseq docs report
- **THEN** the report identifies the applied runner repair, the pinned source and corpus revisions, and the disagreements remaining after the repair
