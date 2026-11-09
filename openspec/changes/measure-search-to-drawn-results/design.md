## Context

`static-search-validation` "Completed-result latency measurement" and `parity-comparison-harness` "Measure search latency per keystroke" say the harness times each keystroke until the results are complete. [PR #365 correct publishing pages that contradict the specs](https://github.com/codekiln/logseq-gardener/pull/365) found that `scripts/parity/perf.py` waits for `data-lsg-search-drawn` instead, and [Issue #371 Reconcile specs and code where they disagree](https://github.com/codekiln/logseq-gardener/issues/371) asks which side is right.

The search dialog in `crates/logseq-gardener-sdk/src/site/search-ui.mjs` sets two attributes on `#lsg-search-results`. `data-lsg-search-drawn` advances when it draws the result list for the input; a block row whose record chunk is still loading shows its page title and a placeholder line. `data-lsg-search-complete` advances once every drawn row has its text. A no-match result advances both.

The `match-logseq-search` change moved the harness to the drawn signal on purpose. Its [verification-performance.md](https://github.com/codekiln/logseq-gardener/blob/main/openspec/changes/archive/2026-10-27-match-logseq-search/verification-performance.md) records the phone warm keystroke p95 at 541 ms timed to complete against the 100 ms budget, and 35 ms timed to the drawn list. The 400 ms or more is the phone profile's network wait for a record chunk, not the dialog's response to the key. That change updated the harness and its design but left both requirements as they were.

## Goals / Non-Goals

**Goals:**

- Both requirements say what the harness measures, and the committed keystroke budgets keep measuring how fast the dialog answers a key.

**Non-Goals:**

- A separate, budgeted time to complete block text per keystroke. The cold keystroke budget already bounds shard downloads, and the per-query table reports bytes downloaded.

## Decisions

- The specs change and the harness stays. A keystroke ends when the dialog first draws a list for that input or a longer input that starts with it, as `metrics.completion_latencies` computes.
- "Completed-result latency measurement" is renamed "Drawn-result latency measurement". The requirement names both attributes so a reader can tell which one the budget uses and which one marks the text arriving.
- The new scenarios point at checks that already exist: `placeholder_rows` in `scripts/search/browser.py` holds the chunk requests, checks the placeholder rows, then checks that `data-lsg-search-complete` advances and the text replaces them.
- `perf.py`'s failure message for unanswered keystrokes and the docstring of `metrics.completion_latencies` say "drawn" instead of "completed". The Search Dialog and Host Integration API reference pages list `data-lsg-search-drawn`.

## Risks / Trade-offs

- [A slow chunk download no longer shows in the keystroke budget] → The block rows still show their page title at once, and the per-query table in the performance report lists each query's download size. A reader who waits for block text on a slow network sees it fill in when its chunk arrives.

## Resolved Questions

### 1 - Should the harness time a keystroke to completed results or to the drawn list?

Decided by the agent without codekiln: to the drawn list, and the specs change. The keystroke budgets of 50 ms desktop and 100 ms phone measure how quickly the dialog answers a key. Timed to complete, a phone keystroke includes a 150 ms-latency network round trip for each record chunk, which no dialog code can meet within 100 ms, so the budget would measure the network profile. Logseq, which the budgets compare against, has every block in memory and needs no such fetch. `match-logseq-search` made this choice in its verification and left the requirement text behind; this change brings the text up to date.

### 2 - Does the drawn signal still count queries with no results?

Decided by the agent without codekiln: yes. `search-ui.mjs` calls `complete()` for a no-match or invalid query, and `complete()` advances `data-lsg-search-drawn` too, so those keystrokes are measured and only keystrokes no drawn list answered count as unanswered.

## Open Questions

None.
