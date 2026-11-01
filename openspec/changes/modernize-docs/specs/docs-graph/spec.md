## ADDED Requirements

### Requirement: Logseq graph in docs

`docs/` SHALL be a Logseq graph that opens in Logseq 0.10 desktop. `docs/logseq/config.edn` SHALL set `:file/name-format :triple-lowbar`, `:publishing/all-pages-public? true`, `:publishing/page-properties? true`, `:publishing/page-properties-collapsed? false`, `:feature/enable-journals? false` and `:default-home {:page "lsg"}`. Every page SHALL be a file in `docs/pages/` named after its title, with each `/` written as `___` and `.md` added; Logseq's `contents.md` is the one file named otherwise. The repository SHALL commit `docs/pages/contents.md` and `docs/logseq/custom.css`, and `.gitignore` SHALL list `docs/logseq/bak/` and `docs/logseq/.recycle/`, so opening the graph in Logseq adds no file that git reports.

#### Scenario: Open the graph in Logseq

- **WHEN** a contributor opens `docs/` as a graph in Logseq 0.10 desktop and re-indexes it
- **THEN** Logseq shows the `lsg` page as its home page, lists every page in `docs/pages/`, and `git status` reports no new or changed file

#### Scenario: Title from the file name

- **WHEN** `lsg` loads `docs/pages/lsg___How To___Install with mise.md` with `--filename-format triple-lowbar`
- **THEN** the page's title is `lsg/How To/Install with mise`

### Requirement: Page titles and tags by reader need

Every page in `docs/pages/` other than `contents.md` and the kind index pages SHALL have a title in the `lsg` namespace that names its Diataxis kind: `lsg/How To/<task>` for a how-to page, `lsg/Tutorial/<goal>` for a tutorial, `lsg/Ref/<subject>` for a reference page, and `lsg` or `lsg/<concept>` for an explanation page. Each such page SHALL carry a `tags::` page property holding the tag of its kind: `[[Diataxis/How To]]`, `[[Diataxis/Tutorial]]`, `[[Diataxis/Reference]]` or `[[Diataxis/Explanation]]`. `mise run docs:check` SHALL fail when a page's kind tag is missing or does not match its title.

#### Scenario: Tag matches the title

- **WHEN** `mise run docs:check` runs on a graph where `lsg/Ref/Command Line` carries `tags:: [[Diataxis/How To]]`
- **THEN** the check fails and names `lsg/Ref/Command Line` and the expected tag `[[Diataxis/Reference]]`

#### Scenario: Explanation named after its concept

- **WHEN** a page explains the parser decision
- **THEN** its title is `lsg/Parser Adoption` and it carries `tags:: [[Diataxis/Explanation]]`

### Requirement: Kind index pages

The graph SHALL contain the pages `Diataxis`, `Diataxis/How To`, `Diataxis/Tutorial`, `Diataxis/Reference` and `Diataxis/Explanation`. `Diataxis/Explanation` SHALL carry `alias:: [[Diataxis/Concept]]`. Each kind page SHALL say what reader need its kind serves, and its Pages tagged with section SHALL list every page of that kind, in Logseq and on the site.

#### Scenario: Index of how-to pages

- **WHEN** the site is built
- **THEN** the `Diataxis/How To` page's Pages tagged with section lists every page whose title starts with `lsg/How To/`

### Requirement: Logseq outline Markdown

Every page SHALL be Logseq outline Markdown: page properties as `key:: value` lines at the top of the file, every block starting with `- `, and child blocks indented with one tab per level. A table or a fenced code block SHALL sit inside one block. A how-to, tutorial, reference or explanation page SHALL follow the page shape that codekiln's Diataxis entity page for its kind describes, leaving out empty sections. Pages converted from the parity reports and the Logseq reference boards SHALL keep the sections of their source.

#### Scenario: Outline parsed as written

- **WHEN** Logseq's file parser and `lsg` load a page with page properties, nested blocks, a table and a fenced code block
- **THEN** both find the same page properties and the same block tree, with the table and the code inside their blocks

### Requirement: Links inside and outside the graph

A link from one page to another SHALL be a page reference such as `[[lsg/Ref/Command Line]]` to a page in the graph. A link from a page to a repository file SHALL be an `https://github.com/codekiln/logseq-gardener/blob/main/` URL, or a `tree/main/` URL for a folder. An image that a page shows SHALL live in `docs/assets/`. A repository file outside the graph that points at a page, such as a code comment or a task message, SHALL name the page's title and its file in `docs/pages/`.

#### Scenario: Link to a missing page

- **WHEN** a page links `[[lsg/Ref/Missing Page]]` and no such page exists
- **THEN** `mise run docs:check` fails and its output names the unresolved page

#### Scenario: Relative link to a repository file

- **WHEN** a page links `[help](../../src/help.rs)`
- **THEN** `mise run docs:check` fails and its output names the link

### Requirement: Non-Markdown files in the graph folder

`docs/parity/publication-speed.json`, `docs/parity/search-performance.json`, `docs/dependency-sources.json` and the files in `docs/logseq-reference/html/` SHALL stay at these paths. They SHALL create no page in `lsg` or in Logseq.

#### Scenario: Data files stay where scripts read them

- **WHEN** a contributor runs `mise run notices:check` and the site is built
- **THEN** the notices check reads `docs/dependency-sources.json`, and the built site's `lsg-site.json` lists no page made from a `.json` or `.html` file

### Requirement: Unconverted files kept out of the graph

While a Markdown file under `docs/` outside `docs/pages/` exists, `:hidden` in `docs/logseq/config.edn` SHALL hold an entry for that file alone, on a line of its own, written as `"/<path from docs/>"`. The pull request that converts the file SHALL delete it and remove its entry. `mise run docs:check` SHALL fail when such a file has no entry.

#### Scenario: Hidden until converted

- **WHEN** `docs/cli.md` exists and `:hidden` holds `"/cli.md"`
- **THEN** neither `lsg` nor Logseq loads a page from `docs/cli.md`

#### Scenario: Missing entry

- **WHEN** `docs/parity/search.md` exists and `:hidden` has no `"/parity/search.md"` entry
- **THEN** `mise run docs:check` fails and names `docs/parity/search.md`

### Requirement: Logseq and lsg agree on the graph

`mise run docs:check` SHALL load the docs graph with Logseq's own file parser, at the revision that `parser:og-check` pins, and with the SDK, and SHALL fail when the two disagree about the graph's pages, titles, aliases, page references, block references or properties. Each reported difference SHALL name its file and the differing item. The check SHALL run locally and in CI without a Logseq checkout that the contributor prepares by hand.

#### Scenario: The docs graph agrees

- **WHEN** `mise run docs:check` runs on the docs graph
- **THEN** the parser check passes

#### Scenario: Deliberate mismatch

- **WHEN** the parser check runs on its test fixture, where Logseq's result and the SDK's result differ in one page's alias
- **THEN** the check fails and names the file and the alias

#### Scenario: Check in CI

- **WHEN** CI runs `mise run ci` on a fresh runner
- **THEN** the parser check runs and its result decides the job's result
