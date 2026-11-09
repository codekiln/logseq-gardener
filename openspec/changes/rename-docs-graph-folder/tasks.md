# Tasks

## 1. Move the graph

- [x] 1.1 Move `docs/pages/` and `docs/logseq/` to `logseq-gardener-docs/` with `git mv`, and verify `git status` reports the files as renamed
- [x] 1.2 Point `docs:site` at `--graph logseq-gardener-docs` and `docs:parser-check` at `logseq-gardener-docs`, and verify `mise run docs:site` publishes 89 pages with no diagnostics
- [x] 1.3 Point `scripts/docs.py` at the new folder, check the relative links of the Markdown files left in `docs/`, empty `:hidden`, and verify `mise run docs:check` passes
- [x] 1.4 Update the parity scripts, `.gitignore`, `dist-workspace.toml` and code comments that name a page file, and verify `rg` finds no `docs/pages`, `docs/logseq/` or `docs/assets` path outside OpenSpec records and `CHANGELOG.md`

## 2. Documentation and instructions

- [x] 2.1 Update `README.md`, `CONTRIBUTING.md`, `THIRD_PARTY_NOTICES.md` and the graph pages that name the folder, fix the relative links in `docs/parity/advanced-queries.md`, and verify `mise run docs:check` passes
- [x] 2.2 Update `.rulesync/rules/overview.md`, run `mise exec -- rulesync generate`, and verify `mise run rulesync:check` passes
- [x] 2.3 Update the `docs-graph` Purpose, and verify `mise run openspec:validate` passes

## 3. Integration

- [x] 3.1 Compare the site built before and after the move, and verify that only the pages naming the folder, page dates and the search files built from them differ
- [x] 3.2 Run `mise run ci` and verify it passes

## Workflow follow-up

- After merge, sync the delta specs into `openspec/specs/` and archive the change, as `lsg/How To/Finish an OpenSpec Change After Merge` describes.
