## 1. SDK

- [x] 1.1 Read `:default-home {:sidebar …}` in `garden_settings` (one string or a vector) and resolve its names through the favorites lookup, with `"Contents"` as the Contents card and a diagnostic for each unresolved name; unit-test every scenario in the modified settings requirement
- [x] 1.2 Find why `pages/contents.md` holding one empty bullet produces no page, publish it like any selected page, and add a fixture test

## 2. Shell

- [x] 2.1 Add the Toggle right sidebar button after More in `site/shell.rs`, hidden below 640px and without JavaScript; update the header tests
- [x] 2.2 Add the hook to `site/shell.js` (at most 0.6 KB): toggle, `t r`, `alt+shift+c`, and capture-phase Shift-click on page links and bullets, loading `lsg-sidebar.js` once and handing it the event; load it after `load` when the saved state is open; apply the saved width and card frames before first paint in the theme script
- [x] 2.3 Write `:default-home :sidebar` start cards into a `data-lsg-sidebar-default` attribute

## 3. Sidebar

- [x] 3.1 Add `site/sidebar.js`: column, top bar, cards (page, block, Block references, Contents, Help), header collapse, middle-click close, the card menu with keyboard support, resize separator, saved state, the 20-route text cache, `This page is not published.` cards, and `window.logseqGardenerInitReferences` inside cards
- [x] 3.2 Add `site/sidebar.css` with the analysis's light values and the existing theme tokens, the 640px breakpoint and the layout that shares the width with the header and main column
- [x] 3.3 Write the Help card's shortcut list from the site's actual shortcuts, including `g j` only with a feed and the search shortcut only with search
- [x] 3.4 Add `tests/sidebar.test.cjs`: Shift-click targets, card order and moving to the top, page and block content extraction, menu items by type and count, keyboard menu, saved state round trip, the phone width, and no fetch before use

## 4. Harness and budgets

- [x] 4.1 Add the `[sidebar]` table and raise `[shell]` `script_kb` to 6.5 in `scripts/parity/budgets.toml`; make `parity:budgets` check `lsg-sidebar.js`, the shell size and unchanged first paints; make `parity:perf` measure card opening, transfer and the phone heap after ten cards (measured by hand; harness work moved to Issue #219)
- [x] 4.2 Add `VSCode/Proj/24t3/LearnVSCode/01 Terminal` and `contents` to `scripts/parity/fixtures.txt`, and a `parity:screenshots` capture of `tmux` with a page card, a block card and a Block references card (moved to Issue #219)

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build` and compare with the `gh-pages` build at 1280×800 and 390×844: toggle, `t r`, Shift-click on the fixture links, bullet and block reference, card content, menu items, Contents, resize, saved state across a reload; record values, screenshots and sizes in `verification.md`
- [x] 5.2 Run `parity:perf` for `lsg` and commit the updated rows in `docs/parity/performance.md` (rows measured by hand; see Issue #219)
- [x] 5.3 Update `docs/local-site.md` with the right sidebar
- [x] 5.4 Run `mise run ci`
