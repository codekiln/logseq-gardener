## 1. SDK loader

- [ ] 1.1 Add `garden.rs` tests for the spec scenarios: Markdown outside the note folders, the skipped paths (`.`, `node_modules/`, `logseq/bak/`, `logseq/.recycle/`, `logseq/version-files/`), symlinks inside and outside the note folders, `:hidden` root and folder entries, git-ignored files with and without a git work tree, and a title collision
- [ ] 1.2 Walk the garden root once in `load_garden`, giving `journals/` paths the journal kind and every other path the page kind, and applying Logseq's skip rules
- [ ] 1.3 Read `:hidden` string entries from `logseq/config.edn` in `garden_config.rs` and skip matching paths with Logseq's prefix rule
- [ ] 1.4 Outside `pages/` and `journals/`, keep only files that `git ls-files --cached --others --exclude-standard` lists when the root is in a git work tree
- [ ] 1.5 Skip a file outside the note folders whose page name matches a note-folder document, and report its path in the build output

## 2. Budgets

- [ ] 2.1 Add a `[loading]` row with `build_slowdown_pct = 5` to `scripts/parity/budgets.toml` and make `parity:perf --baseline-lsg` mark a slower whole-garden build

## 3. Verification

- [ ] 3.1 Build the whole encode garden with `mise run parity:build` and `mise run parity:compare`; record in `verification.md` that `PARAMETERS` and `AI__Coding__Tool__Plugin__Report__26__…` are published, that no page under `tools/` or `worktrees/` is, and that "Other pages" is empty
- [ ] 3.2 Compare the `PARAMETERS` page's blocks with Logseq's `gh-pages` build and record the result in `verification.md`
- [ ] 3.3 Run `parity:perf --baseline-lsg` against the previous `main` binary and record the build times
- [ ] 3.4 Run `mise run ci`

## Workflow follow-up

- File the "Sync and archive" cleanup sub-issue under Issue #252 per `docs/openspec-cleanup.md` before the Apply PR merges
