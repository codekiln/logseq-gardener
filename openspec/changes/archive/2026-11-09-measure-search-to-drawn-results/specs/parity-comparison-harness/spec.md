## MODIFIED Requirements

### Requirement: Measure search latency per keystroke

`mise run parity:perf` SHALL measure each build's search on the whole garden by typing each query in a fixed query list one character at a time and recording a latency for each keystroke. For Logseq a keystroke SHALL end at the last change to its results list before the next keystroke. For `lsg` a keystroke SHALL end when the dialog first draws a result list for that input or a longer one, as the static-search-validation requirement "Drawn-result latency measurement" defines; block rows still waiting for their text do not extend it. It SHALL report the median and 95th percentile per build and profile. For a build without a search input it SHALL report that the build has no search.

#### Scenario: Logseq search
- **WHEN** the harness types `tmux` into Logseq's search
- **THEN** it records four keystroke latencies for that query

#### Scenario: lsg keystroke ends at the drawn list
- **WHEN** the harness types a key into `lsg`'s search and the list is drawn before a block row's record chunk arrives
- **THEN** that keystroke's latency is the time to the drawn list

#### Scenario: Build without search
- **WHEN** the `lsg` build has no search input
- **THEN** the search rows for `lsg` read "no search" and the run completes
