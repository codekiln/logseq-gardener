## Context

The [View as slides analysis](../../../docs/parity/slides.md) measured Logseq's `gh-pages` build of the encode garden (`8132c95`) and an `lsg` build from `48fcc54`. Logseq's View as slides adds a right-sidebar card holding a reveal.js 4.6.0 deck built from the in-browser graph. reveal.js (92 KB, 24.7 KB gzip) loads when the first deck mounts; its styles are compiled into the 1.1 MB `style.css` every Logseq page loads.

Since the analysis, `match-logseq-right-sidebar` has been applied and archived: `lsg` now has the right sidebar, with page, block, Block references, Contents and Help cards, saved state in `localStorage`, and a hook in `lsg-shell.js` that loads `lsg-sidebar.js` on first use. `lsg-shell.js` is 6,463 bytes of its 6.5 KB budget. Every published page already holds its full outline as nested `ul.outline > li` items, so a deck needs no data beyond the page file.

## Goals / Non-Goals

**Goals:**

- Readers can present any page as Logseq does: the same menu item, card, slide order, controls, keys and fullscreen.
- No page's first paint changes apart from the menu item; the slide script and styles load on the first Slides card.
- Memory grows with the slides near the reader's position, not with the length of the deck.

**Non-Goals:**

- A phone presentation mode. Logseq's item does nothing on a phone because the sidebar does not open there; a full-window deck for phones is a separate feature.
- The page-title right-click menu, which Logseq fills with the same items as More.

## Decisions

**Where the work lives.**

| Piece | Lives in | Notes |
| --- | --- | --- |
| Each block's properties, with the existing shown or hidden flag | SDK | The Apply step finds why `background-image::` is dropped today and keeps it in the block's property list, hidden from property rows. |
| `data-slide-*` attributes on block items | Publisher (`site/outline.rs`) | Rendering of SDK properties; written only for slide keys (below). |
| Column and slide order | Publisher (`site/slides.js`) | A walk of the page's own outline markup. It moves to the SDK only if the CLI or the language server needs it. |
| Menu item, Slides card, deck, controls, keys, fullscreen, styles | Publisher | Rendering only. |

**The menu item.** `site/shell.rs` writes a `View as slides` button as the first item of `#lsg-more-menu` on page routes only: not on `index.html` (the home route) nor on the All pages route. CSS hides it below 640px, where the right sidebar does not exist. The More menu's existing keyboard behaviour covers it. Choosing it passes `{type: 'slides', route}` for the current page to the existing sidebar loader in `lsg-shell.js`, which adds the card; the handler is at most 0.1 KB, hence the `[shell]` `script_kb` rise from 6.5 to 6.6.

**The Slides card.** `lsg-sidebar.js` gains the card type `slides`. Its header title is the page's title as a link to the page, its icon is Logseq's `presentation` icon, and its menu offers the page-card items, Open as page included. Its body is the tip line (`Tip: press f to go fullscreen`, `f` in a `code` element) and a `div.reveal` 400px tall filling the card's width. On first render the card loads `lsg-slides.js` once and calls `window.logseqGardenerSlides(deck, outline)`, where `outline` is the page's `ul.outline`: taken from the live document when the card's route is the current page, otherwise from the page file fetched through the sidebar's existing route cache. The card is saved and restored like the other cards; a restored Slides card loads `lsg-slides.js` after the page's `load` event, with the rest of the sidebar.

**Slide order.** Each top-level `li` is a column. A column holds the block itself, without its children, then every descendant at any depth as its own slide, in outline order, as Logseq's `slide.cljs` does. A block whose slide would be empty (no content once bullets, property rows and children are removed) makes no slide, and a column with no slides is dropped. On the AI Engineer Summit page this gives 5 columns of 17, 6, 34, 7 and 9 slides, 73 in all, where Logseq shows 7 columns and 75 slides, two of them blank.

