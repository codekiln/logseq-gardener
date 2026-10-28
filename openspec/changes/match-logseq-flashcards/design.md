## Context

The [flashcards analysis](../../../docs/parity/flashcards.md) measured Logseq's `gh-pages` build of the encode garden (`8132c95`, browser clock 2026-10-27T04:33Z) and an `lsg` build from `48fcc54`. Logseq counts 432 cards, 430 of them due, and renders 29 `{{cards}}` decks on 27 pages. It shows the sidebar count, every deck widget and the review dialog only after loading the whole graph. Scores change the counts for the visit and are gone after a reload.

`lsg` sends `cards` and `cloze` to the literal fallback in `macro_fallback`, renders `#card` as an unlinked `.tag`, and hides `card-*` properties only because it hides every block property. `ReferenceIndex::path_refs` gives the set Logseq's deck queries match on. The `match-logseq-queries` change for [Issue #124](https://github.com/codekiln/logseq-gardener/issues/124), proposed alongside this one, adds the simple query evaluator that the decks need for `and`, `or` and `not`.

[Issue #106](https://github.com/codekiln/logseq-gardener/issues/106) asks for Logseq's look and behaviour without its cost, and for card decks to be SDK operations shared by the CLI, the language server and the publisher.

## Goals / Non-Goals

**Goals:**

- Every deck in the garden shows Logseq's header, counts and first card, and every card block looks like Logseq's.
- A reader can review any deck, or all cards, with Logseq's dialog, keys and scoring.
- No page loads card data or the review script until a reader opens a review.

**Non-Goals:**

- Keeping a reader's scores between visits in browser storage, so the published site works as a personal review tool.
- Publishing the `card` page that `#card` links to.

## Decisions

### The SDK selects and orders cards

A new SDK module, `cards`, provides:

- `cards(index) -> Vec<Card>`: every block whose references include the page `card`, written `#card`, `#[[card]]`, `[[card]]` or `[[Card]]`, outside code. A `Card` holds the block id, its page, its ancestors, and its schedule read from `card-repeats`, `card-next-schedule`, `card-last-interval`, `card-ease-factor`, `card-last-reviewed` and `card-last-score`.
- `deck_queries(index) -> Vec<DeckQuery>`: the argument of every `{{cards}}` macro in the selected documents, in page-name then outline order, with an empty argument meaning `All` and a bare name becoming `[[name]]`, as `srs.cljs` does.
- `deck(query, index, now) -> Deck`: the cards that `query::evaluate` returns for `(and <query> [[card]])`, with `due` and `total`. A card is due when it has no `card-repeats` or fewer than 1, no `card-next-schedule`, or a schedule before `now`. Due cards are sorted by the `card-next-schedule` text, unscheduled first, as Logseq sorts them.
- `HIDDEN_CARD_PROPERTIES`: the six `card-*` keys, joined to the SDK's built-in hidden properties, so no page, reference, embed, search record or review shows them.

The SDK holds no HTML.

### Counts are fixed at build time and corrected when a review opens

`now` at build time is the build time: `SOURCE_DATE_EPOCH` when set, otherwise the system clock. The sidebar badge, each deck header and each widget's first card use it. When a reader opens a review, `flashcards.js` reads that deck's schedules from its deck file, recomputes due cards from the reader's clock, and updates the dialog's counter, the deck header on the page and the sidebar badge. Until then, counts are as of the last build, which the encode garden runs on every push.

### Card blocks on their pages

The outline renderer adds `lsg-card` to a card block's container: `box-shadow` as Tailwind's `shadow-md`, padding `12px 0`, margin-bottom 8px, radius 4px. A `#card` tag and a `[[card]]` or `[[Card]]` reference get `lsg-card-ref`, drawn at opacity 0.3. `card` is treated as a built-in page: the tag stays unlinked, and the build stops reporting it as unresolved.

### The `{{cards}}` widget is static HTML

```html
<div class="cards-review" data-lsg-deck="d7">
  <div class="cards-title">
    <span><i class="ti ti-infinity"></i> [[yazi]] <i class="ti ti-caret-down"></i></span>
    <span><span class="cards-count" title="Overdue/Total">18/18</span>
      <button aria-label="Toggle preview mode">…</button>
      <button aria-label="Toggle random mode">…</button></span>
  </div>
  <div class="cards-first">…breadcrumb and front…</div>
  <button class="cards-review-open">Review cards</button>
</div>
```

The front renders as on its page with the card tag hidden and without its children. A click anywhere in the widget opens the review for that deck; the preview and random buttons open it in that mode. A deck with no cards renders a block at opacity 0.6 with `Cards: <query>` as inline code, then `No matched cards`. Other properties on the macro's block show as usual.

### The review dialog

The dialog is 836px wide at desktop sizes and full width on phones, with Logseq's grey panel, radius, shadow and Close link. The card area is 740px wide with a minimum height of 60vh. The header row repeats the deck header. Its caret opens `Switch to`, which lists every garden deck query, then `All`. Each card shows its breadcrumb and front, with `#card` and `[[card]]` hidden.

The phases are Logseq's: `Show answers` (`s`) reveals the back; `Show clozes` follows when the card has clozes; then `Forgotten` (`f`), `Took a while to recall` (`t`) and `Remembered` (`r`), with Logseq's measured colours and key chips. Preview mode shows every card with `Show answers`/`Hide answers`, `Next` (`n`), a faded `Reset` and the `Current/Total` counter. Random mode shuffles the remaining cards. After the last card the dialog shows Logseq's finished message. Close, Escape, `g f` and `t c` dismiss it.

### Scoring lasts for the visit

Scoring runs Logseq's SM-5 `next-interval` with its default learning fraction and initial interval, in `flashcards.js`, and updates the card's schedule in memory. The counter, the deck header and the badge fall by one; `Forgotten` schedules the card for tomorrow. Closing and reopening the dialog keeps the visit's scores. A reload restores the published counts. The script writes nothing to storage and inserts no summary block.

### Deck files and card batches

The build writes:

- `flashcards/decks/<id>.json` for each deck and for `All`: the card numbers in due order and each card's schedule.
- `flashcards/cards/<n>.html`: batches of 20 rendered cards (breadcrumb, front, back) in `All` order, each card keyed by its number. Decks point into the shared batches, so a card is written once.

Opening a review fetches the deck file and the batch holding its first card, then the next batch before the reader reaches it.

### Code loads only on use

`lsg-shell.js` gains a small loader: a click on the Flashcards item or a `[data-lsg-deck]` widget, or `g f` / `t c`, imports `flashcards.js` and `flashcards.css` once and opens the review. Pages link no flashcard script. Deck pages carry only the widget markup and card styles.

### Cloze

`{{cloze answer}}` renders `span.cloze` showing `[...]`, or `(cue)` for `{{cloze answer \\ cue}}`. On a page a click reveals `[answer]`. In the review the `Show clozes` phase reveals them all.

## Risks / Trade-offs

- Counts on pages go stale between builds. → They are corrected as soon as a review opens, and the garden builds on every push. Live counts on every page would mean shipping schedules to every page.
- Scores vanish on reload. → This matches Logseq's published site; keeping them is listed as a non-goal for a later change.
- Deck selection depends on the queries change. → This change's Apply step waits for that change's Apply step, and the dependency is recorded on both issues.
- `[[Card]]` renders without Logseq's leftover `[[]]` in the review. → Logseq hides the link but leaves its brackets, which reads as a defect.

## Performance budgets

New `[flashcards]` table in `scripts/parity/budgets.toml`:

- `script_kb = 8`: `flashcards.js`, uncompressed, including SM-5.
- `styles_gzip_kb = 1.5`: `flashcards.css`.
- `deck_gzip_kb = 8`: the `All` deck file; every other deck is smaller.
- `card_batch_gzip_kb = 30`: each card batch.
- `deck_page = "Logseq/Flashcard/Review/Favorite/yazi"` and `deck_page_growth_gzip_kb = 4`: the widget and card styles on the yazi deck page.
- `open_ms_desktop = 300` and `open_ms_phone = 1500`: clicking `Flashcards` to the first card of `All`, against Logseq's 23 s before the item appears.
- `review_heap_mb = 20`: phone JS heap after reviewing 50 cards of `All`.
- `build_slowdown_pct = 10`: a whole-garden build against the `--baseline-lsg` binary.

`[shell] script_kb` rises from 6.6 to 6.9 for the loader. Every fixture page stays within the `[desktop]` and `[phone]` first-paint budgets and keeps its size when the fixture garden gains 2,000 pages. Search latency is unaffected.

## Resolved Questions

### 1 - How are due counts kept right as days pass?

Decided by the agent without codekiln: pages carry counts as of the build time, and the review recomputes due cards from the reader's clock when it opens, then corrects the badge and the deck header. Shipping every schedule to every page would put card data on first paint for a count that is correct at each build.

### 2 - What does scoring do?

Decided by the agent without codekiln: what Logseq's published site does. SM-5 schedules the card in memory for the visit, the counts fall, and a reload restores the published state. Nothing is stored, and no summary block is inserted. Keeping scores in browser storage is recorded as a non-goal.

### 3 - Does deck selection wait for, or share, the query evaluator from #124?

Decided by the agent without codekiln: it shares it. Decks are selected by `query::evaluate` with `(and <deck query> [[card]])`, and this change's Apply step starts after the `match-logseq-queries` Apply step merges. One evaluator keeps `{{cards}}` and `{{query}}` answers identical.

### 4 - Where does the Flashcards item sit in the left sidebar?

Decided by the agent without codekiln: between Journals and Graph view, as in Logseq's `container.cljs`. The analysis named All pages as the next item because `lsg` had no Graph view item when it was written.

### 5 - How is card content loaded?

Decided by the agent without codekiln: one deck file per deck with card numbers and schedules, and rendered cards in shared batches of 20, fetched as the review advances. All 432 cards are 66 KB gzip as Markdown, so loading them at once would exceed a phone-friendly first review; shared batches write each card once.

### 6 - When does the flashcard script load?

Decided by the agent without codekiln: on the first click on a deck widget or the Flashcards item, or on `g f` or `t c`, through a loader in `lsg-shell.js`. The widget's header, counts and first card are static HTML, so no page needs the script to look right.

### 7 - What happens to `[[Card]]` in the review?

Decided by the agent without codekiln: the whole reference is hidden, as `#card` and `[[card]]` are. Logseq leaves `[[]]` because its CSS hides only the link inside the brackets.

### 8 - Does `#card` still report an unresolved tag?

Decided by the agent without codekiln: no. `card` is one of Logseq's built-in pages, so the 564 diagnostics are noise. Publishing the `card` page itself is a non-goal here.

### 9 - What lives in the SDK and what in the publisher?

Decided by the agent without codekiln: the SDK selects cards, lists deck queries, selects and orders decks with the shared query evaluator, decides which cards are due at a given time, and holds the hidden `card-*` properties. The publisher renders the sidebar item, card styles, the widget, the dialog, the deck and card files, cloze, and the scoring script. The CLI and language server can then use the same deck operations, as the shared-engine comment on #106 asks.
