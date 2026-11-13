## Why

The existing relationship evidence uses Logseq 0.9.8. Before choosing a parser for page lookup, reviewers need to know whether current Logseq OG interprets the same notes differently. [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3) tracks this prerequisite.

## What Changes

- Compare the current Logseq OG file parser with the historical parser using the shared Markdown fixtures and configured namespace naming.
- Pin the source revisions and mldoc runtime, save repeatable page and block relationship results, and explain differences from the existing snapshot.
- Add mise commands to reproduce the comparison and document what the results mean for SDK page lookup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parser-boundary`: Add [requirements for a current Logseq OG relationship comparison](specs/parser-boundary/spec.md) covering page identity, aliases, journals, hierarchy, and references.

## Impact

The experiment runs archived upstream source from the existing ghq checkout and locked npm dependencies. The comparison adds evidence for the SDK's page lookup design. Pengx's publishing corpus remains pending in [Issue #3 — Finish parser compatibility comparison](https://github.com/codekiln/logseq-gardener/issues/3).

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
