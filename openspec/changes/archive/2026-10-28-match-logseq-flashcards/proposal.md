## Why

Logseq treats every block that references the `card` page as a flashcard. Its published site shows a `Flashcards` item with a due count in the left sidebar, which opens a review dialog over every card; renders each `{{cards QUERY}}` macro as a deck header with its first card; and draws card blocks with a shadow and a faded `#card` tag. `lsg` has none of this: it prints `{{cards …}}` as literal code and `#card` as a plain unlinked tag ([Flashcards analysis](../../../docs/parity/flashcards.md), [Issue #123 — Publish Logseq's flashcards](https://github.com/codekiln/logseq-gardener/issues/123)). The encode garden has 432 cards and 29 `{{cards}}` decks on 27 pages.

Logseq shows the Flashcards count only after loading the whole graph: 23.0 s after navigation on desktop, after a 19.8 MB `db.gz`. `lsg` can select every deck at build time, show each deck's header and first card as static HTML, and load card content only when a reader opens a review.

## What Changes

- The SDK selects cards: blocks whose references include `card` in any form and case, with their children as the back. It selects each deck's cards with the query evaluator from [Issue #124 — Render Logseq queries at build time](https://github.com/codekiln/logseq-gardener/issues/124) as `(and <deck query> [[card]])`, orders due cards as Logseq does, and decides which cards are due at a given time.
- The six `card-*` scheduling properties join the SDK's hidden built-in properties, so no view ever shows them.
- Card blocks get Logseq's shadow, padding, margin and radius, and the `card` tag or reference is drawn at opacity 0.3. `card` no longer produces an unresolved-tag diagnostic.
- The left sidebar gains `Flashcards` between Journals and Graph view, with the `infinity` icon, the `g f` hint and a badge holding the number of cards due at build time. `g f` and `t c` open the review dialog. A garden with `:feature/enable-flashcards? false` shows no item.
- `{{cards QUERY}}` renders Logseq's `.cards-review` widget: the deck header with the query text, `due/total`, and the preview and random buttons, then the first due card's breadcrumb and front, then `Review cards`. A deck with no cards shows `Cards: <query>` and `No matched cards`.
- The review dialog matches Logseq's size, colours and controls: the header with the `Switch to` deck menu, the breadcrumb and front, `Show answers`, then `Forgotten`, `Took a while to recall` and `Remembered` with their keys, preview mode, random mode, the finished message, Close and Escape.
- Scoring follows Logseq's published site: SM-5 schedules the card for the rest of the visit, the counts fall, and a reload restores the published state. Nothing is stored.
- `{{cloze …}}` renders as Logseq does on pages and adds the `Show clozes` phase in the review.
- The build writes one small deck file per deck and card content in shared batches. `flashcards.js` and its styles load only when a reader opens a deck or the Flashcards item. When a review opens, the script recomputes due cards from the reader's clock.
- A new `[flashcards]` budget table caps the script, styles, deck file, card batches, deck-page growth, time to the first card and phone memory during a review.

## Capabilities

### New Capabilities

- `garden-card-decks`: the SDK operations that select cards and decks, list the garden's deck queries, order and count due cards for a given time, and hide the `card-*` properties.
- `published-flashcards`: card block styles, the `{{cards}}` widget, the review dialog, scoring, cloze, deck and card files, loading and budgets.

### Modified Capabilities

- `published-app-shell`: the left sidebar gains the Flashcards item, the shell handles `g f` and `t c`, and the shell script budget grows by 0.3 KB for the review loader.
- `published-inline-markup`: `cards` and `cloze` leave the list of macros that keep their literal rendering.

## Impact

- SDK: a new `cards` module using `query::evaluate` from the `match-logseq-queries` change, `ReferenceIndex` path references, and the build time; the hidden built-in property list in `site/properties.rs` moves its `card-*` keys into the SDK. The CLI and language server can answer "which cards does this deck hold" with the same operation.
- Publisher: the `cards` and `cloze` macro arms in `site/render.rs`; card block classes in the outline renderer; new `site/flashcards.rs`, `site/flashcards.js` and `site/flashcards.css`; the sidebar item and loader in `site/shell.rs` and `site/shell.js`; deck files under `flashcards/decks/` and card batches under `flashcards/cards/`.
- Parity harness: a `[flashcards]` table in `scripts/parity/budgets.toml`, `parity:budgets` and `parity:perf` rows for the yazi deck page and the `All` review, `parity:screenshots` of the widget and the dialog, and a cloze page in the parity fixture garden.
- Order of work: this change's Apply step starts after the `match-logseq-queries` Apply step merges, because deck selection calls its evaluator.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
