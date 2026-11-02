## Context

Logseq 0.10's `block-control` in `src/main/frontend/components/block.cljs` draws a block's bullet unless all of these hold:

- `block-content-empty?` holds: the block has no properties, or only properties Logseq hides; its title is empty; and its body holds nothing but horizontal rules.
- The block has no children and is not collapsed. A block with children always keeps its bullet, because `collapsable?` holds for it.
- The block is not `:block.temp/top?` or `:block.temp/bottom?`. `block-list` sets those flags on the first and last block of a blocks container, which lists the top-level blocks of a page, an embed or a reference group. A page's page-properties block and a heading line written without a dash are blocks of that list, so either one ahead of the first bullet takes the first place. Nested blocks render through `block-children`, which sets neither flag.
- The reader is not editing the block, document mode is off, `:ui/show-empty-bullets?` is false, and the pointer is not over the block's row.

When the rule holds, Logseq renders an empty `span.bullet-container`. Moving the pointer over the block's row sets `*control-show?`, and the bullet comes back.

`lsg` draws a bullet for every block in `site/render.rs`. `outline.js`, `sidebar.js` and `slides.js` find a block by its `.bullet` child, so a block needs that element even when the bullet is not visible.

## Goals / Non-Goals

**Goals:**

- The SDK decides which blocks Logseq shows without a bullet, from the page's parsed blocks.
- A published page hides those bullets and shows each one while the pointer is over its block, as Logseq's site does.
- On the encode garden, `lsg` hides the bullets that Logseq's build hides.

**Non-Goals:**

- Show a block's visible properties as rows under the block. A block with a visible property keeps its bullet under this change, as in Logseq, while `lsg` still shows no property rows for it.

## Decisions

- **The SDK owns the rule.** `OutlineBlock::empty` holds Logseq's `block-content-empty?`, and `Outline::hides_bullet` adds the children, first-block and last-block conditions. `Outline::from_blocks` already reads each block's properties and children, so the check adds one pass over the blocks the publisher already walks. `page_properties::is_hidden_block_property` holds the property names Logseq hides from blocks: its built-in hidden keys and the card scheduling keys.
- **A lone task marker counts as an empty title.** Logseq's parser reads `- TODO` as a task marker with an empty title, while lsdoc gives `TODO` as the block's text. The SDK treats a first line that holds only an upper-case marker word as empty, so the block's bullet is hidden as in Logseq's build.
- **The first and last blocks come from the outline being rendered.** The publisher builds an `Outline` for each list of blocks it renders: a page, an embedded page or block, or a reference copy. The first bullet of that list is Logseq's first block unless page properties, a heading line or other text comes before it; the last top-level bullet is Logseq's last block unless a top-level heading line comes after it. A top-level heading line also ends the body of the bullet before it, because Logseq starts a new block there.
- **The publisher renders a class.** `site/render.rs` adds `lsg-empty` to the block's `<li>` and keeps the bullet element. The shared stylesheet rule `:is(.outline,.refs-outline) li.lsg-empty:not(:hover)>.bullet{visibility:hidden}` hides the bullet until the pointer is over the item. The item of such a block has no children, so hovering the item means hovering the block's row. Pages load no new script, and each page's added work is one class per such block.
- **Hover shows the bullet, as in Logseq.** A static page has no editing state, so the pointer is the one condition left that changes at run time. CSS `:hover` gives the same result as Logseq's mouse-over handler without a script. `visibility: hidden` also takes the hidden bullet out of the keyboard order until it is shown, which matches Logseq, whose hidden bullet is an empty `span` with no action.

## Risks / Trade-offs

- [`:block-hidden-properties` in `logseq/config.edn` hides more property names in Logseq] → The SDK applies only Logseq's built-in list, so a block with no text whose only properties are listed in that setting keeps its bullet in `lsg`. The encode garden leaves the setting commented out. Applying it needs the setting passed into every render path; Resolved Question 2 records the choice.
- [`:ui/show-empty-bullets? true` shows every bullet in Logseq] → `lsg` reads no such setting and always applies the rule. Logseq's default and the encode garden's value are `false`.
- [Logseq's parser and lsdoc split a block's body differently] → The encode-garden comparison in `verification.md` checks every page that has a block with no text.

## Resolved Questions

### 1 - How should a static page treat hover?

Decided by the agent without codekiln: show the bullet while the pointer is over the block's row, with a CSS `:hover` rule. Logseq shows the bullet on mouse-over, and a stylesheet rule needs no script and no work that grows with the garden. On a touch screen, a tap on the row shows the bullet, as a tap does in Logseq's site.

### 2 - Should the rule apply the graph's `:block-hidden-properties`?

Decided by the agent without codekiln: no, for now. `lsg` publishes no block property rows, and the setting reaches only the page properties box and the page sequence today. A block with no text and only such properties keeps its bullet, the same as before this change. The encode garden does not set it.

### 3 - Where should the class go?

Decided by the agent without codekiln: on the block's `<li>`. A reference copy's bullet is filled in later for each listing page, and its `<li>` is written with the copy, so the class on the item reaches the page outline, embeds and reference copies through one path.

### 4 - Should a block written `- TODO` count as having no text?

Decided by the agent without codekiln: yes. Logseq's build of the encode garden hides that block's bullet on `iia/0830 Voice AI Panel`, because its parser takes the word as a task marker and leaves the title empty.

## Open Questions

None.
