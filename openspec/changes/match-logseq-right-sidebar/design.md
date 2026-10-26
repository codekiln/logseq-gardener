## Context

The [Right sidebar analysis](../../../docs/parity/right-sidebar.md) measured Logseq's `gh-pages` build of the encode garden (`8132c95`) and an `lsg` build from `1e72969`. Logseq's sidebar is part of its single-page app: every card renders from the in-browser database, which exists only after the 19.8 MB graph and 7.5 MB of script have loaded.

`lsg` publishes every page as static HTML. Today the header holds Toggle left sidebar, Home, Search garden and More; `lsg-shell.js` (5.7 KB of its 6 KB budget) handles the left sidebar, theme, wide mode and the `g` shortcuts; `lsg-outline.js` zooms into a block at `<page>#zoom/<id>`, where `<id>` is the block's `id::` UUID or its position such as `3.1.2`; `lsg-references.js` loads reference section bodies from `_refs/` when a section opens. A typical page is 27 KB (about 8 KB gzip), of which about 23.6 KB is the inlined head and shell; the largest, `Person`, is 508 KB (52 KB gzip).

## Goals / Non-Goals

**Goals:**

- Readers can keep pages and blocks open beside the current page, with Logseq's controls, menus, look and saved state.
- No page's first paint changes: the sidebar's script and card content load only when a reader uses the sidebar.
- Memory on a phone grows with the cards a reader has open, never with the garden.

**Non-Goals:**

