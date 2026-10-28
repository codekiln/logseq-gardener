# Verification

Every deck count matches Logseq's published build of the encode garden, and the review dialog opens, scores and resets as Logseq's does. The only difference is the order of cards that share a schedule (decision 10 in the design).

Measured on 2026-10-28 with an `lsg` build of the encode garden at commit `083106d` (the analysis's commit), built with `SOURCE_DATE_EPOCH=1793075580` (2026-10-27T04:33Z, the analysis's browser clock) by `scripts/parity/build.py`, served locally and opened in the built-in Chromium browser at 1280×800 and 375×812. Logseq's values are the ones the [flashcards analysis](../../../docs/parity/flashcards.md) measured on its `gh-pages` build.

## Decks

| Deck | Logseq | lsg |
| --- | --- | --- |
| `All` | 430/432 | 430/432 |
| `[[VSCode]]` | 71/73 | 71/73 |
| `(and [[VSCode/Keyshort/Git]] )` | 9/10 | 9/10 |
| `[[yazi]]` | 18/18 | 18/18 |
| `(and [[tmux]] (not [[oh-my-tmux]]))` | 71 cards | 71/71 |
| `(not [[Logseq]])` | 430 cards | 428/430 |
| `(and (or …14 pages…) (not [[oh-my-tmux]]))` | 269/269 | 269/269 |
| `[[Zoxide]]` | No matched cards | No matched cards |
| `Switch to` entries | 29 queries and `All` | 30 (29 and `All`) |

First due cards: Logseq shows `Lazygit/Keyshort/Commit` for `All` and `yazi/Keyshort/Copy GitHub markdown link` for `[[yazi]]`. `lsg` shows `1Password/Quick Access` and the card on `git/Alias/md-url`, the first unscheduled cards in page-title order. 168 cards have no schedule and tie; Logseq breaks the tie by database order, which the files do not record.

## Pages and dialog

- `Logseq/Flashcard/Review/Favorite/yazi`: the widget shows the infinity icon, `[[yazi]]`, `18/18` titled `Overdue/Total`, the preview and random buttons, the first card's breadcrumb and front without its children, and `Review cards`. No literal `{{cards`. Loading the page requests no file under `flashcards/` and no `lsg-flashcards.js`.
- `tmux/Keyshort/Pane/Zoom Pane`: the card block computes `box-shadow: rgba(0,0,0,.1) 0 4px 6px -1px, rgba(0,0,0,.1) 0 2px 4px -2px`, padding-top and padding-bottom 12px, margin-bottom 8px, radius 4px, and the `#card` tag opacity 0.3, as Logseq measured. No `card-` property is visible.
- `Review cards` opens an 836px dialog with the deck header. `s` shows the answers; `r` scores `Remembered`: the dialog counter and the page widget fall from `18/18` to `17/18` and the sidebar badge from `430` to `429`. Escape closes the dialog. After a reload the widget reads `18/18` and the badge `430`. No flashcard key was written to `localStorage`.
- `g` then `f` opens `All` at `430/432` with 30 `Switch to` entries; `g` then `f` again closes it. No console errors.
- At 375×812 the dialog is 375px wide and the page does not scroll sideways.

## Hidden properties

No published page shows any of the six `card-*` keys as text; the only page containing `card-next-schedule` quotes the key in prose on `p-af8832a590df5650.html`. The graph view's neighbourhood files no longer list the keys as property pages (decision 13).

## Sizes

| File | Budget | Measured |
| --- | --- | --- |
| `lsg-shell.js` | 6.9 KB | 6,432 bytes |
| `lsg-flashcards.js` | 8 KB | 7,586 bytes |
| `lsg-flashcards.css` | 1.5 KB gzip | 1,054 bytes gzip |
| `flashcards/decks/all.json` | 8 KB gzip | 3,180 bytes gzip |
| largest of 22 card batches | 30 KB gzip | 12,542 bytes gzip |

## Performance

`parity:perf --flashcards` on a fixture run (60 cloze cards and a deck): opening the review from the deck widget showed the first card in 29 ms on desktop and 542 ms on the phone profile (budgets 300 ms and 1,500 ms), and the JS heap after scoring 50 cards was 2.1 MB on desktop and 2.2 MB on the phone (budget 20 MB). `parity:budgets` passes with the `[flashcards]` table: script 7,586 bytes, styles 1,044 gzip, `All` deck 61 gzip and largest batch 404 gzip on the fixture garden, and the `Tools/Flashcards` deck page grows by 685 bytes gzip.

`[inline] largest_page_growth_kb` rises from 2 to 2.1: the Flashcards item adds about 80 bytes gzip to every page, and `Tools/Queries` exceeded the old limit by 17 bytes.

## Tests

- `crates/logseq-gardener-sdk/src/cards.rs`: the due rule, schedule parsing, UTC times, deck query forms and due ordering.
- `crates/logseq-gardener-sdk/tests/site_flashcards.rs`: deck widgets with counts and first cards, the empty deck, deck files and card batches, the sidebar item and badge, hidden properties, and `:feature/enable-flashcards? false`.
- `site::render` tests: card block and card reference classes without diagnostics, and cloze.
