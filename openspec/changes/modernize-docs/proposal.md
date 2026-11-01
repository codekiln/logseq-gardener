## Why

The documentation is a folder of Markdown files that readers find on GitHub, and some of it is out of date: the README and the installation guide say there is no binary release, while [GitHub Releases](https://github.com/codekiln/logseq-gardener/releases) has carried `lsg` binaries since v0.0.1. Logseq Gardener publishes Logseq gardens, so its documentation can be a Logseq graph that `lsg` publishes as a website. Readers get a site and a graph they can open in Logseq, and every build of the site exercises the publisher on a real garden.

## What Changes

- `docs/` becomes a Logseq graph that opens in Logseq 0.10 desktop. Every Markdown document under `docs/`, `help-assessment.md` and most of `CONTRIBUTING.md` are rewritten as Logseq outline pages in `docs/pages/`. Page titles follow codekiln's [Diataxis convention](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Entity___Diataxis.md): `lsg/How To/<task>`, `lsg/Tutorial/<goal>`, `lsg/Ref/<subject>`, and explanation pages named after their concept. New pages show how to install `lsg` with mise, how to publish a garden to GitHub Pages, and what `lsg` publishes like Logseq and where it differs.
- `mise run docs:site` builds `lsg` from the checkout and publishes the graph with it, with search, and fails when `lsg` reports any diagnostic. `mise run docs:check` runs it in CI, and a further check loads the graph with Logseq's own file parser and fails when Logseq and `lsg` disagree about it.
- A GitHub Actions workflow uploads the site from every pull request and deploys it from `main` to https://codekiln.github.io/logseq-gardener/ with GitHub's Pages actions.
- `lsg help` links pages on the site where it now names repository files. Links come from one base URL and the publisher's readable route rule from the separate change `readable-page-urls`, and a test fails when a linked page is missing from the built site.
- The README installs a released `lsg` with mise, globally or for one project, and links the site for everything else. `README.md`, `CHANGELOG.md`, `LICENSE` and the third-party notice files stay at the repository root, and `CONTRIBUTING.md` stays there as a short entry point.
- `scripts/parity/perf.py --record` writes its results to a graph page, so the specs that name `docs/parity/performance.md` name that page.

## Capabilities

### New Capabilities

- `docs-graph`: the Logseq graph in `docs/`: its configuration, page titles and tags, outline format, links, the files kept out of the graph, and agreement with Logseq's file parser.
- `docs-site`: building the site from the graph with no diagnostics, checking it in CI, deploying it to GitHub Pages, and shipping the graph in release archives.
- `cli-docs-links`: how CLI help links pages on the site, and the test that keeps those links pointing at built pages.

### Modified Capabilities

- `repository-readme`: the README installs released binaries with mise, links the site, and states release facts that match the published releases.
- `parity-comparison-harness`: `parity:perf --record` rewrites the `lsg/Ref/Logseq Compatibility/Performance` page.
- `published-graph-view`: the recorded graph rows go to that page.
- `published-page-outline`: the recorded zoom time goes to that page.
- `cli-static-publishing`: publishing help names the plugin reference pages on the site.

## Impact

- Documentation: the Markdown files under `docs/` become pages in `docs/pages/`. `docs/parity/*.json`, `docs/dependency-sources.json` and `docs/logseq-reference/html/` stay at their paths.
- Tasks and scripts: a new `docs:site` task; `docs:check`, `docs:links` and `scripts/docs.py` read the graph; `scripts/parity/perf.py` writes Logseq outline Markdown; references in `scripts/parity/`, `.mise/tasks/parity/`, `.mise/tasks/release/store-app-key`, `fnox.toml`, `THIRD_PARTY_NOTICES.md` and the experiments' READMEs follow the pages.
- CI and repository settings: a new Pages workflow, full git history in the `ci` job, and GitHub Pages enabled with the GitHub Actions source.
- CLI: `src/help.rs` prints site URLs; `tests/publish.rs` and the saved help assessment change with it.
- Release archives: they keep `docs`, now a graph; `dist-workspace.toml` drops `help-assessment.md`.
- Agent instructions: `.rulesync/rules/overview.md` and the README assessment skills point at the new pages.
- Order: `readable-page-urls` ([Issue #324 Publish pages at readable namespace URLs](https://github.com/codekiln/logseq-gardener/issues/324)) merges before the deployment switches the site to readable URLs and before help links to it.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Balance/Risk vs Reward/wrt Privacy - Make Things As Public As They Can Be](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Balance___Risk%20vs%20Reward___wrt%20Privacy%20-%20Make%20Things%20As%20Public%20As%20They%20Can%20Be.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don%27t%20Repeat%20Yourself%20DRY.md)
- [My/Pref/Writing](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