- Reordering cards by dragging them, as Logseq allows.
- Opening search results in the sidebar, which [Issue #118](https://github.com/codekiln/logseq-gardener/issues/118) can add once this change provides the card API.

## Decisions

**Where the work lives.** The SDK answers what a card shows; the publisher renders the sidebar.

| Piece | Lives in | Notes |
| --- | --- | --- |
| Page routes for titles, aliases and tags | SDK (existing name index) | The same resolution page links use; a card opens whatever a link points at. |
| Block addresses | SDK (existing outline identifiers) | `id::` UUID or position path, the identifiers block zoom uses; they stay stable when the unchanged garden is published again. |
| Breadcrumbs for block cards | SDK (existing ancestors from the reference index) | Already published as the zoom breadcrumb; the card copies it from the fetched page. |
| `:default-home :sidebar` | SDK (`garden_settings`, bounded EDN reader) | Resolved through the favorites lookup: titles and aliases to routes, `"Contents"` to the Contents card, unresolved names to diagnostics. |
| Publishing a one-bullet `contents` page | SDK and publisher | Today the empty `pages/contents.md` produces no page; the Apply step finds why and publishes it like any selected page. |
| Toggle button, styles, `lsg-sidebar.js`, Help content | Publisher | Rendering only. |

**Card content comes from the published page.** A page card fetches `p-<hash>.html`, parses it with `DOMParser`, and takes the content of `<main>` without its `h1` and without the Unlinked References section. A block card fetches the page holding the block and takes the block's `<li>` subtree, found with the same identifier lookup block zoom uses. Reference sections inside a card are initialized with the existing `window.logseqGardenerInitReferences(card)`, so their bodies still come from `_refs/` only when opened. No new files are written per page. If the measured transfer for typical cards exceeds the `card_fetch_gzip_kb` budget, the Apply step adds content-only fragments instead and records the measurement.

**Script split.** `lsg-shell.js` gains a hook of at most 0.6 KB: it draws nothing, but on the toggle, `t r`, `alt+shift+c`, or a Shift-click on a page link or bullet in `<main>` or a card, it loads `lsg-sidebar.js` once and hands it the event. Shift-click is caught on the document in the capture phase, so content a host swaps in needs no extra call. Everything else (column, cards, menus, resize, saved state, Help) is in `lsg-sidebar.js`. Pages with a saved open sidebar load `lsg-sidebar.js` after the `load` event, as the outline and reference scripts load today.

**Layout.** At 640px and wider, `html[data-lsg-right-sidebar=open]` gives the page a right column of `var(--lsg-right-width, 40vw)`, at most 60vw by default, full height from the top of the window; the header and main column use the remaining width. A 3px `role="separator"` on the column's left edge resizes it between `max(320px, 10vw)` and 70vw by drag and by Left and Right arrows (5px per press), and the width is saved. The card list has 16px side padding, 150px bottom padding and scrolls on its own. Styles use the analysis's light values and the existing theme tokens.

**Cards.** Each card is a `<section>` with a header button (`aria-expanded`, `aria-controls`) holding the chevron, the icon (`page`, `block`, `list-details` or `help`, or the page's `icon::`) and the title truncated with an ellipsis, then `⋯` (`aria-label="Card menu"`, `aria-haspopup="menu"`) and `×` (`aria-label="Close card"`). A new card goes to the top and is expanded; a target already present moves to the top. The menu items depend on the card's type, its collapsed state and the number of cards, as the analysis tabulates. The menu is a `role="menu"` list of buttons: Enter or Space opens it with focus on the first item, arrows move, Escape and a click outside close it, and focus returns to `⋯`.

**Memory.** Fetched page HTML is cached as text by route for the life of the tab, at most 20 routes, least recently used dropped first. A collapsed card drops its body; expanding it renders again from the cache. Ten open `tmux`-sized cards hold about ten parsed outlines.

**Saved state.** `localStorage` key `logseq-gardener-right-sidebar` holds `{open, width, cards: [{type, route, block, collapsed}]}`, where `type` is `page`, `block`, `refs`, `contents` or `help`. It is written whenever the sidebar opens or closes, a card is added, closed, collapsed or expanded, or the width changes. On a page load with a saved open sidebar, the column's width and empty card frames apply before first paint from a small inline check in the existing theme script, and card bodies are fetched after the `load` event, so the title shows first. A card whose route no longer exists shows `This page is not published.` with a Close button. With storage unavailable, state lasts for the current page.

**Contents and Help.** An empty sidebar shows the Contents card, which fetches the published `contents` page; when the garden publishes no `contents` page, the card shows one empty bullet, as Logseq does for an empty `contents` page. The top bar holds Contents and Help buttons; Page graph is added by [Issue #120](https://github.com/codekiln/logseq-gardener/issues/120). The Help card lists the published site's shortcuts (`t l`, `t r`, `t t`, `t w`, `g h`, `g a`, `g j` when a feed exists, `alt+shift+c`, Shift-click, and the search shortcut when search is published) and links nowhere outside the garden.

**Phone width.** Below 640px the toggle is not drawn, the hook ignores `t r` and `alt+shift+c`, and Shift-click is left to the browser.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md) on the whole encode garden:

| Measure | Budget |
| --- | --- |
| Every fixture page's first paint, desktop and phone | unchanged except the toggle button (under 0.3 KB), within `data_before_title_kb` 100 and `script_before_title_kb` 16 |
| `[shell]` `script_kb` | 6.5 (from 6); `frame_kb` stays 6 |
| `lsg-sidebar.js`, gzip | 6 KB, never loaded before the first use or, with a saved open sidebar, before `load` |
| Shift-click to the card's outline, p95 | 300 ms desktop, 1,500 ms phone |
| One card's transfer, gzip | 60 KB, so the `Person` card (52 KB) fits |
| Peak phone heap after opening ten cards | 20 MB |

`scripts/parity/budgets.toml` gains a `[sidebar]` table: `parity:budgets` checks the script size, the shell hook and that no fixture page's first paint grows beyond the toggle; `parity:perf` opens a page card, a block card and a Block references card on `tmux`, then ten cards, and marks the times, transfer and heap in `docs/parity/performance.md`.

## Risks / Trade-offs

- [The shell script budget rises] → The hook, at most 0.6 KB, is the only addition to every page; the rest loads on use. The alternative, a separate always-loaded script, costs a request on every page.
- [Whole-page fetches carry the shell] → About 6.7 KB gzip of each card is head and shell the card discards. Content-only fragments would add one file per page (over 10,000); the Apply step measures both and keeps whole pages unless the card budget fails.
- [Saved cards point at pages a later build drops] → The card says the page is not published and offers Close.
- [Block positions change when the author edits the page] → Cards address blocks the same way zoom does; a block with an `id::` keeps its card across edits, a positional one may show a neighbour, as a zoom link would.
- [Shift-click meaning changes] → At 640px and wider, Shift-click no longer opens a browser window; Logseq behaves the same way. Below 640px the browser default stays.

## Resolved Questions

### 1 - Where does a card's content come from?

Decided by the agent without codekiln: from the published page file, fetched on demand and parsed in the browser, with reference bodies from the existing `_refs/` fragments. Issue #106 forbids loading the graph; the page file is already on the server and cached by the browser, and adding content-only fragments would write over 10,000 more files before a measurement shows they are needed.

### 2 - How does a block card address a block without an `id::`?

Decided by the agent without codekiln: by the position identifier block zoom already publishes (`3.1.2`), and by `id::` UUID when the block has one. One addressing scheme for zoom and cards keeps the SDK as the only source of block identity.

### 3 - Is the Page graph button drawn before the graph exists?

Decided by the agent without codekiln: no. The app shell spec allows only controls whose destination exists; Issue #120 adds the button with the Page graph card.

### 4 - What does the Help card contain?

Decided by the agent without codekiln: the shortcuts and gestures the published site supports, with no links to Logseq's forum, documentation or bug tracker. Logseq's Help card sends readers to Logseq's support and lists 157 shortcuts, most for editing, which a published garden does not have.

### 5 - What happens below 640px?

Decided by the agent without codekiln: no toggle, `t r` and `alt+shift+c` do nothing, and Shift-click follows the link. Logseq hides the toggle and ignores Shift-click on a phone but still opens a 156px column on `t r`, which is too narrow to read; following the link keeps the gesture useful.

### 6 - How is the sidebar state saved?

Decided by the agent without codekiln: in `localStorage` under `logseq-gardener-right-sidebar`, by route and block identifier, including collapsed flags and width, and written on every change. Logseq stores database ids, which a static site does not have, and skips saving collapse changes; saving them costs nothing and restores what the reader saw.

### 7 - Does restoring a saved sidebar delay the page?

Decided by the agent without codekiln: no. The column's width and empty card frames apply before first paint so the page does not jump, and card bodies load after the `load` event.

### 8 - Is the card menu keyboard accessible?

Decided by the agent without codekiln: yes, as a `role="menu"` with focusable items, arrow keys and Escape, like the More menu. Logseq's menu items are not focusable; matching that would leave keyboard readers without Close all or Open as page.

### 9 - Is `:default-home {:sidebar …}` supported?

Decided by the agent without codekiln: yes, for a first visit with no saved state. Logseq honours it when publishing, and the SDK's bounded EDN reader already reads the same file.

### 10 - Does the shell script budget rise?

Decided by the agent without codekiln: yes, from 6 KB to 6.5 KB for a hook of at most 0.6 KB. `lsg-shell.js` is 5.7 KB already, and the toggle, `t r` and Shift-click must work on every page without an extra request.

### 11 - Do All pages rows open cards on Shift-click?

Decided by the agent without codekiln: yes. The hook handles Shift-click on any page link in `<main>`, so the All pages table from Issue #117 gets it without extra work, as Logseq's All pages does.
