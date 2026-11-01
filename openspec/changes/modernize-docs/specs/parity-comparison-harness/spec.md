## MODIFIED Requirements

### Requirement: Check lsg against performance budgets

The harness SHALL read budgets for each measure and profile from a committed budget file, and the results SHALL mark every `lsg` value that exceeds its budget. `mise run parity:perf --record` SHALL rewrite the `lsg/Ref/Logseq Compatibility/Performance` page, `docs/pages/lsg___Ref___Logseq Compatibility___Performance.md`, as Logseq outline Markdown with the results and the garden commit, `gh-pages` commit and `lsg` version they came from.

#### Scenario: Page over budget
- **WHEN** an `lsg` fixture page transfers more data before its title is visible than the budget allows
- **THEN** the results mark that value as over budget and name the budget

#### Scenario: Recorded rows
- **WHEN** a contributor runs `mise run parity:perf --record`
- **THEN** the `lsg/Ref/Logseq Compatibility/Performance` page holds one row per fixture page, build and profile, with the commits and version used, and `mise run docs:site` publishes it with no diagnostic
