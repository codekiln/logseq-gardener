## Why

We need to know whether lsdoc reads our notes the way Logseq does before using it in logseq-gardener. We will compare both parsers on the same notes and check how Logseq links and nests those notes, as requested in [Issue #3 Compare Logseq parsers](https://github.com/codekiln/logseq-gardener/issues/3).

## What Changes

- Compare pinned lsdoc and mldoc revisions on the shared fixtures and available public gardens.
- Run Logseq's file-graph parser and a file-compatible graph-validator revision on the fixtures, recording identities, aliases, references, and hierarchy.
- Record disagreements, performance, source-preservation limits, licenses, and the remaining evidence needed for parser adoption.
- Describe the SDK boundaries suggested by the results.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- Extend the [parser selection requirements](../../specs/parser-boundary/spec.md) to require repeatable comparisons of note structure, references, and page relationships, with an explanation of disagreements and the limits of the tested notes.

## Impact

An isolated comparison experiment, mise tasks, parser planning requirements, and architecture documentation. Production parser dependencies and garden commands remain work for the subsequent adoption change.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
