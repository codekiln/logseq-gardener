## 1. SDK outline model

- [x] 1.1 Add the public `outline` module with `Outline::from_blocks`, `OutlineBlock` (depth, parent, children flag, `TaskMarker`, priority, heading level, authored collapsed state, span) and `BlockId` (`Uuid`, `Path`) with `parse` and display; unit-test nested headings and tasks, `heading:: true` at depths 1 and 6, `heading:: 3`, CANCELLED, and a collapsed leaf
- [x] 1.2 Add `Outline::find` and `Outline::ancestors`; unit-test position paths, UUID case, unknown and malformed identifiers, and identical identifiers across two loads of the same source
- [x] 1.3 Make `site/render.rs` read collapse state, heading level, marker and priority from `Outline`, with the existing collapse, transclusion and anchor tests passing unchanged

## 2. Markup and CSS

- [x] 2.1 Add the `outline` class to page and feed-article root lists and the constant bullet element to each block, inside the `summary` for authored collapsed blocks; drop the empty paragraph after page properties
- [x] 2.2 Render task checkboxes, marker labels, struck-through DONE and CANCELED text, and bracketed priorities linked to published pages; extend the render tests
- [x] 2.3 Add the title, bullet, guide, spacing, heading, task and closed-bullet CSS under `.outline` with a `--lsg-guide` theme token for both themes, hiding the native disclosure triangle
- [x] 2.4 Add publication tests: every page and feed root outline has the class, Linked References contexts do not, task and heading markup on fixtures, and authored disclosures still open without JavaScript

## 3. Outline script

- [x] 3.1 Write `lsg-outline.js` once to the site root and load it from `lsg-shell.js` after `load` on every page
- [x] 3.2 Add the hover arrow and the guide strip toggling `folded` or an authored `details`; make `fragments.js` unfold ancestors of a linked block
- [x] 3.3 Add block zoom: `#zoom/<id>` from the bullet, the zoomed view, the breadcrumb with `⋯`, restoring the page view and collapse state, feed bullets opening the journal page, and unknown identifiers leaving the page view
- [x] 3.4 Add browser-controller tests for collapse, fragment unfolding, zoom by UUID and by path, Back, and content swapped into `main`

## 4. Harness and budgets

- [x] 4.1 Add an `[outline]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check the script size and the largest fixture page's size
- [x] 4.2 Add the zoom-time row on `tmux` to `parity:perf` for both profiles

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build` and compare `1Password/Account/Service`, `rulesync/log/25/10/04 Sat - Issues with gitignore`, `rw-cli` (page and zoom into `1.1.1.2`), `AI/Coding/Todos` and `tmux` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record computed styles, bullet positions and screenshots in `verification.md`
- [x] 5.2 Run `parity:perf --record` and commit the new rows in `docs/parity/performance.md`
- [x] 5.3 Check a Gitpa build: outlines restyled, navigation that swaps `main` keeps collapse and zoom working
- [x] 5.4 Run `mise run ci`
