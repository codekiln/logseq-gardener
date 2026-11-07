## Why

Logseq's published site shows no bullet beside a block that has no text, and `lsg` shows one ([Issue #445 — Hide the bullet of a block with no text, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/445)). On `Logseq/Entity/Asset` in the encode garden, the block under "Frontmatter and body" holds only Logseq's "Invalid property names" warning; Logseq's build draws no bullet there, and `lsg` draws one. The same rule covers every block with no text, such as a bare `-` line between two blocks.

## What Changes

- The SDK reports, for each outline block, whether it has no text by Logseq's rule: an empty first line or one holding only a task marker, no property Logseq shows, and nothing after them but horizontal rules.
- The SDK says which blocks Logseq shows without a bullet: a block with no text and no children that is neither the first nor the last block of the outline.
- The publisher marks those blocks, and the shared stylesheet hides their bullet until the pointer is over the block, as Logseq does.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `outline-blocks`: a new requirement, "Report the blocks whose bullet Logseq hides", says when a block has no text and when Logseq hides its bullet.
- `published-page-outline`: a new requirement, "Hide the bullet of a block with no text", says how a published page shows such a block.

## Impact

- `crates/logseq-gardener-sdk/src/outline.rs`: `OutlineBlock::empty` and `Outline::hides_bullet`.
- `crates/logseq-gardener-sdk/src/page_properties.rs`: `is_hidden_block_property`, the block properties Logseq hides.
- `crates/logseq-gardener-sdk/src/site/render.rs` and `site/outline.css`: the `lsg-empty` class on the block's list item and the rule that hides its bullet. Pages load no new script.
- Published sites: pages with such a block change their HTML; every page loads the changed shared stylesheet.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
- [My/AI/Rule/How to Communicate Effectively With Me/A pointer carries a link, an id with a slug, and a reason](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___How%20to%20Communicate%20Effectively%20With%20Me___A%20pointer%20carries%20a%20link%2C%20an%20id%20with%20a%20slug%2C%20and%20a%20reason.md)
