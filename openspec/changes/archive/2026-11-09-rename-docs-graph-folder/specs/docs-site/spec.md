## MODIFIED Requirements

### Requirement: Build the site with lsg from the same commit

`mise run docs:site` SHALL build `lsg` from the checkout and publish the docs graph with that binary into a directory that does not exist yet, using `--graph logseq-gardener-docs`, `--filename-format triple-lowbar`, `--include lsg`, `--include Diataxis`, `--homepage lsg`, `--search` and the readable page URL option of `lsg publish`. It SHALL accept `--output DIR`; without it, it SHALL publish into a new directory under `target/docs-site/`.

#### Scenario: Fresh build

- **WHEN** a contributor runs `mise run docs:site` twice
- **THEN** each run builds `lsg` from the checkout and writes a new directory whose `index.html` shows the `lsg` page and which holds `lsg-site.json` and the search manifest

#### Scenario: Readable page URLs

- **WHEN** `mise run docs:site` publishes the page `lsg/How To/Install with mise`
- **THEN** the page's URL relative to the site root is `lsg/How%20To/Install%20with%20mise/`, and `lsg-site.json` gives that URL as the page's `url`

### Requirement: Docs graph in release archives

Each binary archive of a release SHALL contain the docs graph: `logseq-gardener-docs/logseq/config.edn` and every page in `logseq-gardener-docs/pages/`, so a reader can open the graph in Logseq without network access.

#### Scenario: Graph in an archive

- **WHEN** dist builds a release archive
- **THEN** the archive holds `logseq-gardener-docs/logseq/config.edn`, `logseq-gardener-docs/pages/lsg.md` and `docs/dependency-sources.json`

### Requirement: Check the docs in CI

`mise run docs:check` SHALL run `mise run docs:site`, run every `sh` code block in `README.md`, and check that each relative link in `README.md`, `CONTRIBUTING.md`, `THIRD_PARTY_NOTICES.md` and every Markdown file under `docs/` or under `logseq-gardener-docs/` outside `logseq-gardener-docs/pages/` names an existing file. `mise run ci` SHALL run `mise run docs:check`. The CI job that runs `mise run ci` SHALL check out the repository's full git history, so the site's page dates come from complete history.

#### Scenario: Pull request with a broken page link

- **WHEN** a pull request adds a page that links a page missing from the graph
- **THEN** the CI job fails at `mise run docs:check`

#### Scenario: Page dates in CI

- **WHEN** CI builds the site
- **THEN** the checkout holds the full history and the build reports no diagnostic about a shallow clone

