# Right sidebar

Logseq's right sidebar is a second column of cards. The header's Toggle right sidebar button and `t r` open it; Shift-click on a page link, tag, block reference or bullet adds a card for that page or block at the top. Each card can be collapsed and closed, and its `⋯` menu offers Close, Close others, Close all, Collapse, Collapse others, Collapse all, Expand all and, for pages, Open as page. A row of buttons above the cards adds Contents, Page graph and Help cards. The sidebar opens at 40% of the window, pushes the page to the left, and its cards and open state survive a reload.

`lsg` has no right sidebar. Its header has no toggle, `t r` does nothing, Shift-click on a link opens the browser's new window, and Shift-click on a bullet zooms into the block like a plain click. The garden's `contents` page, which the Contents card shows, is not in the `lsg` build.

Logseq can show any page or block in a card because the whole graph is already in the browser. `lsg` should fill a card from files it already publishes: fetch the target page's HTML when the reader asks for the card, take its outline, and load reference sections from the existing `_refs/` fragments when opened. The sidebar's script loads on first use, so a page's first paint and its script budget stay the same. Resolving link targets and block identities is SDK work; the publisher renders the toggle, cards and menus.

Measured on 2026-10-26 against the `gh-pages` build of the encode garden (`gh-pages` commit `8132c95`, served locally, light theme, 1280×800 and 390×844) and an `lsg` build of the whole garden at commit `0ce74d6` from `lsg` `1e72969` (`mise run parity:build`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where they differ from the checkout or from [Prototype](../logseq-reference/Prototype.md). The Prototype board differs from the live build in several places, listed where they occur: it opens the left sidebar by default, gives the card menu a different item list, describes Help as a list of ten shortcuts, and says new cards open with a page link and the page's outline.

### Opening and closing

- **Toggle button.** `button.toggle-right-sidebar` with title `Toggle right sidebar` and the 20px Tabler icon `layout-sidebar-right`, at the right end of the header after More ([`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L27-L34)). It is not rendered below 640px.
- **`t r`** toggles the sidebar ([`shortcut/config.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/modules/shortcut/config.cljs#L478-L479), [`handler/ui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/ui.cljs#L38-L41)). `alt+shift+c` toggles the Contents card and opens the sidebar with it; pressing it again removes the card and closes the sidebar ([`handler/ui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/ui.cljs#L61-L67)). `shift+/` is bound to toggle Help, but pressing it on the live build did nothing in the measured browser.
- **Default state:** closed ([`state.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L87-L88)). The encode garden does not set `:default-home :sidebar`; when a garden sets it, Logseq adds a card for each listed page (or Contents) on start ([`container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L691-L706)).
- **Empty sidebar.** When the sidebar has no cards, it shows a Contents card ([`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L471-L476)). That card is a placeholder, not a stored card: the first Shift-click replaces it, and after Close all, `t r` shows it again.
- **Closing.** The sidebar closes when its last card is closed, after Close all, or by `t r` and the toggle. Escape does not close it ([`state.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L1145-L1152)).
- **Saved state.** `localStorage` key `ls-right-sidebar-state` holds the open state, the cards (as database ids) and their collapsed flags, for example `{:blocks (["local" 149550 :block-ref]), :collapsed {149550 false}, :open? true}`. Logseq writes it when the sidebar opens or closes or the number of cards changes, not when a card is only collapsed, and restores it after the graph loads, also when publishing ([`handler/ui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/ui.cljs#L43-L59), [`theme.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/theme.cljs#L67-L72)). Following a link keeps the cards; a reload restores them expanded.

### Shift-click

Every Shift-click below calls [`sidebar-add-block!`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L1121-L1132): it removes the same item if present, puts the new card at the top, expands it, opens the sidebar and scrolls the card list to the top. The main page does not change. Below 640px the function does nothing, and the click is swallowed, so the link is not followed either.

| Shift-click on | Card | Source |
| --- | --- | --- |
| Page reference, tag, property value | page | [`open-page-ref`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L503-L512) |
| Bullet | block | [`bullet-on-click`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1683-L1692) |
| Block reference | block references | [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L889-L906) |
| Block reference count beside a block | block references | [`block-refs-count`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2378-L2391) |
| Breadcrumb crumb | block references | [`breadcrumb-fragment`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2528-L2537) |

Links inside a card behave the same way: a plain click opens the page in the main column, a Shift-click adds another card. Shift-clicking a page already in the sidebar moves its card to the top (measured with `Logseq/Template/Today/Link`). A middle click on a card header closes the card.

Measured on `VSCode/Proj/24t3/LearnVSCode/01 Terminal`: Shift-click on `[[Logseq/Template/Today/Link]]`, on `#Goal`, and on the block reference `((6763d7c8-…))` each added a card without changing the URL. On `tmux`, Shift-click on a bullet added a block card titled `tmux`.

### Top bar

```html
<div class="cp__right-sidebar-topbar flex flex-row justify-between items-center">
  <div class="cp__right-sidebar-settings hide-scrollbar gap-1">
    <div class="text-sm"><button class="button cp__right-sidebar-settings-btn">Contents</button></div>
    <div class="text-sm"><button class="button cp__right-sidebar-settings-btn">Page graph</button></div>
    <div class="text-sm"><button class="button cp__right-sidebar-settings-btn">Help</button></div>
  </div>
</div>
```

- Each button adds its card at the top ([`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L435-L460)). Page graph uses the current page, or today's journal on the home route. A fourth button, History, appears only in development builds.
- **Style, live:** bar 48px tall, sticky at the top, background `rgba(255,255,255,0.9)`, 4px side padding. Buttons 14px, weight 400, `rgb(17,24,28)` on `rgb(248,249,250)`, 32px tall, padding `4px 10px`, 6px radius.

### Cards

```html
<div class="flex sidebar-item content color-level shadow-md rounded-md item-type-page">
  <div class="flex flex-col w-full relative">
    <div draggable="true" class="flex flex-row justify-between pr-2 sidebar-item-header color-level rounded-t-md">
      <button aria-expanded="true" id="sidebar-panel-header-0" aria-controls="sidebar-panel-content-0" class="flex flex-row p-2 items-center w-full overflow-hidden">
        <span class="opacity-50 …"><span class="rotating-arrow not-collapsed">…</span></span>
        <div class="ml-1 font-medium overflow-hidden"><div class="flex items-center page-title"><span class="ui__icon ti ti-page …"></span><span class="overflow-hidden text-ellipsis">Logseq/Template/Today/Link</span></div></div>
      </button>
      <div class="item-actions flex items-center">
        <div class="relative ui__dropdown-trigger"><button title="More" class="button"><span class="ui__icon ti ti-dots"></span></button></div>
        <button title="Close" class="button close"><span class="ui__icon ti ti-x"></span></button>
      </div>
    </div>
    <div role="region" id="sidebar-panel-content-0" aria-labelledby="sidebar-panel-header-0" class="initial pt-4 p-1">…</div>
  </div>
</div>
```

[`sidebar-item`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L243-L315) renders every card this way. The header button toggles the card; `⋯` opens the menu; `×` closes the card. The More and Close buttons have only a `title`, no accessible name of their own.

**Card types** ([`build-sidebar-item`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L115-L184)):

| Type | Header | Body |
| --- | --- | --- |
| Contents | `list-details` icon, `Contents` | The outline of the `contents` page, without its title. The encode garden's `pages/contents.md` is one empty bullet, so the card shows one faint bullet. |
| Page | `page` icon (or the page's `icon::`), the page's original name, truncated | The page as in the main column, with these differences ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L463-L526)): no title, no Unlinked References; Pages tagged with, Linked References (collapsed, as the garden's threshold is 0) and Hierarchy stay. |
| Block (bullet) | `block` icon, then the block's breadcrumb: page link and ancestors | The block and its children. Header background `linear-gradient(0deg, rgb(248,249,250), rgb(241,243,245))`. |
| Block references | `Block references` | A breadcrumb row with the `block` icon, indented 32px, then the block and its children. |
| Page graph | `hierarchy` icon, `Page graph` | A `Show Journals` toggle and a canvas graph of the page's neighbours (see #120). |
| Help | `help` icon, `Help` | Five headed lists of links ([`onboarding.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/onboarding.cljs#L12-L57)): Usage (`Keyboard shortcuts`, `Documentation`, `Getting started`, `FAQ`), Community, Development, About, Terms. All but the first point at Logseq's own sites. `Keyboard shortcuts` adds a Keyboard shortcuts card listing all 157 of Logseq's shortcuts, most of them for editing. The Prototype board's ten-shortcut Help card does not match the live build. |

A page with no file still opens a card. `VSCode/Terminal` (referenced, no file) showed `Click here to edit...`, `Pages tagged with "vscode/terminal"`, 6 Linked References and its Hierarchy.

**Style, live:** card background `rgb(248,249,250)`, 2px radius, a 2px `rgb(233,236,239)` ring, `min-height: 100px` while open; 8px between cards. Header button padding 8px; title 19.2px, weight 700, `rgb(17,24,28)`; the chevron is 16px at 50% opacity. More and Close are 32px tall icon buttons at 60% opacity. Card body padding `16px 4px 4px`. A collapsed card is only its 47px header. The 2px radius, the ring and the 19.2px bold title differ from the classes Logseq writes (`rounded-md`, `shadow-md`, `font-medium`); they probably come from the theme that the garden's `logseq/custom.css` imports, which [#106](https://github.com/codekiln/logseq-gardener/issues/106) treats as Logseq's look. This was not traced rule by rule.

### Card menu

[`x-menu-content`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L192-L221) builds the menu from the card's type, its collapsed state, and whether there is more than one card. Measured:

| Sidebar | Menu |
| --- | --- |
| One card, open, page | Close · Collapse · ─ · Open as page |
| Two or more cards, open, block | Close · Close others · Close all · ─ · Collapse · Collapse others · Collapse all · ─ · Expand all |
| Two or more cards, collapsed (from the source, not measured) | Close · Close others · Close all · Collapse others · Collapse all · ─ · Expand · Expand all |

A page card with two or more cards adds `─ · Open as page` at the end. Close others keeps only this card; Close all removes every card and closes the sidebar; Collapse others collapses every other card; Open as page opens the page in the main column. The Prototype board lists six items and describes Collapse others as keeping only this card open; the source and live build differ from it.

**Style, live:** a `.dropdown-wrapper` below the `⋯` button, aligned right, 194px wide, background `rgb(241,243,245)`, 6px radius. Items are links, 14px, 36px tall, padding `8px 16px`, `rgb(104,112,118)`. The menu closes on a click outside it. It has no `role="menu"` and its items are not focusable links (no `href`).

### Width and layout

- **Width:** 40% of the window by default (512px at 1280px, 400px at 1000px, 280px at 700px), at most 60vw ([`container.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.css#L590-L625)). The sidebar is a full-height column from the top of the window; the header and the main column shrink to the remaining width (main 768px at 1280px).
- **Resize:** a 3px `role="separator"` handle on the sidebar's left edge (`aria-label` `Right sidebar resize handler`, `aria-valuemin` 10, `aria-valuemax` 70, focusable). Dragging sets the width between `max(320px, 10%)` and 70% of the window, and dragging below half the minimum closes the sidebar; Left and Right arrows move it by 5px. The width is saved in `localStorage` as `ls-right-sidebar-width` ([`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L336-L421), [`handler/ui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/ui.cljs#L23-L31)). Dragging was not exercised in the measurement.
- **Card list:** 16px side padding, 150px bottom padding, scrolls on its own.
- Cards can be dragged to reorder them.

### Phone width

At 390×844 the toggle is absent and Shift-click does nothing. `t r` still opens the sidebar, as a 156px column (40%) beside a 234px main column, with the Contents card.

### Performance in Logseq

Every card renders from the in-browser database, so the sidebar opens at once, but only after the whole graph has loaded: `static/js/db.gz` (19.8 MB) and `main.js` (7.5 MB). On the desktop profile used here, `tmux` took 22.5 s to show its title. Each page card renders the full page outline and its reference sections in the sidebar; nothing limits how many cards stay rendered.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/shell.rs`, `shell.js`, `outline.js` and the [published app shell spec](../../openspec/specs/published-app-shell/spec.md), checked against the whole-garden build:

- **No right sidebar.** The header holds Toggle left sidebar, Home, Search garden and More. The spec requires that the header contains only controls whose destination exists, and names the right sidebar as one that does not yet.
- **`t r`** is not handled; the shell handles `t l`, `t t`, `t w`, `g h`, `g a` and `g j`.
- **Shift-click on a link** is left to the browser, so Chromium opens the page in a new window (measured on `tmux`).
- **Shift-click on a bullet** zooms into the block like a plain click (`tmux` → `#zoom/1.1`); `outline.js` does not check the Shift key.
- **Block anchors.** Only blocks with an `id::` property get `id="block-<uuid>"`; other blocks are reached by position (`#zoom/1.1`) or by `source-block-N` spans.
- **Unresolved references** (pages with no file, such as `VSCode/Terminal`) render as plain text, so there is nothing to Shift-click.
- **Contents.** The build has no `contents` page; the comparison harness lists it among Logseq-only pages.
- **Reference fragments.** Linked and unlinked reference bodies are already separate files under `_refs/`, loaded when a section opens (`lsg-references.js`). A sidebar card can load them the same way.
- **Sizes.** `lsg-shell.js` is 5,662 bytes of the 6 KB `[shell]` budget. On `tmux` the header and left sidebar markup is 3,522 bytes. Pages are small: median 27 KB, of which about 23.6 KB (6.7 KB gzipped) is the inlined head and shell before `<main>`; the largest, `Person`, is 508 KB (52 KB gzipped).
- **Where the code lives.** The shell is publisher code in the `site` module. Favorites and feature flags come from SDK settings operations.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Toggle and `t r`.** A Toggle right sidebar button at the right end of the header, after More, with the `layout-sidebar-right` icon, an accessible name and `aria-expanded`; hidden below 640px and when JavaScript is off. `t r` toggles the sidebar outside text fields. Check: on `tmux`, the header order is Toggle left sidebar, Home, Search, More, Toggle right sidebar, and `t r` opens and closes the sidebar.
- 2. **Empty state and Contents.** Opening an empty sidebar shows a Contents card with the outline of the `contents` page and no title; the first added card replaces it. `alt+shift+c` toggles the Contents card. `lsg` must publish the `contents` page, or at least its outline, even when it is one empty bullet. Check: on `tmux`, `t r` shows one Contents card with one empty block; Shift-clicking a link leaves one card.
- 3. **Shift-click.** Shift-click on a page reference, tag, page-reference property value, breadcrumb crumb or reference group title adds a card for that page; on a bullet a block card; on a block reference or block reference count a Block references card. The main page does not change. A target already in the sidebar moves to the top and expands. Links inside cards work the same way. Check: on `tmux`, Shift-clicking `tmux/Idea/Pomodoro with status line and silent flash` adds a page card and Shift-clicking the second bullet adds a block card whose header is the breadcrumb starting `tmux`; on `VSCode/Proj/24t3/LearnVSCode/01 Terminal`, Shift-clicking the block reference adds a `Block references` card; the URL stays the same throughout.
- 4. **Card content.** A page card shows the page's properties and outline without its title, then Pages tagged with, Linked References (header only until opened) and Hierarchy, without Unlinked References. A block card shows the block with its children under a breadcrumb header; a Block references card shows `Block references`, a breadcrumb row and the block. Check: the `tmux/Idea/Pomodoro with status line and silent flash` card opened from `tmux` has no `h1`, shows the outline from the heading `Pomodoro in tmux: status-line countdown and silent visual cue`, and shows Linked References as a closed header.
- 5. **Card header and collapse.** Chevron, icon and truncated title in a button with `aria-expanded` and `aria-controls`; `⋯` and `×` buttons with accessible names; middle click on the header closes the card; a collapsed card shows only its header. Check: clicking the header of a card on `tmux` collapses it to one row and back.
- 6. **Card menu.** The items and separators in the table above, depending on card type, collapsed state and card count, including Collapse all, Expand all and Open as page. The menu must also work from the keyboard (focusable items, arrow keys, Escape), which Logseq's does not. Check: with three cards, the first card's menu lists Close, Close others, Close all, Collapse, Collapse others, Collapse all, Expand all; Close all empties and closes the sidebar.
- 7. **Top bar.** Contents, Page graph and Help buttons in a 48px sticky bar with the measured styles. Page graph needs #120; until then the proposal decides whether to leave it out, as the app shell spec does for missing destinations. Check: each shown button adds its card at the top.
- 8. **Help card.** A Help card exists, but copying Logseq's links to its forum, roadmap and bug tracker would send readers of a garden to Logseq's support. The proposal decides the content; one option is the shortcuts the published site supports (`t l`, `t r`, `t t`, `t w`, `g h`, `g a`, `g j`, `mod+k`, Shift-click) in place of Logseq's 157-shortcut list. Check: every link and shortcut in the Help card works on the published site.
- 9. **Layout and width.** At 640px and wider, the sidebar takes 40% of the window (at most 60vw) at the right and the header and main column shrink to the rest; the card list scrolls on its own. Check: at 1280px on `tmux`, the sidebar is 512px wide and the main column 768px, with no sideways scroll.
- 10. **Resize.** A focusable separator on the sidebar's left edge resizes it between `max(320px, 10%)` and 70% by drag and by arrow keys, and saves the width. Check: drag to 600px at 1280px, reload, and the sidebar is 600px wide.
- 11. **Saved state.** The open state, the cards in order and their collapsed flags survive navigation and reloads, stored by route rather than by database id, with an unavailable `localStorage` keeping the state for the current page only. Restoring must not fetch card content before the main page's title is visible. Check: open three cards on `tmux`, follow a link, reload, and the same three cards appear in the same order.
- 12. **Config start state.** `:default-home {:sidebar …}` in `logseq/config.edn` (a page name, `"Contents"`, or a list) adds those cards on a first visit. Check: a fixture garden with `:default-home {:sidebar ["Contents" "tmux"]}` opens with those two cards.
- 13. **Phone width.** Below 640px there is no toggle and no card is added. Logseq swallows the Shift-click and opens a 156px column on `t r`; `lsg` should let the link work and keep `t r` from opening a column that narrow (the proposal decides between ignoring it and an overlay). Check: at 390px, `parity:screenshots` shows no toggle, the page has no sideways scroll, and a Shift-click follows the link.
- 14. **Performance.** The sidebar adds nothing to a page's first paint: one button in the frame, a few lines of key and click handling in `lsg-shell.js`, and a separate `lsg-sidebar.js` loaded on first use. A page card fetches the target page's published HTML (or a content-only fragment the publisher writes) and the reference bodies only when their sections open; a block card fetches its page and selects the block by anchor. Check: `parity:budgets` passes with every fixture page's `[desktop]` and `[phone]` first-paint values unchanged; opening a `Person` page card (the largest page) stays within the new `[sidebar]` budgets below.

## Performance

Logseq makes the sidebar possible by loading the whole graph into the browser. `lsg` must not do that ([#106](https://github.com/codekiln/logseq-gardener/issues/106)). Instead:

- **No graph in the browser.** Each card fetches one published file: the target page, or a content-only fragment of it. Reference bodies come from the existing `_refs/` fragments when a reader opens them. A typical page is 27 KB (about 8 KB gzipped), and the browser can reuse a cached copy if the reader visits the page later.
- **Content-only fragments are optional.** About 23.6 KB of each page is the inlined head and shell. If fetching whole pages exceeds the card budget, the publisher can write each page's `<main>` content as a separate file; the proposal should measure both before adding 10,756 files.
- **Bounded script.** `lsg-shell.js` has 338 bytes of headroom under `[shell]` `script_kb = 6`. The toggle, `t r` and Shift-click delegation must either fit, or the proposal raises the budget with measurements. Everything else goes in `lsg-sidebar.js`, loaded on the first toggle, `t r` or Shift-click, never during first paint.
- **Bounded memory.** Collapsed cards can drop their rendered body and re-render from the cached response when expanded, so memory grows with open cards, not with cards ever added.

Budget rows that apply:

- `[desktop]` and `[phone]`: `data_before_title_kb`, `script_before_title_kb`, `title_ms` and `peak_heap_mb` must not change for any fixture page, including when the saved state has cards to restore.
- `[shell]`: `script_kb` (the hook in `lsg-shell.js`) and `frame_kb` (the toggle button's markup; the header and left sidebar use 3,522 bytes on `tmux`).
- A new `[sidebar]` table in the proposal, for example `script_kb` for `lsg-sidebar.js`, `card_open_ms_desktop` and `card_open_ms_phone` (from Shift-click to the card's outline), `card_fetch_gzip_kb` for one card, and `heap_mb` on the phone profile after opening ten cards. `[references]` (`script_kb = 6`, `open_ms_desktop = 300`, `open_ms_phone = 1500`) is a reasonable starting point.

## Where the work belongs

- **SDK:** resolving a link, tag or alias to the published page it opens (already used by page links and favorites); a stable identity and anchor for every block, so a block card and a Block references card can address a block that has no `id::`; each block's ancestors for the breadcrumb header (the same answer reference breadcrumbs use); the `contents` page, published even when its file is one empty bullet; and reading `:default-home :sidebar` from `config.edn` with the existing bounded EDN reader.
- **Publisher:** the toggle button and its markup, the sidebar column and card styles, `lsg-sidebar.js` (cards, menus, Shift-click, resize, saved state), the Help card's content, writing a block anchor for every block, and any content-only page fragments.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `tmux` | Header with the toggle; many page links and bullets to Shift-click; a favorite | Shift-click on a bullet adds a block card titled `tmux` | Shift-click opens a new window or zooms |
| `VSCode/Proj/24t3/LearnVSCode/01 Terminal` | A block reference to a block on the same page, a tag (`#Goal`), a link to a page with no file (`VSCode/Terminal`), and a sibling of the `03 Basic Python` fixture | Block references, `Goal` and `VSCode/Terminal` cards | No page links: `Goal` and `VSCode/Terminal` have no file and render as text; the block reference links to `#block-6763d7c8-…` |
| `contents` | The Contents card's page; one empty bullet | Shown in the Contents card | Not published |
| `Person` | The largest `lsg` page, for the card size budget | not measured | 508 KB, 52 KB gzipped |
| Home route | Page graph on the home route uses today's journal; toggle present | not measured | — |

Add `VSCode/Proj/24t3/LearnVSCode/01 Terminal` and `contents` to [`scripts/parity/fixtures.txt`](../../scripts/parity/fixtures.txt) in the Apply step. `parity:screenshots` captures pages with the sidebar closed; the Apply step needs a capture with cards open, for example `tmux` with a page card, a block card and a Block references card.

## Out of scope

- The Page graph card's graph, which belongs with [Issue #120 — Add Logseq's graph view and page graph](https://github.com/codekiln/logseq-gardener/issues/120). This issue only places its button.
- Opening search results in the sidebar (Shift+Enter, Shift-click on a result) and search cards (Cmd+Enter), which belong with [Issue #118 — Close the remaining gaps between Logseq search and the static search](https://github.com/codekiln/logseq-gardener/issues/118). Once a card API exists, #118 can call it.
- Slide decks in the sidebar, which belong with [Issue #121 — Add View as slides](https://github.com/codekiln/logseq-gardener/issues/121).
- Shift-click on rows of the All pages table, which belongs with [Issue #117 — Publish Logseq's All pages table](https://github.com/codekiln/logseq-gardener/issues/117) once this issue provides the card API.
- Which reference sections a page card shows follows [Issue #115 — Match Logseq's linked references, filters, unlinked references and tagged pages](https://github.com/codekiln/logseq-gardener/issues/115) and [Issue #116 — Match Logseq's namespace hierarchy section](https://github.com/codekiln/logseq-gardener/issues/116); this issue reuses their output.
- Publishing pages that exist only as references (such as `VSCode/Terminal`), which Logseq shows in cards. The [comparison harness analysis](comparison-harness.md) counts these pages; no parity issue covers publishing them yet.
- Editing in cards, dragging blocks into or out of the sidebar, dragging cards to reorder them, and the development-only History card.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
