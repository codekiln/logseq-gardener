## Why

The SDK and Logseq's file parser name a different page when a page reference has a space inside its brackets ([Issue #421 — Keep the spaces inside a page reference's brackets as Logseq does](https://github.com/codekiln/logseq-gardener/issues/421)). Logseq reads `[[Release Notes ]]` as a reference to the page `Release Notes `, with its trailing space, which is a page of its own. The SDK trims the space, so the page `Release Notes` lists that block in its linked references and the garden graph draws an edge that Logseq does not draw.

## What Changes

- A written page name keeps the spaces at its ends: `[[Release Notes ]]` refers to `Release Notes ` and `[[ Release Notes]]` to ` Release Notes`, and the SDK treats each as a different page from `Release Notes`. No garden page file answers to such a name, and neither does a generated namespace page spelled without the space.
- A page's linked references, its backlink count and the garden graph follow that name. A namespace parent keeps the spaces at both of its ends too, so `[[ Notes/Setup]]` has the parent ` Notes`.
- The rendered link still opens the page `Release Notes`, and now shows `Release Notes` without the space, as Logseq's link does. Before, it showed the name with its spaces.
- The parser check's deliberate mismatch moves from `[[Release Notes ]]`, which now agrees, to a block that starts with inline code holding a property, `` `status:: [[Done]]` ``, which [Issue #431](https://github.com/codekiln/logseq-gardener/issues/431) tracks.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reference-index`: a new requirement, "Keep the spaces inside a page reference's brackets", says that a written name keeps the spaces at its ends, so a block that links `[[Release Notes ]]` is no linked reference of `Release Notes`. It sits beside "Keep the space before a namespace parent's slash", which the change `name-namespace-parent-with-space` adds.

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: `namespace_parent_key` becomes `reference_key`, which keeps spaces at both ends of a name, and `ends_in_space` becomes `has_outer_space`.
- `crates/logseq-gardener-sdk/src/references.rs`: `Contribution::from_document` stops trimming written names.
- `crates/logseq-gardener-sdk/src/graph.rs`, `query.rs` and `site/references.rs`: the callers use the renamed operations, and the graph keeps the spaces of every name.
- `crates/logseq-gardener-sdk/src/site/render.rs`: a page link trims the name it resolves and shows.
- Published sites: a page loses the linked references that reach it only through a reference with a space inside its brackets. The encode garden has none outside code blocks.
- `scripts/logseq-parser/`: the fixture `code-span-property` replaces `page-reference-space`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
