## Context

Logseq's file parser reads every property value with mldoc's property reference rule (`parse-property` in `deps/graph-parser/src/logseq/graph_parser/text.cljs`, at the revision `mise run docs:parser-check` pins). It keeps the top-level page references, tags and nested references of the value, so a nested reference `[[Outer [[Inner]] Name]]` names `Outer [[Inner]] Name` only, and the label of a link to a file names nothing. In a block's text, Logseq walks every inline, so a nested reference names the inner page too, and a file link's label names a page.

lsdoc already reads a properties block this way when it collects the references of a block's text. The SDK's `Contribution::from_document` also parsed each value of a property other than `alias::`, `aliases::` and `tags::` as ordinary text, which added the inner page and the file link's label.

Logseq creates its built-in pages before it reads any file (`built-in-pages-names` in `deps/db/src/logseq/db/default.cljs`), so a reference to `c` finds the page `C`. The SDK's garden graph spelled a page without a file as the first reference that names it.

## Goals / Non-Goals

**Goals:**

- `mise run docs:parser-check` reports no difference for nested references in property values or for the spelling of built-in pages without a file.
- One SDK operation answers Logseq's spelling of a built-in page, for the graph and for any later caller.

**Non-Goals:**

- Give a built-in page that has a file in the garden Logseq's spelling.
- Name the namespace parent of `[[MacOS /Applications Folder]]` as Logseq does. [Issue #412](https://github.com/codekiln/logseq-gardener/issues/412) tracks it, and the parser check's fixture uses it as its deliberate mismatch.

## Decisions

- A block's references take the values of properties other than `alias::`, `aliases::` and `tags::` from the block's text, which lsdoc reads with mldoc's property reference rule. That rule also covers file links, quoted values and `{{query [[Page]]}}` in a value, so the SDK keeps no second copy of it. The three list properties still add the plain names that `linkable_property_names` reads.
- `page_names::built_in_page_title` holds Logseq's sixteen built-in page names in Logseq's spelling. The graph's `BUILT_IN_PAGES` list, which also holds `started`, keeps deciding which nodes the built-in pages setting hides.
- The parser check's deliberate mismatch is the namespace parent of `[[MacOS /Applications Folder]]`: Logseq names `MacOS ` with its trailing space, and the SDK names `MacOS`.

## Risks / Trade-offs

- [A property value that lsdoc does not read as a property, while the SDK's AST does, would lose its references] → Both read the same lsdoc parse; the parser check agrees on the docs graph and finds no new difference on the encode garden.

- [The `reference-index` delta restates the requirement as `read-task-markers-and-alias-values` modifies it, and that change is not synced yet] → Sync and archive `read-task-markers-and-alias-values` ([Issue #380](https://github.com/codekiln/logseq-gardener/issues/380)) before this change, so this delta replaces the newer text.

## Resolved Questions

### 1 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: the namespace parent of a reference with a space before its slash, `[[MacOS /Applications Folder]]`. It is a one-line fixture, its every reported line names the same file and the same page, the encode garden has it in several journals, and it needs no file of its own to show the difference. Issue #412 records it.

### 2 - Does `STARTED` keep Logseq's spelling too?

Decided by the agent without codekiln: no. `STARTED` is no built-in page at the pinned Logseq revision, so the first reference spells it, as Logseq does. The graph still treats it as built-in for the built-in pages setting, as before.

## Open Questions

None.
