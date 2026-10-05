## Why

The Logseq docs comparison reports false disagreements for Org page links because the reference worker applies Markdown rules. Correcting the comparison gives reviewers reliable evidence for the parser decision tracked in [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3).

## What Changes

- Change the extracted lsdoc comparison program so mldoc receives each note’s Markdown or Org format when reading page links. Stop setup if the expected call has changed.
- Exercise the actual runner with a small Org garden so a missing format argument fails a regression check.
- Regenerate the pinned Logseq docs report and update the findings to describe the corrected results.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- The [parser comparison requirements](specs/parser-boundary/spec.md) now require the comparison worker to read links using each note’s format and a test that exercises that worker.

## Impact

Changes the isolated parser comparison experiment and its mise check. The SDK and CLI retain their current behavior. Current Logseq OG and Pengx's publishing garden remain separate pending comparisons under Issue #3.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
