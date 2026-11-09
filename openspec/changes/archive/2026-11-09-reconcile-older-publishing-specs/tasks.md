## 1. Spec deltas

- [x] 1.1 Modify `local-static-publishing` "Render supported syntax and report unsupported references" to defer to the query, block reference and macro requirements
- [x] 1.2 Modify `published-graph-view` "Load graph data only on the graph page" to allow an open Page graph card to fetch the graph files
- [x] 1.3 Modify `static-search-contract` "Selected-source search corpus" to let a YouTube timestamp add its displayed time
- [x] 1.4 Modify `publishing-namespace-selection` "Explicit namespace inclusion" and `cli-static-publishing` "Explicit SDK-backed publishing" for `--include-all`
- [x] 1.5 Modify `authored-page-sequences` "Present accessible sequence navigation" for hidden `prev` and `next` keys

## 2. Code fixes

- [x] 2.1 Add a test in `references.rs` for a hidden property naming a namespace child, see it fail on `origin/main`, and stop leading property blocks from naming the parents of hidden rows
- [x] 2.2 Add a test in `site_build_plugins.rs` that changes only contributed metadata, see it fail on `origin/main`, and add page plugin values and the host script to the revision digest
- [x] 2.3 Add a test in `site_search.rs` that a timestamp is the only macro that adds search text

## 3. Follow-up

- [x] 3.1 Open [Issue #576 Check page behaviour after a plugin navigation and align the page initializers spec](https://github.com/codekiln/logseq-gardener/issues/576)
- [x] 3.2 Run `mise run openspec:validate` and `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the six deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #371.
