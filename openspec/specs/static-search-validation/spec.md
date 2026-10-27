## Purpose

Define reproducible static-search loading, latency, scale and failure validation.

## Requirements

### Requirement: Completed-result latency measurement

The shared controller SHALL coalesce pending input without a fixed debounce delay and cancel pending evaluation on close/revision changes. The existing parity harness SHALL measure completed-result latency, including no-match/unchanged query outcomes, with explicit unchanged counts. Desktop and phone p95 budgets SHALL remain 50ms and 100ms.

#### Scenario: Rapid input and dismissal

- **WHEN** input changes repeatedly before evaluation or search closes
- **THEN** only current input can publish results and closed/stale work cannot update the dialog

### Requirement: Reproducible loading and scale evidence

The existing performance harness SHALL measure small, larger generic and actual selected-host corpora on desktop and throttled phone profiles. Evidence SHALL distinguish ordinary page work, first-open readiness/transfer/heap, and cached reopening. Corrupt/missing/offline/revision-mismatched artifacts SHALL retain actionable error/retry behavior and ordinary navigation.

#### Scenario: Cold and warm search

- **WHEN** a fresh browser opens search and then reopens the same publication
- **THEN** measurements include validated loading before readiness and report whether reopening performs artifact requests

#### Scenario: Failed publication loading

- **WHEN** loading encounters an unavailable or corrupt artifact
- **THEN** the dialog exposes retry/reload without automatic navigation or stale results

### Requirement: Whole-garden search budgets

`mise run parity:build` SHALL publish search, and `parity:perf --search-site` SHALL measure `lsg` and Logseq on the whole encode garden with the committed query list. The `[search]` budgets SHALL cover title index size, largest shard, bytes fetched per keystroke, first open, cold and warm keystroke latency, phone heap growth and build time. `parity:budgets` SHALL check the size budgets in CI.

#### Scenario: Budget rows for both builds

- **WHEN** `parity:perf --search-site` runs on the whole encode garden
- **THEN** the committed search rows hold numbers for both `lsg` and Logseq, and every `lsg` value within a `[search]` budget is unmarked

#### Scenario: Oversized shard fails CI

- **WHEN** a build writes a shard larger than the shard budget
- **THEN** `parity:budgets` fails and names the shard
