## Why

Logseq spells an alias name that a page writes twice in different case with the last spelling. With `alias:: [[homebrew]], [[Homebrew]]`, Logseq reads `Homebrew`, and the SDK read `homebrew` ([Issue #509 — Spell a repeated alias name as Logseq does](https://github.com/codekiln/logseq-gardener/issues/509)). The encode garden has five such pages, for example `alias:: [[CUI space]], [[CUI Space]], ...` in `pages/Security___CUI space.md`.

## What Changes

- An alias name written more than once in one `alias::` value, in different case, takes its last spelling and keeps the position of its first. `[[homebrew]], [[Homebrew]]` gives `Homebrew`.
- The parser check's deliberate mismatch moves from `fixtures/alias-spelling-case`, which now agrees, to `fixtures/alias-spelling-reference`: a page `Run` with `alias:: [[Runtime]]` and a block `- Uses [[runtime]]` elsewhere, which Logseq spells `runtime` and the SDK spells `Runtime`. [Issue #515](https://github.com/codekiln/logseq-gardener/issues/515) tracks that difference.
- The published search data lists the alias as `Homebrew`, not `homebrew`, for a page that writes both.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-graph`: adds the requirement "Spell a repeated alias name with its last spelling".

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: `page_alias_spellings` keeps the last spelling of a repeated name.
- `crates/logseq-gardener-sdk/tests/`: the fixture `alias-spelling-case` and a test in `og_relationships.rs`.
- `scripts/logseq-parser/`: the fixture `alias-spelling-reference` and `test_check.py`.
- A published page that writes an alias twice shows the last spelling in the search data.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
