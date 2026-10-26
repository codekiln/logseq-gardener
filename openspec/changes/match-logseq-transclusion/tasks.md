## 1. SDK

- [x] 1.1 Add the `transclusion` module with `ReferenceTargets`, moving the target index out of `site/render.rs`; register quote bullets and drop duplicate uuids; unit-test both
- [x] 1.2 Add `reference_content`; unit-test a `DONE` task with a page reference, a block with properties and children, a heading and a quote bullet
- [x] 1.3 Add `embed_outline` with the shared nesting limit of 5; unit-test block and page embeds, unavailable and withheld targets, a self embed and a chain of seven embeds

## 2. Markup

- [x] 2.1 Render block references as `span.block-ref` with the SDK's content and invalid references as `span.warning`; extend the render tests
- [x] 2.2 Render embeds as `div.embed-block` and `div.embed-page` wells with `data-level`, the page header and icon, and embedded outlines through the outline renderer; test unique IDs and `collapsed::` inside wells
- [x] 2.3 Render unavailable embeds, self embeds and depth warnings as the published transclusion spec requires; update the visibility sentinel tests

## 3. Styles

- [x] 3.1 Add the embed shade tokens for both themes to `site/theme.css`, measured on the `gh-pages` build in each theme
- [x] 3.2 Add `site/transclusion.css` and inline it into every page next to the outline styles

## 4. Harness and budgets

- [x] 4.1 Add a `[transclusion]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check the style size and that a fixture page with embeds stays within the first-paint budget

## 5. Verification

- [x] 5.1 Build the encode garden with `parity:build` and compare `Claude/Code`, `Spotify/Tool/TUI`, `AI/Coding/History`, `Py/Zen of Python`, `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` and `VSCode/Proj/24t3/LearnVSCode/03 Basic Python` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record computed styles, screenshots, fallback counts and page sizes before and after in `verification.md`
- [x] 5.2 Run `parity:perf --record` and commit the updated rows in `docs/parity/performance.md`
- [x] 5.3 Run `mise run ci`
