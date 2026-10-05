## Why

The Logseq docs comparison reports false disagreements for Org page links because the reference worker applies Markdown rules. Correcting the comparison gives reviewers reliable evidence for the parser decision tracked in [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3).

## What Changes

- Apply a checked, documented format-argument repair to the archived lsdoc comparison runner.
- Exercise the actual runner with a small Org garden so a missing format argument fails a regression check.
- Regenerate the pinned Logseq docs report and update the findings to describe the corrected results.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parser-boundary`: Require format-aware public-garden reference comparisons and a runner regression check.

## Impact

Changes the isolated parser comparison experiment and its mise check. The SDK and CLI retain their current behavior. Current Logseq OG and Pengx's publishing garden remain separate pending comparisons under Issue #3.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
