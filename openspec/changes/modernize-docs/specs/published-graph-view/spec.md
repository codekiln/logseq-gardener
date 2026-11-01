## MODIFIED Requirements

### Requirement: Keep the graph view within budgets

The graph view SHALL meet the `[graph]` budgets in `scripts/parity/budgets.toml`, measured by the comparison harness on the whole encode garden: `_graph/default.bin` at most 600 KB gzip; `lsg-graph.js` with its styles at most 30 KB minified gzip, with the worker separate; the first frame with every node drawn within 1,000 ms on the desktop profile and 3,000 ms on the phone profile; a peak JS heap of at most 50 MB on the phone profile after the first frame, a pan and a zoom; a 95th-percentile frame time of at most 50 ms while panning on the phone profile; and a whole-garden build at most 25% slower than the `--baseline-lsg` binary's. `parity:budgets` SHALL check the sizes in CI and `parity:perf` SHALL mark the times, heap and build values.

#### Scenario: Budgets in CI

- **WHEN** `mise run ci` runs `parity:budgets` on the parity fixture build
- **THEN** it fails if the graph file or script exceeds its budget

#### Scenario: Measured against Logseq

- **WHEN** `parity:perf --record` runs on the whole encode garden
- **THEN** the `lsg/Ref/Logseq Compatibility/Performance` page gains graph rows for `lsg` and Logseq with the first frame, bytes, heap and frame time
