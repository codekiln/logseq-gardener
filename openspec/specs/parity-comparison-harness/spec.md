## Purpose

Build the whole encode garden with lsg, compare it page by page with the site Logseq publishes for the same garden, and measure both builds so lsg can be held to its performance budgets.

## Requirements

### Requirement: Build the whole garden with lsg

`mise run parity:build` SHALL publish every page of a garden that `lsg` can select, using one `--include` root for each top-level namespace found among the garden's page files. It SHALL default to the encode garden resolved with `ghq` and accept another garden path. It SHALL write into a new directory for each run and record, in a settings file beside the output, the garden commit, the `lsg` version and every `lsg` option it passed. It SHALL leave the garden's files unchanged.

#### Scenario: Whole-garden build
- **WHEN** a contributor runs `mise run parity:build` with the encode garden checked out
- **THEN** a new output directory holds the `lsg` site for every top-level namespace, and the settings file names the garden commit, the `lsg` version and the options used

#### Scenario: Repeat run
- **WHEN** the task runs a second time
- **THEN** it writes a second new directory and leaves the first run's output in place

#### Scenario: Title outside lsg's namespace rules
- **WHEN** a page file's decoded top-level segment fails `lsg`'s selector rules
- **THEN** the build skips that root, finishes the rest, and lists the skipped file in the settings file

### Requirement: Read Logseq's published build

The harness SHALL read Logseq's build from the garden's `gh-pages` branch without running Logseq. It SHALL decode `static/js/db.gz` into page records with the page name, original title, whether the page is a journal, and whether the page has content blocks. It SHALL record the `gh-pages` commit it read.

#### Scenario: Page records from db.gz
- **WHEN** the harness reads a `gh-pages` build
- **THEN** it lists each page entity with its name, title, journal flag and content flag, and reports the counts of pages, journals and pages with content

#### Scenario: Escaped characters in the graph data
- **WHEN** a page title contains `&`, `<`, `>`, `"` or `'`
- **THEN** the decoded title contains the original character

### Requirement: Map titles to both builds

The harness SHALL map each page title to its `lsg` file and to its Logseq route `#/page/<name>`, where `<name>` is the lower-cased page name, URL-encoded. Titles SHALL match across builds after lower-casing and Unicode NFC normalization, the same comparison Logseq applies to page names.

#### Scenario: Same page in both builds
- **WHEN** a contributor asks the harness for the page `tmux`
- **THEN** it returns the `lsg` file for `tmux` and the Logseq URL ending in `#/page/tmux`

#### Scenario: Title with spaces and slashes
- **WHEN** a contributor asks for a namespaced title containing spaces
- **THEN** the Logseq route encodes the spaces and slashes and the `lsg` file is the one the index links to that title

### Requirement: Report missing pages by group

`mise run parity:compare` SHALL write a Markdown report and a JSON file listing pages present in one build and missing from the other. It SHALL place each Logseq-only page in exactly one group, checked in this order: journals; pages without content blocks (reference-only pages); pages whose file `:hidden` in `logseq/config.edn` matches; pages whose title `lsg` cannot select; and other pages. It SHALL list `lsg`-only pages in their own group. The report SHALL start with the counts for each group and name every page in the `lsg`-only and other-pages groups.

#### Scenario: Journal missing from lsg
- **WHEN** a journal with content is in Logseq's build and absent from `lsg`'s
- **THEN** the report counts it under journals and leaves it out of other pages

#### Scenario: Reference-only page
- **WHEN** a Logseq page has no content blocks and no `lsg` page
- **THEN** the report counts it under reference-only pages

#### Scenario: Unexplained page
- **WHEN** a Logseq page with content belongs to none of the named groups
- **THEN** the report names it under other pages

#### Scenario: Page only in lsg
- **WHEN** an `lsg` page has no Logseq page with the same normalized name
- **THEN** the report names it under `lsg`-only pages

### Requirement: Save side-by-side screenshots

`mise run parity:screenshots` SHALL serve both builds on loopback addresses and save a full-page screenshot of each page in a fixture list from each build. Before each capture it SHALL wait for the page title to render and scroll to the bottom of the page so lazily rendered sections are drawn. Both captures of a page SHALL use the same viewport size and colour scheme. It SHALL write an HTML index that shows the two captures of each page next to each other. A page missing from one build SHALL appear in the index with a note naming the build that lacks it.

