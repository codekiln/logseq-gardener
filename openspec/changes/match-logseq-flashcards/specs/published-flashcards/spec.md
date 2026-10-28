## ADDED Requirements

### Requirement: Style card blocks like Logseq

A card block SHALL render with Logseq's card style: the `shadow-md` box shadow, padding `12px 0`, margin-bottom 8px and radius 4px. A `#card` tag or a `[[card]]` or `[[Card]]` reference SHALL be drawn at opacity 0.3. The build SHALL report no unresolved-tag diagnostic for `card`.

#### Scenario: Card on its page
- **WHEN** a reader opens `tmux/Keyshort/Pane/Zoom Pane`
- **THEN** the card block's computed padding, radius and box shadow match Logseq's, the `#card` tag has opacity 0.3, and no `card-` property is visible

### Requirement: Show a deck widget for `{{cards}}`

`{{cards QUERY}}` with matching cards SHALL render `.cards-review` holding the `.cards-title` header (the infinity icon, the query text, a caret, the `due/total` counter titled `Overdue/Total`, and the `Toggle preview mode` and `Toggle random mode` buttons) with Logseq's measured styles, then the first due card's breadcrumb and front with the card tag hidden and without its children, then a `Review cards` button. Counts and the first card SHALL be computed at the build time. Activating the widget SHALL open the review for that deck, in preview or random mode when the matching button is used. A query with no matching cards SHALL render a block at opacity 0.6 with `Cards: <query>` as inline code, then `No matched cards`.

#### Scenario: yazi deck
- **WHEN** a reader opens `Logseq/Flashcard/Review/Favorite/yazi` built for 2026-10-27
- **THEN** the widget reads `[[yazi]]` and `18/18`, shows the deck's first due card, the unscheduled card on `git/Alias/md-url`, and no literal `{{cards`

#### Scenario: Empty deck
- **WHEN** a reader opens `Logseq/Flashcard/Review/Favorite/Zoxide`
- **THEN** the page shows `Cards: [[Zoxide]]` as code and `No matched cards`

### Requirement: Review cards in Logseq's dialog

The review dialog SHALL match Logseq's measured size, colours, radius and Close link, with an 836px panel at desktop widths, full width on phones, and a 740px card area at least 60vh tall. Its header SHALL repeat the deck header, and its caret SHALL open `Switch to` listing every garden deck query and then `All`. Each card SHALL show its breadcrumb and front with `#card`, `[[card]]` and `[[Card]]` hidden. The phases SHALL be `Show answers` (`s`), `Show clozes` when the card has clozes, then `Forgotten` (`f`), `Took a while to recall` (`t`) and `Remembered` (`r`) in Logseq's measured colours with key chips. Preview mode SHALL show every card with `Show answers`/`Hide answers`, `Next` (`n`), `Reset` and a `Current/Total` counter. Random mode SHALL shuffle the remaining cards. After the last card the dialog SHALL show Logseq's finished message. Close, Escape, `g f` and `t c` SHALL dismiss it. On open, the dialog SHALL recompute due cards from the reader's clock and update its counter, the deck header and the sidebar badge.

#### Scenario: Deck menu
- **WHEN** a reader opens the Flashcards dialog on the encode garden and opens the caret
- **THEN** `Switch to` lists 30 entries ending with `All`

#### Scenario: Reader's clock
- **WHEN** a site built on 2026-10-20 is opened with the browser clock at 2026-10-27T04:33Z and the reader opens `All`
- **THEN** the counter reads `430/432` and the sidebar badge reads `430`

### Requirement: Score cards for the visit

Scoring SHALL compute the next schedule with Logseq's SM-5 rules and default settings, keep it in memory for the visit, and reduce the due counters and the badge by one; `Forgotten` SHALL schedule the card for the next day. Closing and reopening the dialog SHALL keep the visit's scores. A reload SHALL restore the published counts. Scoring SHALL write nothing to browser storage and SHALL insert no block.

#### Scenario: Remembered
- **WHEN** a reader opens `All`, shows the answers on the first card and chooses `Remembered`
- **THEN** the counter changes from `430/432` to `429/432`, the badge reads `429`, and the next card appears

#### Scenario: Reload
- **WHEN** the reader then reloads the page
- **THEN** the badge reads `430` and `All` starts with the same first card

### Requirement: Render cloze

`{{cloze answer}}` SHALL render `span.cloze` showing `[...]`, and `{{cloze answer \\ cue}}` SHALL show `(cue)`. On a page, activating a cloze SHALL reveal `[answer]`. In the review, the `Show clozes` phase SHALL reveal every cloze on the card.

#### Scenario: Cloze in review
- **WHEN** a reader reviews the parity fixture garden's cloze card
- **THEN** `Show clozes` follows `Show answers` and reveals the answers

### Requirement: Load card data only on use

The build SHALL write one deck file per deck and for `All`, holding card numbers in due order and each card's schedule, and SHALL write rendered cards once each in shared batches of 20. `flashcards.js` and `flashcards.css` SHALL load only when a reader activates a deck widget or the Flashcards item, or presses `g f` or `t c`. Opening a review SHALL fetch the deck file and the batch holding its first card, and later batches as the review advances.

#### Scenario: Page view
- **WHEN** a reader opens the yazi deck page and does not interact with it
- **THEN** no deck file, card batch or `flashcards.js` is requested

### Requirement: Keep flashcards within budgets

`flashcards.js` SHALL be at most 8 KB and `flashcards.css` at most 1.5 KB gzip. The `All` deck file SHALL be at most 8 KB gzip and each card batch at most 30 KB gzip. The yazi deck page SHALL grow by at most 4 KB gzip and stay within the `[desktop]` and `[phone]` first-paint budgets, and every fixture page SHALL keep its size when the fixture garden gains 2,000 pages. Clicking `Flashcards` SHALL show the first card of `All` within 300 ms on desktop and 1,500 ms on the phone profile, and the phone JS heap after reviewing 50 cards SHALL stay within 20 MB. A whole-garden build SHALL be at most 10% slower than the `--baseline-lsg` binary. CI SHALL check the size budgets through `parity:budgets`.

#### Scenario: Budgets in CI
- **WHEN** `parity:budgets` runs on the fixture garden
- **THEN** it checks the `[flashcards]` sizes and fails when one is exceeded
