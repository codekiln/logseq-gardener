# View as slides

Logseq's More menu offers View as slides on every page. It adds a card to the right sidebar that shows the page as a reveal.js deck: each top-level block is a column the reader moves across, and the block's descendants are stacked below it, one block per slide. The deck has arrow controls and a progress bar, responds to reveal.js's keys once the reader clicks it, and `f` puts it in the browser's fullscreen mode. Block properties become reveal.js attributes, so `background-image::` sets a slide background.

`lsg` has no View as slides. Its More menu holds only Toggle theme and Use system theme, and it has no right sidebar to hold the deck. The [app shell design](../../openspec/changes/archive/2026-10-25-match-logseq-app-shell/design.md) reserves the top of More for this item.

The gap is the menu item, a place to show the deck, the slide structure and the presentation itself. A deck needs no data beyond the page the reader is already on: `lsg` publishes every block of the page in its outline, so a slide script can build the deck from the page's own markup. Logseq instead loads the deck from its in-browser copy of the whole graph, then fetches reveal.js and two fonts. The `lsg` slide script and its styles should load only when a reader opens slides, leaving every page's first paint as it is. The SDK supplies each block's properties; the publisher renders the menu item, the deck and its script.

Measured on 2026-10-26 against the `gh-pages` build of garden commit `083106d` (`gh-pages` `8132c95`, served locally, light theme, 1280×800 and 390×844, Playwright's Chromium) and an `lsg` build of the whole garden from `lsg` `48fcc54` (`mise run parity:build`, run `20261027T002614Z`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). The live build ships reveal.js 4.6.0. Measured live values come first and win where they differ from the checkout or from [lsg/Ref/Logseq Published Site/Clickable Prototype](../pages/lsg___Ref___Logseq%20Published%20Site___Clickable%20Prototype.md). The Prototype board's slide model, card and fullscreen overlay differ from the live build; the differences are listed under [Differences from the Prototype board](#differences-from-the-prototype-board).

### The menu item

- **Where.** `View as slides` is the first item of the header's More menu (`⋯`), and the same list appears when a reader right-clicks the page title ([`page_menu.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page_menu.cljs#L118-L125), [`header.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/header.cljs#L85-L87), [`content.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/content.cljs#L328-L336)). The string is `:page/slide-view` in [`dicts/en.edn`](https://github.com/logseq/og/blob/6e7afa8eb/src/resources/dicts/en.edn#L170-L171).
- **When.** On any page route, including journal pages and pages that are not slide decks. It is absent on the home route and in the native mobile apps. The publishing guard that hides Favorite and Delete does not apply to it, so the published site shows it.
- **Measured menu** on the AI Engineer Summit page: `View as slides`, `Export page`, a divider, `Export graph`, `Toggle theme`. On the home route: `Export graph`, `Toggle theme`. The menu is a `.dropdown-wrapper` 194px wide below `⋯`, background `rgb(241,243,245)`; items are 14px links, 36px tall, padding `8px 16px`, `rgb(104,112,118)`.
- **Action.** The item calls `sidebar-add-block!` with the type `:page-slide-view`, so it behaves like every other right-sidebar card ([Right sidebar](right-sidebar.md#shift-click)): the card goes to the top, the sidebar opens at 40% of the window (512px at 1280px), and the main page stays as it is.
- **Phone width.** At 390×844 the item is still listed, but choosing it does nothing, because `sidebar-add-block!` does nothing below 640px.

### The slides card

[`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L173-L178) renders the card: the header is the page's original name as a link to the page (`a.page-title`), and the body is the deck from [`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L63-L71).

- A line `Tip: press f to go fullscreen` (14px, `f` in a `code` chip).
- `div.reveal`, 400px tall, filling the card's width (440px in a 512px sidebar). reveal.js lays out a 960×700 slide and scales it to fit, `--slide-scale: 0.44` in that card.
- The card has the usual header, `⋯` menu and close button. Its class is `item-type-page-slide-view`.
- The card is saved with the other cards in `localStorage` `ls-right-sidebar-state`, as `["local" <database id> :page-slide-view]`. A reload restores the card and its deck, which loads reveal.js during start-up.

### Loading

- reveal.js is a separate file, `static/js/reveal.js` (92,420 bytes, 24.7 KB gzipped), loaded when the first deck mounts; a spinner shows meanwhile ([`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L74-L87), [`gulpfile.js`](https://github.com/logseq/og/blob/6e7afa8eb/gulpfile.js#L62)).
- reveal.js's styles and its `black` theme are compiled into `style.css`, which every page loads ([`tailwind.all.css`](https://github.com/logseq/og/blob/6e7afa8eb/tailwind.all.css#L13-L14)); the `.reveal` rules are about 59 KB of the 1.1 MB file.
- Opening the AI Engineer Summit deck fetched `reveal.js`, the theme's Source Sans Pro regular and italic fonts (114 KB and 99 KB `woff`), both YouTube players the page embeds, and the page's images. reveal.js renders every slide at once, so media on later slides loads when the deck opens.
- The slides come from the in-browser graph, at most 1,000 blocks of the page ([`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L93-L95)). That graph is the 19.8 MB `db.gz` every Logseq page loads first; Logseq's fixture pages take 17 to 24 s to show their title on the desktop profile ([Parity performance](performance.md)).

### Slide structure

[`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L47-L61) and [`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L99-L107):

- Each top-level block is a horizontal `section`.
- A top-level block with children becomes a vertical stack: the block itself, without its children, is the first slide, and then every descendant at any depth is its own slide, in outline order. Depth is lost: a grandchild is the next slide after its parent, not part of it.
- A top-level block without children is a single slide.
- Text a block holds on its own lines, such as `*` list lines under a heading, stays on that block's slide. This is the structure the garden's `logseq-slides` skill writes: each nested block is one slide with `*` lines as its points.
- The page's property block (`tags:: [[Logseq/Slideshow]]`) is the first top-level block, and because slides hide properties, it is a blank first slide. An empty top-level block is also a blank slide.
- Journal pages drop the first item of the tree ([`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L96-L98)); on `2026-10-23 Fri` both top-level blocks, `Filed` and `Updated`, still appear as columns, so the dropped item is not a content block.
- Measured on `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`: 7 columns with 1, 17, 6, 34, 7, 1 and 9 slides, 75 slides in all. Column 1 is the blank property slide and column 6 a blank block. Column 2 starts `What I learned at the [[AI Engineer Summit 2025 NYC Workshops]]`, then `What is the [[AI Engineer Summit]]?`, then `How I found out about the AI Engineer Summit...` alone on its slide, then the `flash back` block with its image.

### Slide content

Each slide renders its block with the normal block component and `:slide? true` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2842-L2932)):

- No bullet, collapse arrow, drag handle, selection, task checkbox or marker ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2003-L2004), [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2914-L2915)).
- No property rows ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L2362-L2368)).
- Everything else as on the page: headings keep their tag (`h1` to `h4`) and the small level label (`h3`) Logseq shows beside them, page references keep the garden's brackets, images, block references, embeds and YouTube players render.
- Nothing limits a slide's height. On the card, the `flash back` slide's image sits below the 400px deck and is cut off; fullscreen shows it.

### Properties become reveal.js attributes

[`with-properties`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L18-L28) writes each block property as an attribute on the slide's `section`: `key::` becomes `data-key`, and `data-key::` stays `data-key`. reveal.js reads the ones it knows.

- Measured on `Logseq/Doc/slide`: `background-image:: https://images.unsplash.com/…` becomes `data-background-image` and draws the image behind that slide. Headings add `data-heading="1"` or `"2"`, numbered lists `data-logseq.order-list-type="number"`, and the property block `data-tags="Logseq/Slideshow"`; reveal.js ignores those.
- The garden's documentation of this is [`Logseq/Doc/slide`](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Doc___slide.md), copied from Logseq's [`slide`](https://github.com/logseq/docs/blob/master/pages/slide.md) page.

### Navigation and keys

reveal.js is started with `embedded`, `controls`, `center`, `transition: "slide"`, `history: false` and `keyboardCondition: "focused"` ([`slide.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.cljs#L32-L43)).

- **Focus.** Keys reach the deck only after a click inside it, which adds the class `focused`. Before that, arrow keys do nothing to the deck. Logseq's own up and down shortcuts stand aside while the deck is focused ([`editor.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/editor.cljs#L3243-L3251)).
- **Controls.** Four arrow buttons at the bottom right, `aria-label` `previous slide`, `next slide`, `above slide`, `below slide`, faint when disabled. A progress bar runs along the bottom.
- **Keys** (reveal.js 4.6.0 defaults): Left or `h`, Right or `l` move between columns and return to the top of the column; Up or `k`, Down or `j` move within a column; `n` or Space next slide, `p` or Shift+Space previous; Shift with an arrow jumps to the first or last slide; `b` or `.` pauses with a black screen; `g` jumps to a slide by number; Escape or `o` toggles the overview; `f` enters fullscreen.
- **URL.** Moving through slides leaves the URL unchanged.

### Fullscreen

- `f` asks the browser to put the `.reveal` element in fullscreen; the slide scales to the screen (`--slide-scale` 1.097 at 1280×800).
- Pressing `f` again does not leave fullscreen. In the measured Chromium, Escape toggled the overview and fullscreen stayed on; a desktop browser normally leaves fullscreen on Escape at the browser level, which was not measured.

### Look, light theme

- Deck background `rgb(255,255,255)`. Body text `rgb(104,112,118)`, from `--ls-primary-text-color` ([`slide.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/slide.css#L1-L39)); headings `rgb(0,0,0)` through the light-theme override, and upper case from reveal.js's `black` theme. On the 960×700 slide an `h3` is 50.4px.
- The garden's [`logseq/custom.css`](https://github.com/codekiln/logseq-encode-garden/blob/main/logseq/custom.css) adds `.reveal` rules for code blocks: 0.55em text, no height limit, light and dark colours.
- Dark-theme values were not measured.

### Differences from the Prototype board

- **Structure.** The board makes each direct child of a top-level block one slide that includes all of that child's descendants. The live build gives every descendant its own slide.
- **Card and fullscreen.** The board draws a 4:3 deck with its own controls, a `Fullscreen (f)` button and a fixed overlay that Escape or `f` closes, with the label `Slide {h.v} · arrows to move · f or Esc to exit`. The live build uses reveal.js's controls in a 400px-tall card and the browser's fullscreen mode.
- **Keys.** The board's deck takes keys when it has keyboard focus (`tabIndex=0`). The live deck has no `tabindex` and needs a click.
- The board's menu item is followed by a divider; the live build puts `Export page` after it and the divider after that.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/shell.rs`, `outline.rs`, `properties.rs` and the [published app shell spec](../../openspec/specs/published-app-shell/spec.md), checked against the whole-garden build:

- **No menu item.** `#lsg-more-menu` holds `Toggle theme` and `Use system theme`. The spec says the menu contains only those items and that the header holds only controls whose destination exists.
- **No right sidebar.** [Issue #119](https://github.com/codekiln/logseq-gardener/issues/119) adds it; its [analysis](right-sidebar.md) is written, and nothing is built yet.
- **The deck's content is already on the page.** The outline is nested `ul.outline > li` items, one per block, with headings as `h1` to `h4` inside the item, images with `loading="lazy"`, and each block's source position in a `source-block-N` span. A script can read the top-level items and their descendants from this markup without fetching anything.
- **Properties.** Block properties render as visible rows unless `properties.rs` marks the key as internal (`heading`, `background-color`, `id` and others). No property reaches the markup as an attribute a slide could read. On `Logseq/Doc/slide` the `background-image::` value appears nowhere in the `lsg` page.
- **Media.** The AI Engineer Summit page has 23 images and no video player: `{{video}}` belongs to [Issue #122](https://github.com/codekiln/logseq-gardener/issues/122).
- **Sizes.** The AI Engineer Summit page is 70,276 bytes (19.5 KB gzipped), of which 23,623 bytes are the inlined head and shell before `<main>`. `lsg-shell.js` is 5,662 bytes of the 6 KB `[shell]` budget. No page script mentions slides or reveal.js.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Menu item.** `View as slides` as the first item of More on every page route, journal pages included, and absent from the home route and All pages. It needs JavaScript, so it stays hidden like More when scripts are off. Check: on the AI Engineer Summit page More lists `View as slides` first; on `Home` it is absent.
- 2. **Where the deck opens.** Logseq opens a right-sidebar card. `lsg` can do the same once #119 provides cards, or open the deck on its own (a full-window layer over the page) without waiting. The proposal decides; if it uses the sidebar, this issue follows #119. Check: choosing the item shows the deck with the page link as its title and `Tip: press f to go fullscreen`, and the page underneath keeps its scroll position and URL.
- 3. **Slide structure.** Top-level blocks across; under each, the block itself and then every descendant as its own slide, in outline order; `*` lines stay on their block's slide. Logseq's blank slides for the property block and for empty blocks are a quirk; the proposal decides whether to keep them. Check: the AI Engineer Summit deck has 7 columns with 1, 17, 6, 34, 7, 1 and 9 slides (or 5 columns of 17, 6, 34, 7 and 9 without blank slides), and its fourth slide in column 2 is the `flash back` block.
- 4. **Slide content.** Each slide shows its block without bullet, collapse control, property rows or children, with headings at their level and images, references and embeds as on the page. Check: no slide in the AI Engineer Summit deck contains a `.bullet` or a property row; the `How I found out` slide holds one `h3`.
- 5. **Properties as attributes.** Each block property is available to the deck as `data-<key>`, with `data-` not doubled, and `background-image` draws the slide's background. This needs properties that `lsg` drops or hides today. Check: on `Logseq/Doc/slide`, the first slide and the `To add a background image` slide each have the background image from their property.
- 6. **Navigation.** Next, previous, up and down controls with accessible names, a progress bar, and the keys listed under [Navigation and keys](#navigation-and-keys); Left and Right return to the top of a column; the URL does not change. Logseq needs a click before keys work; `lsg` should also give the deck keyboard focus when it opens, so a keyboard reader can start without a mouse. Check: open the AI Engineer Summit deck, press Right, then Down three times, and the `flash back` slide shows, the fifth of 75, with the progress bar at 4/74 of its width.
- 7. **Fullscreen.** `f` puts the deck in the browser's fullscreen mode and scales the slide to the screen; Escape leaves it. Check: `f` sets `document.fullscreenElement` to the deck; Escape clears it.
- 8. **Look.** In the card or layer, a 960×700 slide scaled to fit, white background, body text in the page text colour, black upper-case headings at reveal.js's sizes, centred content, and the garden's `custom.css` `.reveal` rules applying to code blocks. The proposal decides whether `lsg` uses reveal.js itself (MIT, 24.7 KB gzipped) or a smaller script of its own that keeps the class names `custom.css` targets. Check: computed styles on the AI Engineer Summit deck match the values under [Look, light theme](#look-light-theme), and a code block in a slide gets the `custom.css` font size.
- 9. **Phone width.** Logseq lists the item below 640px and does nothing when it is chosen. `lsg` should either hide the item or open the deck at full width; the proposal decides. Check: at 390×844 the item is absent or opens a deck with no sideways scroll.
- 10. **Saved state.** Logseq restores an open slides card after a reload. If the deck is a sidebar card, it follows #119's saved state; restoring must not load the slide script before the page title is visible. Check: with a deck open, reload, and the page's first-paint values stay within budget.
- 11. **Performance.** Opening slides adds nothing to a page's first paint and fetches nothing from the garden: the deck is built from the page's outline, and its script and styles load on first use. Media on slides far from the current one waits until the reader gets near it, unlike Logseq, which starts every image and video at once. Check: `parity:budgets` passes with every fixture page's `[desktop]` and `[phone]` values unchanged; opening the AI Engineer Summit deck stays within the new `[slides]` budgets below.

## Performance

Logseq can build any page's deck at once because the whole graph is already in the browser, and it adds reveal.js and two fonts on top. `lsg` must not load the graph ([#106](https://github.com/codekiln/logseq-gardener/issues/106)), and a deck does not need it:

- **No extra data.** The page's own outline holds every block a deck shows. Slide properties need to be in the page's markup (for example as attributes on the block's item), which adds a few bytes per property on the pages that have them.
- **Script on first use.** `lsg-shell.js` has 338 bytes of headroom under `[shell]` `script_kb = 6`. The menu item and a loader for `lsg-slides.js` must fit, or the proposal raises the budget with measurements. Deck styles load with the script, not in every page's inlined head.
- **Media on demand.** Images and players on slides far from the current slide load only when the reader moves near them. `lsg` already marks images `loading="lazy"`.
- **Fonts.** Logseq loads 213 KB of Source Sans Pro for the theme. The proposal decides whether to load those fonts or use the garden's existing type.

Budget rows that apply:

- `[desktop]` and `[phone]`: `data_before_title_kb`, `script_before_title_kb`, `title_ms` and `peak_heap_mb` must not change for any fixture page.
- `[shell]`: `script_kb` for the hook in `lsg-shell.js`, and `frame_kb` for the menu item.
- A new `[slides]` table in the proposal, for example `script_kb` for `lsg-slides.js` and its styles, `open_ms_desktop` and `open_ms_phone` from choosing the item to the first slide, `step_ms` for moving one slide, and `open_heap_mb` on the phone profile after opening the AI Engineer Summit deck. `[references]` (`script_kb = 6`, `open_ms_desktop = 300`, `open_ms_phone = 1500`) is a reasonable starting point; reveal.js alone would need about 25 KB gzipped.

## Where the work belongs

- **SDK:** each block's properties, including the ones `lsg` hides or drops today (`background-image`, `data-*` keys), with a flag for which ones a page shows. Building the column and slide sequence from the outline is simple enough for the publisher; it moves to the SDK only if the CLI or the language server needs it.
- **Publisher:** the `View as slides` menu item, attributes for slide properties on each block's markup, `lsg-slides.js` with its styles (deck, controls, keys, fullscreen, progress, media on demand), the card or layer that holds the deck, and the `[slides]` budget rows.

## Fixture pages

| Page | Why | Logseq | lsg today |
| --- | --- | --- | --- |
| `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025` | The garden's slide deck, tagged `Logseq/Slideshow`, the Prototype board's example; headings, images, embeds, videos, `*` lines | 7 columns, 75 slides | Page only; 70,276 bytes |
| `Logseq/Doc/slide` | `background-image::` properties and numbered lists | 2 slides with background images | Properties absent |
| `2026-10-23 Fri` | A journal page; already a fixture | 2 columns | Page only |
| `Home` | No menu item on the home route; already a fixture | Item absent | — |

Both new pages are added to [`scripts/parity/fixtures.txt`](../../scripts/parity/fixtures.txt). `parity:screenshots` captures pages without the deck open; the Apply step needs a capture of the AI Engineer Summit deck on its first content slide and in fullscreen.

## Out of scope

- The right sidebar and its cards, menus and saved state, which belong with [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119). This issue uses its card if the proposal chooses the sidebar.
- The video and YouTube players inside slides, which belong with [Issue #122](https://github.com/codekiln/logseq-gardener/issues/122); slides show whatever the page renders.
- Block embeds inside slides, which follow [Issue #114](https://github.com/codekiln/logseq-gardener/issues/114).
- The page-title right-click menu, which Logseq fills with the same items as More; `lsg` has no page-title menu yet.
- reveal.js features Logseq does not set up: speaker notes, plugins, PDF export, fragments and the URL hash.
- Dark-theme values, which were not measured; the styles use the existing theme tokens.
