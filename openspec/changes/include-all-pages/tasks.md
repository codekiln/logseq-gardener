# Tasks

## 1. SDK: select every page

- [x] 1.1 Add `NamespaceSelection::including_all()` and `includes_all()` in `crates/logseq-gardener-sdk/src/publishing.rs`, with exclusion and candidate validation unchanged; verify with unit tests for every scenario of "Select every page"
- [x] 1.2 Verify with an SDK test in `crates/logseq-gardener-sdk/tests/` that `select` with an all-pages selection publishes `SolveSpace` from `pages/Solvespace.md` with `alias:: [[SolveSpace]]`, leaves an excluded namespace out and withholds a `public:: false` page

## 2. CLI: `--include-all`

- [x] 2.1 Add `--include-all` to `lsg publish` in `src/cli.rs`, building the selection through the SDK and leaving journals to `--journals`; verify with `tests/publish.rs` cases for every scenario of "Publish every page and journal"
- [x] 2.2 Describe the flag in the publish usage line and Behavior section of `src/help.rs`; verify `mise run test` and `mise run clilint:check` pass
- [x] 2.3 Add the flag to `docs/pages/lsg___Ref___Publish Options.md` and `docs/pages/lsg___Ref___Selection and Visibility.md`, keeping the edits small; verify `mise run docs:check` passes

## 3. Parity harness

- [x] 3.1 Make `scripts/parity/build.py` pass `--include-all --journals` instead of derived roots, drop `outside_files` and `skipped_files`, and update `scripts/parity/test_parity.py`; verify the parity Python tests pass
- [x] 3.2 Update the `parity:build` description and `docs/pages/lsg___Ref___Parity Tasks.md`; verify `mise run ci` passes

## Workflow follow-up

- File the cleanup sub-issue "Sync and archive include-all-pages" under #350, naming the pull request and the change.
- After merge, sync the three delta specs and archive the change, following `lsg/How To/Finish an OpenSpec Change After Merge`.
- In logseq-encode-garden, switch `mise-tasks/site/build` to `--include-all` with its existing `--journals` and drop its title derivation.
