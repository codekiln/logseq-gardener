## Why

Logseq spells an alias name as the first file it reads that refers to the name does, whether that file lists the name as an alias or refers to it in a block. A page `Run` with `alias:: [[Runtime]]` and a block `- Uses [[runtime]]` in `Notes` gives the alias the spelling `runtime` in Logseq. The SDK kept the alias's own spelling, `Runtime` ([Issue #515 — Spell an alias name as Logseq does when a block references it in another case](https://github.com/codekiln/logseq-gardener/issues/515)).

## What Changes

- An alias name that is no page's title takes the spelling of the first file in Logseq's read order that refers to it. Within that file, the last block that refers to it gives the spelling. In the properties before the first bullet, the name written last wins, as in `alias:: [[homebrew]], [[Homebrew]]`.
- A name in an `alias::` value that Logseq creates in every graph, such as `todo`, takes Logseq's spelling, `TODO`.
- The published search data lists the alias in that spelling.
- The parser check's deliberate mismatch moves from `fixtures/alias-spelling-reference`, which now agrees, to a fixture that still differs. `fixtures/alias-spelling-reference` stays in the parser check tests as a fixture that must agree.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-graph`: adds the requirement "Spell an alias name as the first file that refers to it does".

## Impact

- `crates/logseq-gardener-sdk/src/title_spellings.rs`: reads the alias names with the page titles and records their spelling.
- `crates/logseq-gardener-sdk/src/page_names.rs`: `page_alias_spellings` uses the recorded spelling.
- `crates/logseq-gardener-sdk/tests/`: the fixture `alias-spelling-reference` and a test in `og_relationships.rs`.
- `scripts/logseq-parser/`: the deliberate mismatch fixture and `test_check.py`.
- A published page whose alias a block refers to in another case shows that spelling in the search data.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
