## Context

Logseq's parser adds the namespace parents of every name a block writes to the block's references, so each parent becomes a page. The alias value in a properties block is such a name. In the SDK, `Contribution::from_document` in `crates/logseq-gardener-sdk/src/references.rs` kept the parents of a name only when the parent resolved to a page, and `GraphIndex::new` in `graph.rs` made a node for a parent only through the names that open no page. A name that opens a page through an alias therefore brought in no parent that has no file.

## Goals / Non-Goals

**Goals:**

- Pages without a file that Logseq creates for the namespace parents of an alias name exist in the SDK, in the global graph and in the page graphs of the pages that write the name.
- `mise run docs:parser-check` reports no `page without a file` difference of this kind.

**Non-Goals:**

- The six `card-*` pages and four date pages that the parser check still reports on the encode garden. They follow other parts of Logseq's parser.
- Changes to the publisher.

## Decisions

- `GraphRefs` gains `unresolved_parents`: the namespace parents of the names a block writes that name no known page, as written. `Contribution::from_document` fills it for every written name, so a parent of an alias name is kept. `GraphIndex::new` makes each one a node, in the same place it makes nodes for the other unresolved names.
- The publisher reads the graph through the SDK and needs no change. The global graph files and the page graph files of the pages that write such a name change.
- The parser check's deliberate mismatch moves to `fixtures/trailing-slash-name`: a link `[[a/b/]]`, which Logseq reads as the page `a/b` with the parent `a`, and the SDK reads as `a/b/` with no parent. It is filed as [Issue #495](https://github.com/codekiln/logseq-gardener/issues/495). `fixtures/empty-page-file`, the earlier choice, now agrees after #477.

## Risks / Trade-offs

- [A parent that Logseq creates is dropped when its name is excluded or private] → The existing rule for reference-only nodes applies unchanged: references from pages outside the selection create no node.

## Open Questions

None.
