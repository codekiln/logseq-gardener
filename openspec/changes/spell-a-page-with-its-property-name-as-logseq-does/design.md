## Context

Logseq's file parser merges the pages a file refers to by lower-cased name (`with-ref-pages` in `deps/graph-parser/src/logseq/graph_parser/extract.cljc`), and the later reference wins. The first file Logseq reads that refers to a page creates it. `title_spellings.rs` applies this to pages with a file and to alias names. A page without a file got the spelling of the first name the graph index met, in the order of the SDK's documents, with the names of one block in alphabetical order.

`scripts/logseq-parser/check.py` showed the rule on small gardens. Each case differs from the SDK before the change and agrees after it.

| Notes.md | Logseq's spelling |
| --- | --- |
| `- Uses [[Gadget]]` with `gadget:: 1` in one block | `gadget` |
| `- Uses [[Sprocket]]`, then a block with `sprocket:: 1` | `sprocket` (the last block) |
| a block with `cog:: 1`, then `- Uses [[Cog]]` | `Cog` (the last block) |
| `- Uses [[PULLEY]]` with `Pulley:: 1` | `pulley` (a property name is lower case) |
| `- Uses [[Lever]]`, then `- Uses [[lever]]` | `lever` (the last block) |

## Goals / Non-Goals

**Goals:**

- A page without a file takes the spelling Logseq gives it when one file refers to it in more than one block or case.
- `check.py` reports no difference on `fixtures/property-name-reference`.

**Non-Goals:**

- The order in which Logseq reads several files that refer to the same page. The graph index still reads files in document order. The alias spelling already follows Logseq's read order in `title_spellings.rs`.
- The order of several names in one block's text. The graph index still lists them alphabetically.

## Decisions

- Spell the page in `GraphIndex::new`, where the index creates a page without a file. For each block, note the spelling of each name the block refers to: the first name of that page in the block's list, then the block's property names, which replace it. At the end of the block, give each page that this file created the noted spelling, so the last block wins. A page created by an earlier file, a built-in page such as `TODO` and a journal keep their spelling.
- Leave `title_spellings.rs` as it is. A page without a file has no title for it to record, and the graph index already holds the references.
- Drop a block's page reference that equals one of its property names from the SDK dump that `check.py` compares. The comparison treats the property names as properties already, because Logseq cannot tell the page reference from the property name. A block that writes `[[gadget]]` and `gadget::` in the same case differed in the same way.

## Risks / Trade-offs

- [A published graph shows a page without a file in another spelling] → It is the spelling Logseq shows.
- [The dump hides an SDK reference that the property name also gives] → The reference names the same page, and the SDK's relations are unchanged.

## Open Questions

None.
