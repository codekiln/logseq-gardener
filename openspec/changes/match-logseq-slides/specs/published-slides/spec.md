## ADDED Requirements

### Requirement: Open a page as slides

Choosing View as slides in the More menu SHALL add a Slides card for the current page to the top of the right sidebar, expanded, and open the sidebar, as any other card is added; a Slides card for the same page already present SHALL move to the top. The main page, its scroll position and its URL SHALL stay the same. The card's header title SHALL be the page's title as a link to the page, and its body SHALL show the line `Tip: press f to go fullscreen`, with `f` in a `code` element, above a deck 400px tall that fills the card's width. A Slides card SHALL be saved and restored with the other cards.

#### Scenario: Open the AI Engineer Summit deck
- **WHEN** a reader at 1280px on `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025` chooses More, then View as slides
- **THEN** the sidebar opens with a Slides card at the top titled with a link to that page, showing the tip line and a 400px-tall deck, and the URL is unchanged

#### Scenario: Restore after reload
- **WHEN** a reader with a Slides card open reloads the page
- **THEN** the Slides card is restored, and `lsg-slides.js` is requested only after the page's `load` event

### Requirement: Build the deck from the page outline

The deck SHALL have one column per top-level block of the page's outline. A column SHALL hold the block itself without its children as its first slide, then one slide for every descendant at any depth, in outline order. A block whose slide would show nothing once bullets, property rows and children are removed SHALL make no slide, and a column with no slides SHALL be dropped. The deck SHALL be built from the page's published outline, from the live document for the current page and from the published page file otherwise, without fetching any other garden data.

#### Scenario: AI Engineer Summit columns
- **WHEN** the AI Engineer Summit deck opens
- **THEN** it has 8 columns holding 17, 6, 34, 7, 1, 22, 18 and 1 slides, and the fourth slide of the first column is the `flash back` block with its image

#### Scenario: Journal page
- **WHEN** a reader opens slides on `2026-10-23 Fri`
- **THEN** the deck has 2 columns, starting `Filed` and `Updated`

### Requirement: Show each block as a slide

Each slide SHALL show its block's content as the page shows it, without bullet, collapse control, property rows or child blocks: headings at their level, text lines the block holds on its own lines, images, page links, block references, embeds and code blocks. Links inside a slide SHALL behave as links inside any card. The deck SHALL lay out a 960×700 slide scaled to fit the deck and centred, with the structure `div.reveal > div.slides > section`, so rules the garden's `custom.css` writes for `.reveal` apply. In the light theme the deck background SHALL be white, body text the page text colour and headings black and upper case.

#### Scenario: Slide without outline controls
- **WHEN** the AI Engineer Summit deck shows any slide
- **THEN** the slide contains no `.bullet`, no collapse control and no property row

#### Scenario: Heading slide
- **WHEN** the deck shows the `How I found out about the AI Engineer Summit` slide
- **THEN** the slide holds one `h3` in upper case

#### Scenario: Garden code-block rule
- **WHEN** a slide holds a fenced code block and the garden's `custom.css` sets `.reveal pre code { font-size: 0.55em }`
- **THEN** the code block's computed font size is 0.55 of the slide's font size

### Requirement: Apply slide properties

For each block property whose key starts with `background-` or `data-`, or is `transition`, `transition-speed`, `auto-animate` or `visibility`, the published block item SHALL carry the attribute `data-slide-<key>` with the property's value, with a leading `data-` in the key written once. These properties SHALL stay hidden from property rows as they are today. The deck SHALL copy each to the slide's `section` as `data-<key>`; `background-image`, `background-color`, `background-size` and `background-position` SHALL set the slide's background, and `transition` the slide's transition. Pages without such properties SHALL gain no bytes.

#### Scenario: Background image
- **WHEN** a reader opens slides on `Logseq/Doc/slide`
- **THEN** the `Logseq's slide feature is powered by` slide and the `To add a background image` slide each show the image named by their `background-image::` property as their background

#### Scenario: No attributes without slide keys
- **WHEN** the `tmux` page is published
- **THEN** its markup contains no `data-slide-` attribute

### Requirement: Move through the deck

The deck SHALL have four arrow controls with the accessible names `previous slide`, `next slide`, `above slide` and `below slide`, shown faint when they lead nowhere, and a progress bar along its bottom whose width is the current slide's position over the last slide's position. The deck SHALL take keyboard focus when its card opens and after a click inside it, and SHALL receive keys only while focused. While focused: Left or `h` and Right or `l` SHALL move to the top of the previous or next column; Up or `k` and Down or `j` SHALL move within a column; `n` or Space SHALL go to the next slide and `p` or Shift+Space to the previous, through the whole deck; Shift with an arrow SHALL go to the first or last slide in that direction; `b` or `.` SHALL black out the deck until the next key; `g` SHALL open a field that takes a slide number or `column/slide` and goes there on Enter; Escape or `o` SHALL toggle an overview of every slide where a click or Enter chooses one. The shell's and sidebar's shortcuts SHALL be ignored while the deck has focus, and the URL SHALL never change.

#### Scenario: Keyboard path to the flash back slide
- **WHEN** a reader opens the AI Engineer Summit deck and presses Down three times
- **THEN** the `flash back` slide shows, the fourth of 106, and the progress bar is 3/105 of the deck's width

#### Scenario: Right returns to the top
- **WHEN** a reader on the third slide of the first column presses Right
- **THEN** the first slide of the second column shows

#### Scenario: Keys before focus
- **WHEN** focus is on the page outside the deck and a reader presses Right
- **THEN** the deck does not move

### Requirement: Present in fullscreen

While the deck has focus, `f` SHALL put the deck's `div.reveal` in the browser's fullscreen mode and scale the slide to the screen, and `f` in fullscreen SHALL leave it. Escape SHALL leave fullscreen through the browser.

#### Scenario: Enter and leave fullscreen
- **WHEN** a reader focuses the deck and presses `f`, then `f` again
- **THEN** `document.fullscreenElement` is the deck's `div.reveal` after the first press and null after the second

### Requirement: Keep slides within budgets

The deck's behaviour and styles SHALL ship as `lsg-slides.js` at the site root, never requested before a Slides card renders. Only the current slide and its neighbours in each direction SHALL be in the document outside the overview, so images and players on other slides load only when the reader moves near them. The comparison harness SHALL check, on the whole encode garden: every fixture page's first paint unchanged apart from the View as slides item and within the `[desktop]` and `[phone]` budgets; `lsg-slides.js` at most 6 KB gzip; choosing View as slides to the first slide at p95 within 300 ms on the desktop profile; one step to the next slide drawn at p95 within 50 ms; and JS heap growth of at most 10 MB after opening the AI Engineer Summit deck and stepping through every slide. `parity:budgets` SHALL check the size budgets in CI.

#### Scenario: No script before use
- **WHEN** a reader opens the AI Engineer Summit page and does not choose View as slides
- **THEN** `lsg-slides.js` is not requested

#### Scenario: Distant images wait
- **WHEN** a reader opens the AI Engineer Summit deck on its first slide
- **THEN** no image from the third column or later is requested
