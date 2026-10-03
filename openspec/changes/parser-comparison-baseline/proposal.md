## Why

Logseq Gardener needs a parser choice before it can read gardens, but the current comparison plan has no runnable cases or recorded reference output. A small baseline using the parser version in the locally checked-out Logseq source will give later candidates the same inputs and a result they can reproduce.

## What Changes

- Add a compact Markdown fixture garden covering page references, aliases, nested blocks, namespaces, tags, and text that resembles references inside code.
- Record the official `mldoc` 1.5.9 parse and reference output for those fixtures with a repeatable runner.
- Run the baseline check through `mise run ci` in GitHub Actions.
- Document the source revision, commands, and limits of this first comparison step. Keep parser selection and garden commands in issue #3 and issue #4.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parser-boundary`: Require a reproducible reference baseline as evidence for the parser comparison before adopting a parser or adding garden operations.

## Impact

The change adds files under the comparison experiment, mise tasks to check and update its results, and a check in `mise run ci`. It updates the parser-boundary planning requirement. It does not change the SDK, CLI, production dependencies, or existing gardens.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Pref/Writing/Use Plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
