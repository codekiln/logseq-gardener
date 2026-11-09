# Proposal

## Why

The documentation graph lives in `docs/`, a name that is easy to confuse with Logseq's own documentation graph (https://docs.logseq.com/) and with the `docs/` folders of other repositories. During the check for [Issue #369](https://github.com/codekiln/logseq-gardener/issues/369) the wrong graph was opened because of it. [Issue #387 — Rename the docs graph folder to logseq-gardener-docs](https://github.com/codekiln/logseq-gardener/issues/387) asks for a name that says which project the graph documents.

## What Changes

- The Logseq graph moves from `docs/` to `logseq-gardener-docs/`: `docs/pages/` becomes `logseq-gardener-docs/pages/` and `docs/logseq/` becomes `logseq-gardener-docs/logseq/`.
- The files in `docs/` that are not part of the graph stay where they are: `docs/dependency-sources.json`, `docs/parity/` and `docs/logseq-reference/html/`. The three parity reports in `docs/parity/` now sit outside the graph folder, so the graph's `:hidden` list no longer names them.
- Every path that names the graph follows it: the `docs:site` and `docs:parser-check` tasks, `scripts/docs.py`, the parity scripts, `.gitignore`, the release archive contents in `dist-workspace.toml`, the RuleSync overview rule, `README.md`, `CONTRIBUTING.md`, code comments that name a page file, and the graph's own pages.
- `mise run docs:check` keeps checking the local links of the Markdown documents left in `docs/`.
- Release archives carry `logseq-gardener-docs/` as the graph and keep `docs/` for the dependency inventory and the reports.
- The site's page URLs and page text stay the same, apart from the pages that name the folder.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docs-graph`: the graph's folder is `logseq-gardener-docs/`; the data files and parity reports stay in `docs/`, outside the graph; `:hidden` applies to Markdown inside the graph folder only.
- `docs-site`: `docs:site` publishes `--graph logseq-gardener-docs`, `docs:check` checks the links of the Markdown files under `docs/`, and release archives hold the graph at `logseq-gardener-docs/`.
- `repository-readme`: the README site-link check names pages in `logseq-gardener-docs/pages/`.
- `parity-comparison-harness`: `parity:logseq-export` publishes `logseq-gardener-docs` by default, and `parity:perf --record` rewrites the Performance page in `logseq-gardener-docs/pages/`.

## Impact

- Files: 92 graph files move with `git mv`; paths change in mise tasks, Python scripts, `.gitignore`, `dist-workspace.toml`, `.rulesync/rules/overview.md`, `README.md`, `CONTRIBUTING.md`, `THIRD_PARTY_NOTICES.md`, code comments and graph pages.
- Page dates: `lsg` reads a page's first and last commit dates from git without following renames, so after the move every docs page shows the move's commit as both its created and updated date on the site until page dates follow renames.
- Archived OpenSpec changes keep the paths they were written with. Active changes keep their records too, except relative links that would otherwise break.
- Contributors who opened `docs/` in Logseq desktop add `logseq-gardener-docs/` as a new graph.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/How to Communicate Effectively With Me](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me.md)
