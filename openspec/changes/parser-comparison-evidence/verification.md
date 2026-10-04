# Verification

- `mise run parser:comparison-check` passed after a clean npm install and pinned source build.
- Adding a page reference to `Graph Cases.md` caused the check to fail. Restoring the file made it pass.
- Repeated fixture runs produced unchanged snapshots, including normalized graph relationships and validator assertions.
- The upstream garden runner completed both pinned public corpora. Corpus hashes and raw findings are recorded beside the reports.
- The standalone memory probe completed against the pinned docs corpus.
- `mise run ci` and `mise exec -- openspec validate --all --strict` passed.

Current Logseq OG, Pengx's public garden, publication filtering, query evaluation, a full SDK graph comparison, and editing behavior remain outside this recorded result. [Issue #3 Compare Logseq parsers](https://github.com/codekiln/logseq-gardener/issues/3) remains open.
