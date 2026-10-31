## Why

GitHub CI checks only the mldoc syntax baseline. The comparison of Gardener's relationship rules with Logseq OG runs locally from a ghq checkout, so a change that breaks how the SDK names pages, resolves aliases or recognises journals can pass CI ([Issue #7 — Check SDK page relationships against Logseq fixtures in CI](https://github.com/codekiln/logseq-gardener/issues/7)). Page lookup has shipped ([Issue #4 — Page lookup through the SDK](https://github.com/codekiln/logseq-gardener/issues/4)), so the SDK now has results to compare.

## What Changes

- Copy the eight synthetic fixture notes and an expected-results file derived from the recorded current Logseq OG relationships into the SDK's tests, with the Logseq revision, runtime versions and note hashes.
- Add an SDK test that loads the notes and compares page names, titles, files, aliases, alias ambiguity, namespace parents, journal dates and pages without a file with the expected results, for the default and triple-lowbar filename formats.
- Add a check, run by `mise run ci`, that the copied notes and expected results still match the recorded evidence, and an update command that regenerates them.
- Let the SDK's graph index report a page's namespace parent, so callers and the test read it from the SDK.
- Document how the expected results are derived, reviewed and regenerated.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `parser-boundary`: Add a requirement that CI compares the SDK's page relationships with the recorded Logseq OG results.
- `garden-graph`: Add a requirement that the graph index reports a page's namespace parent.

## Impact

The SDK gains `GraphIndex::namespace_parent`. New files: the fixture notes and `expected.json` under `crates/logseq-gardener-sdk/tests/fixtures/og-relationships/`, the test `tests/og_relationships.rs`, `scripts/og_relationship_fixtures.py`, and the `parser:sdk-fixtures-check` and `parser:sdk-fixtures-update` mise tasks. `mise run ci` runs the new check, and `mise run test` runs the new test. An existing alias test reads the copied notes instead of OpenSpec change folders. `docs/parser-adoption.md` describes the new check. No dependency changes.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
