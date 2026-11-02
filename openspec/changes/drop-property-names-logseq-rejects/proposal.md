## Why

A block that starts with inline code holding a property, such as `` `status:: [[Done]]` marks a finished task. ``, gives the SDK a property named `` `status `` and a reference to `Done`. Logseq gives the block neither ([Issue #431 — Read code at the start of a block as code, not as a property, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/431)). The page `Done` then lists the block among its linked references, and the garden graph draws an edge to `Done`. Logseq's published site has neither. The encode garden has 13 such blocks in 11 files.

Logseq's Markdown parser reads the same property the SDK reads. Logseq's graph parser then drops it, because `` `status `` is not a property name Logseq accepts: the backtick cannot start an EDN keyword. The published block shows no text and a warning that names the rejected property.

## What Changes

- The SDK drops each property whose name Logseq rejects when it loads a document: its row, its page references, and the graph node for its name. Logseq keeps a name only when it reads as an EDN keyword, holds none of `"`, `|`, `^`, `(`, `)`, `{` and `}`, and does not start with `#`.
- A page loses the linked references that reach it only through such a property. In the encode garden, `Logseq/Entity/Asset` goes from 801 to 800 linked references, and 14 other pages, such as `Logseq/Entity` and `Logseq/Entity/Hardware`, lose one or more.
- The parser check's deliberate mismatch moves from `fixtures/code-span-property`, which now agrees, to `fixtures/duplicate-block-id`: two pages give a block the same `id::`, and Logseq keeps the id on one of them. [Issue #438](https://github.com/codekiln/logseq-gardener/issues/438) tracks that difference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-garden-loading`: a new requirement, "Keep only the property names Logseq accepts", says that a loaded document holds no property whose name Logseq rejects.
- `reference-index`: a new requirement, "Read no references from a property Logseq rejects", says that the page names in such a property's value give the block no reference.

## Impact

- `crates/logseq-gardener-sdk/src/property_names.rs` (new): `valid_property_name`, Logseq's rule for property names.
- `crates/logseq-gardener-sdk/src/garden.rs`: `parse_document` drops the properties Logseq rejects, so every caller of the loaded garden sees the same properties.
- `crates/logseq-gardener-sdk/src/references.rs`: `Contribution::from_document` reads no page names from a rejected property's value.
- Published sites: linked reference counts, the all-pages list, the garden graph and the reference fragments change for the pages those blocks name. The publisher's code is unchanged.
- `scripts/logseq-parser/`: the fixture `duplicate-block-id` replaces `code-span-property`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
