## Why

Logseq's right sidebar lets a reader keep other pages and blocks open beside the current page: Shift-click a link, tag, block reference or bullet and a card for it appears in a second column. `lsg` has no right sidebar, so Shift-click opens a browser window or zooms into the block, and `t r` does nothing ([Right sidebar analysis](../../../docs/parity/right-sidebar.md), [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119)).

Logseq fills cards from the whole graph it loads into the browser, which takes 22.5 s before the title of `tmux` shows on the desktop profile. `lsg` can fill each card from the page file it already publishes, fetched when the reader asks for the card, so no page's first paint changes.

## What Changes

- A Toggle right sidebar button at the right end of the header, after More, and the `t r` shortcut open and close a right column of cards at 640px and wider. The column is 40% of the window, pushes the page left, scrolls on its own and can be resized with a separator.
- Shift-click on a page reference, tag, page-valued property, breadcrumb or reference group title adds a page card; on a bullet, a block card; on a block reference, a Block references card. The main page stays put. A target already in the sidebar moves to the top. Links inside cards behave the same way.
- A page card shows the page without its title: properties, outline, Pages tagged with, Linked References (closed until opened) and Hierarchy, without Unlinked References. A block card shows the block and its children under a breadcrumb header.
- Each card has a collapse chevron, a `⋯` menu (Close, Close others, Close all, Collapse or Expand, Collapse others, Collapse all, Expand all, and Open as page for page cards) usable from the keyboard, and a Close button. A middle click on a card header closes it.
- An empty sidebar shows a Contents card with the garden's `contents` page; `alt+shift+c` toggles it. A top bar adds Contents and Help cards. The Help card lists the shortcuts the published site supports.
- The open state, width, cards and their collapsed flags survive navigation and reloads, stored by route. `:default-home {:sidebar …}` in `logseq/config.edn` sets the cards of a first visit.
- `lsg-shell.js` gains a small hook for the toggle, `t r` and Shift-click; everything else lives in `lsg-sidebar.js`, which loads on first use. Cards fetch the published page file and reuse the existing `_refs/` fragments.
- The `contents` page is published even when its file is one empty bullet.
- Below 640px there is no toggle, `t r` is ignored and Shift-click follows the link.
- New `[sidebar]` budgets cap the script, the time from Shift-click to a card, one card's transfer and the phone heap after ten cards. The `[shell]` script budget rises from 6 KB to 6.5 KB for the hook.

## Capabilities

### New Capabilities

- `published-right-sidebar`: the right sidebar column, its cards, menus, Shift-click, top bar, Contents and Help cards, saved state, phone behaviour and budgets.

### Modified Capabilities

- `published-app-shell`: the header gains the Toggle right sidebar button; the shortcuts gain `t r` and `alt+shift+c`; the SDK settings operation also reads `:default-home :sidebar`; the shell script budget becomes 6.5 KB.

## Impact

- SDK: `garden_settings` reads `:default-home :sidebar`; the publisher's block addresses come from the existing outline identifiers (`id::` UUID, or position such as `3.1.2`) that block zoom already uses; a single empty bullet page is published like any page.
- Publisher: header markup in `site/shell.rs`, the hook in `site/shell.js`, new `site/sidebar.js` and `site/sidebar.css`, the Help card's content, and a `data-lsg-sidebar-default` attribute for configured start cards.
- Parity harness: `scripts/parity/budgets.toml` gains a `[sidebar]` table and a raised `[shell]` `script_kb`; `parity:budgets` checks the sizes, `parity:perf` measures card opening and heap; `scripts/parity/fixtures.txt` gains `VSCode/Proj/24t3/LearnVSCode/01 Terminal` and `contents`.
- Gitpa receives the sidebar as part of the shared shell. Shift-click is handled on the document, so content a host swaps in works without a new call.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
