# Page title, outline, headings, collapse and block zoom

A Logseq page reads as an outline: a light 36px title, small round bullets with a faint guide line down each level of children, headings that keep the outline's indentation, task checkboxes, and bullets that open a zoomed view of one block with a breadcrumb back to the page. `lsg` publishes the same blocks as plain nested HTML lists with the browser's default discs, bold browser-default headings, task markers as plain words, and no zoom. The content matches; the look and the reader controls do not.

Most of the gap is CSS on markup `lsg` already writes. Two items need behaviour: readers collapsing and expanding blocks that are not authored as collapsed, and block zoom. Both can work on the page already loaded, so neither needs one HTML file per block or any graph data in the browser.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `09cf7fb` (served locally, dark theme) and an `lsg` build of the whole garden from `2cead35` (`mise run parity:build`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `1.0.0-12-g6e7afa8eb`, the file-graph continuation of 0.10 (`frontend` = `src/main/frontend`). Its CSS differs from the live build in a few weights and line heights; the measured live values are given first and win, as the parity rules require.

### Page title

- Markup: `div.ls-page-title > h1.page-title.title > span.title` ([`components/page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L340-L378)).
- Live: 36px, weight 300, margin-bottom 24px. The checkout's CSS says weight 500 ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L338-L343), `--ls-page-title-size: 36px` in `packages/ui/src/vars-classic.css`).
- Text: the `title::` property, otherwise the file name. Namespace pages show the whole path as plain text (`1Password/Account/Service`); the segments are not separate links.

### Blocks and bullets

[`components/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2862-L2935) builds each block as `div.ls-block` holding a control row (hover arrow, then `a.bullet-link-wrap > span.bullet-container > span.bullet`) and the content, followed by `div.block-children-container > div.block-children-left-border + div.block-children`. CSS in [`components/block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L166-L210):

- **Bullet:** a 6px round dot centred in a 14px container (live; 16px in the checkout), grey at opacity 0.8. Hovering the bullet shows a faint disc behind it and scales the dot to 1.2.
- **Nesting guide:** `.block-children` draws a left border (live: 2px, `rgba(226,240,253,0.082)` in dark), and is indented 29px. A 4px invisible strip over the line darkens on hover and toggles the parent's collapse when clicked.
- **Spacing:** each block has 2px top and bottom padding; text is 16px with a 25.6px line height (1.6) live.
- **Hidden properties:** `id::`, `collapsed::`, `heading::` and other built-in properties are never shown.

### Collapse

- `collapsed:: true` sets the block collapsed; its children are not drawn until expanded ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1712-L1718)).
- A collapsed bullet sits on a filled disc (live: `rgba(226,240,253,0.082)` 14px circle, `.bullet-container.bullet-closed`). `rw-cli` has four.
- An arrow appears left of the bullet while the row is hovered, for any block with children; it points right when collapsed and down when open ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1754-L1774), `ui.css` `.rotating-arrow`).
- Readers can collapse and expand any block with children through the arrow or the guide strip. The change lives only in the reader's browser; nothing is saved.

### Headings

- `- # Text` through `- ###### Text` render as `h1`–`h6` inside the block, with the `#` characters removed and the heading keeping its bullet. `heading:: true` renders as the heading for the block's depth (`h2` at the top level); no encode-garden page uses it ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1946-L1958)).
- Live sizes and weights: h1 32px/300, h2 24px/300, h3 19.2px/600, h4 16px/600, h5 13.28px/600, h6 12px/600. h1 and h2 have a 1px bottom rule and margins of 4px and 3px; h3 and h4 have small top margins; nothing else is added ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L390-L447)).
- The bullet aligns with the heading's text baseline.

### Task markers

[`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1829-L1928) renders, in order, checkbox, marker label, priority, then the text:

- **TODO, DOING, NOW, LATER:** an unchecked 16px checkbox, then the marker word as a small label (85%, weight 650, opacity 0.7). Live `AI/Coding/Todos` shows `☐ DOING install…` and `☐ TODO install…`.
- **WAIT, WAITING, IN-PROGRESS:** unchecked checkbox and the marker word.
- **DONE:** a checked checkbox, no word, text struck through at opacity 0.7.
- **CANCELED, CANCELLED:** no checkbox, no word, text struck through at opacity 0.7.
- **Priority:** `[#A]` as a faint link to page `A`.
- Clicking a checkbox on the published site changes only the reader's copy.

The encode garden has 454 TODO, 395 DONE and 10 DOING blocks, and no priorities.

### Block zoom

- Clicking a bullet opens `#/page/<block uuid>` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1683-L1704)). Blocks without `id::` get a new random UUID on every build, so these links are not stable across builds.
- The zoomed view ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L467-L505)) has no page title. It shows a breadcrumb, then the block as the outline root with its bullet and all its children expanded, and nothing else: no references or hierarchy.
- The breadcrumb ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2559-L2621)) is `div.breadcrumb.block-parents`: the page link, then up to three nearest ancestors as inline text (with `⋯` after the page when more are hidden), separated by chevron icons at opacity 0.5. Each item opens that ancestor's zoomed view. The zoomed block itself is not in the breadcrumb. Live, zooming into `rw-cli`'s collapsed table block shows `rw-cli › rw-cli - An open-source… › Commands › rw-cli list - list items in Readwise/Reader`.

## What lsg does today

From the renderer (`crates/logseq-gardener-sdk/src/site/render.rs`, `site/mod.rs`) and the live build:

