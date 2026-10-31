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
