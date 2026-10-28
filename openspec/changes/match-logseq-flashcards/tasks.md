## 0. Before starting

- [ ] 0.1 Confirm the `match-logseq-queries` Apply step has merged to main, and that the `published-inline-markup` base text in main still matches this change's delta; update the delta if the queries sync changed it

## 1. SDK

- [ ] 1.1 Add the `cards` module: card selection in every reference form and case, ignoring code, with schedules read from the six `card-*` properties; test 432 cards on the encode garden and the `[[Card]]` form
- [ ] 1.2 Add deck query listing (empty argument as `All`, bare name as a page reference) and deck selection through `query::evaluate` with `(and <query> [[card]])`; test the 29 garden decks' totals from the analysis
- [ ] 1.3 Add the due rule and ordering for a given time; test `430/432`, `71/73`, `9/10`, `18/18` and the first `All` card at 2026-10-27T04:33Z
- [ ] 1.4 Add the `card-*` keys to the SDK's hidden built-in properties and use them in `site/properties.rs`

## 2. Publisher

- [ ] 2.1 Add the `lsg-card` block class and `lsg-card-ref` tag class in the outline renderer; stop reporting `card` as an unresolved tag
- [ ] 2.2 Add `site/flashcards.rs`: the `{{cards}}` widget with build-time counts and the first card, the no-match form, and routing of `cards` and `cloze` out of `macro_fallback`
- [ ] 2.3 Write `flashcards/decks/<id>.json` for each deck and `All`, and `flashcards/cards/<n>.html` batches of 20 rendered cards
- [ ] 2.4 Add the Flashcards sidebar item with the build-time badge, the `:feature/enable-flashcards?` switch, and the loader and `g f` / `t c` shortcuts in `site/shell.rs` and `site/shell.js`
- [ ] 2.5 Add `site/flashcards.js`: the dialog, phases, keys, deck menu, preview and random modes, finished message, reader-clock recount, SM-5 scoring in memory, and batch fetching
- [ ] 2.6 Add `site/flashcards.css` with Logseq's measured sizes and colours, theme tokens for dark mode, and the phone layout
- [ ] 2.7 Render cloze on pages and in the `Show clozes` phase
- [ ] 2.8 Update the `published-inline-markup` and `tests/site.rs` tests that expect the literal `{{cards}}`

## 3. Harness and fixtures

- [ ] 3.1 Add a cloze card and a deck with no matches to `tests/fixtures/parity-garden/`, and the analysis's fixture pages to `scripts/parity/fixtures.txt`
- [ ] 3.2 Add the `[flashcards]` table to `scripts/parity/budgets.toml`, raise `[shell] script_kb` to 6.9, and make `parity:budgets` check the script, styles, deck file, batches and yazi page growth
- [ ] 3.3 Make `parity:perf` record clicking `Flashcards` to the first card on desktop and phone and the phone heap after 50 cards; make `parity:screenshots` capture the yazi widget and the open dialog at 1280×800 and 390×844

## 4. Verification

- [ ] 4.1 Build the encode garden with `parity:build` and compare the fixture pages with the `gh-pages` build with the browser clock at 2026-10-27T04:33Z: counts, first cards, styles, dialog size and colours, scoring and reload; record values and screenshots in `verification.md`
- [ ] 4.2 Run `parity:perf --record` and commit the flashcard rows in `docs/parity/performance.md`
- [ ] 4.3 Run `mise run ci`
