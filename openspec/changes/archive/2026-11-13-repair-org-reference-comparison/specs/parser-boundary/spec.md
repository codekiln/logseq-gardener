## ADDED Requirements

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
