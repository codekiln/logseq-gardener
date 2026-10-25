## 1. SDK

- [ ] 1.1 Add `GardenSettings::show_brackets` read from `:ui/show-brackets?`, with a diagnostic for a non-boolean value; unit-test missing, commented-out, `false`, `true` and string values
- [ ] 1.2 Add the `inline_references` module with `tag_page_name`; unit-test `#Keyshort`, `#[[multi word]]` and a tag and a page reference resolving to the same aliased page
- [ ] 1.3 Make `site/search.rs` use `tag_page_name`, with the search tests passing unchanged

## 2. Markup

- [ ] 2.1 Render page references and labelled references in `span.page-reference` with bracket spans when brackets are on, moving the `unresolved` class to the outer span; extend the render tests for both settings
- [ ] 2.2 Render tags as `a.tag` or `span.tag`, and as text inside link labels; add render tests
- [ ] 2.3 Add `external-link`, `target="_blank"` and `rel="noopener noreferrer"` to `http` and `https` links, leaving `mailto:` unchanged
- [ ] 2.4 Read image size maps into `width` and `height` attributes and add `loading="lazy"` and `title`; test valid, partial and invalid maps
- [ ] 2.5 Dispatch macros: unknown names to the warning chip, `renderer` to nothing, owned names unchanged; test `{{pdf x}}`, `{{renderer x}}` and `{{query x}}`

## 3. Styles

- [ ] 3.1 Add the theme tokens for both themes to `site/theme.css`, light values from the analysis and dark values from Logseq's dark theme
- [ ] 3.2 Add `site/inline.css` and inline it into every page next to the outline styles

## 4. Harness and budgets

- [ ] 4.1 Add an `[inline]` table to `scripts/parity/budgets.toml` and make `parity:budgets` check the inline styles' gzip size and the `tmux` page growth

## 5. Verification

- [ ] 5.1 Build the encode garden with `parity:build` and compare `tmux`, `1Password/Quick Access`, `1Password/Dev/Connect Server`, `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether` and `LangSmith/26/09/29 Tue - Deep Agents/1045 Engine Workshop/Take-Aways` with the `gh-pages` build at 1280×800 and 390×844 in both themes; record computed styles and screenshots in `verification.md`, including the highlight result
- [ ] 5.2 Run `parity:perf --record` and commit the updated rows in `docs/parity/performance.md`
- [ ] 5.3 Run `mise run ci`
