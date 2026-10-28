## Why

Logseq's More menu offers View as slides on every page: it opens a right-sidebar card that presents the page as a deck, with top-level blocks across and their descendants down, arrow controls, a progress bar and `f` for fullscreen. `lsg` has no View as slides ([View as slides analysis](../../../docs/parity/slides.md), [Issue #121 — Add View as slides](https://github.com/codekiln/logseq-gardener/issues/121)).

Logseq builds the deck from the whole graph it loads into the browser, then fetches reveal.js (24.7 KB gzip) and 213 KB of fonts, and renders every slide and its media at once. `lsg` can build the deck from the page's own published outline, with a small script that loads only when a reader opens slides and keeps only the slides near the current one in the document.

## What Changes

- `View as slides` becomes the first item of the More menu on every page route, journal pages included, at 640px and wider. It is absent from the home page and All pages, and below 640px.
- Choosing it adds a Slides card to the top of the right sidebar, as Logseq does. The card's title links to the page; its body shows `Tip: press f to go fullscreen` and a 400px-tall deck.
- The deck has one column per top-level block. A column's first slide is the block without its children, followed by one slide per descendant in outline order. Blocks that show nothing on a slide (the page's property block, an empty block) make no slide.
- Each slide shows its block as the page does, without bullet, collapse control, property rows or children. A 960×700 slide scales to fit, with Logseq's light-theme colours, upper-case headings and centred content.
- Block properties that reveal.js reads as slide attributes, such as `background-image::` and `data-*` keys, reach the page's markup as attributes on the block's item, and the deck applies them; `background-image` draws the slide's background.
- Four arrow controls with accessible names, a progress bar, and reveal.js's keys as Logseq configures them: arrows and `h j k l`, `n`, `p`, Space, Shift with an arrow, `b` or `.` to pause, `g` to jump, Escape or `o` for the overview, `f` for the browser's fullscreen. The deck takes keyboard focus when it opens and after a click.
- The Slides card is saved with the other sidebar cards and restored after the page's `load` event.
- `lsg-slides.js` with its styles loads on the first Slides card. It keeps the reveal.js class names the garden's `custom.css` targets (`.reveal`, `.slides`) so the garden's code-block rules apply.
- A new `[slides]` budget table caps the script, the time to the first slide, one step, and the phone heap after opening the largest fixture deck. The `[shell]` script budget rises from 6.5 KB to 6.6 KB for the menu item's handler, and the `[sidebar]` script budget from 6 KB to 6.5 KB gzip for the Slides card type.

## Capabilities

### New Capabilities

- `published-slides`: the View as slides item, the deck's structure, content, property attributes, navigation, fullscreen, look, loading and budgets.

### Modified Capabilities

- `published-app-shell`: the More menu gains View as slides on page routes; the shell script budget becomes 6.6 KB.
- `published-right-sidebar`: a Slides card type joins page, block, Block references, Contents and Help cards, with its own saved state; `lsg-sidebar.js` may be 6.5 KB gzip.

## Impact

- SDK: each block's properties, including ones the page hides or drops today, with the existing flag for which ones a page shows. The publisher reads them; no new graph data.
- Publisher: the menu item in `site/shell.rs` and its handler in `site/shell.js`; `data-slide-*` attributes on block items in `site/outline.rs`; the `slides` card type in `site/sidebar.js`; new `site/slides.js` and `site/slides.css`.
- Parity harness: `scripts/parity/budgets.toml` gains a `[slides]` table and a raised `[shell]` `script_kb`; `parity:budgets` checks the sizes; `parity:perf` opens the AI Engineer Summit deck and steps through it; `parity:screenshots` captures the deck on its first slide and in fullscreen. The fixture pages are already in `scripts/parity/fixtures.txt`.
- Gitpa receives the item and the card through the shared shell and sidebar.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
