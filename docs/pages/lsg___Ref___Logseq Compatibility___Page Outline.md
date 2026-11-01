tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Outline and Blocks]], [[lsg/Ref/Published Pages]]

- # Page Outline Compatibility
	- `lsg` publishes the page title, bullets, guide lines, headings and task markers with Logseq's sizes, weights and colours, lets readers collapse and expand any block with children, and opens a zoomed view of a block with Logseq's breadcrumb when a reader clicks its bullet. Its zoom addresses use the block's `id::` or its position among its siblings, so they keep working after a rebuild, where Logseq's change on every build. Its task checkboxes are read-only, each expandable block has its own arrow that stays visible while closed or focused, and referenced blocks show no reference count.
	  id:: 8cb05199-293c-41ca-a714-ba609a9c7b36
	- Measured on 2026-10-25 against the `gh-pages` build of garden commit `09cf7fb` (served locally, dark theme) and an `lsg` build of the whole garden from `2cead35` (`mise run parity:build`). The OpenSpec change [match-logseq-page-outline](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-25-match-logseq-page-outline) then built the outline, and [published-page-outline](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-page-outline/spec.md), [outline-blocks](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/outline-blocks/spec.md) and [authored-outline-collapse](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/authored-outline-collapse/spec.md) hold its requirements.
	- ## What Logseq's published site shows
		- Source links point at the local `logseq/og` checkout, `1.0.0-12-g6e7afa8eb`, the file-graph continuation of 0.10 (`frontend` = `src/main/frontend`). Its CSS differs from the live build in a few weights and line heights; the measured live values are given first and win.
		- ### Page title
			- Markup: `div.ls-page-title > h1.page-title.title > span.title` ([`components/page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L340-L378)).
			- Live: 36px, weight 300, margin-bottom 24px. The checkout's CSS says weight 500 ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L338-L343), `--ls-page-title-size: 36px` in `packages/ui/src/vars-classic.css`).
			- Text: the `title::` property, otherwise the file name. Namespace pages show the whole path as plain text (`1Password/Account/Service`); the segments are not separate links.
		- ### Blocks and bullets
			- [`components/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2862-L2935) builds each block as `div.ls-block` holding a control row (hover arrow, then `a.bullet-link-wrap > span.bullet-container > span.bullet`) and the content, followed by `div.block-children-container > div.block-children-left-border + div.block-children`. CSS in [`components/block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L166-L210):
				- **Bullet:** a 6px round dot centred in a 14px container (live; 16px in the checkout), grey at opacity 0.8. Hovering the bullet shows a faint disc behind it and scales the dot to 1.2.
				- **Nesting guide:** `.block-children` draws a left border (live: 2px, `rgba(226,240,253,0.082)` in dark), and is indented 29px. A 4px invisible strip over the line darkens on hover and toggles the parent's collapse when clicked.
				- **Spacing:** each block has 2px top and bottom padding; text is 16px with a 25.6px line height (1.6) live.
				- **Hidden properties:** `id::`, `collapsed::`, `heading::` and other built-in properties are never shown.
		- ### Collapse
			- A block with `collapsed:: true` starts collapsed; its children are not drawn until expanded ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1712-L1718)).
			- A collapsed bullet sits on a filled disc (live: `rgba(226,240,253,0.082)` 14px circle, `.bullet-container.bullet-closed`). `rw-cli` has four.
			- An arrow appears left of the bullet while the row is hovered, for any block with children; it points right when collapsed and down when open ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1754-L1774), `ui.css` `.rotating-arrow`).
			- Readers can collapse and expand any block with children through the arrow or the guide strip. The change lives only in the reader's browser; nothing is saved.
		- ### Headings
			- `- # Text` through `- ###### Text` render as `h1`–`h6` inside the block, with the `#` characters removed and the heading keeping its bullet. `heading:: true` renders as the heading for the block's depth (`h2` at the top level); no encode-garden page uses it ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1946-L1958)).
			- Live sizes and weights: h1 32px/300, h2 24px/300, h3 19.2px/600, h4 16px/600, h5 13.28px/600, h6 12px/600. h1 and h2 have a 1px bottom rule and margins of 4px and 3px; h3 and h4 have small top margins; nothing else is added ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L390-L447)).
			- The bullet aligns with the heading's text baseline.
		- ### Task markers
			- [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1829-L1928) renders, in order, checkbox, marker label, priority, then the text:
				- **TODO, DOING, NOW, LATER:** an unchecked 16px checkbox, then the marker word as a small label (85%, weight 650, opacity 0.7). Live `AI/Coding/Todos` shows `☐ DOING install…` and `☐ TODO install…`.
				- **WAIT, WAITING, IN-PROGRESS:** unchecked checkbox and the marker word.
				- **DONE:** a checked checkbox, no word, text struck through at opacity 0.7.
				- **CANCELED, CANCELLED:** no checkbox, no word, text struck through at opacity 0.7.
				- **Priority:** `[#A]` as a faint link to page `A`.
				- Clicking a checkbox on the published site changes only the reader's copy.
			- The encode garden has 454 TODO, 395 DONE and 10 DOING blocks, and no priorities.
		- ### Block zoom
			- Clicking a bullet opens `#/page/<block uuid>` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1683-L1704)). Blocks without `id::` get a new random UUID on every build, so these links are not stable across builds.
			- The zoomed view ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L467-L505)) has no page title. It shows a breadcrumb, then the block as the outline root with its bullet and all its children expanded, and nothing else: no references or hierarchy.
			- The breadcrumb ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2559-L2621)) is `div.breadcrumb.block-parents`: the page link, then up to three nearest ancestors as inline text (with `⋯` after the page when more are hidden), separated by chevron icons at opacity 0.5. Each item opens that ancestor's zoomed view. The zoomed block itself is not in the breadcrumb. Live, zooming into `rw-cli`'s collapsed table block shows `rw-cli › rw-cli - An open-source… › Commands › rw-cli list - list items in Readwise/Reader`.
	- ## What lsg publishes
		- The page title is 36px, weight 300, 24px above the content. The outline follows the page properties directly, with no empty paragraph between them.
		- The root outline of each page and each journal-feed article carries the `outline` class. Its items show no browser list markers; each block has a 6px round bullet centred in a 14px column at every depth, with a hover disc that scales the dot to 1.2. Children indent 29px with a 2px guide line, and blocks have 2px vertical padding and a 1.6 line height. Colours come from theme tokens, `--lsg-guide` among them, in both themes. Lists inside a block's content and Linked References contexts keep their own style.
		- Headings `h1`–`h6` use Logseq's sizes and weights (32px/300, 24px/300, 19.2px/600, 16px/600, 13.28px/600, 12px/600), with a 1px bottom rule under h1 and h2, and keep their bullet. `heading:: true` gives the heading for the block's depth plus one, capped at 6, and `heading:: <n>` gives `h<n>`. A heading line written without a bullet starts a block of its own, as in Logseq.
		- Task blocks show a disabled unchecked checkbox and the marker word as a label for TODO, DOING, NOW, LATER, WAIT, WAITING and IN-PROGRESS; a disabled checked checkbox and struck-through text for DONE; and struck-through text without checkbox or label for CANCELED and CANCELLED. A priority keeps its brackets, `[#A]`, linked to page `A` when that page is published.
		- Readers can collapse and expand any block with children with its arrow, left of the bullet, or with the strip over its children's guide line. Each expandable block has its own focusable arrow, at least 24px square and centred on its bullet, headings included. A collapsed block shows its bullet on a filled disc, and the state lasts only in the open tab. A link to a block inside a collapsed block expands its collapsed ancestors.
		- A block written with `collapsed:: true` starts closed as a native disclosure, shows its bullet on the filled disc in place of the browser's triangle, and opens without JavaScript.
		- Clicking a bullet, or pressing Enter on a focused bullet, opens the block at `<page>#zoom/<id>`. `<id>` is the block's `id::` UUID when it has one, otherwise its 1-based position among its siblings at each level, such as `3.1.2`, so the address opens the same block after the unchanged garden is published again. The zoomed view shows a breadcrumb and the block as the root with all its descendants expanded, and hides the title, page properties, the Hierarchy section, Linked References and every other block. The breadcrumb links the page, then up to three nearest ancestors to their own zoomed views, with `⋯` after the page when more are hidden. Back restores the page with the reader's earlier collapse state. `rw-cli` at `#zoom/1.1.1.2` shows `rw-cli › rw-cli - An open-source… › Commands › rw-cli list - list items in Readwise/Reader`.
		- In a journal-feed article, a bullet opens the journal's own page at the block's zoom address. An unknown identifier leaves the page view.
		- A word too long for its block breaks to fit, so no outline block makes the page wider than the screen; code blocks keep their own horizontal scrolling.
		- The SDK's `outline` module reads each page's blocks with their depth, parent, task marker, priority, heading level, authored collapsed state and identifier, for the publisher, the CLI and the language server. The publisher renders the markup, the styles, the read-only checkboxes and `lsg-outline.js`, which handles collapse and zoom on the page already loaded. A host that swaps page content calls `window.logseqGardenerInitOutline(root)`, which [[lsg/Ref/Host Integration API]] describes.
		- [[lsg/Ref/Logseq Compatibility/Performance]] recorded on 2026-10-26 a bullet click on `tmux` painting the zoomed view in 20 ms on desktop and 23 ms on the phone profile, where Logseq took 77 ms and 612 ms. The same run marked `tmux` over its budget, with 101 KB of data before the title on both profiles.
	- ## Performance
		- The outline adds no file per block and no graph data to the browser.
		- The `[outline]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) holds the budgets: `lsg-outline.js` at most 8 KB uncompressed, loaded after the page's `load` event; the outline styles at most 1.5 KB gzip on each page; and a bullet click on `tmux` painting the zoomed view within 50 ms on the desktop profile and 150 ms on the phone profile. The `tmux` page stays under the 100 KB first-paint budget and grows by at most 2 KB gzip. The existing budgets for data and script before the title and for peak heap hold as well.
		- `parity:budgets` checks the script and page sizes in CI, and `parity:perf --record` records the zoom times on [[lsg/Ref/Logseq Compatibility/Performance]].
	- ## Differences from Logseq
		- Zoom addresses are `<page>#zoom/<id>` on the page already loaded. Logseq opens `#/page/<uuid>` with a random UUID for every block without `id::`, so its links change on every build; a position path changes only when the author inserts a block above it.
		- Task checkboxes are disabled. In Logseq's published site a reader can tick one, but the change is lost on reload.
		- Each expandable block has its own arrow, and closed arrows stay visible without hover; keyboard focus and touch pointers reach the arrows before hover. Logseq shows one arrow on the hovered row.
		- Bullets in the journals feed open the journal's own page at the zoom address. The feed itself does not zoom.
		- Linked References contexts keep their own bullets; only root outlines carry Logseq's outline style.
		- Referenced blocks show no reference count at their right.
		- `:LOGBOOK:` drawers show as the literal word `logbook`, and the build reports each one as a diagnostic.
		- When the match-logseq-page-outline change closed on 2026-10-25, a `collapsed:: true` written after a block's first child, at the parent's indentation, attached to the child, so `lsg` showed two of `rw-cli`'s four collapsed blocks open; Logseq attaches the property to the parent.
		- Page properties show only when the garden sets `:publishing/page-properties? true`; [[lsg/Ref/Published Pages]] describes them.
		- `lsg` publishes a read-only site: it has no title renaming, marker or priority changes, or saved collapse state.
	- ## Before the change
		- The `lsg` build measured on 2026-10-25, from `2cead35`, rendered pages from `crates/logseq-gardener-sdk/src/site/render.rs` and `site/mod.rs`.
		- **Title:** a bare `<h1>` with browser defaults: 32px, weight 700.
		- **Blocks:** nested `<ul><li>` with the content directly in the `li`. Bullets were the browser's disc, circle and square markers, with no guide lines. A blank line after page properties left a stray `<p><br></p>` before the outline.
		- **Collapse:** an authored collapsed block rendered as `<li class="outline-collapsed"><details class="outline-disclosure"><summary>…</summary><ul>…</ul></details></li>`. Readers opened it with the browser's triangle; the disc was hidden. Blocks not authored as collapsed could not be collapsed. `fragments.js` opened the `<details>` around a linked block.
		- **Headings:** `<h1>`–`<h6>` inside the `li` with browser defaults (bold, 32/24/18.72px…, margins about 0.67em), no rule under h1 and h2. `heading::` was ignored.
		- **Tasks:** the marker was plain text, and the priority lost its brackets (`TODO [#A] x` rendered as `TODO A x`). No checkbox, no strike-through. `AI/Coding/Todos` also showed its `:LOGBOOK:` drawers as a literal `logbook`.
		- **Zoom:** none. Blocks with `id::` got `<span id="block-<uuid>">` anchors, and blocks that backlinks or search pointed at got `source-block-<offset>` anchors. Bullets were not links.
		- **Properties:** block properties were hidden, as in Logseq.
		- **Size:** `tmux`, 1,061 blocks, was 94 KB gzip against the 100 KB first-paint budget. A simulated extra wrapper per block added 150 bytes gzip.
	- ## Fixture pages
		- | Page | Why |
		  | --- | --- |
		  | `1Password/Account/Service` | h1, h2 and h3 blocks, nesting four levels deep, an `id::` block |
		  | `rulesync/log/25/10/04 Sat - Issues with gitignore` | h3 to h6 headings |
		  | `rw-cli` | Four authored collapsed blocks; zoom target `#zoom/1.1.1.2`, which Logseq's 2026-10-25 build reached as `6add3ffb-e52b-4e8f-908a-9958186e4dfb`, a UUID Logseq generated for that build |
		  | `AI/Coding/Todos` | TODO, DOING and DONE blocks with logbooks |
		  | `tmux` | The largest fixture page, for the size budget |
