## RENAMED Requirements

- FROM: `### Requirement: Logseq graph in docs`
- TO: `### Requirement: Logseq graph in logseq-gardener-docs`

- FROM: `### Requirement: Non-Markdown files in the graph folder`
- TO: `### Requirement: Data files outside the graph folder`

## MODIFIED Requirements

### Requirement: Logseq graph in logseq-gardener-docs

`logseq-gardener-docs/` SHALL be a Logseq graph that opens in Logseq 0.10 desktop. `logseq-gardener-docs/logseq/config.edn` SHALL set `:file/name-format :triple-lowbar`, `:publishing/all-pages-public? true`, `:publishing/page-properties? true`, `:publishing/page-properties-collapsed? false`, `:feature/enable-journals? false` and `:default-home {:page "lsg"}`. Every page SHALL be a file in `logseq-gardener-docs/pages/` named after its title, with each `/` written as `___` and `.md` added; Logseq's `contents.md` is the one file named otherwise. The repository SHALL commit `logseq-gardener-docs/pages/contents.md` and `logseq-gardener-docs/logseq/custom.css`, and `.gitignore` SHALL list `logseq-gardener-docs/logseq/bak/` and `logseq-gardener-docs/logseq/.recycle/`, so opening the graph in Logseq adds no file that git reports.

#### Scenario: Open the graph in Logseq

- **WHEN** a contributor opens `logseq-gardener-docs/` as a graph in Logseq 0.10 desktop and re-indexes it
- **THEN** Logseq shows the `lsg` page as its home page, lists every page in `logseq-gardener-docs/pages/`, and `git status` reports no new or changed file

#### Scenario: Title from the file name

- **WHEN** `lsg` loads `logseq-gardener-docs/pages/lsg___How To___Install with mise.md` with `--filename-format triple-lowbar`
- **THEN** the page's title is `lsg/How To/Install with mise`

### Requirement: Page titles and tags by reader need

Every page in `logseq-gardener-docs/pages/` other than `contents.md` and the kind index pages SHALL have a title in the `lsg` namespace that names its Diataxis kind: `lsg/How To/<task>` for a how-to page, `lsg/Tutorial/<goal>` for a tutorial, `lsg/Ref/<subject>` for a reference page, and `lsg` or `lsg/<concept>` for an explanation page. Each such page SHALL carry a `tags::` page property holding the tag of its kind: `[[Diataxis/How To]]`, `[[Diataxis/Tutorial]]`, `[[Diataxis/Reference]]` or `[[Diataxis/Explanation]]`. `mise run docs:check` SHALL fail when a page's kind tag is missing or does not match its title.

#### Scenario: Tag matches the title

- **WHEN** `mise run docs:check` runs on a graph where `lsg/Ref/Command Line` carries `tags:: [[Diataxis/How To]]`
- **THEN** the check fails and names `lsg/Ref/Command Line` and the expected tag `[[Diataxis/Reference]]`

#### Scenario: Explanation named after its concept

- **WHEN** a page explains the parser decision
- **THEN** its title is `lsg/Parser Adoption` and it carries `tags:: [[Diataxis/Explanation]]`

### Requirement: Links inside and outside the graph

A link from one page to another SHALL be a page reference such as `[[lsg/Ref/Command Line]]` to a page in the graph. A link from a page to a repository file SHALL be an `https://github.com/codekiln/logseq-gardener/blob/main/` URL, or a `tree/main/` URL for a folder. An image that a page shows SHALL live in `logseq-gardener-docs/assets/`. A repository file outside the graph that points at a page, such as a code comment or a task message, SHALL name the page's title and its file in `logseq-gardener-docs/pages/`.

#### Scenario: Link to a missing page

- **WHEN** a page links `[[lsg/Ref/Missing Page]]` and no such page exists
- **THEN** `mise run docs:check` fails and its output names the unresolved page

#### Scenario: Relative link to a repository file

- **WHEN** a page links `[help](../../src/help.rs)`
- **THEN** `mise run docs:check` fails and its output names the link

### Requirement: Data files outside the graph folder

`docs/parity/publication-speed.json`, `docs/parity/search-performance.json`, `docs/dependency-sources.json` and the files in `docs/logseq-reference/html/` SHALL stay at these paths, outside the graph folder. They SHALL create no page in `lsg` or in Logseq.

#### Scenario: Data files stay where scripts read them

- **WHEN** a contributor runs `mise run notices:check` and the site is built
- **THEN** the notices check reads `docs/dependency-sources.json`, and the built site's `lsg-site.json` lists no page made from a `.json` or `.html` file

### Requirement: Unconverted files kept out of the graph

While a Markdown file under `logseq-gardener-docs/` outside `logseq-gardener-docs/pages/` and `logseq-gardener-docs/logseq/` exists, `:hidden` in `logseq-gardener-docs/logseq/config.edn` SHALL hold an entry for that file alone, on a line of its own, written as `"/<path from logseq-gardener-docs/>"`. The pull request that converts the file SHALL delete it and remove its entry. `mise run docs:check` SHALL fail when such a file has no entry. Markdown files under `docs/`, such as the parity reports in `docs/parity/`, are outside the graph folder and need no entry; `mise run docs:check` SHALL check their relative links.

#### Scenario: Hidden until converted

- **WHEN** `logseq-gardener-docs/cli.md` exists and `:hidden` holds `"/cli.md"`
- **THEN** neither `lsg` nor Logseq loads a page from `logseq-gardener-docs/cli.md`

#### Scenario: Missing entry

- **WHEN** `logseq-gardener-docs/notes/search.md` exists and `:hidden` has no `"/notes/search.md"` entry
- **THEN** `mise run docs:check` fails and names `logseq-gardener-docs/notes/search.md`

#### Scenario: Report outside the graph

- **WHEN** `docs/parity/advanced-queries.md` links a page file that does not exist
- **THEN** `mise run docs:check` fails and names the link, and asks for no `:hidden` entry for the report
