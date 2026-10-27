# Flashcards

Logseq treats every block that references the `card` page (`#card` or `[[card]]`) as a flashcard. Its published site offers three views of them: a `Flashcards` item in the left sidebar with a count of cards due, which opens a review modal over every card; a `{{cards QUERY}}` macro, which shows a small deck header and the first card's front on the page, or `No matched cards`; and the card blocks themselves, drawn with a shadow and a faded `#card` tag. The scheduling properties that Logseq writes under each card (`card-last-interval::`, `card-next-schedule::` and four others) are never shown. On the encode garden there are 432 cards, 430 of them due on 2026-10-27, and 29 `{{cards}}` macros on 27 pages.

`lsg` has none of this. It draws no Flashcards item, prints each `{{cards …}}` macro as literal code, and renders `#card` as an unlinked tag with no card styling. The scheduling properties are hidden only because `lsg` drops every block property, including ones Logseq shows, such as `favorite-deck::`.

Most of the gap is the review UI. Choosing a deck's cards is a query over the whole garden, which belongs in the SDK and should share its evaluator with [Issue #124 — Render Logseq queries at build time](https://github.com/codekiln/logseq-gardener/issues/124). The publisher renders the sidebar item, the deck header, the review modal and the card styles. Performance matters here: Logseq shows the Flashcards count only after the whole graph has loaded (23.0 s on the desktop run below, `static/js/db.gz` 19.8 MB). `lsg` can compute every deck at build time and load card content only when a reader opens a review.

The proposal must settle three decisions. Scores in Logseq's published site last only until the page reloads, so the proposal decides how `lsg` treats scoring (gap 6). Due counts depend on the reader's clock (gap 3). The `{{cards}}` query evaluator is shared with #124 (gap 4).

Measured on 2026-10-27 against the `gh-pages` build of garden commit `083106d` (`gh-pages` `8132c95`, served locally, light theme, 1280×800, Playwright Chromium, browser clock 2026-10-27T04:33Z) and an `lsg` build of the whole garden from `lsg` `48fcc54` (`mise run parity:build`, run `20261027T002614Z`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where they differ from the checkout or from the reference boards. The boards in `docs/logseq-reference/` disagree with the live build in several places:

- `Flashcards.md` draws a 760px white modal with both `1/430` and `[overdue]/430` counters, a `Hide answers` button beside the scoring buttons, and a visible `#card` tag on the card front. The live modal is 836px wide and grey, shows one counter, has no `Hide answers` button while scoring, and hides the tag.
- `Prototype.md` draws the `{{cards}}` widget as a full inline reviewer, lists cards in page-source order, gives the modal no Close button, and says scoring writes nothing. The live widget shows only the first card's front and a `Review cards` button. Cards are ordered by next review date, the modal has a Close button, and scoring changes the counts until the page reloads.
- Both boards use red, blue and green scoring buttons (`#ce2c31`, `#3e63dd`, `#18794e`). The live colours are listed under [Review modal](#review-modal).

### What counts as a card

- A card is a block whose references include the page `card`, written as `#card`, `[[card]]` or `[[Card]]`, since page names ignore case ([`extensions/srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L35-L47), [`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L128-L132)). The card's front is the block. Its back is the block's children.
- A card can contain `{{cloze …}}`, which adds a third phase, `Show clozes` ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L222-L248)). The encode garden has no cloze.
- Measured: `All` reads `430/432`, so the garden has 432 cards. A static count of blocks referencing `card` outside code also finds 432 lines in 268 pages and no journals. 264 cards carry scheduling properties.

### Due cards and the counters

- A card is due when it has no `card-repeats` or fewer than 1, no `card-next-schedule`, or a schedule before the reader's current time ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L291-L308)). The due cards are sorted by the `card-next-schedule` text, so cards that have no schedule come first.
- The deck header shows `due/total` (tooltip `Overdue/Total`), with `due` counting down as the reader scores cards. In preview mode it shows `index/total` (tooltip `Current/Total`) ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L643-L659)).
- Measured counts on 2026-10-27: `All` `430/432`, `[[VSCode]]` `71/73`, `(and [[VSCode/Keyshort/Git]] )` `9/10`, `[[yazi]]` `18/18`. They change as the reader's date passes each `card-next-schedule`.

### Left sidebar `Flashcards`

