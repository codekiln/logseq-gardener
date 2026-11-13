## ADDED Requirements

### Requirement: Reproducible syntax reference baseline

The parser comparison experiment SHALL include a checked-in Markdown fixture garden and a repeatable command that checks the recorded parse and reference output of the Logseq-used `mldoc` version. The experiment SHALL identify the source revision and SHALL state that the syntax baseline does not establish graph compatibility or text-preserving writes.

#### Scenario: Reproduce the baseline

- **WHEN** a contributor runs the documented mise check
- **THEN** the command compares each fixture and parser result with the saved version and fails when either changes

#### Scenario: Review the comparison scope

- **WHEN** a reviewer reads the experiment guide
- **THEN** it identifies the fixtures, parser version, source revision, and the remaining graph, candidate-parser, and text-preservation checks
