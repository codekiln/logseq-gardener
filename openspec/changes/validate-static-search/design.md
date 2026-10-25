## Context

Shared search already has selected-source privacy, revisioned artifacts, safe result links and functional loader/dialog coverage. The parity harness supplies desktop and CPU/network-throttled phone profiles and unchanged 50/100ms p95 limits. Its shared-search selector is unset and its current opening wait cannot distinguish an empty container from loaded data.

## Goals / Non-Goals

**Goals:** completed-result input latency within existing budgets, honest first-open/warm cost, reproducible corpus/profile evidence and actionable failure behavior.

**Non-Goals:** production deployment and changes to publication selection.

## Decisions

Use one cancellable animation frame for pending input. Clear stale results immediately, evaluate only the newest field value, and expose ready/error/loading/searching state plus a monotonically increasing completed-result marker. Measure that completion marker after DOM/status updates, including empty/unchanged results. Retain the existing metric and report unchanged keystrokes explicitly.

Extend `scripts/parity/perf.py` and its profiles/probe/result writer. A standalone search-site option serves an existing selected publication without requiring a separate Logseq export or unrelated feed/build measurements. Record no eager search artifacts, first-open readiness/transfer/decoded size, prepared heap growth, and warm reopen requests/readiness. Keep representative results in focused documentation with disclosed corpus counts and reproduction commands.

Reuse the existing Rust privacy/anchor fixtures, pure engine/loader tests and Chromium functional proof. Add only missing offline/corrupt-data and scheduling-completion regressions. Actual host playback/navigation evidence comes from the independently reviewed Gitpa consumer.

## Risks / Trade-offs

- A synthetic corpus may hide authored-content costs → include the actual selected Gitpa corpus and a larger generic corpus.
- Observer timing can mistake stale-result clearing for completion → observe the explicit completion marker after final rendering.
- Large corpora can exceed the UI budget → use measured results to decide whether worker/chunking or query changes are required.

## Migration Plan

Rebuild search-enabled output to receive the fingerprinted controller. Existing artifact/schema and host initializer remain compatible.

## Resolved Questions

### 1 - Which budgets apply?

Retain50ms desktop and 100ms phone p95. First-open transfer/readiness limits are documented separately from per-key latency using the measured corpus sizes and throttled network.

## Open Questions

None.
