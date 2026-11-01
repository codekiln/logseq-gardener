## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Test the harness logic in CI

`mise run test` SHALL run unit tests for the Logseq data decoding, the title normalization, the grouping, and the export comparison's word and property comparison, export log reading and scrolling loop, using small fixtures stored in the repository. The tests SHALL run without the encode garden, network access or a browser. `mise run parity:logseq-export` SHALL first read small HTML fixture pages in Chromium and stop when the page reading keeps a reference-count badge, splits a word across highlight spans or misses blocks added on scroll.

#### Scenario: CI without the garden
- **WHEN** `mise run ci` runs on a machine without the encode garden
- **THEN** the harness unit tests pass using repository fixtures only

#### Scenario: Page reading checked in a browser
- **WHEN** a contributor runs `mise run parity:logseq-export`
- **THEN** the task reads the fixture pages in Chromium before it builds either site, and stops when a fixture page reads differently from its expected text
