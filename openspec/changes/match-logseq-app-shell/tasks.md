## 1. SDK settings and favorites

- [ ] 1.1 Move the bounded EDN reader out of `site/config.rs` into a module both the site and the new settings module call, with its existing tests
- [ ] 1.2 Add the public `garden_settings` module: read `:favorites` and the three `:feature/enable-*` flags with defaults and diagnostics; unit-test absent, malformed and valid configs
- [ ] 1.3 Add `resolve_favorites` over the selected title and alias lookup, keeping config order and first occurrence, with diagnostics for unresolved entries; unit-test alias, duplicate, private, excluded and missing cases
- [ ] 1.4 Add the unresolved-favorite diagnostics to the publish report and the CLI output

## 2. Shell markup and styles

- [ ] 2.1 Render the header (Toggle left sidebar, Home except on `index.html`, More) and the sidebar (All pages, FAVORITES) from an ordered item list in `html_document`, with accessible names, `aria-current` and Tabler icon paths
- [ ] 2.2 Record the Tabler Icons notice following `docs/licensing.md`
- [ ] 2.3 Replace the default palette in `theme.css` with Logseq's indigo values, add `--lsg-sidebar` and `--lsg-header`, and check every default text and control pair for 4.5:1 contrast in both themes
- [ ] 2.4 Add the inline shell styles: sticky 48px header, 246px pushing sidebar from 640px, 74vw overlay and shade below 640px, 810px and 1280px columns, 16/32px padding, body font stack at 16px
- [ ] 2.5 Extend the `<head>` theme snippet to apply the saved sidebar and wide-mode state before paint at 640px and wider

## 3. Shell behaviour

- [ ] 3.1 Write `lsg-shell.js` once to the site root and reference it with `defer` from every page
- [ ] 3.2 Implement the sidebar toggle, saved state, overlay closing (shade, link, Escape) and focus return
- [ ] 3.3 Implement the More menu: keyboard opening, arrow keys, Escape and outside-click closing, Toggle theme and Use system theme
- [ ] 3.4 Move appearance changes into `theme.js` functions the menu and `t t` call; keep host-inserted selectors and `window.logseqGardenerInitTheme(root)` working; remove the default Appearance selector
- [ ] 3.5 Implement `t l`, `t t`, `t w`, `g h` and `g a` with the one-second window and the form-field and modifier exclusions
- [ ] 3.6 Hide the toggle and More without JavaScript; keep Home and content usable

## 4. Tests and budgets

- [ ] 4.1 Add publication tests for header order, Home on `index.html`, All pages target with and without `--homepage`, favorites order, alias resolution, current-page marking and the shared script reference
- [ ] 4.2 Add browser-controller tests for the sidebar, More menu, theme toggle, shortcuts and storage failures
- [ ] 4.3 Add three favorites (one through an alias, one missing) to `tests/fixtures/parity-garden/logseq/config.edn` and the `lsg-shell.js` and per-page shell size checks to `parity:budgets` and `scripts/parity/budgets.toml`
- [ ] 4.4 Add a 390×844 capture to `parity:screenshots`
- [ ] 4.5 Update help text if publish output changes, and run `mise run clilint:check`

## 5. Verification

- [ ] 5.1 Build the encode garden with `parity:build` and compare `tmux`, the home page, `gh-dash` and `Lazygit` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record screenshots and computed colours and widths in `verification.md`
- [ ] 5.2 Run `parity:perf --record` and commit the new rows in `docs/parity/performance.md`
- [ ] 5.3 Check a Gitpa build: its inserted navigation, appearance selector and fragment swaps still work
- [ ] 5.4 Run `mise run ci`
