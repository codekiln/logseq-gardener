## Why

When two blocks give the same `id::`, Logseq keeps the id on the block it reads first and gives the other block a new id ([Issue #438 — Keep a duplicated block id on one block only, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/438)). The SDK keeps the id on both blocks, so `lsg` treats the id as ambiguous: a block reference to it shows `Block ref invalid`, and a block embed of it shows nothing. Logseq's published site shows the block it kept.

The encode garden's `pages/json___5.md` and `pages/json___jsonc.md` both give their `## About` block `id:: 67c822b0-3271-41da-a14f-4f023a1f4556`, and the journal `2025-03-05 Wed` embeds it. Logseq's build shows the `About` block of `json/5` in that embed; `lsg` shows an empty space.

## What Changes

- When the SDK loads a garden, the block Logseq reads first keeps a shared `id::`. Each later block with that id loses its `id`, `custom-id` and `custom_id` properties, as Logseq removes them. Logseq reads journals first, from the last path to the first, then the files whose path holds `contents.`, then every other file by path, and the blocks of a file in source order.
- A block reference or embed of a shared id shows the block Logseq kept. In the encode garden, the embed on `2025-03-05 Wed` shows the `About` block of `json/5`.
- The parser check's deliberate mismatch moves from `fixtures/duplicate-block-id`, which now agrees, to `fixtures/alias-title-case`: a page that lists its own name as an alias in other letter case, which Logseq titles with the alias's spelling. [Issue #444](https://github.com/codekiln/logseq-gardener/issues/444) tracks that difference.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-garden-loading`: a new requirement, "Keep each block id on the block Logseq reads first", says which block of a loaded garden keeps an id that several blocks give.

## Impact

- `crates/logseq-gardener-sdk/src/block_ids.rs` (new): Logseq's file parse order and the removal of repeated ids.
- `crates/logseq-gardener-sdk/src/garden.rs`: `load_garden` applies it after parsing every document, so references, embeds, the outline, the properties box, search and the parser check see one owner for each id.
- Published sites: block references and embeds of a shared id show the kept block, and the build reports no diagnostic for them. The publisher's code is unchanged.
- `scripts/logseq-parser/`: the fixture `alias-title-case` replaces `duplicate-block-id`, and its test follows.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
