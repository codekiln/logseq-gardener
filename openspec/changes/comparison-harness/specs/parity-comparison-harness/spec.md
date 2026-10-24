## ADDED Requirements

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

### Requirement: Test the harness logic in CI

`mise run test` SHALL run unit tests for the Logseq data decoding, the title normalization and the grouping, using small fixtures stored in the repository. The tests SHALL run without the encode garden, network access or a browser.

#### Scenario: CI without the garden
- **WHEN** `mise run ci` runs on a machine without the encode garden
- **THEN** the harness unit tests pass using repository fixtures only
