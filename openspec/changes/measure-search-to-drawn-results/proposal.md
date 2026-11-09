## Why

Two search requirements say the parity harness times each `lsg` keystroke until its results are complete, but since `match-logseq-search` the harness stops the clock when the result list is drawn, before block rows' text arrives. [Issue #371 Reconcile specs and code where they disagree](https://github.com/codekiln/logseq-gardener/issues/371) lists this as the last spec/code disagreement on search, found in [PR #365 correct publishing pages that contradict the specs](https://github.com/codekiln/logseq-gardener/pull/365). The code is right, so the specs change.

## What Changes

- `static-search-validation` "Completed-result latency measurement" becomes "Drawn-result latency measurement": the harness times each `lsg` keystroke to the first result list drawn for that input, counts keystrokes no list answered, and keeps the 50 ms desktop and 100 ms phone p95 budgets. Block rows whose text is still loading show a placeholder line, and the search browser check verifies that the text replaces it.
- `parity-comparison-harness` "Measure search latency per keystroke" states the end of a keystroke for each build: for Logseq, the last change to its results list before the next keystroke; for `lsg`, the first drawn result list for that input.
- The `lsg/Ref/Search Dialog` documentation page lists `data-lsg-search-drawn` and says the harness waits for it. The harness's failure message and the docstring of `completion_latencies` say "drawn" instead of "completed".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `static-search-validation`: keystroke latency ends at the drawn result list.
- `parity-comparison-harness`: the end of a search keystroke is stated per build.

## Impact

- `openspec/specs/static-search-validation/spec.md` and `openspec/specs/parity-comparison-harness/spec.md`, after sync.
- `scripts/parity/perf.py` and `scripts/parity/metrics.py`: wording only; the measurement and the budgets stay as they are.
- `logseq-gardener-docs/pages/lsg___Ref___Search Dialog.md` and `lsg___Ref___Host Integration API.md`: the attribute tables.
- The other item left on #371, the page initializers, is tracked in [Issue #576 Check page behaviour after a plugin navigation and align the page initializers spec](https://github.com/codekiln/logseq-gardener/issues/576).

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/How to Communicate Effectively With Me/A pointer carries a link, an id with a slug, and a reason](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___A%20pointer%20carries%20a%20link%2C%20an%20id%20with%20a%20slug%2C%20and%20a%20reason.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