**Slide content.** A slide is a clone of the block's content element without the `ul` of children, bullets, collapse controls and property rows. Headings, images, page links, block references, embeds and code blocks keep their page markup and styles. Links inside a slide behave as links in a card do, so Shift-click adds a card. Slides have no height limit, as in Logseq; fullscreen shows tall slides.

**Properties as attributes.** For each block property whose key starts with `background-` or `data-`, or is `transition`, `transition-speed`, `auto-animate` or `visibility`, the publisher writes `data-slide-<key>` on the block's `li` (with a leading `data-` in the key not doubled). These are the slide attributes reveal.js reads. The deck copies each to the slide's `section` as `data-<key>` and applies the ones it implements: `background-image`, `background-color`, `background-size` and `background-position` set the slide's background, `transition` the slide's transition. Pages without such properties gain no bytes.

**Own deck script.** `lsg-slides.js` is the publisher's own script, not reveal.js. It keeps reveal.js's structure and class names (`div.reveal > div.slides > section > section`, `.controls`, `.progress`, `.present`, `.focused`) so the garden's `custom.css` `.reveal` rules apply, and it lays out a 960×700 slide scaled to the deck with `--slide-scale`, centred. Only the current slide and its four neighbours (left, right, up, down) are in the document; moving replaces the slides that fall out of reach. The script implements what Logseq's reveal.js setup uses: the controls, the progress bar, the `slide` transition, the keys below, the overview and fullscreen.

**Keys and focus.** Keys reach the deck while it has focus: the deck has `tabindex="0"`, takes focus when the card opens, and the class `focused` while focused, as a click gives it in Logseq. Left or `h`, Right or `l` move between columns and return to the top of the column; Up or `k`, Down or `j` move within a column; `n` or Space go to the next slide, `p` or Shift+Space the previous; Shift with an arrow goes to the first or last slide in that direction; `b` or `.` pauses with a black screen until any key; `g` opens a field that takes a slide number (counted through the whole deck, as the progress bar counts) or `column/slide` and goes there on Enter; Escape or `o` toggles the overview, a scaled grid of every slide where a click or Enter chooses one. The sidebar's and shell's own shortcuts are ignored while the deck has focus. The URL never changes.

**Fullscreen.** `f` calls `requestFullscreen` on the `div.reveal`; the slide rescales to the screen. Escape leaves fullscreen through the browser. Pressing `f` in fullscreen leaves it, which Logseq's build does not do; the key otherwise does nothing in fullscreen.

**Look.** In the light theme the deck background is white, body text uses the page text colour (`--ls-primary-text-color`, `rgb(104,112,118)` in light), headings are black and upper case at the sizes Logseq's build shows on the 960×700 slide (an `h3` is 50.4px). The deck uses the garden's font stack, not reveal.js's Source Sans Pro files. Dark theme uses the existing theme tokens. The Apply step compares computed styles with the analysis's light values and records any difference.

**Media.** Slides out of reach are not in the document, so their images and players load only when the reader moves near them; images keep `loading="lazy"`. The overview renders every slide with lazy images.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md) on the whole encode garden:

| Measure | Budget |
| --- | --- |
| Every fixture page's first paint, desktop and phone | unchanged except the menu item (under 0.15 KB), within `data_before_title_kb` 100 and `script_before_title_kb` 16 |
| `[shell]` `script_kb` | 6.6 (from 6.5); `frame_kb` stays 6 |
| `lsg-slides.js` with its styles, gzip | 6 KB, never requested before a Slides card renders |
| Choosing View as slides to the first slide, p95, desktop | 300 ms on the AI Engineer Summit page |
| One step (Right or Down) to the next slide drawn, p95, desktop | 50 ms |
| JS heap growth after opening the AI Engineer Summit deck and stepping through all its slides, desktop | 10 MB |

`scripts/parity/budgets.toml` gains a `[slides]` table (`script_gzip_kb`, `open_ms_desktop`, `step_ms_desktop`, `deck_heap_mb`) and `[shell]` `script_kb = 6.6`. `parity:budgets` checks the sizes; `parity:perf` opens the AI Engineer Summit deck, steps through it, and marks the times and heap in `docs/parity/performance.md`; `parity:screenshots` captures the deck on its first slide at 1280×800 and in fullscreen. Phones have no Slides card, so there are no phone rows.