- **Title:** a bare `<h1>` with browser defaults: 32px, weight 700.
- **Blocks:** nested `<ul><li>` with the content directly in the `li`. Bullets are the browser's disc, circle and square markers; there are no guide lines. A blank line after page properties leaves a stray `<p><br></p>` before the outline.
- **Collapse:** an authored collapsed block renders as `<li class="outline-collapsed"><details class="outline-disclosure"><summary>…</summary><ul>…</ul></details></li>`. Readers open it with the browser's triangle; the disc is hidden. Blocks not authored as collapsed cannot be collapsed. `fragments.js` opens the `<details>` around a linked block.
- **Headings:** `<h1>`–`<h6>` inside the `li` with browser defaults (bold, 32/24/18.72px…, margins about 0.67em), no rule under h1 and h2. `heading::` is ignored.
- **Tasks:** the marker is plain text; the priority loses its brackets (`TODO [#A] x` renders as `TODO A x`). No checkbox, no strike-through. `AI/Coding/Todos` also shows its `:LOGBOOK:` drawers as a literal `logbook`.
- **Zoom:** none. Blocks with `id::` get `<span id="block-<uuid>">` anchors, and blocks that backlinks or search point at get `source-block-<offset>` anchors. Bullets are not links.
- **Properties:** block properties are hidden, as in Logseq.

## Gaps

Each gap names a check the apply step can run against the fixture pages.

- 1. **Title style.** 36px, weight 300, 24px below. Check: on `1Password/Account/Service` the title's computed size and weight match Logseq's (36px, 300).
- 2. **Bullets.** A 6px dot in a 14px container replaces the browser markers at every depth, with the hover disc. Check: no `li` in the outline shows a `list-style` marker; the dot's size and colour match Logseq's in both themes.
- 3. **Nesting guide and spacing.** Children indent by Logseq's distance and show the left guide line; blocks have 2px vertical padding and a 1.6 line height. Check: on `1Password/Account/Service` the x position of each level's bullet matches Logseq's within 2px at 1280px wide.
- 4. **Collapsed bullets.** An authored collapsed block shows the bullet on a filled disc instead of the browser's triangle. Check: `rw-cli` shows four closed bullets styled as Logseq's and no native disclosure triangle.
- 5. **Reader collapse.** Any block with children shows the arrow on hover, and the arrow or the guide line toggles it. Without JavaScript, authored collapsed blocks still open, as they do today. Check: on `1Password/Account/Service`, collapsing `Important limitations` hides its children and expanding restores them.
- 6. **Headings.** Logseq's sizes, weights and h1/h2 rules, aligned with the bullet; `heading:: true` maps to the depth's level. Check: on `1Password/Account/Service` the h1, h2 and h3 computed styles match the values above; on `rulesync/log/25/10/04 Sat - Issues with gitignore` h3–h6 match.
- 7. **Task markers.** Checkbox, marker label, DONE and CANCELED strike-through, `[#A]` kept with its brackets. Checkboxes are read-only on the static site. Check: `AI/Coding/Todos` shows DOING and TODO with unchecked boxes and labels, and the DONE item checked and struck through.
- 8. **Block zoom.** Each bullet opens a zoomed view of its block: breadcrumb (page, then up to three ancestors, `⋯` when more), the block expanded as the root, and no title or references. The link must be stable across builds: the block's `id::` UUID when it has one, otherwise a position-based id. Check: zooming into `rw-cli`'s block `6add3ffb-e52b-4e8f-908a-9958186e4dfb` shows the same breadcrumb as Logseq, and the URL still works after a rebuild.
- 9. **Stray paragraph.** No empty paragraph between page properties and the outline. Check: `1Password/Account/Service` has no `<p><br></p>` before its outline.
- 10. **Performance.** All of the above is CSS plus one small deferred script. Zoom works on the loaded page (by hiding everything but the block and building the breadcrumb from the DOM), so it adds no files and no data. Per-block wrapper markup must stay small: a simulated extra wrapper per block on `tmux` (1,061 blocks, 94 KB gzip today against the 100 KB first-paint budget) added 150 bytes gzip. Check: `parity:budgets` passes, `tmux` stays under 100 KB gzip, and the new script stays outside the 16 KB pre-title script budget.

## Where the work belongs

- **SDK:** the block tree with each block's depth, marker, priority, heading level (from `#` or `heading::`) and collapsed state; the ancestor path of any block; and a stable block identifier (UUID, else position). The CLI and the language server need the same block identities and paths, for example to resolve a block reference or show where it sits.
- **Publisher:** the markup, the CSS, the hover arrow and reader collapse, the read-only checkboxes, and the zoomed view with its breadcrumb.

## Fixture pages

| Page | Why |
| --- | --- |
| `1Password/Account/Service` | h1, h2 and h3 blocks, nesting four levels deep, an `id::` block |
| `rulesync/log/25/10/04 Sat - Issues with gitignore` | h3 to h6 headings |
| `rw-cli` | Four authored collapsed blocks; zoom target `6add3ffb-e52b-4e8f-908a-9958186e4dfb` |
| `AI/Coding/Todos` | TODO, DOING and DONE blocks with logbooks |
| `tmux` | The largest fixture page, for the size budget |

## Out of scope

- The `:LOGBOOK:` drawer shown as `logbook`, and other inline rendering (#112).
- Page properties, which the encode garden shows in Logseq but `lsg` publishes only with `:publishing/page-properties? true` (published-page-properties spec).
- The block-reference count badge at the right of referenced blocks (#114).
- Shift-click to open a block in the right sidebar (#119).
- Editing: renaming the title, changing markers or priorities, and saving collapse state.
