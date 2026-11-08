## Why

Logseq counts a block's property name as a reference to the page of that name, in lower case. Logseq merges the pages a file refers to by name, and the later reference wins. A block `- Uses [[Gadget]]` with the property `gadget:: 1` gives the page the spelling `gadget`. The SDK spelled it `Gadget`, the spelling of the first reference it read ([Issue #521 — Spell a page as Logseq does when a block writes a property name and a reference to it in another case](https://github.com/codekiln/logseq-gardener/issues/521)).

## What Changes

- A page without a file takes the spelling of the last block, in the first file that refers to it, that names it. A property name counts as a reference in lower case. Within one block, the property name comes after the names written in the text, so it wins.
- The parser check no longer reports a block's page reference that has the same name as one of the block's properties. Logseq lists a property name as a page the block refers to, and `check.py` compares those names as properties, so the reference of the same name is the same page. The issue reported this as a second difference; it appeared for a reference in the same case too.
- `fixtures/property-name-reference` now agrees and gains files that vary the order of the blocks and the case of the property name. The parser check has no deliberate mismatch left.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-graph`: adds the requirement "Spell a page without a file as the last block that names it does".

## Impact

- `crates/logseq-gardener-sdk/src/graph.rs`: the graph index spells a page that a file creates from the last block that names it.
- `crates/logseq-gardener-sdk/tests/relationships/mod.rs`: the parser check's SDK dump leaves out a block's page reference that is also one of its property names.
- `scripts/logseq-parser/`: the fixture and `test_check.py`.
- A published graph shows a page without a file in the spelling Logseq gives it.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
