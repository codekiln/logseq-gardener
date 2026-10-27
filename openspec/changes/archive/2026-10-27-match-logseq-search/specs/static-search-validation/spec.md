## ADDED Requirements

### Requirement: Whole-garden search budgets

`mise run parity:build` SHALL publish search, and `parity:perf --search-site` SHALL measure `lsg` and Logseq on the whole encode garden with the committed query list. The `[search]` budgets SHALL cover title index size, largest shard, bytes fetched per keystroke, first open, cold and warm keystroke latency, phone heap growth and build time. `parity:budgets` SHALL check the size budgets in CI.

#### Scenario: Budget rows for both builds

- **WHEN** `parity:perf --search-site` runs on the whole encode garden
- **THEN** the committed search rows hold numbers for both `lsg` and Logseq, and every `lsg` value within a `[search]` budget is unmarked

#### Scenario: Oversized shard fails CI

- **WHEN** a build writes a shard larger than the shard budget
- **THEN** `parity:budgets` fails and names the shard
