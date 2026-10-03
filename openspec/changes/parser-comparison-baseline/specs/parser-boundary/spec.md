## ADDED Requirements

### Requirement: Reproducible syntax reference baseline

The parser comparison experiment SHALL include a checked-in Markdown fixture garden and a repeatable command that checks the recorded parse and reference output of the Logseq-used `mldoc` version. The experiment SHALL identify the source revision and SHALL state that the syntax baseline does not establish graph compatibility or text-preserving writes.

#### Scenario: Reproduce the baseline

- **WHEN** a contributor installs the pinned experiment dependencies and runs the documented check
- **THEN** the command checks each fixture against the committed `mldoc` output and fails if the output or fixture bytes differ

#### Scenario: Review the comparison scope

- **WHEN** a reviewer reads the experiment guide
- **THEN** it identifies the fixtures, parser version, source revision, and the remaining graph, candidate-parser, and text-preservation checks
