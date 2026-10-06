## Why

The parser comparison favors lsdoc, but adopting it requires a review of its source and dependency licenses. Recording that evidence now lets the SDK contributor choose a pinned dependency and preserve its required notices.

## What Changes

- Record the tested lsdoc revision, its declared license, and the license files of every package in its locked dependency graph.
- Add a reproducible local audit and a guide to the remaining SDK adoption work.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parser-boundary`: Require reproducible license evidence for the Rust parser candidate before adoption.

## Impact

Adds research evidence and documentation for [Issue #3 — Parser compatibility decision](https://github.com/codekiln/logseq-gardener/issues/3). The SDK dependency decision still requires the remaining corpus comparison and maintainer review.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
