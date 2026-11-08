## Context

mldoc, Logseq's Markdown parser, reads `key:: value` for any key without spaces or colons, so `` `status:: [[Done]]` marks a finished task. `` is a property drawer with the key `` `status `` and the value `` [[Done]]` marks a finished task. ``. lsdoc, the SDK's parser, matches mldoc and reads the same property.

Logseq's graph parser then checks each name. `extract-properties` in `deps/graph-parser/src/logseq/graph_parser/block.cljs` lower-cases the name, turns spaces and underscores into hyphens, and keeps it only when `valid-property-name?` in `property.cljs` accepts `:<name>`. That function reads `{:<name> nil}` with ClojureScript's EDN reader and also rejects `"`, `|`, `^`, `(`, `)`, `{`, `}` and a leading `#`. A rejected name goes into `:block/invalid-properties`, and its value's page references are dropped with it. Both files are at the revision that `scripts/logseq-parser/source.json` pins, `6efedb75588763af256bc7dfd0ed5526dc91fe7c` of `logseq/logseq`.

The EDN reader's rules come from `cljs.tools.reader.edn` 1.3.2. `read-keyword` reads a token that ends at whitespace (including the comma) or at one of `"`, `;`, `^`, `(`, `)`, `[`, `]`, `{`, `}` and `\`. The token fails when it holds `@`, `` ` `` or `~`. `parse-symbol` splits it at its first `/`: the part after the slash must not be empty, must not start with a digit and must hold no other slash, unless the whole token is `/`.

Logseq's build of the encode garden, decoded from `static/js/db.gz` of the `gh-pages` commit `8132c95`, has 13 blocks with rejected names in 11 pages. Each has no properties and no references. On `Logseq/Entity/Asset`, the block under "Frontmatter and body" shows no text and a box that reads "Invalid property names: `logseq-entity".

## Goals / Non-Goals

**Goals:**

- `mise run docs:parser-check` reports no difference for a block that starts with code holding a property.
- A page lists no block that names it only in a property Logseq rejects, and the garden graph has no node for a rejected name.

**Non-Goals:**

- Show Logseq's "Invalid property names" warning on the published block. The SDK would need to report each block's rejected names, and the publisher would render Logseq's markup. [Issue #439](https://github.com/codekiln/logseq-gardener/issues/439) tracks it.

## Decisions

- `property_names::valid_property_name` holds Logseq's rule. It checks the characters Logseq rejects, reads the first EDN token as `read-keyword` and `parse-symbol` do, and accepts text after the token only when that text is EDN whitespace. Its unit test lists 51 names with the answer `valid-property-name?` gave for each under nbb-logseq.
- `garden::parse_document`, which every garden load uses, drops the rejected properties from each properties block. The block stays, possibly empty, so outline positions, block identifiers and source spans do not move. Every caller that reads properties, such as page titles, aliases, `public::`, queries, the properties box and the parser check, then sees what Logseq keeps.
- `Contribution::from_document` reads a block's page names from its text with `lsdoc::refs`, which parses the text again and would find the rejected property again. Before that call, it blanks the value of each property line whose name Logseq rejects, inside the block's properties spans. Blanking the value keeps the line's key and `::`, so the properties after it still read as properties.
- The publisher changes nothing. The block already showed no text; its linked references, counts and graph edges change because the SDK's index changes.
- The parser check's deliberate mismatch is the fixture `scripts/logseq-parser/fixtures/duplicate-block-id`. Resolved Question 2 records why.

## Risks / Trade-offs

- [Logseq accepts a few names whose text after the first token reads as more EDN forms, such as `a,b,c`, which reads as `{:a b, c nil}`] → The SDK rejects them. Neither the encode garden nor the docs graph has such a name, and the parser check would report one.
- [A property line inside a properties span that lsdoc did not read as a property] → `rejected_property_value` applies the same key rule as lsdoc's Markdown property reader: optional spaces, then a key with no spaces or colons, then `::`. A line without that shape is left as it is.

## Resolved Questions

### 1 - Should the SDK read the block as code, as the issue's title says?

Decided by the agent without codekiln: no. Logseq reads a property and then rejects its name. Logseq's build shows the block with no text and the "Invalid property names" warning, and its database holds no property and no reference for the block. Reading the block as code would publish text that Logseq's site does not show.

### 2 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: two pages that give a block the same `id::`. Logseq's `fix-block-id-if-duplicated!` keeps the id on the block it reads first and gives the other block a new id without its `id::` property; the SDK keeps the id on both. The check reports two lines, both about `pages/JSONC.md`, and the difference is current: the encode garden's `pages/json___5.md` and `pages/json___jsonc.md` share `id:: 67c822b0-3271-41da-a14f-4f023a1f4556`. Its cause is unrelated to property names, so it cannot disappear with this fix by accident. It is filed as [Issue #438](https://github.com/codekiln/logseq-gardener/issues/438). The other candidates were journal references that the two sides spell with different weekdays, which depend on the garden's journal format, and page titles that the two sides spell in different letter case, whose cause is not yet traced.

### 3 - Should the published block show Logseq's warning?

Decided by the agent without codekiln: not in this change. The issue is about references and graph edges, which the SDK fixes alone. The warning needs a new SDK output and publisher markup, so it has its own issue, [Issue #439](https://github.com/codekiln/logseq-gardener/issues/439).

### 4 - Should `valid_property_name` reproduce the EDN reader exactly?

Decided by the agent without codekiln: no. A full reader of the text after the first token would cover only names such as `a,b,c`, `x[a]b` and `x\a\b`, which Logseq accepts by reading them as maps with more entries. The function rejects them, and its test records them as known differences from Logseq.

## Open Questions

None.
