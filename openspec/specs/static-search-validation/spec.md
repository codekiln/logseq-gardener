## Purpose

Define reproducible static-search loading, latency, scale and failure validation.

## Requirements

### Requirement: Drawn-result latency measurement

The shared controller SHALL coalesce pending input without a fixed debounce delay and cancel pending evaluation on close/revision changes. The existing parity harness SHALL time each keystroke until the dialog draws a result list for that input or for a longer input that starts with it, which `data-lsg-search-drawn` on `#lsg-search-results` marks. A drawn list SHALL include no-match outcomes, and the harness SHALL count, as unanswered, each keystroke no drawn list answered. A drawn list SHALL wait for the shards its query needs; a block row whose record chunk is still loading SHALL show its page title and a placeholder line until the chunk arrives, which `data-lsg-search-complete` then marks. Desktop and phone p95 budgets SHALL remain 50ms and 100ms.

#### Scenario: Rapid input and dismissal

- **WHEN** input changes repeatedly before evaluation or search closes
- **THEN** only current input can publish results and closed/stale work cannot update the dialog

#### Scenario: Keystroke ends at the drawn list

- **WHEN** a keystroke's query matches block rows whose record chunk has not arrived
- **THEN** the harness ends that keystroke when the list is drawn with placeholder lines, and the search browser check sees `data-lsg-search-complete` advance and the block text replace the placeholders once the chunk arrives

#### Scenario: Query with no results

- **WHEN** a keystroke's query matches nothing
- **THEN** the dialog draws the no-results state, `data-lsg-search-drawn` advances, and the keystroke is measured rather than counted as unanswered

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
