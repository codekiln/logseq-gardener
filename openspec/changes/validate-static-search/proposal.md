## Why

The shared search controller adds a fixed 60ms input delay, exceeding the existing 50ms desktop keystroke budget before query work begins. Search validation needs to include cold loading and completed results on real selected corpora.

## What Changes

- Coalesce input evaluation into animation frames and expose explicit loading/completion state for consumers and measurements.
- Extend the existing parity harness with shared search controls, first-open/warm transfer and readiness evidence, and standalone selected-site measurement.
- Reuse existing privacy/failure/browser fixtures and record small, large and actual host corpus measurements against unchanged latency budgets.

## Capabilities

### New Capabilities

- `static-search-validation`: Reproducible completed-result and loading measurements with documented budgets and failure evidence.

### Modified Capabilities

None.

## Impact

The shared controller scheduling and existing parity performance harness change. The selected-source contract and host navigation APIs remain stable. Implements [issue96 — static search validation](https://github.com/codekiln/logseq-gardener/issues/96); Gitpa integration remains separately reviewed.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
