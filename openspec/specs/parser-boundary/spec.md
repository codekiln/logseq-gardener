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