- Shown unless `config.edn` sets `:feature/enable-flashcards? false`, between Journals and Graph view ([`components/container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L453-L455), [`state.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L615-L619)). The encode garden leaves it on.
- Markup: the `infinity` icon, the label `Flashcards`, the shortcut hint `g f`, and a rounded badge with the due count, hidden when the count is 0 ([`container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L258-L275)). The count is refreshed one second after the item mounts and then every hour ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L809-L821)).
- Measured: link 14px, weight 500, `rgb(17,24,28)`, opacity 0.9, padding `6px 8px`, radius 6px, 213×33. Icon 16px, opacity 0.7, margin-right 8px. Badge `430`: 12px, weight 500, padding `0 12px`, fully rounded, margin-left 4px. At 1280×800 the left sidebar starts closed, so the reader opens it first.
- Click, `g f` or `t c` opens the review modal over all cards; either shortcut closes it again ([`shortcut/config.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/modules/shortcut/config.cljs#L469-L470), [`handler/ui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/ui.cljs#L286-L290), [`handler/events.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/events.cljs#L332-L334)).

### Review modal

- Panel: 836×602 at 1280×800, background `rgb(236,238,240)`, radius 6px, the shared modal shadow, and a Close link (24px icon, `rgb(104,112,118)`, opacity 0.6) at the top right. Escape and Close dismiss it. The card area is 740px wide with a minimum height of 60vh ([`ui.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/ui.css#L131-L138)).
- Header row `.cards-title`: background `rgb(233,236,239)` in the modal and `rgb(248,249,250)` on a page, padding `4px 6px`, radius 4px, 34px tall ([`srs.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.css#L1-L17)). On the left are the 20px `infinity` icon and the deck name (14px, weight 500, `All` or the query text) with a caret. On the right are the counter (14px, opacity 0.6, margin-right 8px) and two 36×24 icon buttons, `Toggle preview mode` (`letter-a`) and `Toggle random mode` (`arrows-shuffle`) ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L622-L695)).
- Deck menu: the caret opens a list with the placeholder `Switch to`. It holds the arguments of every `{{cards}}` macro in the graph and then `All`; choosing one reloads the modal with that query ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L590-L600)). Measured: 29 queries and `All`.
- Card: a breadcrumb of the card's page and parent blocks (16px, margin `8px 0`), then the front block. Inside the reviewer, `#card` and `[[card]]` are hidden with `display: none` ([`components/block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L690-L696)). A `[[Card]]` reference keeps its brackets, so the front reads `Reword the Last Commit [[]]`.
- Phase 1 shows the front and `Show answers` with the key hint `s`. Phase 2 shows the children and replaces that button with `Forgotten` `f`, `Took a while to recall` `t` and `Remembered` `r`. There is no `Hide answers` button while scoring ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L458-L500)). Labels come from [`dicts/en.edn`](https://github.com/logseq/og/blob/6e7afa8eb/src/resources/dicts/en.edn#L483-L503).
- Buttons: 14px, weight 500, padding `4px 12px`, radius 4px, 28px tall, with a small key chip. Measured backgrounds: `Show answers` and `Took a while to recall` `rgba(62,99,221,0.9)`; `Forgotten` `rgba(239,68,68,0.9)`; `Remembered` `rgba(41,151,100,0.9)`; text near white.
- Preview mode shows every card, not only due ones, with `Show answers` / `Hide answers`, `Next` `n` and a faded `Reset`. Measured counter `0/432`.
- After the last card: `Congrats, you've reviewed all the cards for this query, see you next time! 💯`. A graph with no cards shows `Time to create a card!` with a link to the documentation ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L711-L721)).
- Measured first card of `All`: `Lazygit/Keyshort/Commit`. Of `[[yazi]]`: `yazi/Keyshort/Copy GitHub markdown link`.

### What scoring does in the published build

- Scoring computes the next interval with SM-5 and writes the six `card-*` properties back into the block ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L315-L358)). `Forgotten` also schedules the card for tomorrow. When a `{{cards}}` deck is finished, Logseq inserts a `Summary: N items, N review counts [[date]]` block after the macro ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L366-L383)). The garden's `Logseq/Flashcard/Review/Favorite` holds two such blocks from the desktop app.
- Measured: after `Show answers` and `Remembered` on the first card, the counter fell from `430/432` to `429/432`, the sidebar badge to `429`, and the next card appeared. Closing and reopening the modal kept `429`. After a reload the badge and the counter read `430` again, and the same first card returned. `localStorage` gained no flashcard key and no IndexedDB database existed. Scores last for the visit only and nothing reaches the garden.

### `{{cards QUERY}}` on a page