## Risks / Trade-offs

- [An own deck script instead of reveal.js] → Fewer reveal.js features (fragments, speaker notes, plugins, auto-animate effects). Logseq configures none of them, so the reader sees no difference; the script is about a quarter of reveal.js's size and keeps memory bounded by rendering only nearby slides.
- [Dropping blank slides changes Logseq's counts] → The AI Engineer Summit deck has 73 slides where Logseq has 75. The two missing slides are blank: the page's property block and an empty block. Tests count against 73 and record Logseq's 75.
- [The shell script budget rises again] → 0.1 KB for one handler on every page; the alternative, putting the item in `lsg-sidebar.js`, cannot draw the item before the sidebar loads.
- [Slide attributes add markup to some pages] → Only blocks with slide keys gain attributes; on the encode garden that is `Logseq/Doc/slide`.
- [A tall slide is cut off in the 400px card] → Logseq behaves the same way; fullscreen shows the whole slide.

## Resolved Questions

### 1 - Where does the deck open?

Decided by the agent without codekiln: in a right-sidebar card, added to the top of the sidebar like any other card, as Logseq does. The right sidebar now exists (`match-logseq-right-sidebar`), so the card reuses its column, menus and saved state.

### 2 - Does `lsg` use reveal.js or its own deck script?

Decided by the agent without codekiln: its own script, keeping reveal.js's class names. reveal.js is 24.7 KB gzip and renders every slide and its media at once; an own script within 6 KB gzip renders only nearby slides, which keeps phone-class memory bounded and matches every reveal.js behaviour Logseq configures. Keeping the class names lets the garden's `custom.css` `.reveal` rules apply.

### 3 - Does the deck keep Logseq's blank slides?

Decided by the agent without codekiln: no. A block that shows nothing on a slide makes no slide. The blank slides come from the page's property block and from empty blocks; they carry no content and a reader has to step past them.

### 4 - What happens below 640px?

Decided by the agent without codekiln: the menu item is absent. The right sidebar does not exist below 640px, and Logseq's item does nothing there; an item that does nothing is worse than none.

### 5 - Which block properties reach the deck?

Decided by the agent without codekiln: the keys reveal.js reads as slide attributes (`background-*`, `data-*`, `transition`, `transition-speed`, `auto-animate`, `visibility`), written as `data-slide-<key>` on the block's item. Logseq writes every property as an attribute, but only these change a slide; writing every property would add bytes to most pages for nothing.

### 6 - Does the deck take keyboard focus when it opens?

Decided by the agent without codekiln: yes. Logseq needs a click before keys reach the deck; giving the deck focus when it opens lets a keyboard reader present without a mouse, and a click still focuses it.

### 7 - Does the deck load Source Sans Pro?

Decided by the agent without codekiln: no. The deck uses the garden's font stack, which already names Source Sans 3 and Source Sans Pro first; loading reveal.js's 213 KB of font files would change little where those fonts are installed.

### 8 - Does `f` leave fullscreen?

Decided by the agent without codekiln: yes, `f` toggles. In Logseq's build `f` only enters fullscreen and Escape opened the overview while fullscreen stayed on; toggling with the same key is what a reader expects, and Escape still leaves fullscreen through the browser.

### 9 - Is the slide order built in the SDK?

Decided by the agent without codekiln: no, in `lsg-slides.js` from the page's outline markup. The order is a walk of the outline the page already publishes, and only the published site uses it; the SDK supplies the properties the deck needs.

### 10 - Does the shell script budget rise?

Decided by the agent without codekiln: yes, from 6.5 KB to 6.6 KB for the menu item's handler, at most 0.1 KB. `lsg-shell.js` has 37 bytes left, and the item must work before the sidebar script has loaded.
