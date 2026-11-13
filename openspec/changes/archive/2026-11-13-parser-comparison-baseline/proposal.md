## Why

Logseq Gardener needs a parser choice before it can read gardens. The [parser comparison plan](https://github.com/codekiln/logseq-gardener/blob/85eaa574062cbe8b8d816b30ec1718e3bb350986/openspec/changes/project-foundation/parser-comparison.md) calls for shared cases and recorded results, which this baseline begins to provide using the parser version in the locally checked-out Logseq source.

## What Changes

- Add a compact Markdown fixture garden covering page references, aliases, nested blocks, namespaces, tags, and text that resembles references inside code.
- Record the official `mldoc` 1.5.9 parse and reference output for those fixtures with a repeatable runner.
- Run the baseline check through `mise run ci` in GitHub Actions.
- Document the source revision, commands, and limits of this comparison step. We will choose a parser under [Issue #3 Compare Logseq parsers before adding garden operations](https://github.com/codekiln/logseq-gardener/issues/3), then add the first read-only garden command through the SDK under [Issue #4 Add the first read-only garden command through the SDK](https://github.com/codekiln/logseq-gardener/issues/4).

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