- The argument is a query in Logseq's simple query language, joined with `AND` to "references `card`". A bare page name becomes `[[name]]`, and an empty argument means all cards ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L257-L289)). A page reference matches on the block's path references: the block's own references, those of its parent blocks, and its page ([`deps/db/…/rules.cljc`](https://github.com/logseq/og/blob/6e7afa8eb/deps/db/src/logseq/db/rules.cljc#L140-L143)). This is how `{{cards [[yazi]]}}` finds `#card` blocks under a parent block tagged `[[yazi]]`.
- With matches, the page shows the deck header described above inside `.cards-review` (padding 12px), then the first due card's breadcrumb and front, then a `Review cards` button: 12px, weight 500, `rgba(62,99,221,0.9)`, padding `0 12px`, radius 4px, 90×24 ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L510-L513)). The page does not show the answers. A click anywhere in the card area opens the review modal with this query ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L696-L700)).
- Without matches: a block at opacity 0.6 holding `Cards: [[Zoxide]]` as inline code (12.8px, background `rgb(241,243,245)`, padding `3px 5px`, radius 4px), then `No matched cards` (16px, weight 500, margin `8px 0 0 8px`) ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L722-L725)).
- Measured on the encode garden: 28 of the 29 macros have matches and show a header. `{{cards [[Zoxide]]}}` shows `No matched cards`. The second macro on `Logseq/Flashcard/Review/Favorite` sits under a collapsed block and renders when the reader opens it.
- Other properties on the macro's block are shown as usual. Measured: `favorite-deck: [[yazi]]` appears under the yazi deck.

### A card block on its own page

