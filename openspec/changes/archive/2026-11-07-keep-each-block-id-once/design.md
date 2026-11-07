## Context

Logseq's file parser gives each block a UUID from its `id::` property, or from `custom-id::` or `custom_id::`, which its property reader renames to `id`. `extract-pages-and-blocks` in `deps/graph-parser/src/logseq/graph_parser/extract.cljc` passes every block through `fix-block-id-if-duplicated!` in `block.cljs`, with one set of UUIDs shared by every file of the parse. A block whose UUID is already in the set goes through `fix-duplicate-id`, which gives it a new UUID and removes `id` from its properties and its text. Which block keeps the UUID depends on the order in which Logseq parses the files.

`filter-files` in `deps/graph-parser/src/logseq/graph_parser.cljs` sets that order for the desktop app and for the graph parser's command line. It sorts the files by path, then returns the paths that hold `journals/` from the last to the first, then the paths that hold `contents.`, `.edn` or `custom.css`, then every other path. The parser check runs the same code.

The SDK keeps the id on every block that gives it. `ReferenceTargets` in `transclusion.rs` treats an id that several published blocks give as ambiguous, so `lsg` shows `Block ref invalid` for a reference to it and nothing for an embed of it.

Logseq's build of the encode garden, the `gh-pages` commit `8132c95` from garden commit `083106d`, keeps `67c822b0-3271-41da-a14f-4f023a1f4556` on the `About` block of `json/5`. The `About` block of `json/jsonc` has another UUID, and the embed on `2025-03-05 Wed` shows the `About` block of `json/5`.

## Goals / Non-Goals

**Goals:**

- Each block id of a loaded garden names the block Logseq keeps it on, for every caller of `load_garden`.
- `mise run docs:parser-check` reports no difference for two pages that give a block the same id.

**Non-Goals:**

- Give the other blocks Logseq's new UUIDs. Logseq makes them at random, so no page can refer to them.

## Decisions

- `block_ids::keep_each_block_id_once` runs once in `load_garden`, after every document is parsed. It walks the documents in Logseq's order and keeps a set of the UUIDs it has seen. The work grows with the number of blocks, and each page's rendering stays the same size.
- A block's UUID comes from its `custom-id`, `custom_id` or `id` property, in that order, as `Outline::from_blocks` and `page_targets` read it. A later block with a UUID already in the set loses every property whose name, lower-cased with spaces and underscores read as hyphens, is `id` or `custom-id`; Logseq renames both to `id` and then removes `id`. The rest of the block's properties stay, and so does its place in the outline.
- The walk visits every properties block of a document, including the page's own properties and the blocks inside quotes, in source order. `page_targets` reads blocks inside quotes as targets too.
- The order uses each document's path relative to the garden, with `/` between folders. Logseq's `contents.` group also holds `.edn` and `custom.css` files, which the SDK does not load.
- `ReferenceTargets` still treats an id that two published blocks give as ambiguous. After this change only a caller that builds documents without `load_garden` can give it one.
- The parser check's deliberate mismatch is the fixture `scripts/logseq-parser/fixtures/alias-title-case`. Resolved Question 2 records why.

## Risks / Trade-offs

- [Logseq sorts paths as JavaScript strings, by UTF-16 code unit; Rust compares UTF-8 bytes] → The orders differ only between a character from U+E000 to U+FFFF and one above U+FFFF at the same position of two paths. Neither graph has such a pair.
- [Logseq's desktop app skips a file whose page title another file already has, after it has read that file's block ids] → The SDK reads every loaded file's ids, as Logseq does, before `Garden::remove_shadowed_documents` drops a file outside `pages/` and `journals/`. The two agree.
- [A block whose `custom-id::` and `id::` give different UUIDs] → The SDK uses `custom-id`, as it did before; Logseq uses the last of them in the block. Neither graph has such a block.

## Resolved Questions

### 1 - Which block keeps an id that several blocks give?

Decided by the agent without codekiln: the block Logseq's file parser reads first, in the order of `filter-files`. Logseq's build of the encode garden keeps the id on `json/5`, which that order reads before `json/jsonc`. Keeping the id on neither block, as the SDK did, shows nothing where Logseq shows a block.

### 2 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: a page that lists its own name as an alias in other letter case. The encode garden's `pages/Solvespace.md` holds `alias:: [[SolveSpace]]`; Logseq titles the page `SolveSpace` and the SDK titles it `Solvespace`. The check reports one line about one file, the difference is current, and its cause lies in how Logseq names a page, apart from block ids. It is filed as [Issue #444](https://github.com/codekiln/logseq-gardener/issues/444). The other candidates were journal references that the two sides spell with different weekdays, which depend on the garden's journal format, and a page title that a journal reference spells first, as `brew` for `pages/Brew.md`, which depends on the parse order this change adds and could change with it.

### 3 - Should the SDK remove the id from the losing block's text as well as its properties?

Decided by the agent without codekiln: the properties only. The SDK reads ids, the properties box and the parser check from the parsed properties. Published pages hide `id` and `custom-id` lines from block text already, so the text needs no change.

## Open Questions

None.
