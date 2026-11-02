## Why

The SDK and Logseq's file parser name a different namespace parent when a page reference has a space before a slash ([Issue #412 — Name the namespace parent of a reference with a space before its slash as Logseq does](https://github.com/codekiln/logseq-gardener/issues/412)). Logseq names the parent of `[[MacOS /Applications Folder]]` `MacOS `, with its trailing space, which is a page of its own. The SDK trims the space and names `MacOS`, so a garden page named `MacOS` gains a linked reference and a graph edge that Logseq does not show.

## What Changes

- A namespace parent keeps the space before its slash: the parent of `MacOS /Applications Folder` is `MacOS `, and the SDK treats it as a different page from `MacOS`. No garden page file answers to it.
- A page's linked references, its namespace children, the `(namespace …)` query and the garden graph follow that parent. A generated namespace page spelled `MacOS ` still answers to it.
- A written page name still has the spaces around it trimmed, as before. Logseq keeps those too; [Issue #421](https://github.com/codekiln/logseq-gardener/issues/421) tracks that difference.
- The parser check's deliberate mismatch moves from `[[MacOS /Applications Folder]]`, which now agrees, to `[[Release Notes ]]`, a written reference with a space inside its brackets.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `reference-index`: a new requirement says that a namespace parent keeps the space before its slash, so a reference to `MacOS /Applications Folder` counts for no page named `MacOS`.
- `garden-graph`: the requirement "Report a page's namespace parent" says that the parent keeps the space before the slash and is a node of its own.

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: new public operations `namespace_parent`, `namespace_parent_key` and `ends_in_space`; `built_in_page_title` no longer reads `A ` as the built-in page `A`.
- `crates/logseq-gardener-sdk/src/references.rs`, `graph.rs`, `query.rs` and `site/references.rs`: namespace parents use those operations.
- Published sites: on the encode garden, the page graph data changes for the pages that reference such a parent. The verification records the counts.
- `scripts/logseq-parser/`: the fixture `namespace-parent-space` becomes `page-reference-space`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
