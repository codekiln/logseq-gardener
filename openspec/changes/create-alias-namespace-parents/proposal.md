## Why

When a page lists a namespaced name as an alias, Logseq creates the namespace parents of that name as pages without a file, and the SDK creates none ([Issue #462 — Create the namespace parents of an alias name, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/462)). With `pages/AGI.md` holding `alias:: [[AI/General/Intelligence]]`, Logseq's file parser has the pages `ai` and `ai/general`; the SDK has neither, and `mise run docs:parser-check` reports 75 such missing pages on the encode garden.

The cause is in the graph. Logseq's parser adds the namespace parents of every name a block writes to the block's references, and the alias value is a name the page's properties block writes. The SDK adds the parents of a name that opens no page, but drops them when the name opens a page through an alias, so `AI/General` and `AI` never become pages.

## What Changes

- A name a block writes brings in its namespace parents as pages without a file, whether or not the name itself opens a page. The parents become nodes of the global graph and join the page graphs of the pages that write the name, as in Logseq.
- The parser check's deliberate mismatch moves from `fixtures/alias-namespace-parents`, which now agrees, to `fixtures/empty-page-file`: an empty page file, from which Logseq reads no page and the SDK reads one. [Issue #468](https://github.com/codekiln/logseq-gardener/issues/468) tracks that difference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-graph`: "Include reference-only pages as nodes" adds the namespace parents of a name that opens a page through an alias.

## Impact

- `crates/logseq-gardener-sdk/src/references.rs`: `GraphRefs::unresolved_parents` is new. It holds the namespace parents of the names a block writes that open no page.
- `crates/logseq-gardener-sdk/src/graph.rs`: `GraphIndex::new` makes each of those parents a node.
- The publisher's code is unchanged. The global graph files and the page graph files of the pages that write such a name change; pages load no new data.
- `crates/logseq-gardener-sdk/tests/`: the new fixture `alias-namespace-parents` and a test in `og_relationships.rs`.
- `scripts/logseq-parser/`: the fixture `empty-page-file` replaces `alias-namespace-parents`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
