## ADDED Requirements

### Requirement: Build the site with lsg from the same commit

`mise run docs:site` SHALL build `lsg` from the checkout and publish the docs graph with that binary into a directory that does not exist yet, using `--graph docs`, `--filename-format triple-lowbar`, `--include lsg`, `--include Diataxis`, `--homepage lsg`, `--search` and the readable page URL option of `lsg publish`. It SHALL accept `--output DIR`; without it, it SHALL publish into a new directory under `target/docs-site/`.

#### Scenario: Fresh build

- **WHEN** a contributor runs `mise run docs:site` twice
- **THEN** each run builds `lsg` from the checkout and writes a new directory whose `index.html` shows the `lsg` page and which holds `lsg-site.json` and the search manifest

#### Scenario: Readable page URLs

- **WHEN** `mise run docs:site` publishes the page `lsg/How To/Install with mise`
- **THEN** the page's URL relative to the site root is `lsg/How%20To/Install%20with%20mise/`, and `lsg-site.json` gives that URL as the page's `url`

### Requirement: Fail on any diagnostic

`mise run docs:site` SHALL fail when publishing reports one or more diagnostics. It SHALL print every diagnostic and keep the output directory for inspection.

#### Scenario: Unsupported syntax

- **WHEN** a page holds syntax that `lsg` reports as unsupported
- **THEN** `mise run docs:site` exits with a non-zero status and prints the diagnostic

### Requirement: Check the docs in CI

`mise run docs:check` SHALL run `mise run docs:site`, run every `sh` code block in `README.md`, and check that each relative link in `README.md`, `CONTRIBUTING.md`, `THIRD_PARTY_NOTICES.md` and every Markdown file under `docs/` outside `docs/pages/` names an existing file. `mise run ci` SHALL run `mise run docs:check`. The CI job that runs `mise run ci` SHALL check out the repository's full git history, so the site's page dates come from complete history.

#### Scenario: Pull request with a broken page link

- **WHEN** a pull request adds a page that links a page missing from the graph
- **THEN** the CI job fails at `mise run docs:check`

#### Scenario: Page dates in CI

- **WHEN** CI builds the site
- **THEN** the checkout holds the full history and the build reports no diagnostic about a shallow clone

### Requirement: Deploy to GitHub Pages

A GitHub Actions workflow SHALL build the site with `mise run docs:site` on every pull request, every push to `main` and every manual run. Each run SHALL upload the site as an artifact with `actions/upload-pages-artifact`. On a push to `main`, or a manual run on `main`, a deploy job SHALL publish that artifact with `actions/deploy-pages` to https://codekiln.github.io/logseq-gardener/. The build job SHALL have read access to repository contents only, and only the deploy job SHALL have `pages: write` and `id-token: write`. Deploy jobs SHALL run one at a time and SHALL let a running deployment finish.

#### Scenario: Pull request preview

- **WHEN** the workflow runs for a pull request
- **THEN** the run holds the site as a downloadable artifact and deploys nothing

#### Scenario: Deployment from main

- **WHEN** a pull request merges into `main`
- **THEN** the workflow deploys the site built from the merge commit, and https://codekiln.github.io/logseq-gardener/ shows its `lsg` page

### Requirement: Docs graph in release archives

Each binary archive of a release SHALL contain the docs graph: `docs/logseq/config.edn` and every page in `docs/pages/`, so a reader can open the graph in Logseq without network access.

#### Scenario: Graph in an archive

- **WHEN** dist builds a release archive
- **THEN** the archive holds `docs/logseq/config.edn` and `docs/pages/lsg.md`