- The block gets the class `shadow-md`, padding `12px 0`, margin-bottom 8px and radius 4px ([`extensions/srs.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.css#L19-L27), [`components/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2856-L2870)). Measured shadow `0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -2px rgba(0,0,0,0.1)`.
- The `#card` tag and `[[card]]` reference are drawn at opacity 0.3 ([`block.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.css#L690-L692)). Measured on `tmux/Keyshort/Pane/Zoom Pane`, the tag is 12.8px, weight 600, `rgb(52,81,178)`, padding `0 4px`. The size and weight come from the garden's `custom.css` theme (see #106).
- The six scheduling properties are registered as hidden built-in properties ([`srs.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/srs.cljs#L762-L768), [`graph_parser/property.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/property.cljs#L68-L79), [`util/property.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/util/property.cljs#L18-L37)). Measured: the Zoom Pane card shows no property row, though its source carries all six.

### Performance in Logseq

The Flashcards count, the review modal and every `{{cards}}` widget need the whole graph in the browser. The build downloads `static/js/db.gz` (19.8 MB) and about 8 MB of script, then runs a database query per deck. On the desktop run, with no throttling, the Flashcards item appeared 23.0 s after navigation and its count 0.7 s later. [Parity performance](performance.md) records 17.4–23.8 s to the title on desktop and 185–205 s on the phone profile across its fixture pages, with a peak JS heap near 2,000 MB, before any flashcard view can render.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/render.rs`, `site/shell.rs`, `garden_settings.rs` and `references.rs`, checked against the whole-garden build:

- **Sidebar.** The left sidebar holds Journals, All pages and the favorites (`site/shell.rs`). No Flashcards item, no badge, no `g f` or `t c`. `garden_settings.rs` parses `:feature/enable-flashcards?`, but nothing reads it.
- **`{{cards}}`.** `macro_fallback` in `site/render.rs` sends `cards` and `cloze` to the literal fallback. The yazi deck page shows `<code>{{cards [[yazi]]}}</code>`, and the build records an "Unsupported macro (including embeds and queries); preserved as literal text" diagnostic.
- **Card blocks.** `#card` renders as `<span class="tag">#card</span>` with no link, because no `card` page is published. The build reports 564 "Unresolved or excluded tag: card" diagnostics. `[[Card]]` renders as `<span class="page-reference unresolved">`. Card blocks get no class or style of their own. Tags use `lsg`'s own `.tag` style (0.9em, opacity 0.8), not Logseq's 0.3 for `card`.
- **Scheduling properties.** Hidden, but only because the outline renderer skips every `Block::Properties` (`render.rs`, the `match block` arm near line 532). `favorite-deck::` and `deck-exclude::` are hidden too, though Logseq shows them. The page-level list in `site/properties.rs` (`internal`) does not name the `card-*` keys, so a future block-property display would show them.
- **What the SDK already has.** `ReferenceIndex` in `references.rs` gives `linked(page)` (the blocks that reference a page, with namespace ancestors added) and `path_refs(block)` (the block's pages, its ancestors' pages and its page). Together they can find the blocks that reference `card` and filter them by a page reference. The SDK has no evaluator for `and`, `or` and `not`, which the garden's decks use; [Issue #124](https://github.com/codekiln/logseq-gardener/issues/124) covers query evaluation.
- **Size.** The yazi deck page is 26,585 bytes. The source of all card blocks and their children, without `card-*` lines, is 194,868 bytes (66,381 gzip). This is the Markdown size, not the rendered size, and it is the starting point for sizing a deck file.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Card selection.** A card is any block whose references include `card`, in any case and in any form (`#card`, `[[card]]`, `#[[card]]`, `[[Card]]`). The card's children are its back. Check: the SDK returns 432 cards for the encode garden and none from code blocks or inline code.
- 2. **Card block styling.** Card blocks on their pages get Logseq's shadow, `12px 0` padding, 8px bottom margin and 4px radius, and the `card` tag or reference is drawn at opacity 0.3. Check: on `tmux/Keyshort/Pane/Zoom Pane` the card block's computed padding, radius and box-shadow and the tag's opacity match [A card block on its own page](#a-card-block-on-its-own-page).
- 3. **Due cards and counters.** Due follows Logseq's rule, compared with the reader's current time, and due cards are sorted by `card-next-schedule` with unscheduled cards first. Counts computed at build time go stale as days pass. The proposal decides whether the build ships each card's schedule and the browser computes the due count, or the build fixes the date and accepts stale counts. Check: with the browser clock at 2026-10-27, `All` reads `430/432`, `[[VSCode]]` `71/73`, `(and [[VSCode/Keyshort/Git]] )` `9/10`, and the first `All` card is `Lazygit/Keyshort/Commit`.
- 4. **Deck queries.** `{{cards}}` arguments are evaluated as Logseq simple queries over path references, joined with "references `card`". A bare name becomes a page reference, and an empty argument means all cards. The proposal decides whether this waits for, or shares an evaluator with, [Issue #124](https://github.com/codekiln/logseq-gardener/issues/124). Check: the 29 garden decks give the totals measured above, including `[[yazi]]` 18, `(and [[tmux]] (not [[oh-my-tmux]]))` 71, `(not [[Logseq]])` 430 and the 14-page `(and (or …) (not [[oh-my-tmux]]))` 269.
- 5. **Inline `{{cards}}` widget.** With matches: the `.cards-review` header (icon, query text, caret, counter, preview and random buttons) with the measured styles, the first due card's breadcrumb and front with the card tag hidden, and `Review cards`. A click opens the review modal for that deck. Without matches: `Cards: <query>` as code and `No matched cards` at opacity 0.6. Check: `Logseq/Flashcard/Review/Favorite/yazi` shows `[[yazi]]` and `18/18` with `yazi/Keyshort/Copy GitHub markdown link` first. `Logseq/Flashcard/Review/Favorite/Zoxide` shows `No matched cards`.
- 6. **Scoring.** Logseq's published site schedules cards with SM-5 for the visit and forgets the scores on reload. It writes nothing to storage and inserts a summary block after a finished deck. The proposal decides whether `lsg` matches that (memory only), keeps scores per reader in browser storage, or leaves the scoring buttons out. Check: whatever is decided, scoring `Remembered` on the first `All` card moves the counter from `430/432` to `429/432`, and a reload shows the decided state.
- 7. **Review modal.** The Flashcards modal at Logseq's size and colours: the header row, the `Switch to` deck menu (every garden deck query, then `All`), the breadcrumb, `Show answers` `s`, then `Forgotten` `f`, `Took a while to recall` `t`, `Remembered` `r`, the preview mode (`Next` `n`, `Reset`, `Current/Total`), random mode, the finished message, and the Close and Escape exits. Check: on the encode garden the menu lists 30 entries, and the button labels, order and computed colours match [Review modal](#review-modal).
- 8. **Sidebar item and shortcuts.** `Flashcards` with the `infinity` icon, the `g f` hint and the due badge, between Journals and All pages. It is shown unless `:feature/enable-flashcards? false`, and `g f` and `t c` open the modal. Check: on `tmux` the sidebar lists Journals, Flashcards, All pages, the badge reads the due count, and a garden with the setting off shows no item.
- 9. **Scheduling properties stay hidden.** The six `card-*` keys are never shown on a page, in references, in embeds or in the review, including after `lsg` starts showing block properties. Check: no published HTML in the whole-garden build contains `card-last-interval`, `card-repeats`, `card-ease-factor`, `card-next-schedule`, `card-last-reviewed` or `card-last-score` as visible text.
- 10. **Cloze.** `{{cloze answer}}` renders as `[...]` (or `(cue)` after `\\`) and reveals `[answer]` on click or in the third phase. The encode garden has no cloze, so a fixture in `tests/fixtures/parity-garden/` covers it. Check: the fixture card shows `Show clozes` after `Show answers`.
- 11. **Performance.** No page loads card data until a reader opens a review, so the `[desktop]` and `[phone]` first-paint budgets hold on every page, including deck pages. The sidebar item adds only markup within `[shell]`. A deck page carries its header, counts and first card's front as static HTML; the review loads that deck's cards on open. Check: `parity:budgets` shows the yazi deck page within the first-paint budgets and unchanged when 2,000 pages are added; `parity:perf` records opening `All` within the proposal's budget.

## Performance and budgets

Logseq loads the whole database before a reader can see a deck. In `lsg`, the reader's browser should hold only the deck being reviewed.

- **Decks at build time.** The SDK returns, for each `{{cards}}` query and for `All`, the ordered card ids and each card's schedule. The publisher writes one small deck file per query: ids, page, schedule dates, and where to fetch each card's front and back.
- **Card content on demand.** Fronts and backs are rendered HTML fragments, fetched in batches as the review advances rather than all 432 at once. The Markdown of every card is 66 KB gzip, so the rendered `All` deck should be fetched in parts.
- **Static first view.** The inline widget's header, counts as of the build, and first card front are in the page HTML, so the widget shows without script. Script updates the due count from the reader's clock (gap 3).
- **Starting points for the proposal's `[flashcards]` budgets:** review script loaded only when a review opens; deck file gzip size per query; time from clicking `Flashcards` to the first card on desktop and phone, against Logseq's 23 s before the item even appears; build slowdown against `--baseline-lsg`. The proposal sets the values from measurements.

## Where the work belongs

- **SDK (`garden-core`):** card selection (gap 1), the deck query evaluator over path references shared with #124 (gap 4), deck ordering and the due rule as a pure function of schedules and a date (gap 3), the list of deck queries in the garden, and the set of hidden built-in properties including the `card-*` keys (gap 9). The CLI and the language server can then answer "which cards does this deck hold" the same way.
- **Publisher:** the sidebar item and shortcuts, card block styles, the inline widget, the review modal and its script, the deck and card fragment files, cloze rendering, scoring behaviour as the proposal decides, and the budget rows.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `Logseq/Flashcard/Review/Favorite/yazi` | The board's deck; path-reference matching; `favorite-deck::` shown | header `[[yazi]]` `18/18` | `{{cards [[yazi]]}}` as code |
| `Logseq/Flashcard/Review/Favorite/Zoxide` | No matched cards | `Cards: [[Zoxide]]`, `No matched cards` | macro as code |
| `Logseq/Flashcard/Review/Favorite` | `and`, `or`, `not`; a deck under a collapsed block; summary blocks from the desktop app | `269/269`; second deck after expanding | macros as code |
| `Logseq/Flashcard` | Two decks on one page, `[[Logseq]]` and `(not [[Logseq]])` | `2/2`, `428/430` | macros as code |
| `tmux/Keyshort/Pane/Zoom Pane` | The board's card; shadow, faded tag, six hidden `card-*` properties | card styled, no properties | plain block, tag unlinked |
| `tmux/Keyshort/Copy Mode` | `[[Card]]` form; `[[]]` left in the reviewer | reference at opacity 0.3 | unresolved reference |
| `All` (sidebar `Flashcards`) | Global deck, badge, modal, scoring | `430/432`, badge `430` | none |

## Out of scope

- Publishing the reference-only `card` page that `#card` links to, which belongs with [Issue #111](https://github.com/codekiln/logseq-gardener/issues/111) and [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115).
- Showing ordinary block properties such as `favorite-deck::`. That is a gap in the outline, not in flashcards; gap 9 only requires the `card-*` keys to stay hidden when it is fixed.
- `{{query}}` and other query macros, which belong to [Issue #124](https://github.com/codekiln/logseq-gardener/issues/124). This issue shares its evaluator.
- `{{cards}}` in the right sidebar, which belongs with [Issue #119](https://github.com/codekiln/logseq-gardener/issues/119).
- Editing actions that the published site does not offer: `Make a Flashcard`, the `/cards` and `/cloze` commands, and the SM-5 settings `:srs/learning-fraction` and `:srs/initial-interval`, except as gap 6 decides.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
