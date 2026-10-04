## Why

[Issue #3 Compare Logseq parsers](https://github.com/codekiln/logseq-gardener/issues/3) needs runnable evidence before the SDK can interpret garden notes. The existing mldoc baseline supplies shared inputs but leaves syntax differences and graph relationships untested.

## What Changes

- Compare pinned lsdoc and mldoc revisions on the shared fixtures and available public gardens.
- Run Logseq's file-graph parser and a file-compatible graph-validator revision on the fixtures, recording identities, aliases, references, and hierarchy.
- Record disagreements, performance, source-preservation limits, licenses, and the remaining evidence needed for parser adoption.
- Describe the SDK boundaries suggested by the results.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parser-boundary`: Require reproducible candidate syntax and graph relationship evidence, including classified differences and limits of the tested corpus.

## Impact

An isolated comparison experiment, mise tasks, parser planning requirements, and architecture documentation. Production parser dependencies and garden commands remain work for the subsequent adoption change.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
