## Purpose

Gardens such as Gitpa publish with `lsg` partly because it is fast. The SDK records how long each phase of a publication takes, a benchmark publishes a generated Gitpa-sized garden and the encode garden and records the results, and a CI check fails when a build becomes slower than the recorded results allow.

## Requirements

### Requirement: Publication phase timings from the SDK

The SDK SHALL record the wall-clock time of each publication phase in `SiteReport::timings`, in this order: `read` (finding and reading source files and checking options), `parse`, `index` (selection, names, references, queries and search records), `plugins` (waiting for build plugins), `render`, `history` (waiting for the git history read), `search` (search files), `graph` (waiting for graph layouts and neighbourhoods) and `write`. Each phase SHALL appear once, with time spent in it across separate intervals added together. `garden::load_garden_timed` SHALL return the same garden as `load_garden` and add its time to the `read` and `parse` phases of a caller's stopwatch. Timings SHALL hold phase names and durations only, and recording them SHALL leave the published files unchanged.

#### Scenario: Timed loading
- **WHEN** a caller loads a garden with `load_garden_timed`
- **THEN** the garden equals the result of `load_garden`, and the stopwatch holds `read` and then `parse`, whose sum is the total

#### Scenario: Every publication reports its phases
- **WHEN** the SDK publishes a garden
- **THEN** `SiteReport::timings` lists the nine phases in order, including phases with no work such as `plugins` without build plugins

### Requirement: Reproducible publication benchmark

`mise run parity:publish-bench` SHALL build a release `lsg` and publish each requested corpus with `--journals`, `--search`, `--timings` and every top-level namespace included: once cold and then `--runs` times warm. The `gitpa-sized` corpus SHALL be a garden generated from a fixed seed with Gitpa's page count, word count, namespace depth and link, block and property density, identified by a digest of its files. The `encode-garden` corpus SHALL be a checkout of the encode garden, identified by its commit. Each result row SHALL record the date, the lsg commit, the corpus and its identity, the runner, operating system, processor model and processor count, the cold time, the fastest and median warm times, the fastest warm build's phase times, peak memory, the published counts, the output's file count and size, and the file count and size of each part of the output: page HTML at the site root, `_refs/`, `assets/`, search files, `_graph/` and everything else. `--record` SHALL append the rows to `docs/parity/publication-speed.json`.

#### Scenario: Same generated garden everywhere
- **WHEN** the benchmark generates the `gitpa-sized` corpus on two machines
- **THEN** both rows record the same digest

#### Scenario: Record a row
- **WHEN** a contributor runs the benchmark with `--record`
- **THEN** the recorded file gains one row per corpus with every field above

#### Scenario: Parts add up
- **WHEN** a row records the encode garden's output
- **THEN** the sizes of its parts add up to the output's size, and their file counts to its file count

### Requirement: Publication speed regression check

With `--check`, the benchmark SHALL compare each corpus's fastest warm build with the recorded rows for the same corpus identity and runner. When rows exist for the same processor model, the limit SHALL be the slowest of them plus `[publication] slowdown_pct` from `scripts/parity/budgets.toml`. Otherwise, when rows exist for the corpus and runner, the limit SHALL be the slowest of them plus `unknown_cpu_slowdown_pct`. A build over its limit SHALL fail the check; a corpus without a comparable row SHALL be reported as not checked. CI SHALL run the check on the `gitpa-sized` corpus and on a pinned encode garden commit, and keep the rows it measured as a downloadable result.

#### Scenario: Slower build on a known processor
- **WHEN** rows for the same corpus, runner and processor model have a slowest warm time of 11 seconds and `slowdown_pct` is 20
- **THEN** a fastest warm build over 13.2 seconds fails the check

#### Scenario: Unrecorded processor
- **WHEN** no row matches the processor model, the slowest row for the corpus and runner is 30 seconds and `unknown_cpu_slowdown_pct` is 100
- **THEN** the limit is 60 seconds

#### Scenario: Another corpus or runner
- **WHEN** the only rows are for another corpus identity or another runner
- **THEN** the check reports the corpus as not checked and does not fail
