## Why

The SDK gives a garden two links that Logseq's file parser does not ([Issue #378 — Read nested references in property values and built-in page names as Logseq does](https://github.com/codekiln/logseq-gardener/issues/378)). A nested reference in a property value such as `type:: [[Notes on [[Mldoc]]]]` refers to `Notes on [[Mldoc]]` only in Logseq, and to `Mldoc` as well in the SDK. A built-in page without a file, such as `C` from the priority `[#C]`, keeps Logseq's spelling `C` in Logseq, while the SDK spells it as the first reference does, so `#c` on an earlier page gives `c`.

## What Changes

- The SDK reads the value of every property with Logseq's property rule, which `alias::`, `aliases::` and `tags::` values already follow: a nested reference names its outer page only, and the label of a link to a file, such as `[Report.md](../assets/Report.md)`, names no page. In a block's text, both still name pages, as in Logseq.
- A page without a file that is one of Logseq's built-in pages, such as `C`, `TODO`, `NOW`, `Favorites` or `card`, takes Logseq's spelling in the garden graph, whatever the case of the reference that brings it in.
- The parser check's deliberate mismatch moves from the nested property reference, which now passes, to a page reference with a space before its slash, `[[MacOS /Applications Folder]]`, which [Issue #412](https://github.com/codekiln/logseq-gardener/issues/412) tracks.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reference-index`: the requirement "Answer linked references from one shared index" says that a nested reference in any property value supplies its outer page only, and that a file link's label in a property value supplies none.
- `garden-graph`: the requirement "Include reference-only pages as nodes" gives a built-in page without a file Logseq's spelling.

## Impact

- `crates/logseq-gardener-sdk/src/references.rs`: a block's references take the values of properties other than `alias::`, `aliases::` and `tags::` from the block's text alone, which lsdoc reads as Logseq reads a property value.
- `crates/logseq-gardener-sdk/src/page_names.rs`: a new public operation, `built_in_page_title`, returns Logseq's spelling of a built-in page.
- `crates/logseq-gardener-sdk/src/graph.rs`: a reference-only node that names a built-in page uses `built_in_page_title`.
- Published sites: only the page graph data changes. On the encode garden, the built-in page `NOW` was spelled `Now`, and one page lost a node that the label of a file link in its `source-md::` value brought in. No HTML page changes.
- `scripts/logseq-parser/`: the fixture `nested-property-reference` becomes `namespace-parent-space`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
