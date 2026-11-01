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

### Requirement: Build Logseq's own export of a garden

`mise run parity:logseq-export` SHALL build a static export of a garden, the docs graph by default, by running `logseq.tasks.dev.publishing` from tag `0.10.15` of logseq/logseq with `nbb-logseq`, which calls `logseq.publishing/export`. The npm packages SHALL be installed from a committed lockfile.

The export code SHALL come from a local ghq checkout of logseq/logseq that holds the pinned commit, otherwise from GitHub. Either copy SHALL match a pinned digest before it is used, and the task SHALL cache it under `~/.cache/logseq-gardener/`.

The built publishing frontend SHALL come from the folder passed with `--static-dir`, otherwise from Logseq.app on macOS, otherwise from the official 0.10.15 release archive, which SHALL match its pinned SHA-256 before it is unpacked into the cache. The frontend's files SHALL match a pinned digest. A Logseq.app frontend of another version SHALL leave the task to use the release archive; a `--static-dir` folder that fails the digest SHALL stop the task.

The task SHALL record the export's log in the run directory. It SHALL list the files the export failed to copy into its `assets/` folder in the report without counting them as differences, and it SHALL stop when the export fails.

#### Scenario: Export code from a local checkout
- **WHEN** a ghq checkout of logseq/logseq holds tag `0.10.15` and the cache is empty
- **THEN** the task archives the pinned paths from that checkout, checks their digest and caches them

#### Scenario: Export code from GitHub
- **WHEN** no local checkout holds the pinned commit
- **THEN** the task fetches only the pinned paths from GitHub and uses them after their digest matches

#### Scenario: Digest mismatch
- **WHEN** the Logseq source or a `--static-dir` frontend differs from its pinned digest
- **THEN** the task stops before it builds anything and names the expected and the found digest

#### Scenario: No Logseq.app
- **WHEN** the machine has no Logseq.app 0.10.15 and no `--static-dir` is passed
- **THEN** the task downloads the pinned release archive once, checks its SHA-256, and caches the frontend for later runs

#### Scenario: Image named inside a code example
- **WHEN** the export logs that it failed to copy an image file that a code example names
- **THEN** the report lists the file, and the task's exit status depends only on the page comparison

### Requirement: Build the same garden with lsg for the export comparison

`mise run parity:logseq-export` SHALL build `lsg` from the checkout and publish the garden into the same run directory. For the docs graph it SHALL use the options `docs:site` uses. For another garden it SHALL use the `lsg publish` options passed to the task, and it SHALL stop when another garden is given without options. It SHALL read the published pages' titles and URLs from the site's `lsg-site.json`.

#### Scenario: Docs graph by default
- **WHEN** a contributor runs `mise run parity:logseq-export` without arguments
- **THEN** the task publishes `docs` with `--filename-format triple-lowbar --include lsg --include Diataxis --homepage lsg --search --readable-urls`

#### Scenario: Another garden without options
- **WHEN** a contributor passes `--garden` with a garden other than `docs` and no `lsg publish` options
- **THEN** the task stops and asks for the options that select the garden's pages

### Requirement: Compare rendered pages with Logseq's export

`mise run parity:logseq-export` SHALL serve both builds from one server on 127.0.0.1 and read each page `lsg-site.json` lists as a page or journal in headless Chromium, in the `lsg` build and at the Logseq route of the same title. It SHALL load each Logseq page from a blank page and wait until the document title names the page. It SHALL scroll Logseq's page until the text of its blocks stops growing.

On both sides the task SHALL read the browser's rendered text of the page's blocks, without the page properties box and without Logseq's reference-count badge, and compare the words, ignoring letter case and punctuation and counting repeats. It SHALL read the rows of each build's page properties box as key and value and compare them.

The task SHALL write a JSON report and a Markdown summary into a new directory `target/parity/<UTC time>/`, naming for each page the words only one build shows, the property rows only one build shows, and a page that did not load. It SHALL list the Logseq pages with blocks that the `lsg` build does not publish. It SHALL exit 1 when any page differs and leave the garden's files unchanged.

#### Scenario: Same words in both builds
- **WHEN** a page's blocks show the same words in both builds
- **THEN** the report counts the page as matching

#### Scenario: Reference-count badge
- **WHEN** Logseq shows a count beside a block that other blocks reference
- **THEN** the count is left out of Logseq's text, and the page matches when the blocks otherwise show the same words

#### Scenario: Word split across highlight spans
- **WHEN** syntax highlighting puts `2` and `px` of `2px` in separate elements
- **THEN** both builds read the word `2px`

#### Scenario: Blocks Logseq renders after scrolling
- **WHEN** Logseq renders the end of a long page only after it is scrolled into view
- **THEN** Logseq's text includes the end of the page

#### Scenario: Property row only in Logseq
- **WHEN** Logseq's properties box on a page shows a row, such as `alias: Logseq Gardener`, that the `lsg` box lacks
- **THEN** the report names that row for the page, and the task exits 1

#### Scenario: Logseq page that does not load
- **WHEN** Logseq's document title never names an `lsg` page's title
- **THEN** the report names the page as missing from Logseq's export, and the task exits 1

#### Scenario: Page outside the lsg options
- **WHEN** Logseq's export holds a page with blocks that the `lsg` options do not select
- **THEN** the report lists the page, and the listing alone leaves the exit status at 0

#### Scenario: Garden unchanged
- **WHEN** the task finishes on a garden in a Git checkout
- **THEN** `git status` reports the same files in the garden as before the run

### Requirement: Test the harness logic in CI

`mise run test` SHALL run unit tests for the Logseq data decoding, the title normalization, the grouping, and the export comparison's word and property comparison, export log reading and scrolling loop, using small fixtures stored in the repository. The tests SHALL run without the encode garden, network access or a browser. `mise run parity:logseq-export` SHALL first read small HTML fixture pages in Chromium and stop when the page reading keeps a reference-count badge, splits a word across highlight spans or misses blocks added on scroll.

#### Scenario: CI without the garden
- **WHEN** `mise run ci` runs on a machine without the encode garden
- **THEN** the harness unit tests pass using repository fixtures only

#### Scenario: Page reading checked in a browser
- **WHEN** a contributor runs `mise run parity:logseq-export`
- **THEN** the task reads the fixture pages in Chromium before it builds either site, and stops when a fixture page reads differently from its expected text
