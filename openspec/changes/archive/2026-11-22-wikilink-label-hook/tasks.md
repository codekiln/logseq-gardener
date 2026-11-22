# Tasks

## 1. Pre-commit fixer

- [x] 1.1 Add the `logseq-wikilink-labels` command to a new `pre-commit` section in `lefthook.yml`, keeping `post-merge` and `post-checkout`, and verify with `lefthook validate`
- [x] 1.2 Run `mise run logseq:fix-wikilink-labels` with no arguments to fix existing pages, then verify `mise run logseq:fix-wikilink-labels --check` exits 0
- [x] 1.3 Commit the change through the hook and verify the pre-commit command runs and passes

## Workflow follow-up

- After the implementation PR merges, sync the `docs-graph` delta, archive the change and submit the cleanup PR, following `lsg/How To/Finish an OpenSpec Change After Merge`.
