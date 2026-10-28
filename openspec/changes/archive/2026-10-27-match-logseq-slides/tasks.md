## 1. SDK and page markup

- [x] 1.1 Find why `background-image::` is absent from the published `Logseq/Doc/slide` page; keep slide keys (`background-*`, `data-*`, `transition`, `transition-speed`, `auto-animate`, `visibility`) in the block's property list, hidden from property rows; unit-test with a fixture block
- [x] 1.2 Write `data-slide-<key>` attributes on block items in `site/outline.rs`, with a leading `data-` written once and values escaped; test `Logseq/Doc/slide`-shaped and `tmux`-shaped fixtures (no attribute)

## 2. Menu item and card

- [x] 2.1 Add the `View as slides` button as the first More item on page routes in `site/shell.rs`, absent on `index.html` and All pages, hidden below 640px; update the shell tests
- [x] 2.2 Handle the item in `site/shell.js` by passing `{type: 'slides', route}` to the sidebar loader; raise `[shell]` `script_kb` to 6.6 and confirm `lsg-shell.js` fits
- [x] 2.3 Add the `slides` card type to `site/sidebar.js`: title link, `presentation` icon, tip line, 400px deck, Open as page in its menu, saved state, loading `lsg-slides.js` once and passing the outline from the live document or the cached page file

## 3. Deck

- [x] 3.1 Add `site/slides.js`: column and slide order from the outline, dropping empty slides and empty columns; slide content cloned without bullets, controls, property rows and children; only the current slide and its neighbours in the document
- [x] 3.2 Add the controls, progress bar, focus handling, keys, `g` jump field, overview and blackout; ignore shell and sidebar shortcuts while the deck has focus
- [x] 3.3 Add fullscreen on `f` with rescaling, and `f` to leave
- [x] 3.4 Apply `data-slide-*` attributes as `data-<key>` on sections, with backgrounds and `transition`
- [x] 3.5 Add `site/slides.css` (served with the script) with the 960×700 scaled slide, centring, light values and theme tokens, keeping the `.reveal` structure `custom.css` targets
- [x] 3.6 Test the slide order, empty-slide rule and key handling in `tests/` against fixture outlines

## 4. Harness and budgets

- [x] 4.1 Add the `[slides]` table (`script_gzip_kb = 6`, `open_ms_desktop = 300`, `step_ms_desktop = 50`, `deck_heap_mb = 10`) to `scripts/parity/budgets.toml` and `[shell]` `script_kb = 6.6`; make `parity:budgets` check `lsg-slides.js` and that no fixture page grows beyond the menu item
- [x] 4.2 Make `parity:perf` open the AI Engineer Summit deck, step through every slide and record open time, step time and heap growth; make `parity:screenshots` capture the deck on its first slide at 1280×800 and in fullscreen

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build`; compare the AI Engineer Summit deck with the `gh-pages` build: column and slide counts (106 against Logseq's 75), the `flash back` path, controls, progress, `f`, and computed styles against the analysis's light values; record values and screenshots in `verification.md`
- [x] 5.2 Check `Logseq/Doc/slide` backgrounds, `2026-10-23 Fri` columns, the home page and the 390×844 menu without the item
- [x] 5.3 Run `parity:perf --record` and commit the slides rows in `docs/parity/performance.md`
- [x] 5.4 Run `mise run ci`
