## 1. Move selection into the SDK

- [ ] 1.1 Add a `publication` module and move page selection, `public:: false` withholding, route assignment, journal identities and alias collection out of `site/mod.rs` into it; the publisher calls it
- [ ] 1.2 Check that the whole encode garden built without plugins gives the same `lsg-site.json` revision as `main`, and record both revisions in the PR

## 2. Publication view

- [ ] 2.1 Build the view (pages with id, title, url, kind; blocks with page-scoped ids, text without property lines, children) borrowing from the loaded garden
- [ ] 2.2 Filter page properties per plugin with the published-properties visibility rules, and leave `blocks` empty for plugins that do not ask for blocks
- [ ] 2.3 Compute the view revision from its content and the `lsg` version
- [ ] 2.4 Tests: excluded and withheld pages absent, hidden properties absent, no source paths, shared UUIDs fall back to paths, identical views for identical builds

## 3. Contributions in the SDK

- [ ] 3.1 Add the `contributions` module: `BuildPlugin`, `PluginView`, `Contribution` (rows and metadata) and inline node types with serde for the protocol shapes
- [ ] 3.2 Validate contributions against each plugin's view (kinds, page and block ids, link targets, URL schemes, empty rows, metadata keys, metadata size, module required) with one test per rejection
- [ ] 3.3 Resolve row links to page URLs and block anchors, and return the set of block anchors target pages must emit
- [ ] 3.4 Add `SiteOptions::build_plugins` and reject an id shared with a manifest

## 4. Process plugins

- [ ] 4.1 Read and check the manifest `build` section; make `module` and `activation` optional with it and reject a manifest with neither `module` nor `build`
- [ ] 4.2 Run a process plugin as a `BuildPlugin`: start the command in the manifest's folder, stream the request to stdin, read stdout up to 64 MB, pass stderr through, enforce the timeout and kill on expiry
- [ ] 4.3 Tests with fixture scripts: success, non-zero exit, malformed JSON, wrong `apiVersion`, oversized output, timeout, and one process start per build

## 5. Publisher

- [ ] 5.1 Build the view and start plugin threads right after selection only when there is a build plugin; join and validate before the first page renders
- [ ] 5.2 Render derived rows as outline bullets in `.lsg-derived[data-lsg-plugin]`, after the target block's children or at the end of the outline, and emit the anchors links need
- [ ] 5.3 Merge contributed metadata into `data-lsg-page`; add no host or metadata element when no plugin has a module, and list only plugins with a module in `lsg-site.json`
- [ ] 5.4 Tests: reference fragments, search files and graph data identical with and without row contributions; embeds of a target block show authored children only; output identical for a plugin that contributes nothing; deterministic output with plugins finishing in different orders

## 6. CLI and docs

- [ ] 6.1 Report build plugin failures and rejected contributions from `lsg publish` with exit one and no output directory; update publishing help and the clilint checks, and review the help examples per `help-assessment.md`
- [ ] 6.2 Document the `build` section, the view, the protocol, the contribution kinds, the indexing rules and the trust model in `docs/publishing.md`; add the `publication` and `contributions` modules to `docs/architecture.md`
- [ ] 6.3 Add an SDK example that registers an in-process build plugin

## 7. Budgets and CI

- [ ] 7.1 Add fixture build plugins under `tests/fixtures/plugins/`: a Node no-op (with and without blocks) and a Node plugin that contributes rows and metadata
- [ ] 7.2 Add a `[build_plugins]` table to `scripts/parity/budgets.toml` and `parity:budgets` checks on the fixture garden: derived rows per page, first paint, pages without contributions unchanged
- [ ] 7.3 Add a `parity:build-bench` task and a CI job that checks out a pinned commit of the encode garden, builds it without plugins, with the in-process no-op example and with the process no-op fixture with and without blocks, and checks added time per page, `lsg` peak memory and identical output for the no-op plugins
- [ ] 7.4 Record the whole-garden measurements in the PR

## 8. Coordination

- [ ] 8.1 Comment on [issue #195](https://github.com/codekiln/logseq-gardener/issues/195) with the shipped view, protocol and contribution kinds and their differences from Gitpa's `PublicationView`, for [Gitpa issue #93](https://github.com/codekiln/gitpa/issues/93) to adopt