#### Scenario: Fixture page in both builds
- **WHEN** the fixture list names `tmux`
- **THEN** the run saves a Logseq capture and an `lsg` capture of `tmux`, and the index shows them side by side

#### Scenario: Lazily rendered references
- **WHEN** Logseq draws Linked References only once they scroll into view
- **THEN** the Logseq capture includes the Linked References section

#### Scenario: Fixture missing from lsg
- **WHEN** the fixture list names a journal that `lsg` does not publish
- **THEN** the index shows the Logseq capture and states that the `lsg` build lacks the page

#### Scenario: Local servers only
- **WHEN** the screenshot task serves the builds
- **THEN** both servers listen on 127.0.0.1 and stop when the task ends

### Requirement: Measure load performance for both builds

`mise run parity:perf` SHALL load each fixture page from both builds, served on 127.0.0.1 with an empty browser cache, on a desktop profile and on a phone profile with a 412 by 823 viewport, four times CPU slowdown, 150 ms round trip, 1.6 Mbps download and 750 Kbps upload. For each page, build and profile it SHALL record the median of three loads for: time until the page title is visible, first contentful paint, gzip size of the data transferred before the title is visible, script size before the title is visible, and peak JavaScript heap. It SHALL write the results to the run directory as JSON and Markdown.

#### Scenario: Same page, both builds, both profiles
- **WHEN** the fixture list names `tmux`
- **THEN** the results hold desktop and phone rows for `tmux` from Logseq's build and from `lsg`'s build, each with the five measures

#### Scenario: Logseq's graph data counts before first paint
- **WHEN** Logseq's build loads `db.gz` before it shows the page title
- **THEN** the data-before-title measure for Logseq includes `db.gz` at its stored size

#### Scenario: Page missing from one build
- **WHEN** a fixture page exists in only one build
- **THEN** the results record the measures for that build and mark the other build's row as missing

### Requirement: Measure search latency per keystroke

`mise run parity:perf` SHALL measure each build's search on the whole garden by typing each query in a fixed query list one character at a time and recording, for each keystroke, the time until the results list stops changing. It SHALL report the median and 95th percentile per build and profile. For a build without a search input it SHALL report that the build has no search.

#### Scenario: Logseq search
- **WHEN** the harness types `tmux` into Logseq's search
- **THEN** it records four keystroke latencies for that query

#### Scenario: Build without search
- **WHEN** the `lsg` build has no search input
- **THEN** the search rows for `lsg` read "no search" and the run completes

### Requirement: Check lsg against performance budgets

The harness SHALL read budgets for each measure and profile from a committed budget file, and the results SHALL mark every `lsg` value that exceeds its budget. `mise run parity:perf --record` SHALL rewrite the `lsg/Ref/Logseq Compatibility/Performance` page, `docs/pages/lsg___Ref___Logseq Compatibility___Performance.md`, as Logseq outline Markdown with the results and the garden commit, `gh-pages` commit and `lsg` version they came from.

#### Scenario: Page over budget
- **WHEN** an `lsg` fixture page transfers more data before its title is visible than the budget allows
- **THEN** the results mark that value as over budget and name the budget

#### Scenario: Recorded rows
- **WHEN** a contributor runs `mise run parity:perf --record`
- **THEN** the `lsg/Ref/Logseq Compatibility/Performance` page holds one row per fixture page, build and profile, with the commits and version used, and `mise run docs:site` publishes it with no diagnostic

### Requirement: Check page size budgets in CI

`mise run parity:budgets` SHALL build the repository fixture garden with the release `lsg`, then build it again with 2,000 added pages that no fixture page references. It SHALL fail when a fixture page's first-paint size (its HTML plus the styles and scripts it loads before rendering) or its script size exceeds the budget file, or when a fixture page's first-paint size differs between the two builds. `mise run ci` SHALL run it.

#### Scenario: Pages within budget
- **WHEN** every fixture page is within budget and the same size in both builds
- **THEN** the task passes and prints each page's sizes

#### Scenario: First paint grows with the garden
- **WHEN** a change makes a fixture page larger in the build with 2,000 added pages
- **THEN** the task fails and names the page and both sizes

### Requirement: Test the harness logic in CI

`mise run test` SHALL run unit tests for the Logseq data decoding, the title normalization and the grouping, using small fixtures stored in the repository. The tests SHALL run without the encode garden, network access or a browser.

#### Scenario: CI without the garden
- **WHEN** `mise run ci` runs on a machine without the encode garden
- **THEN** the harness unit tests pass using repository fixtures only
