## MODIFIED Requirements

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
