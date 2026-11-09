## 1. Spec deltas

- [x] 1.1 Rename and modify `static-search-validation` "Completed-result latency measurement" to "Drawn-result latency measurement"
- [x] 1.2 Modify `parity-comparison-harness` "Measure search latency per keystroke" to state where a keystroke ends for each build

## 2. Harness wording and documentation

- [x] 2.1 Say "drawn" in the unanswered-keystroke failure message in `scripts/parity/perf.py` and in the docstring of `completion_latencies` in `scripts/parity/metrics.py`
- [x] 2.2 Add `data-lsg-search-drawn` to the attribute tables of `lsg/Ref/Search Dialog` and `lsg/Ref/Host Integration API`, and say the harness waits for it
- [x] 2.3 Run `mise run openspec:validate` and `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the two deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under its cleanup sub-issue of #371.
