## Verification

`lsg`'s View as slides matches Logseq's on the menu item, the card, the slide order, the slide look in both themes, the controls and the keys. The AI Engineer Summit deck has more slides than Logseq's, because Logseq's deck stops partway through the page; every column both decks show has the same slides.

Measured on 2026-10-28 with the whole encode garden (`mise run parity:build`, runs `20261028T003322Z` to `20261028T004849Z`, 8,697 pages) against Logseq's `gh-pages` build `b491e00`, both served locally and read in Chromium at 1280×800.

### AI Engineer Summit deck

| Check | Logseq (`gh-pages`) | `lsg` |
| --- | --- | --- |
| More menu on the page | View as slides first | View as slides, Toggle theme |
| Card | title links to the page, `Tip: press f to go fullscreen`, deck 400px tall | the same |
| Columns and slides | 7 columns: 1, 17, 6, 34, 7, 1, 9 (75) | 8 columns: 17, 6, 34, 7, 1, 22, 18, 1 (106) |
| Down three times from the first slide | `flash back` slide with its image | the same; progress 3/105 |
| `How I found out` slide | one `h3`, 50.4px, upper case | the same |
| Controls' accessible names | previous slide, next slide, above slide, below slide | the same |
| Deck background and text, light | `rgb(255,255,255)`, `rgb(104,112,118)` (analysis) | `rgb(255,255,255)`, `rgb(104,112,118)`; headings `rgb(0,0,0)` |
| Deck background, text and headings, dark | `rgb(21,23,24)`, `rgb(155,161,166)`, `rgb(255,255,255)` | the same through the theme tokens |
| Slides in the document | every slide | the current slide and its neighbours (2 to 5) |

Logseq's extra column is the blank slide for the page's `tags::` block. Its deck then ends inside the workshop 3 block at `Feedback Loops from Online Evaluators`, the block before that section's second embed of the same block, so it leaves out 13 more workshop 3 slides, the workshop 4 block (18 slides) and the closing image. `lsg` builds the deck from the whole published outline. The design and the `published-slides` scenarios now give `lsg`'s counts.

### Other fixture pages

- `Logseq/Doc/slide`: the item reaches `data-slide-background-image` on both blocks that set `background-image::`, including the one whose property line has a three-space indent; the deck draws `https://www.ai.engineer/…` behind `Logseq's slide feature is powered by…` and the Unsplash image behind `To add a background image…`, and no background on the other slides.
- `2026-10-23 Fri`: 2 columns, `Filed` (6 slides) and `Updated` (17 slides).
- Home (the journals feed) and All pages: More lists Toggle theme and Use system theme, without View as slides.
- `tmux` at 390×844: More lists Toggle theme and Use system theme, with focus on Toggle theme.
- A Slides card restored after a reload requests `lsg-slides.js` after the page's `load` event (56 ms against `load` at 48 ms on the AI Engineer Summit page) and does not take focus.

### Fullscreen

`mise run parity:screenshots --slides` captured `target/parity/20261028T004849Z/screenshots/slides-desktop-lsg.png` (the card on its first slide) and `slides-fullscreen-lsg.png`: `f` made the deck's `div.reveal` the fullscreen element and scaled the slide to 1280×800. `tests/slides.test.cjs` checks that a second `f` leaves fullscreen. The Claude browser pane refuses fullscreen requests, so fullscreen was checked only in Playwright's Chromium.

### Budgets

`mise run parity:budgets` passed: `lsg-shell.js` is 6,594 bytes of 6,600, `lsg-sidebar.js` 6,184 bytes gzip of the raised 6,500, `lsg-slides.js` with its styles 4,505 bytes gzip of 6,000, and no fixture page requests it. `mise run parity:perf --slides --record` wrote the View as slides rows in [Parity performance](../../../docs/parity/performance.md): opening the deck took 83 ms at the 95th percentile (budget 300), a step 14 ms (budget 50), and the heap grew 1.5 MB after stepping through all 106 slides (budget 10).
