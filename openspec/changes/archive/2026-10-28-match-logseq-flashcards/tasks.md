## 0. Before starting

- [x] 0.1 Confirm the `match-logseq-queries` Apply step has merged to main, and that the `published-inline-markup` base text in main still matches this change's delta; update the delta if the queries sync changed it

## 1. SDK

- [x] 1.1 Add the `cards` module: card selection in every reference form and case, ignoring code, with schedules read from the six `card-*` properties; test 432 cards on the encode garden and the `[[Card]]` form
- [x] 1.2 Add deck query listing (empty argument as `All`, bare name as a page reference) and deck selection through `query::evaluate` with `(and <query> [[card]])`; test the 29 garden decks' totals from the analysis
- [x] 1.3 Add the due rule and ordering for a given time; test `430/432`, `71/73`, `9/10`, `18/18` and the first `All` card at 2026-10-27T04:33Z
- [x] 1.4 Add the `card-*` keys to the SDK's hidden built-in properties and use them in `site/properties.rs`

## 2. Publisher

- [x] 2.1 Add the `lsg-card` block class and `lsg-card-ref` tag class in the outline renderer; stop reporting `card` as an unresolved tag
- [x] 2.2 Add `site/flashcards.rs`: the `{{cards}}` widget with build-time counts and the first card, the no-match form, and routing of `cards` and `cloze` out of `macro_fallback`
- [x] 2.3 Write `flashcards/decks/<id>.json` for each deck and `All`, and `flashcards/cards/<n>.html` batches of 20 rendered cards
- [x] 2.4 Add the Flashcards sidebar item with the build-time badge, the `:feature/enable-flashcards?` switch, and the loader and `g f` / `t c` shortcuts in `site/shell.rs` and `site/shell.js`
- [x] 2.5 Add `site/flashcards.js`: the dialog, phases, keys, deck menu, preview and random modes, finished message, reader-clock recount, SM-5 scoring in memory, and batch fetching
- [x] 2.6 Add `site/flashcards.css` with Logseq's measured sizes and colours, theme tokens for dark mode, and the phone layout
- [x] 2.7 Render cloze on pages and in the `Show clozes` phase
- [x] 2.8 Update the `published-inline-markup` and `tests/site.rs` tests that expect the literal `{{cards}}`

## 3. Harness and fixtures

- [x] 3.1 Add a cloze card and a deck with no matches to `tests/fixtures/parity-garden/`, and the analysis's fixture pages to `scripts/parity/fixtures.txt`
- [x] 3.2 Add the `[flashcards]` table to `scripts/parity/budgets.toml`, raise `[shell] script_kb` to 6.9, and make `parity:budgets` check the script, styles, deck file, batches and yazi page growth
- [x] 3.3 Make `parity:perf` record clicking `Flashcards` to the first card on desktop and phone and the phone heap after 50 cards; make `parity:screenshots` capture the yazi widget and the open dialog at 1280×800 and 390×844

## 4. Verification

- [x] 4.1 Build the encode garden with `parity:build` and compare the fixture pages with the `gh-pages` build with the browser clock at 2026-10-27T04:33Z: counts, first cards, styles, dialog size and colours, scoring and reload; record values and screenshots in `verification.md`
- [x] 4.2 Run `parity:perf --flashcards` and record the flashcard rows; measured on a fixture run and recorded in `verification.md`, because `--record` rewrites `docs/parity/performance.md` from a full run with Logseq's build, which the next full `parity:perf` run refreshes
- [x] 4.3 Run `mise run ci`
