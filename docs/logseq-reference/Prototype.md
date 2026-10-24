# Clickable prototype

This board is a working React-style prototype of the published Logseq garden (screens, routes, search, sidebars, flashcards, queries, slides) driven by 24 embedded page sources; its script is the most complete statement of behaviour in the reference.

How to read this file:

- Everything below comes from `html/Prototype.dc.html` (markup, CSS and the `<script type="text/x-dc">` block). Nothing here comes from the live site unless the board says so.
- Where the prototype differs from the other boards or from Logseq, the difference is listed under "Known gaps" and "Differences from other boards".
- The page sources are reproduced exactly in the appendix at the end.
- Canvas note on the board (from `canvas.json`): "Clickable prototype: press Play". Usage tips: Search (header), type `tmux`, arrows, Enter, Shift+Enter opens in the right sidebar, `/` shows filters, Cmd+Enter sends the search to the sidebar. Open the "AI Engineer Summit" page from search, then More, View as slides, then arrow keys and `f`. The Omer Hamerman YouTube page has `{{youtube-timestamp}}` links that seek the video placeholder. `AI/Batch Inference` shows Mermaid as a fenced code block ("Logseq does not draw Mermaid without a plugin"). More, Toggle theme gives dark mode. Shift-click any link to open it in the right sidebar.

## 1 · Screens and routes

Route state is `route = {type, ...}`. There are no URLs in the prototype; the type names are the routes.

| Route type | Screen | Reached by |
| --- | --- | --- |
| `journals` (initial) | Journals feed: four journal pages stacked, each as an excerpt without references sections | Left sidebar "Journals"; Home button; search commands "Go to journals" / "Go to home" |
| `page` (`name` = lower-cased resolved name) | One page with title, properties, outline, then "Pages tagged with", Linked References, Hierarchy, Unlinked References | Any page link; favorites; search results; recents |
| `block` (`id`) | Zoomed block: breadcrumb and the block's outline, no title and no references | Click a bullet; click a block ref; click a breadcrumb crumb; search block result |
| `all` | All pages table | Left sidebar "All pages"; command "Go to all pages" |
| `graph` | Graph view | Header "Graph" button; command "Go to graph view" |
| `wb` | Whiteboards placeholder | Left sidebar "Whiteboards" |

Shared behaviour:

- Opening a page resolves aliases first (`alias::` values map to the owning page). Names compare lower-cased.
- Opening a page adds it to the recents list (most recent first, max 6, de-duplicated), closes search and the More menu, sets Linked References open and Unlinked References closed.
- Any route change scrolls the main column to the top.
- Shift-click on any page link, tag, block ref or bullet opens it in the right sidebar instead (see section 2).
- A page name with no source (for example a link target not in the 24 sources) still opens: title is the link text, outline is a single empty block, linked references are computed.
- The `contents` page source is a single empty block (`-`) and is removed from All pages.

### Layout

- App: full viewport height, column: header, then `shell` (flex row: left nav, main, right aside). Each of nav, main, aside scrolls independently.
- At width 760px or less the shell becomes a column and the sidebars lose their fixed width (`@media (max-width: 760px)`).
- Header: 48px, padding `0 12px`, background `var(--hd)`, border-bottom `1px solid var(--bd)`.
- Left sidebar: 246px wide, background `--bg2`, border-right, padding `12px 8px`, gap 2px. Open by default.
- Main: `padding: 28px 28px 80px`, content max-width 810px, centred.
- Right sidebar: width 40%, min-width 320px, background `--bg2`, border-left, padding `8px 10px 40px`, gap 10px. Closed by default.
- Fonts: 'Source Sans 3' (body), 'JetBrains Mono', 'Fira Code' (code, falling back to Monaco, Menlo, monospace).
- Page title: `h1`, 36px (journal pages 30px), weight 300, line-height 1.3, colour `--title`, margin-bottom 16px.

## 2 · Interactive behaviour

### Header

Left group: "Toggle left sidebar" (title "Toggle left sidebar (t l)") and "Search" (title "Search (⌘K)"). Right group:

- "Home" icon button (title "Home (g h)"): shown only when the route is not `journals`; goes to Journals.
- "Graph" text button (14px, weight 500, muted): goes to the graph route.
- "More" icon button: toggles the More menu.
- "Toggle right sidebar" (title "Toggle right sidebar (t r)").

The `(t l)`, `(g h)`, `(t r)` and `(⌘K)` parts are title text only. See "Known gaps" for which keys are actually bound.

### Left sidebar

Buttons (15px, min-height 34px, padding `6px 10px`, radius 6px, hover and active background `--bg4`):

1. "Journals": goes to Journals; active on the journals route.
2. "Whiteboards": goes to the `wb` route; active on that route.
3. "Flashcards" with a "430" badge (12px, weight 600, `--bg4` background, radius 9px, padding `1px 7px`): opens the flashcards modal.
4. "All pages": goes to All pages; active on the all route.

Then "FAVORITES" (12px, weight 600, letter-spacing 0.06em, muted; margin `18px 10px 6px`) and the favorites list (14px; 6px round dot; text truncated with ellipsis). A favorite is highlighted when the current page route name equals it (case-insensitive). The list, in order (19 entries):

1. fd
2. gh-dash
3. GitHub/CLI/Extension/gh-dash
4. tuicr
5. tmux/Q/How can I put a thick border around the focused pane?
6. LazyVim
7. Logseq/Flashcard/Review/Favorite
8. Obsidian
9. Codex
10. chezmoi
11. DevContainer
12. mise
13. rg
14. vim
15. nvim
16. Zoxide
17. yazi
18. tmux
19. Lazygit

Below the favorites: a dashed separator (`margin: 22px 10px 6px; padding-top: 12px; border-top: 1px dashed`) and a label "EXAMPLE PAGES · NOT LOGSEQ UI" (11px, weight 600, letter-spacing 0.06em, muted). It lists five pages, shown as their last two path segments in muted 14px text:

| Shown as | Page name |
| --- | --- |
| Fixture/Headings | Prototype/Fixture/Headings |
| Fixture/Queries | Prototype/Fixture/Queries |
| loop/label | Rust/loop/label |
| Concept/AWS vs OpenAI vs LangChain | AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain |
| Favorite/yazi | Logseq/Flashcard/Review/Favorite/yazi |

Labels are computed as `name.split('/').slice(-2).join('/')`. Only the two `Prototype/Fixture/*` pages are invented for the prototype (their text says "This page is a prototype fixture, not a page in the garden"). The HTML does not say whether the other three are verbatim garden pages.

### More menu

Fixed at `top: 46px; right: 52px`, width 240px, background `--card`, border `1px solid --bd`, radius 8px, shadow `0 12px 32px rgba(0,0,0,0.18)`, padding 4px, `z-index: 41`; a transparent full-screen layer behind it closes it on click. Items (`.mi`: 14px, min-height 38px, padding `8px 12px`, radius 6px, icon 16px):

1. "View as slides": only on a page route. Closes the menu and adds a slides item for the current page to the right sidebar (opens the sidebar). Followed by a divider (`border-top: 1px solid --bd; margin: 4px 0`). It is offered on any page, not only pages tagged `Logseq/Slideshow`.
2. "Export graph": closes the menu and opens the Export dialog.
3. "Toggle theme": toggles dark mode and closes the menu.

### Export dialog

Overlay (`position: fixed`, `--ov` background, `z-index: 60`, click outside closes). Dialog `role="dialog"` aria-label "Export", width 440px, background `--card`, radius 10px, padding `20px 24px`. Title `h2` "Export" (24px, weight 500, margin-bottom 12px). Options (link colour, `.mi` buttons), in order:

1. Export as EDN
2. Export as JSON
3. Export as standard Markdown (no block properties)
4. Export as OPML
5. Export as Roam JSON

Clicking an option closes the dialog and shows a toast "`<option text>`: the browser downloads the file". Nothing is downloaded.

### Theme toggle and tokens

Theme toggles by adding class `dark` to the root `.lsq`. Toggle sources: More, Toggle theme; search command "Toggle between dark/light theme". Both token sets:

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| `--bg` | `#ffffff` | `#1a1d1e` | page background |
| `--bg2` | `#f8f9fa` | `#151718` | sidebars, embeds, search panel |
| `--bg3` | `#f1f3f5` | `#202425` | hover, table stripe, placeholders |
| `--bg4` | `#e9ecef` | `#2b2f31` | active nav, badges, closed bullet, kbd |
| `--bd` | `#e9ecef` | `#2b2f31` | borders, guidelines |
| `--tx` | `#11181c` | `#ecedee` | body text |
| `--tx2` | `#3a3f44` | `#c1c8cd` | secondary text |
| `--mu` | `#687076` | `#9ba1a6` | muted text, icons |
| `--ln` | `#106ba3` | `#a3cef1` | links |
| `--brk` | `#6b7280` | `#787f85` | `[[ ]]` brackets |
| `--dot` | `#9ba1a6` | `#697177` | bullet dot, graph node |
| `--ac` | `#3e63dd` | `#849dff` | accent: tags, slide progress, graph centre |
| `--title` | `#000000` | `#ffffff` | page title |
| `--icbg` | `#e9ecef` | `#2b2f31` | inline code background |
| `--ictx` | `#3a3f44` | `#c1c8cd` | inline code text |
| `--warn` | `#b54708` | `#f1a10d` | invalid ref, query error |
| `--prop` | `#f8f9fa` | `#202425` | properties box |
| `--ov` | `rgba(17,24,28,.45)` | `rgba(0,0,0,.6)` | modal overlay |
| `--card` | `#ffffff` | `#202425` | cards, menus, dialogs |
| `--hd` | `rgba(255,255,255,.9)` | `rgba(26,29,30,.92)` | header |
| `--sel` | `#e6edfe` | `#1f2d5c` | defined, not used by any rule |
| `--hr` | `#dee2e6` | `#313538` | heading rule |
| `--codebg` | `#f8f9fa` | `#151718` | code block |
| `--ck` | `#bd4b00` | `#ff8b3e` | code keyword |
| `--cs` | `#297c3b` | `#71d083` | code string |
| `--cc` | `#7e868c` | `#787f85` | code comment |
| `--cn` | `#cd1d8d` | `#ff8dcc` | code number, builtin |
| `--cd` | `#0078a1` | `#75c7f0` | code definition name, python attribute, bash option |
| `--co` | `#793aaf` | `#d19dff` | code operator, rust macro |
| `--cv2` | `#946800` | `#f0c000` | defined, not used by any rule |

Fixed colours outside the token sets: Forgotten `#ce2c31`, Took a while to recall `#3e63dd`, Remembered `#18794e`, filter include `#18794e`, filter exclude `#ce2c31`, highlight (`.mk`) background `#fef3ac` text `#262626`, video placeholder `#0f0f0f`.

Other common CSS:

- Links: `var(--ln)`, no underline, underline on hover. External links (`a.ext`): `border-bottom: 1px solid currentColor`, no underline on hover.
- Tags (`.tag`): `font-size: .9em; opacity: .7; padding: 0 2px; border-radius: 4px`, colour `--ac`.
- Inline code (`.ic`): Fira Code, `.9em`, background `--icbg`, colour `--ictx`, radius 3px, padding `1px 4px`.
- Highlight (`.mk`): padding `2px 4px`, radius 3px.
- Block ref (`a.bref`): `border-bottom: .5px solid currentColor; padding: 2px 0; cursor: alias`, no underline on hover.
- Warning (`.warn`): colour `--warn`.
- Timestamp link (`a.yts`): inline-flex, gap 3px, `font-size: .9em`, with a 13px clock icon in the outline view.
- Properties box (`.props`): `background: var(--prop); border-radius: 4px; padding: 4px 32px 4px 8px; font-size: 12.8px; line-height: 1.6; margin: 4px 0`; key link `.pk` weight 500, colour `--mu`.
- Key hint (`.kbd`): inline-flex, min-width 18px, height 18px, padding `0 4px`, radius 4px, background `--bg4`, 11px, colour `--tx2`.
- Embed well (`.embed`): background `--bg2`, radius 4px, padding `4px 12px 8px`, margin `4px 0`.

### Outline interaction

- Rows: `pad = depth * 29px`; guide lines (`.gl`, 1px, `--bd`) at `x = i*29 + 8` for each ancestor level.
- Bullet button (`.bul`): 16x16, round; inner dot 6px `--dot` at opacity .85; hover turns the dot `--tx`; aria-label "Zoom into block". Click zooms into the block; Shift-click opens the block in the right sidebar.
- Closed bullet (`.bul.closed`, background `--bg4`): block has children and is collapsed.
- A disclosure arrow button (aria-label "Toggle children", 14px chevron-down) appears left of the bullet (`left: depth*29 - 20px`) when the block row is hovered and the block has children; it rotates -90 degrees when collapsed. Click toggles collapse in place.
- Initial collapse state comes from the `collapsed:: true` property. A collapsed block's children are not rendered.
- Heading blocks get a bullet offset: `bulTop = round(((HPX*1.6 - 16)/2)*10)/10`, plus 2px for h1 and h2.
- Block row gap 6px; body line-height 1.6.
- Block properties (visible ones) render in a properties box under the block content.

### Right sidebar

- Top row: three buttons "Contents", "Page graph", "Help" (13px `.sbtn`). Each adds an item (new items go on top; the sidebar opens).
  - "Page graph" uses the current page if the route is a page, else the whole-graph variant.
- Empty state text: "Shift-click any page link, or press ⇧⏎ in search, to open it here."
- Item card (`.card`: `--card` background, radius 6px, border, shadow `0 1px 3px rgba(0,0,0,.08)`). Header: title button (15px, weight 600, icon then title, truncated), a "More" button (aria-label "More") and a "Close" button (aria-label "Close"), both 30x30.
- Clicking the title toggles the card open or closed.
- Card menu items, in order (shown when "More" is clicked):
  1. "Close"
  2. "Close others" (only when there is more than one item)
  3. "Close all" (also closes the sidebar)
  4. "Collapse" (when open) or "Expand" (when collapsed)
  5. "Collapse others" (only when there is more than one item; keeps only this card open)
  6. "Open as page" (only for page and block items)
- Item types:

| Type | Header icon and title | Body |
| --- | --- | --- |
| `contents` | `☰` "Contents" | outline of the `contents` page (empty block) |
| `page` | `▤` page name | page link (20px, weight 300) then the page's top-level outline |
| `block` | `•` block's plain text (or "Block") | the block's outline |
| `graph` | `⌘` "Page graph" | graph 300px high; caption "Neighbours of {name}" or "Open a page to see its neighbourhood; showing the whole prototype graph." |
| `help` | `?` "Help" | "Keyboard shortcuts" (weight 600) and the ten shortcuts listed below, each with key chips |
| `search` | `⌕` the query (or "Blank input") | 'Search results for "{q}"' and up to 8 matching page names followed by matching blocks (stops adding blocks once there are 16 results); blocks show their page as a 12px sub-line |
| `slides` | `▭` "Slides" | slide deck (see Slides below) |

- Sidebar rendering is simplified: lists show a "•" marker, code is shown without extra chrome besides `.code`, videos show "YouTube {id}", images show "[external image: {alt}]", embeds show "embed: {title or ref}". Sidebar `{{query}}` and `{{cards}}` items are not rendered.

Help shortcuts (also the keys named in the search commands):

| Name | Keys |
| --- | --- |
| Search pages and blocks | ⌘ k |
| Go to journals | g j |
| Go to home | g h |
| Go to all pages | g a |
| Go to graph view | g g |
| Toggle flashcards | t c |
| Toggle left sidebar | t l |
| Toggle right sidebar | t r |
| Toggle between dark/light theme | t t |
| Toggle wide mode | t w |

### Slides (right sidebar item and fullscreen)

Slide model, from `slidesFor` and `slideVM`:

- Columns: one per top-level block (depth 0) of the page.
- Rows in a column: the top-level block first, then each of its direct children (only direct children).
- The first row of a column (the top block) shows only that block's own items. Every later row shows the child block's items and, recursively, all of its descendants' items.
- Slide label: `{column}.{row}` (1-based), for example `2.1`.
- Progress bar (3px high in the card, 4px in fullscreen, colour `--ac` on `--bg3`): width = `round(slidesBefore / totalSlides * 100)%`, where `slidesBefore` = all slides in earlier columns + the current row number.

Card view: link to the page (15px, weight 600), then "Tip: press `f` to go fullscreen" (13px), then the deck (`tabIndex=0`, aria-label "Slide deck: use arrow keys", 4:3 aspect ratio, border `1px solid --bd`, radius 4px). Slide body: centred text, 15px; headings sized by heading level; lists left aligned with "•"; images max 80% wide and 150px high (placeholder 70% x 90px "[external image: {alt}]"); video placeholder 70% wide, 16:9, text `YouTube {id}{?start=n}`; embeds shown as `[embed {ref}{title}]`; code shown as plain text (no highlighting, 11px). Controls (30x30, colour `--ac`; disabled ones get class `off`, opacity .2):

- "Previous slide" (left triangle) and "Next slide" (right triangle): move to the previous or next column and reset the row to the first.
- "Slide up" and "Slide down" (24x18 triangles): move within a column.
- A "Fullscreen (f)" button below the deck.

Deck keys (when the deck has focus): ArrowLeft, ArrowRight (and Space, same as ArrowRight), ArrowUp, ArrowDown, and `f` for fullscreen.

Fullscreen overlay (`position: fixed; inset: 0; z-index: 90`, background `--bg`, centred, padding 40px, gap 14px, focused on open): text sizes by heading level h1 56px, h2 44px, h3 36px, h4 30px, other 26px (weight per heading, line-height 1.25, max-width 1000px); list items 24px; images up to 60vw by 45vh; video placeholder 50vw wide; progress bar 4px at the bottom; label bottom right (13px): `Slide {h.v} · arrows to move · f or Esc to exit`. Keys: Escape or `f` exit; the arrow keys and Space behave as in the deck.

### Flashcards modal (left sidebar "Flashcards", command "Toggle flashcards")

- Overlay `z-index: 60`, click outside closes (no Close button in the modal). Dialog `role="dialog"`, aria-label "Flashcards", width 720px (max 100%), background `--card`, radius 12px, padding `18px 24px 22px`, gap 14px.
- Top row: "∞ All" (14px, weight 500) on the left; counter on the right (14px, muted): `{current}/{total} in this prototype (430 published)`.
- Cards are every block that references `card` (a `#card` tag), in page-source order.
- Phases:
  1. Front: the block text without `#card` (bold) followed by a `#card` tag. A "Show answers" button with key chip `s`.
  2. Back (after the button): the block's children as an indented outline (margin-left 29px, left border). The button becomes "Hide answers" (chip `s`). Three scoring buttons appear on the right: "Forgotten" (chip `f`, background `#ce2c31`), "Took a while to recall" (chip `t`, `#3e63dd`), "Remembered" (chip `r`, `#18794e`), all white text. Any of them advances to the next card and returns to the front phase. Scoring writes nothing.
  3. Done (after the last card): "Congrats, you've reviewed all the cards for this query, see you next time! 💯".
- The key chips `s`, `f`, `t`, `r` are labels only; the modal has no key handler (see Known gaps).
- No cloze phase: `{{cloze}}` is not implemented.

### Linked-reference filter dialog

Opened by the filter icon button next to the Linked References header (aria-label "Filter linked references", title "Filter", 22px icon, opacity 0.6). Dialog `role="dialog"`, aria-label "Filter", width 560px, background `--card`, radius 10px, padding `20px 24px`.

- Title "Filter" (18px, weight 500); hint "Click to include and shift-click to exclude. Click again to remove." (14px, muted).
- "Includes:" row (shown only if any), chips in colour `#18794e` on `--bg3`; "Excludes:" row (shown only if any), chips in colour `#ce2c31`. Chip text: `{page name} {count}`.
- Remaining chips: `{name} {count}` with the count muted. All chips are `.sbtn` on `--bg3`.
- Chips list the other pages referenced by the blocks that reference the current page, counted per block, most frequent first. Counts ignore the current filter.
- Click on a chip: if the page is already in the filter, remove it; otherwise include it (`true`). Shift-click: exclude it (`false`).
- Filter semantics: a linked block stays if it references every included page and none of the excluded pages. State is per page and lives only in memory.
- The header label becomes `{filtered} of {total} Linked References` when any filter is set. Singular: "1 Linked Reference". No search input in this dialog (the Backlinks board shows one).

### Linked References, Hierarchy, Unlinked References (page route only)

- "Pages tagged with "{title}"" section (h2, 16px, weight 700; margin-top 40px) when any page has a `tags::` value pointing at this page, plus the extra list for `tmux`: `Person/Brian P. Hogan/Book/25/tmux 3` and `tmux/Tutorial/Use tmux on Mac`.
- Linked References header button (`.sech`: 16px, weight 500, min-height 32px) with a 14px chevron that rotates -90 degrees when collapsed. Label: `{N} Linked References` (`N` = number of referencing blocks on other pages, before filters). Body shows groups on `--bg2` (radius 6px, padding `10px 14px`): page title link (weight 600), then each matching block with breadcrumb (14px muted, margin-left 24px, chevron-right separators at 50% opacity) and bullet. Heading blocks show at 18px. Groups sort journals first in the order Journals shows them (2026-10-23 Fri, 2026-10-22 Thu, 2026-09-15 Tue, 2026-08-25 Tue), then other pages in source order. Empty text: "No pages in this prototype link here."
- Hierarchy (`h2`-style button "Hierarchy", weight 700, opacity 0.3, toggles; open by default): list with `margin: 12px 24px`, rows `margin: 8px 0`; each path segment is a bracketed link to its full prefix; separators "/" with `margin: 0 8px; opacity: 0.3`. Members: every known page name that starts with `{name}/` (any depth, sorted by lower-case), plus, if the page name contains `/`, every ancestor prefix of length 2 or more segments. Shown only if non-empty.
- Unlinked References: header button with chevron, label "Unlinked References", collapsed by default. When opened it shows only the text: `Full-text search for "{title}" runs only when this opens; matching plain-text blocks are listed in the same groups as linked references.` No results are computed.
- Section order: outline, Pages tagged with, Linked References, Hierarchy, Unlinked References.
- What counts as a reference: a block references a page if its first text, list or quote items contain `[[page]]`, `#tag`, `#[[tag]]` or `[label]([[page]])`, or a property value contains `[[page]]`; names resolve through aliases. Queries, embeds and block refs do not create references. The count of backlinks on All pages uses the same rule.

### All pages

- Title `h1` "All pages" (36px, weight 300). Line: "Total {n} pages in this prototype (the published graph has 16693)". Paging: label `{page}/{pages}` then buttons "Prev" and "Next", 20 rows per page.
- Table columns: "Page name" (link), "Backlinks" (count), "Created At" ("—"), "Updated At" ("—"). The two date columns are placeholders in the prototype.
- Page names come from the known names list: the 24 sources, the extra names list and every link target found in the sources, sorted by lower-case name, with `contents` removed. The prototype list is not the real list of the garden.
- Styles: `.th` weight 600, 14px, muted, padding `10px 12px`, border-bottom; `.td` padding `8px 12px`, 15px.

### Graph route

- Container 640px high, `--bg2`, radius 8px, border; nodes are absolutely positioned buttons; edges are 1px lines rotated into place.
- Nodes: the 36 pages with the highest link degree (journal pages excluded unless "Show Journals" is checked), placed on three rings around the centre (ring radius `min(w,h) * (0.18 + ring * 0.12)`). Node diameter 12 if the page has more than 2 backlinks, else 8. Label: last path segment, 12px, below the node. Click opens the page. Node colour `--dot`.
- Edges: any pair of pages where a block on one references the other.
- Settings panel (`aside`, aria-label "Graph settings"): top 12px, right 12px, width 220px, `--card`, radius 8px, padding `10px 12px`, 14px. Sections:
  - "Nodes": "Show Journals" (functional checkbox, default off), "Orphan pages" (checked, no effect), "Built-in pages" (unchecked, no effect).
  - "Search": text "Click to search".
  - "Forces": text "Link Distance · Charge Strength · Charge Range".
  - "Export" (title only).
- Caption bottom left (13px, muted): "Pages from this prototype; the published graph draws all 16,693 with a force layout."
- Page graph (right sidebar): centred page (diameter 18, colour `--ac`) with up to 14 neighbours on one ring at radius `0.36 * min(w,h)`, in a 360x300 area.

### Whiteboards route

`h1` "Whiteboards" (36px, weight 300) and a dashed box (`border: 1px dashed --bd`, radius 8px, padding 40px, centred, muted): "[Whiteboard dashboard: a grid of whiteboard thumbnails. Not captured in this prototype.]"

### Toast

`role="status"`, fixed, bottom 24px, centred, background `--tx`, text `--bg`, padding `10px 16px`, radius 8px, 14px, `z-index: 95`, disappears after 2200 ms. Messages: "Wide mode widens the 810px content column" (command Toggle wide mode); "Copied block ref (({uuid}))" (search Cmd/Ctrl+C on a block; `uuid` literally if the block has no id); "Seeking video to {time}"; "{export option}: the browser downloads the file"; "Query re-run"; "Properties settings for this query: {block, page | page, created-at, updated-at}".

## 3 · Search

### Opening and shell

- Opened by the header "Search" button (no key binding for ⌘K, see Known gaps) or by sidebar actions. Opening resets query, filter, highlight and "show more" state and focuses the input.
- Overlay `z-index: 70`, `--ov` background, top padding `8vh`, click outside closes.
- Dialog `role="dialog"` aria-label "Search": width `90vw`, max-width 896px, background `--bg2`, radius 8px, shadow `0 24px 64px rgba(0,0,0,0.35)`.
- Input (label "Search", id `cmdk-input`): placeholder "What are you looking for?", 20px, padding `12px 4px`, no border.
- When a filter is active a chip appears before the input: "Search only: {name}" (13px, `--bg4` background, radius 6px) with an X button (aria-label "Remove filter").
- Results area: `max-height: 62vh; min-height: 220px`, scrolls.
- Group header: 32px high, padding `0 12px`, 12px, background `--bg2`; the group name is a bold (700) muted button (clicking it toggles show more or less), then a count pill (11px, `--bg4` background, radius 9px, padding `0 6px`; shows `99+` above 99), then on the right, when the group has more than 5 items, a button "Show more" or "Show less" followed by two key chips `⌘` and `↓` (or `↑` when expanded). Group bottom border `1px solid --bd`.
- Result row (`.res`): 15px, min-height 40px, padding `8px 12px`, icon tile 28x28 (radius 6px, `--bg4`, icon 16px muted), label with the matched text in bold (`.hit`, weight 600, colour `--tx`) and truncated, optional sub-line (12px muted: the block's page name), optional key chips (commands), optional info text (12px muted). The highlighted row has background `--bg4` and colour `--tx`.
- Icons by type: page (document), block (bullet list), command (a command-key glyph), recent (clock), filter (funnel).
- Highlight follows the mouse (`mousemove` on a row sets it).
- The highlight index counts only visible rows (after the 5-row limit).
- Footer (13px, muted, `--bg2`, border-top): "Tip:" followed by one tip, and on the right the hint buttons for the highlighted result.
- GROUP-LIMIT is 5 rows per group; "Show more" reveals the whole group; changing the query resets all groups to collapsed.

### Empty query, no filter

Groups in order:

1. "Recents": the recents list (pages opened this session, newest first, max 6), shown only when non-empty. Row type: recent.
2. "Commands": all nine commands.

Commands (label, keys, action):

| Command | Keys | Action |
| --- | --- | --- |
| Go to journals | g j | Journals route |
| Go to all pages | g a | All pages route |
| Go to graph view | g g | Graph route |
| Go to home | g h | Journals route |
| Toggle flashcards | t c | Open the flashcards modal |
| Toggle left sidebar | t l | Toggle the left sidebar |
| Toggle right sidebar | t r | Toggle the right sidebar |
| Toggle between dark/light theme | t t | Toggle theme |
| Toggle wide mode | t w | Toast "Wide mode widens the 810px content column" |

Running a command closes search first (`runCommand` sets `search: false`).

### Typed query (no filter), group order

For a query `q` that does not start with `/`, matching is case-insensitive substring. Groups in order, each only if non-empty:

1. "Pages": page names containing `q`; names that start with `q` come first, then shorter names first. Row type: page.
2. "Commands": commands whose label contains `q`.
3. "Current page": blocks of the page being viewed whose text contains `q` (only on a page route).
4. "Blocks": matching blocks on all other pages.

Block rows show the block's plain text (with `[[ ]]`, `**`, backticks, link markup and leading `#` removed, timestamps as `MM:SS`) and, as the sub-line, the page name. Blocks whose plain text is `[`, or empty, are skipped.

### The `/` filter list

When the query starts with `/` and no filter is chosen:

- Group "Filters": the six filters below whose text or name contains the text after `/`. A bare `/` lists all six. Each row has the info text "Add filter to search".
- If the text after `/` is non-empty, also group "Pages": page names containing that text.

Filters (exact texts, group key, chip name):

| Text | Group | Chip name |
| --- | --- | --- |
| Search only pages | pages | pages |
| Search only current page | current-page | current page |
| Search only blocks | blocks | blocks |
| Search only commands | commands | commands |
| Search only whiteboards | whiteboards | whiteboards |
| Search only files | files | files |

Choosing a filter (Enter or click) sets it, clears the query and resets the highlight. With a filter active:

| Filter | Groups shown |
| --- | --- |
| pages | "pages" (lower-case name): matching pages (needs a non-empty query) |
| blocks | "Current page" then "blocks" (lower-case name) |
| current page | "Current page" |
| commands | "commands" (lower-case name) |
| whiteboards, files | no group is ever produced (always "No matched results") |

Code behaviour for an empty query with a filter: "commands" lists all commands; "pages" lists nothing; "blocks" and "current page" list every block (the empty string matches all).

### Keyboard

| Key | Action |
| --- | --- |
| ArrowDown, Ctrl-N | Highlight next (wraps) |
| ArrowUp, Ctrl-P | Highlight previous (wraps) |
| Enter | Run the highlighted result (open page, zoom block, trigger command, apply filter) |
| Shift+Enter | Open the highlighted page or block in the right sidebar (falls back to Enter for commands and filters) |
| Cmd/Ctrl+Enter | Open the current query as a search item in the right sidebar |
| Escape | Remove the filter if one is set; otherwise close search |
| Backspace | With an empty query and a filter set: remove the filter |
| Cmd/Ctrl+C | If the highlighted result is a block: toast "Copied block ref ((uuid))" (nothing is placed on the clipboard) |

Cmd+ArrowDown also moves the highlight down like ArrowDown; there is no keyboard path for Show more or Show less (the buttons are click-only, though they display the `⌘↓` and `⌘↑` chips).

Shift-click on a result opens it in the sidebar; a plain click runs it.

### Footer tips and hint buttons

Tip text (exactly one is shown):

| State | Tip |
| --- | --- |
| A filter is set | "Type" `ESC` "to clear search filter" |
| No filter and empty query | "Type" `/` "to filter search results" |
| No filter and non-empty query (including queries that start with `/`) | `⌘` `⏎` "to open search in the sidebar" |

Hint buttons for the highlighted result (label then key chips):

| Highlighted type | Hints |
| --- | --- |
| page, recent | "Open" `⏎`; "Open in sidebar" `⇧` `⏎` |
| block | "Open" `⏎`; "Open in sidebar" `⇧` `⏎`; "Copy ref" `⌘` `c` |
| command | "Trigger" `⏎` |
| filter | "Filter" `⏎` |

### No results

When the query is non-empty or a filter is set and there are no rows: "No matched results" (14px, muted, padding `16px 12px`).

## 4 · Markdown renderer rules

### Page and block parsing

- A page source starts with optional page properties: leading lines that are not block lines, each matching `key:: value` (key characters `A-Z a-z 0-9 _ . -`). They end at the first block line.
- A block line matches `^(\t*)-( (.*)|$)`: depth is the number of leading tabs; the text after `- ` is the block's first line. Parent is the nearest earlier block with smaller depth.
- Any other line is a continuation of the current block: leading tabs are removed, then up to two leading spaces. A line that opens a code fence (` ``` ` or `~~~`) on the block's first line, or later, makes following lines continuation lines until the fence closes (so `- ` lines inside a fence do not start blocks).
- Each continuation or first line is classified in this order:
  1. Inside an advanced query (`#+BEGIN_QUERY` to `#+END_QUERY`): collected as the query text.
  2. `#+BEGIN_QUERY`: starts an advanced query.
  3. A line of the form `| a | b |`: a table row. A separator row (cells all `:?-+:?`) is dropped. The first row is the header. A table ends at the next non-table line.
  4. Inside a code fence: collected as code. A fence opens on a line matching `` ^(```|~~~)([\w-]*)$ `` and closes on a line that is only ` ``` ` or `~~~` (either marker closes either kind).
  5. Property: `^([A-Za-z0-9_.-]+):: ?(.*)$`.
  6. `{{video URL}}` or `{{youtube URL}}` as the whole line.
  7. `{{embed X}}` as the whole line.
  8. `{{query X}}` as the whole line.
  9. `{{cards X}}` (the space before `X` is optional) as the whole line.
  10. Image as the whole line: `![alt](src)` optionally followed by `{:width N, :height N}`.
  11. List item: `^(\s*)\* (.*)$`, depth = floor(indent length / 4).
  12. Quote: `^> ?(.*)$`.
  13. Heading: `^(#{1,6}) (.*)$`.
  14. Otherwise plain text.
- A block can have several items (for example a heading line, then a list, then properties).
- Task markers (`TODO`, `DOING`, `DONE`, `LATER`, `NOW`, `WAITING`, `CANCELED` at the start of the first text line) are detected, but only to support the `task` query operator. The prototype draws no checkboxes (see Known gaps).
- A block's "plain text" (used in search, breadcrumbs, block refs, query tables) is its first text or list item with `{{youtube-timestamp N}}` replaced by `MM:SS`, `[[` and `]]` removed, `**` removed, backticks removed, `[label](url)` replaced by `label`, and a leading `#`s plus space removed. If a block has no text or list item, it is the first raw line.
- Page properties and block properties: the hidden-property list below is applied when displaying them.

### Properties

Hidden keys (exact list in the script, not shown anywhere): `id`, `collapsed`, `heading`, `background-color`, `created-at`, `updated-at`, `card-last-interval`, `card-repeats`, `card-ease-factor`, `card-next-schedule`, `card-last-reviewed`, `card-last-score`, `filters`, `public`, `title`, `query-table`, `query-properties`, `query-sort-by`, `query-sort-desc`, `prototype-slow`.

(`prototype-slow` is a prototype-only key. The Frontmatter board also lists `hl-*`, `ls-type`, `query-*` and task markers as never shown; the script hides only the exact keys above.)

Display:

- Page properties show in a box under the title when at least one is visible. Block properties show in a box under the block's content.
- Key: link (weight 500, muted) to the page named like the key; then `:` with a 4px right margin; then the value.
- `tags` and `alias`: the values are split into page names (from `[[...]]`, or comma-separated text) and shown as links without brackets, separated by ", ".
- A value that is a bare URL (`http:` or `https:`) shows as an external link.
- Other values render as inline markup, so `[[Page]]` keeps its brackets. A list such as `see-also:: [[a]], [[b]]` shows `[[a]], [[b]]` with the comma as plain text.

### Inline markup (`parseSegs`)

The tokenizer is a single regex with these forms, in priority order at each position:

| Syntax | Result |
| --- | --- |
| `{{youtube-timestamp N}}` | timestamp link, see below |
| `((uuid))` (36 chars of `0-9a-f-`) | block ref, see below |
| `#[[Tag]]` | tag link, shown as `#Tag` |
| `[label]([[Page]])` | link to the page, label only, no brackets |
| `[[Page]]` | page link with grey `[[`/`]]` |
| `[label](http...)` | external link (`a.ext`), opens in a new tab with `rel="noopener"` |
| `**bold**` | bold (backticks inside are stripped) |
| `` `code` `` | inline code (`.ic`) |
| `==text==` | highlight (`.mk`) |
| `*italic*` | italic (asterisks only; the text must not start with a space or `*`) |
| `#word` | tag link when preceded by line start or whitespace; the word starts with a letter, digit or `@` and continues with word characters, `/`, `.`, `@`, `-` |

Not implemented (the text stays literal): `_italic_`, `~~strike~~`, `<email>` autolinks, bare URLs, `{{cloze}}`, and other macros. Images are only recognised as a whole line, never inline. Inline segments of a heading are handled like any text line.

Where each segment kind is drawn: outline text supports all kinds; list items support text, page link, bold, italic, code, external link; quotes support text, bold, page link; table cells support text, bold, code, italic, page link; unsupported kinds inside those containers produce no output (for example an external link inside a quote disappears).

### Headings

- Constants: `HPX = {0:16, 1:32, 2:24, 3:19.2, 4:16, 5:13.28, 6:12}`, `HFW = {0:400, 1:300, 2:300, 3:600, 4:600, 5:600, 6:600}` (font size in px and weight by heading level; 0 is body text). The h5 value 13.28 equals 0.83em of 16px; h6 is 12px.
- h1 and h2 have a rule below: `border-bottom: 1px solid var(--hr)`, `padding-bottom: 0.125em`, `margin: 0.125em 0`. Other levels have none.
- Line-height 1.6 for headings in the outline.
- The fixture page `Prototype/Fixture/Headings` states: "h1–h4 were measured on the published site; h5–h6 come from Logseq's block.css." It labels: Heading 1 32px weight 300 rule below; Heading 2 24px weight 300 rule below; Heading 3 19.2px weight 600; Heading 4 16px weight 600; Heading 5 0.83em weight 600; Heading 6 0.75em weight 600; Body text 16px weight 400 line-height 1.6.
- Linked reference blocks that are headings display at 18px, other blocks at 16px.
- Fullscreen slides use 56, 44, 36, 30, 26 px for h1 to h5 and above (see Slides).

### Lists, quotes, tables

- List item `* text`: row with a muted "•" and the text, `display: flex; gap: 8px; padding-left: {18 * depth}px; line-height: 1.6`. Depth is the indent length divided by 4 (floored).
- Quote `> text`: `blockquote` with `margin: 4px 0; padding: 4px 12px; border-left: 3px solid var(--bd); color: var(--tx2)`.
- Table (`.ltab`): width 100%, collapsed borders, `margin: 1rem 0`, 15px; `th` 14px, weight 400, colour `--tx`, `border-bottom: 2px solid var(--bd)`, padding `10px 8px`; `td` padding `6px 8px`; even rows have background `--bg3`. In a query list view the wrapper scrolls horizontally.
- Ordered lists have no special rendering (text like `1/3` stays text).

### Code fences

- Container `.code`: relative, background `--codebg`, radius 2px, Fira Code (fallback Monaco, Menlo, monospace) 12px, line-height 1.45, padding `22px 12px 10px`, `white-space: pre`, horizontal scroll, margin `4px 0`, colour `--tx`. Each line is a `.cl` row with min-height 1.45em (empty lines render as a space).
- Language label: `.lang` positioned top right, 12px, padding `4px 6px`, background `--bg2`, opacity .8, 'Source Sans 3', line-height 1; shows the fence language (empty if none).
- Highlighting applies to `python` (also `py`), `rust` (also `rs`) and `bash` (also `sh`, `shell`, `zsh`). Other languages and no language (for example `mermaid`) render as plain text, one span per line. The language name match is case-insensitive.
- Token classes and colours (tokens defined in section 2):

| Class | Colour token | Used for |
| --- | --- | --- |
| `t-k` | `--ck` (`#bd4b00`, dark `#ff8b3e`) | keywords |
| `t-s` | `--cs` (`#297c3b`, dark `#71d083`) | strings |
| `t-c` | `--cc` (`#7e868c`, dark `#787f85`), italic | comments |
| `t-n`, `t-b` | `--cn` (`#cd1d8d`, dark `#ff8dcc`) | numbers (`t-n`), python builtins (`t-b`) |
| `t-d`, `t-p`, `t-a` | `--cd` (`#0078a1`, dark `#75c7f0`) | name after a definition keyword (`t-d`), python attribute after `.` (`t-p`), bash options (`t-a`) |
| `t-o` | `--co` (`#793aaf`, dark `#d19dff`) | operators, rust macros like `println!` |

- Python rules: comment `#...` to end of line; strings with `"` or `'` and backslash escapes; numbers `\d+(\.\d+)?`; words (identifiers); operators `[=+\-*/<>%&|]+`.
  - Keywords: `and as assert async await break class continue def del elif else except finally for from global if import in is lambda nonlocal not or pass raise return try while with yield None True False`.
  - Builtins: `print open len range str int dict list set tuple isinstance super`.
  - Definition keywords: `def`, `class` (the next word gets `t-d`).
  - A word right after `.` that is not a keyword or builtin gets `t-p`.
- Rust rules: comment `//...`; string `"..."`; lifetime or label `'name` (matched but left unstyled); numbers; macros `name!`; words; operators.
  - Keywords: `as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while async await dyn`.
  - Builtins: none.
  - Definition keywords: `fn`, `struct`, `enum`, `trait`, `mod`.
- Bash rules: only comments (`#...` preceded by line start or whitespace, `t-c`) and options (`-x`, `--flag`, preceded by line start or whitespace, `t-a`).
- Code blocks in the right sidebar use the same `.code` styling and tokens; slides show code as plain text at 11px.

### Images

- Recognised only as a whole line: `![alt](src)` with an optional `{:height H, :width W}` suffix.
- Image source rule: only paths starting with `../assets/` map to a known asset (`image_1742292361076_0.png` and `image_1742369972123_0.png`, both mapped to raw.githubusercontent.com URLs under `codekiln/logseq-encode-garden/main/assets/`). The two slideshow images on `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025` use `../assets/` paths and render the real images; every other image uses an absolute `https://` URL and is drawn as a placeholder.
- Placeholder: background `--bg3`, radius 4px, margin `6px 0`, 13px muted centred text `[external image: {alt}]` (alt falls back to the file name), width = `:width` in px (default 480px) with max-width 100%, height = `:height` in px (default 220px).
- A real image (not used by the sources) would be `display: block`, width = `:width` px (or auto), max-width 100%, radius 4px.

### `{{video}}` and `{{youtube}}`

- `{{video URL}}` and `{{youtube URL}}` are treated alike. The video id is read from `v=`, `youtu.be/` or `embed/` followed by 11 characters. The start time comes from `t=` (digits, optional trailing `s`) in the query string.
- Placeholder (iframe is not loaded): 560px wide, max-width 100%, 16:9, background `#0f0f0f`, radius 4px, margin `6px 0`, white text: red YouTube play icon (68x48), line `youtube.com/embed/{id}` plus `?start={n}` when a start time is given (13px, mono, opacity .8), and "iframe placeholder · 560×315 in Logseq" (12px, opacity .55).
- Examples in the sources: `{{video https://www.youtube.com/watch?v=6cqVzgVQJfE}}` (no start); `{{video https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s}}` shows `youtube.com/embed/TQsP_PlCY1I?start=205`; `...&t=806s` shows `?start=806`.

### `{{youtube-timestamp}}`

- `{{youtube-timestamp N}}` renders as a link with a clock icon (outline view only) and the time. Time format: `MM:SS` zero-padded, with `HH:` prepended only when the hour is non-zero (so 0 is `00:00`, 41 is `00:41`, 620 is `10:20`, 3600 is `01:00:00`).
- The link has title "Seek video".
- Seek behaviour: the link seeks the most recent video earlier in the same page (the walk through blocks remembers the last `{{video}}` seen, at or before the block, on that page). It sets that video's seek time and shows the toast "Seeking video to {time}". The video placeholder then shows a badge bottom left (13px, `rgba(0,0,0,0.75)` background, radius 4px, padding `4px 10px`): "Seeked to {time}". If no video precedes the link on its page, clicking does nothing.
- In block ref text, search results and breadcrumbs, the timestamp appears as plain `MM:SS` text (for example "10:20 Agent skill").

### Embeds

- `{{embed ((uuid))}}` and `{{embed [[Page]]}}` draw a grey well (`.embed`).
- Block embed: the target block and all its descendants, flattened in document order, up to 10 rows. Page embed: all top-level blocks of the page with their descendants, up to 10 rows, preceded by a header row (page icon, link to the page, weight 600, gap 10px, padding `4px 0`).
- Row indent: `depth * 29px`; font size from the block's heading level; each row has a bullet and shows only the block's first text line with inline markup.
- Missing target (unknown uuid or page not in the sources): "[embedded block {ref} lives on a page outside this prototype]" (14px, muted).
- Embeds inside embeds are not expanded further (only the first text line of each row is shown).
- In slides: `[embed {ref}{title}]`; in the right sidebar: `embed: {title}{ref}`.

### Block refs

- `((uuid))` where the uuid belongs to a block with an `id::` property in the 24 sources: a link (`a.bref`) showing the target's plain text, title "Block from {page name}". Click zooms to the block; Shift-click opens it in the right sidebar.
- Unknown uuid: `span.warn` with title "Block ref invalid" showing the raw `((uuid))`.
- Example: `((6aa57707-6afe-46ec-bf96-766804d117dc))` resolves to "10:20 Agent skill"; `((67da7fc2-1c1f-46e9-84b0-dee824e45395))` on the AI Engineer Summit page does not resolve and shows the warning form.

### `{{query ...}}` and `#+BEGIN_QUERY`

Every query item has a header row (14px, muted, padding `2px 0 4px`) with a toggle on the left and tools on the right.

- Simple query (`{{query ...}}`): left shows a search icon and the label "Live query" (or "Live query for pages" for `page-tags`). Always open.
- Advanced query: left shows the `:title` (16px, `--tx`; default "Query"). If it has `:collapsed? true` it starts closed and, while closed, shows the result count in parentheses after the title (for example `(3)`). The title button toggles open and closed. Tools appear only while open.
- Tools (right): result count text "{n} result" or "{n} results" (only if n > 0), a view toggle button (title "Switch to table view" or "Switch to list view", not for `page-tags`), a "Setting properties" button (toast on click: "Properties settings for this query: block, page" or "...: page, created-at, updated-at"), and, for slow queries, a refresh button (aria-label "Refresh query"; toast "Query re-run").
- Slow query: a block with property `prototype-slow:: {ms}` gets the refresh button with title "This query takes {ms}ms to finish, it's a bit slow so that auto refresh is disabled. Click the refresh button instead if you want to see the latest result." (the property itself is prototype-only).
- States:
  - Error: `warn` text "Query failed:" followed by the error message in Fira Code 13px; tools hidden.
  - Zero results: "No matched result" (14px, margin-top 8px, opacity .9).
  - List view (default): groups on `--bg2` (radius 6px, padding `8px 12px`), one per page: page link (weight 600) and each matching block with breadcrumb and bullet. Only top-level hits are shown (a hit whose ancestor is also a hit is dropped).
  - Table view (`query-table:: true`, or toggled): columns "block" and "page" (block text and page link); for `page-tags` queries the columns are "page", "created-at", "updated-at" with "—" in the date columns.
- Query language implemented:
  - Tokens: `(`, `)`, `[[Page]]` (optionally `#[[Page]]`), quoted strings, other non-space words.
  - Operators: `and`, `or`, `not` (true when none of its arguments match), `task` (block marker is one of the arguments), `page` (block is on one of the pages), and `page-tags` as the top-level operator (returns pages, matched on their `tags::` values).
  - A bare `[[Page]]` matches a block when the page is the block's own page, or the block or any of its ancestors references that page (name resolved through aliases).
  - The block that holds the query is excluded from its own results.
  - Errors: "Empty query", "EOF while reading", "Unmatched delimiter: )", "Unexpected input after query", "Missing :query", "Unknown query operator: {op}".
- Advanced query syntax read: a `:title "..."`, a `:collapsed? true` flag, and `:query` followed by a parenthesised expression or a `[[Page]]`. Unbalanced `()[]{}` gives "EOF while reading".
- Query examples in the fixtures (`Prototype/Fixture/Queries`): simple query; table view; page query `(page-tags [[Logseq/Slideshow]])`; advanced with title ("Cards about yazi", query `(and [[yazi]] [[card]])`); advanced collapsed ("Blocks that mention tmux"); no results `(and [[yazi]] [[Rust]])`; slow `(or [[nvim]] [[vim]])` with `prototype-slow:: 120`; failed (unbalanced `(and [[tmux]]}`); flashcards query; flashcards query with no matches.
- Path refs: queries and cards match on the block's path set (page name plus refs of the block and all its ancestors), which is how `{{cards [[yazi]]}}` finds a card whose `[[yazi]]` tag is on the parent block.

### `{{cards ...}}` widget

- `{{cards}}` (no query) or `{{cards QUERY}}` where QUERY uses the same expression language. Cards are blocks that reference `card`, filtered by the query; the query is matched on the block's path set.
- Container: border `1px solid --bd`, radius 8px, padding `12px 16px`, margin `4px 0`.
- States:
  - No match: dimmed (opacity .6) inline-code chip "Cards: {query}" and "No matched cards" (weight 500).
  - Header (matches exist): review icon, the query text (or "All"), and on the right `{current}/{total}` (title "Current/Total") and `{overdue}/{total}` (title "Overdue/Total"). Overdue = cards with no `card-next-schedule` or a schedule date on or before `2026-10-24` (the prototype's fixed "today").
  - Active card: front (bullet, text with bold, code, tag and page links), children shown after the button flips: bullet outline with a left guide. Buttons: "Show answers" or "Hide answers" with chip `s`; when showing the back: "Forgotten" `f` (`#ce2c31`), "Took a while to recall" `t` (`#3e63dd`), "Remembered" `r` (`#18794e`). Each scoring button advances to the next card.
  - Done: "Congrats, you've reviewed all the cards for this query, see you next time! 💯".
- State is per block, in memory; nothing is saved.

## 5 · Garden pages included

The prototype embeds 24 page sources (full text in the appendix). Which are fixtures:

- Invented for the prototype, not garden pages: `Prototype/Fixture/Headings`, `Prototype/Fixture/Queries`.
- Listed under "EXAMPLE PAGES · NOT LOGSEQ UI" in the sidebar but not named `Prototype/Fixture/*`: `Rust/loop/label`, `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain`, `Logseq/Flashcard/Review/Favorite/yazi`. The HTML does not say whether their text is verbatim from the garden.
- Page sources otherwise presented as garden pages (excerpts):

| Group | Pages |
| --- | --- |
| Main page example | `tmux` |
| Cards and Keyshorts | `tmux/Keyshort/Pane/Zoom Pane`, `yazi/Keyshort/Leader`, `yazi/Keyshort/Sort/Time/Modification` |
| Video and refs | `Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs`, `tuicr/Agent/Skill` |
| Slides and images | `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`, `Latent Space/Pod` |
| Embeds and aliases | `Spotify/Tool/TUI`, `spotify-player`, `Logseq/Frontmatter`, `Logseq/Frontmatter/alias`, `Logseq/Slideshow` |
| Favorite flashcard decks | `Logseq/Flashcard/Review/Favorite`, `Logseq/Flashcard/Review/Favorite/yazi` |
| Journals | `2026-10-23 Fri`, `2026-10-22 Thu`, `2026-09-15 Tue`, `2026-08-25 Tue` |
| Empty | `contents` |

- Page names shown in All pages also include an extra-names list (mostly `tmux/...` pages, favorites, and a few link targets) that have no source; those pages open as empty pages. The extra list is: `tmux/Bug`, `tmux/Bug/Vim Right Pane Insert Shows Up in Left Pane`, `tmux/Cheat Sheet`, `tmux/Command`, `tmux/Command/choose-tree`, `tmux/Command/last-window`, `tmux/Command/new-session`, `tmux/Command/new-session/-t`, `tmux/Command/new-window`, `tmux/Command/set-option`, `tmux/Command/source-file`, `tmux/Config`, `tmux/Config/local`, `tmux/Docs`, `tmux/Docs/Getting Started`, `tmux/Docs/Getting Started/Basic Concepts`, `tmux/How To`, `tmux/How To/Reload`, `tmux/Idea`, `tmux/Idea/Pomodoro with status line and silent flash`, `tmux/Idea/Toggl Track status in status line`, `tmux/Keyshort`, `tmux/Keyshort/Pane`, `tmux/man`, `tmux/Tutorial/Use tmux on Mac`, `Person/Brian P. Hogan/Book/25/tmux 3`, `fd`, `gh-dash`, `tuicr`, `LazyVim`, `Obsidian`, `Codex`, `chezmoi`, `mise`, `nvim`, `vim`, `yazi`, `Lazygit`, `Catppuccin`, `oh-my-tmux`, `Keyshort`, `card`, `Logseq/Slideshow`, `Logseq/Frontmatter/tags`, `TUI`, `Rust`, `Model Context Protocol`, `AI Engineer Summit`, `Diataxis/Explanation`, `Filed`, `Updated`, `Ghostty`, `Claude/Code`, `GitHub/CoPilot/CLI`.
- Journals shown on the Journals route, in order: `2026-10-23 Fri`, `2026-10-22 Thu`, `2026-09-15 Tue`, `2026-08-25 Tue`.
- The prototype's "today" is `2026-10-24`.

## 6 · Known gaps

Stated by the prototype itself or visible in its code:

- Iframes are placeholders: videos are drawn as a black 16:9 box with a text line, no player loads ("iframe placeholder · 560×315 in Logseq").
- External images are placeholders ("[external image: {alt}]"). Only the two `../assets/` images on the slideshow page render.
- Mermaid is shown as a fenced code block, as Logseq does without a plugin.
- Whiteboards are a placeholder text box.
- Journal and page content are excerpts of the garden pages, and the All pages list, backlink counts, hierarchy and graph are computed only over the prototype's pages. The real graph has 16,693 pages and 430 flashcards; the prototype states this in captions.
- Graph layout is not a force simulation (fixed rings); graph settings controls other than "Show Journals" do nothing; no sliders, "Clear All", "Reset Forces" or "Excluded pages" switch.
- Created At and Updated At columns show "—".
- Unlinked References performs no search.
- Search is bound only to the header button. The titles "Search (⌘K)", "(t l)", "(g h)", "(t r)" and the shortcut list name keys that are not bound: there is no global keydown handler for ⌘K or the `g`/`t` sequences. Flashcard keys `s`, `f`, `t`, `r` are not bound either. Only these key handlers exist: the search input (arrows, Ctrl-N/P, Enter variants, Escape, Backspace, Cmd/Ctrl-C), the slide deck (arrows, Space, `f`) and the fullscreen deck (Escape, `f`, arrows, Space).
- "Show more" and "Show less" in search work by click only; the `⌘↓` and `⌘↑` chips are labels.
- Flashcard scoring, saved filters, and "Copy ref" write nothing (toast only).
- Tasks (`TODO`/`DONE`) are not drawn as checkboxes; `~~strike~~`, `_italic_`, `<email>`, `{{cloze}}` and unknown-macro warnings are not implemented.
- `Whiteboards` and `files` search filters list nothing.
- The properties-box hidden list is shorter than the Frontmatter board's list.
- Block refs inside quotes, list items and table cells are not rendered as refs; external links and tags inside quotes are dropped.
- The filter dialog has no "Search in linked pages" input; the filter icon does not change colour with filter state.

### Differences from other boards

- Favorites: 19 entries here (including `GitHub/CLI/Extension/gh-dash`, `tmux/Q/How can I put a thick border around the focused pane?`, `Logseq/Flashcard/Review/Favorite`, `Codex`, `DevContainer`, `rg`, `Zoxide`); the Page view and Journals boards show 11.
- Source of the `tmux` page differs from the Page view board: here the first block is `# [tmux](https://github.com/tmux/tmux)`, "customization" has an extra child "For coordinated palette setup (e.g.  with TPM ports)", and "learning" has `[[Person/Brian P. Hogan/Book/25/tmux 3]]`.
- All pages here has a "Prev"/"Next" pair; the All pages board draws only "Next".
- Dialog widths: flashcards 720px (board 760px), filter 560px (board says Logseq's `.ls-filters` max-width 704px), search 90vw up to 896px (board 720px).
- Search group names use "Recents", "Commands", "Pages", "Current page", "Blocks" here; the Overlays board shows only "PAGES" and "BLOCKS" in upper case with no Current page group.
- Heading sizes: Prototype fixtures state h5 as 0.83em and h6 as 0.75em; the Outline board gives h5 0.83em and does not state h6.

## Appendix · page sources

Exact text of the `SRC[...]` entries in the script, tabs preserved. Page properties come first, then blocks. Where a source contains fenced code, the fence here is made of tildes so the inner fences stay intact.

### `tmux`

~~~~~~~~markdown
created-by:: [[Person/Nicholas Marriott]]
logseq-entity:: [[Logseq/Entity/Software/Project]]

- # [tmux](https://github.com/tmux/tmux)
	- [[GitHub/Star]]s: 47,343 as of [[2026-07-04 Sat]]
	- A terminal multiplexer: run many terminal sessions in one screen, detach and reattach them, and script window and pane layouts.
	- ideas
		- [[tmux/Idea/Pomodoro with status line and silent flash]]
		- [[tmux/Idea/Toggl Track status in status line]]
	- customization
		- For coordinated palette setup (e.g.  with TPM ports)
		- [[tmux/Config]]
		- [[Catppuccin]]
	- extensions
		- [[oh-my-tmux]] — by [[Person/Grégory Pakosz]]
	- learning
		- [[tmux/Docs/Getting Started]]
		- [[Person/Brian P. Hogan/Book/25/tmux 3]]
		- [[tmux/man]]
~~~~~~~~

### `tmux/Keyshort/Pane/Zoom Pane`

~~~~~~~~markdown
- [[Keyshort]] [[tmux]] [[tmux/Keyshort]] [[tmux/Keyshort/Pane]]
	- **Zoom pane (toggle)** #card
	  card-last-interval:: 10.24
	  card-repeats:: 3
	  card-ease-factor:: 2.56
	  card-next-schedule:: 2026-07-04T12:38:29.316Z
	  card-last-reviewed:: 2026-06-24T07:38:29.316Z
	  card-last-score:: 5
		- Shortcut: `<prefix> z` (same chord toggles zoom **off**; tmux calls this key the **prefix**)
		- [[My Notes]] : **Zooms** the active [[tmux/Pane]] it **fills the window**; other panes in that window are **hidden until you press the same shortcut again** to restore the split layout. You can see that a window is zoomed because it has a **Z** at the end of its window name, or at any rate, in oh-my-tmux it does.
~~~~~~~~

### `Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs`

~~~~~~~~markdown
logseq-entity:: [[Logseq/Entity/YouTube]]
created-by:: [[Person/Omer Hamerman]]
date-created:: [[2026/07/28]]
readwise-link:: https://read.readwise.io/read/01m25dyqe1zk4hs4yq903vjkr4

- # [The Holy Grail of Code Review TUIs - YouTube](https://www.youtube.com/watch?v=6cqVzgVQJfE)
	- Speaker: [[Person/Omer Hamerman]] (channel credit in Readwise: **DevOps Toolbox**)
	- Channel: [DevOps Toolbox](https://www.youtube.com/@devopstoolbox) · ~16 mins · published [[2026/07/28]]
	- Topic: [[tuicr]] — terminal code review that can push real GitHub/GitLab comments; compared with hunk; agent skill via [[PiAI]]; pairs with [[gh-dash]]
	- ## [[Video]]
		- Raw transcript: [[Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs/Video]]
		- {{video https://www.youtube.com/watch?v=6cqVzgVQJfE}}
			- ### {{youtube-timestamp 0}} The review gap
				- Opens on a Mitchell tweet thread about "vibe coding" quality metrics, then the pitch: review a PR from the terminal, leave inline comments like GitHub, and push them back without opening a browser.
				- That was the missing piece versus **hunk**, a diff TUI that stops short of team-visible review comments.
			- ### {{youtube-timestamp 41}} Introducing [[tuicr]]
				- "When I saw tuicr, pronounced tweaker, I got curious."
				- Checklist it hits: single binary, [[vim]] motions, works with [[git]] / [[jj-vcs]] / [[mercurial]], and can submit real GitHub and GitLab review comments.
			- ### {{youtube-timestamp 106}} Features vs hunk
				- tuicr is built around collaborative review: suggestion / issue / note / praise [[tuicr/Comment/Type]], GitHub-style diffs, themes, clipboard Markdown export, and an agent skill.
			- ### {{youtube-timestamp 187}} Creator and stack
				- [[Person/Almog Gavra]]: Cornell, staff engineer at LinkedIn and Confluent, now doing his own thing.
				- Written in [[Rust]]. Pronounced **tweaker**.
			- ### {{youtube-timestamp 220}} Install and local review
				- `tuicr tui` matches running the binary; `tuicr -w` reviews uncommitted local changes.
			- ### {{youtube-timestamp 400}} Comment types and submit
			  id:: 6aa57707-e737-4321-9b40-aaac81aaba45
				- `:submit` pushes the review via [[GitHub/CLI]] (`gh`); GitLab needs `glab`.
			- ### {{youtube-timestamp 520}} End-to-end with Worktrunk, [[PiAI]], and [[gh-dash]]
				- For non-review GitHub life cycle (issues, notifications), he leans on [[gh-dash]].
			- ### {{youtube-timestamp 620}} Agent skill
			  id:: 6aa57707-6afe-46ec-bf96-766804d117dc
				- [[tuicr/Agent/Skill]] uses the CLI to add/read comments on **active** sessions and can open a review pane in [[tmux]] or [[Zellij]].
				- Inline agent comments beat a generic AI summary — "game changer."
~~~~~~~~

### `tuicr/Agent/Skill`

~~~~~~~~markdown
tags:: [[Diataxis/Concept]]
logseq-entity:: [[Logseq/Entity/Concept]]
see-also:: [[tuicr/Comment/Type]], [[tuicr]]

- # [[tuicr]] [[Agent/Skills]]
	- ## Overview
		- Design line from the skill file itself: **"the TUI is where the human reviews code; the CLI is how the agent discovers active sessions, reads user comments, and, only when appropriate, adds agent-authored comments."**
		- Video coverage: ((6aa57707-6afe-46ec-bf96-766804d117dc)) in [[Person/Omer Hamerman/YouTube/26/07/The Holy Grail of Code Review TUIs]] — install via the packaged `pi-tuicr` skill rather than reinventing it; inline agent comments beat a generic AI summary.
~~~~~~~~

### `AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025`

~~~~~~~~markdown
tags:: [[Logseq/Slideshow]]

- # What I learned at the [[AI Engineer Summit 2025 NYC Workshops]]
	- ## What *is* the [[AI Engineer Summit]]?
		- ### How I *found out* about the AI Engineer Summit...
			- #### ⚡ flash back to [[@swyx]]'s [[GraphQL]] presentation at [[Conference/React Boston/2018]] ...
			  `babel-blade`: **solving the GraphQL double declaration Problem**
			  ![image.png](../assets/image_1742292361076_0.png){:height 331, :width 397}
			  [swyx 🌉 on X - link](https://x.com/swyx/status/1046108066037583872)
			- [[Wayfair]] threw a great party. Got to know [[Person/Shawn @swyx Wang]], who at that time was working at [[Netlify]] as a DevRel.
			- ... Time Passes ...
			- Now my favorite AI podcast is ... [[The Latent Space Podcast]]
			- {{embed ((67d943c3-9827-4274-9daa-b2069d73d4b0))}}
		- ### so, *what* is the AI Engineer Summit?
			- ![AI engineer Summit logo](../assets/image_1742369972123_0.png)
			- #### [[AI Engineer Summit NYC 2025]]
			  {{embed ((67d94f45-ee8f-42df-844b-c8a44d9a3cdd))}}
		- ### so, *when* was the AI Engineer Summit?
		  * Thu Feb 20 - **Leadership Track**
		  * Fri Feb 21 - **Engineering Track**
		  * Sat Feb 22 - **Workshops**
		      * *I only attended the workshops this year. Would love to attend the full event in the future*
		- ### *where* was the AI Engineer Summit?
		  *Downtown Manhattan, NYC*
		  ![Pic 02](https://lh3.googleusercontent.com/pw/pic02){:height 309, :width 512}
		  * **Jay Conference Bryant Park** - 109 W 39th St
		  * **AWS** -  JFK27 (12 W 39th St)
	- ## How I *got* to the AI engineer summit
	  *Many thanks to the leadership team and employer for sponsoring my attendance*
	  ![Pic 01](https://lh3.googleusercontent.com/pw/pic01){:height 382, :width 678}
- # Before [[AIES 25 WS 1 - Building Agents with MCP - Mahesh Murag]]
	- ![Pic 01](https://lh3.googleusercontent.com/pw/pic01b){:height 389, :width 747}
	- ## Discussions before the first workshop
		- ### 1/3 [[Person/Junrui (Tom) Hu]]
		  Getting coffee before the first workshop, I met Tom, **AI Lead** at [[HanoverPark]], where he's building **AI portfolio management**, **AI inbox analyzer**, real-time fund metrics, and an investor portal.
		- ### 2/3 - [[Person/Mike Christensen]]
		  ((67da7fc2-1c1f-46e9-84b0-dee824e45395))
- # [[AIES 25 WS 1 - Building Agents with MCP - Mahesh Murag]] - #MCP Talk that went viral
	- ## Context - #MCP #Examples from [[David Ondrej - Build Anything with MCP Agents]]
		- ### #Example 1 - using [[GitHub/MCP]] enable [[CursorAI]] to take actions in GitHub
		  {{video https://www.youtube.com/watch?v=TQsP_PlCY1I&t=205s}}
		- ### #Example 2 - enabling [[CursorAI]] to interact with [[Chrome/DevTools]] with [[AgentDeskAI/GitHub/browser-tools-mcp]]
		  {{video https://www.youtube.com/watch?v=TQsP_PlCY1I&t=806s}}
	- ## What problem does [[Model Context Protocol]] solve?
		- ### What does this mean ... for [[AI Coding]]? AI Agents in Code editors can now ...
		  * develop technical plans using Jira, Confluence
		  * submit merge requests to GitHub, GitLab, etc
		  * check out a ticket and start working on it
		  * introspect into database, etc
~~~~~~~~

### `AI/Batch Inference/Concept/AWS vs OpenAI vs LangChain`

~~~~~~~~markdown
tags:: [[Diataxis/Explanation]]

- # AWS Bedrock vs OpenAI vs LangChain - Batch Inference Conceptual Overview
	- ## Overview
		- Batch inference submits many prompts or embedding requests as one asynchronous job, trading higher latency for lower cost and greater throughput.
	- ## Key Principles
		- Control-plane job APIs separate **submission** from **execution**.
		- External object storage stages **inputs** and **outputs**.
		- Discounted pricing incentivises deferred processing.
	- ## Mechanism
		- ### OpenAI
			- Upload data with `POST /v1/files`, then launch `POST /v1/batches` and inspect with `GET /v1/batches/{id}`. Full reference in the [OpenAI Batch API docs](https://platform.openai.com/docs/api-reference/batches).
		- ### Comparative Flow
			- ```mermaid
			  graph TD
			    A[Client] -->|Upload JSONL| B(S3 / OpenAI Files)
			    A -->|Create Job| C(Bedrock Job API)
			    A -->|Create Job| D(OpenAI /v1/batches)
			    C -->|Write output| B
			    D -->|Write output| B
			  ```
	- ## Examples
		- ### Bedrock CLI
			- ~~~bash
			  aws bedrock create-model-invocation-job \
			    --model-id anthropic.claude-3-sonnet-20240229-v1:0 \
			    --input-data-config file://input.jsonl \
			    --output-data-config s3://my-bucket/results/ \
			    --role-arn arn:aws:iam::123456789012:role/BedrockBatchRole
			  ~~~
		- ### OpenAI Python
			- ~~~python
			  from openai import OpenAI
			  client = OpenAI()
			
			  file = client.files.create(
			      file=open("input.jsonl", "rb"),
			      purpose="batch"
			  )
			
			  batch = client.batches.create(
			      input_file_id=file.id,
			      endpoint="/v1/chat/completions",
			      completion_window="24h"
			  )
			  print(batch.status)  # "queued"
			  ~~~
~~~~~~~~

### `Spotify/Tool/TUI`

~~~~~~~~markdown
tags:: [[TUI]]
alias:: [[Spotify TUI Options]]

- # Spotify [[TUI]] Projects
	- ## [[spotify-player]]
		- {{embed ((6a3f6744-5403-429d-ad4a-4391e01224ea))}}
	- ## [[ncspot]]
		- more minimal than [[spotify-player]] minimalist pick: ncurses-style, resource-friendly, Vim-keyed, and closer to an MPD-client mental model.
	- ## [[spotify-tui]]
		- historical reference: older, once popular Rust Spotify TUI. dependent on an external playback client such as the official Spotify app or spotifyd.
~~~~~~~~

### `spotify-player`

~~~~~~~~markdown
- ## Features
  id:: 6a3f6744-5403-429d-ad4a-4391e01224ea
	- [configurable](https://github.com/aome510/spotify-player/blob/master/docs/config.md)
	- To enable [fuzzy search](https://en.wikipedia.org/wiki/Approximate_string_matching), build with the `fzf` feature (disabled by default).
~~~~~~~~

### `Logseq/Frontmatter`

~~~~~~~~markdown
- # Logseq Frontmatter
	- This page defines shared page-level frontmatter conventions for the knowledge garden.
	- Page-level attributes are bare `key:: value` lines at the top of the file, before any bullet. `---` fences around them are illegal in Logseq; if they appear, unwrap them.
	- {{embed [[Logseq/Frontmatter/alias]]}}
~~~~~~~~

### `Logseq/Frontmatter/alias`

~~~~~~~~markdown
- # `alias::` for more convenient or alternate names
	- In Logseq, if page `[[Super Long Name]]` has frontmatter `alias:: [[Shortname]]`, then clicking `[[Shortname]]` anywhere in the graph navigates to `[[Super Long Name]]`.
	- ## Aliases are human-curated, not agent-generated
		- Agents should suggest alias candidates in the chat response rather than writing them to the page without explicit instruction.
~~~~~~~~

### `Logseq/Slideshow`

~~~~~~~~markdown
tags:: [[Slideshows]]
alias:: [[Logseq Slideshow]], [[Logseq Slideshows]]

-
~~~~~~~~

### `contents`

~~~~~~~~markdown
-
~~~~~~~~

### `2026-10-23 Fri`

~~~~~~~~markdown
- # [[Filed]]
	- [[GitP/A/Session/26/10/22-Thu/Asset/Audio/Trimmed/mp3]]
	- [[Obsidian/Q/Should a team vault in git commit its community plugins' code, or only the list of plugins?]]
	- [[My/Pref/Dev/Tool/Obsidian/Plugins in Git]]
	- [[LangSmith/Deployment/Q/Can a LangSmith Cloud deployment call Amazon Bedrock through an IAM role instead of stored AWS keys?]]
- # [[Updated]]
	- GitP episodes
		- [[GitP/A/Session/26/09/25-Fri]]
		- [[GitP/A/Session/26/10/22-Thu]]
	- [[My/Pref/Dev/mise/Tasks]]
	- [[My/Pref/Writing/Use Plain language]]
~~~~~~~~

### `2026-10-22 Thu`

~~~~~~~~markdown
- # [[Updated]]
	- Music
		- [[GitP/A/Session/26/10/20-Tue]]
		- [[GitP/How To/Prepare Podcast Metadata]]
		- [[Microfreak/Preset/016 Voidom]]
		- [[Logseq/Entity/Preset/Synth/Microfreak]]
- # [[Filed]]
	- [[GitP/A/Session/26/10/22-Thu]]
~~~~~~~~

### `2026-09-15 Tue`

~~~~~~~~markdown
- I was pleasantly surprised to find this morning that clicking on `Opened [PR #101](https://github.com/codekiln/dotfiles/pull/101)` in [[GitHub/CoPilot/CLI]] in [[Ghostty]] in [[tmux]] actually opened the PR in my browser.
- # [[Filed]]
	- [[MacOS/How To/Find What Blocked a Software Update Install]]
~~~~~~~~

### `2026-08-25 Tue`

~~~~~~~~markdown
- # Experiments
	- This morning, tried [[ChatGPT/Remote/Voice]] chat for the first time for reviewing lazyvim flashcards. It worked pretty well; has a lot of promise.
- # I wonder
	- if there is a way that [[Claude/Code]] in [[tmux]] could surface links which, if clicked, would open [[nvim]] in an adjacent pane.
~~~~~~~~

### `Rust/loop/label`

~~~~~~~~markdown
- when loops are nested in rust, it's possible to lablel them. This is useful if you have to break out of one loop but the other.
- The syntax is `'loop_name: loop { ... break 'loop_name}`.
	- [[Confus/ing]]
		- Syntactically, the (`'identifier`) token shape appears elsewhere in [[Rust/Lifetime]]s, but there it serves a different purpose
		- | Context | Meaning |
		  | ---- | ---- | ---- |
		  | `'label:` before a loop | loop label |
		  | `'label` in `break 'label` | labeled control flow |
		  | `'a` in `&'a T` | lifetime parameter |
		  | `'static` | the static lifetime |
		  | `'_'` | anonymous lifetime |
		- So `'thing` is *not* “the same feature” being reused — it’s the same **token class** used in multiple grammar positions.
- [[Example]]
	- ```rust
	  fn loop_labels_to_disambiguate_multiple_loops() {
	      println!("Loop labels to disambiguate multiple loops:");
	      let mut count = 0;
	
	      'counting_up: loop {
	          println!("count = {count}");
	          let mut remaining = 10;
	
	          loop {
	              println!("remaining = {remaining}");
	              if remaining == 9 {
	                  break;
	              }
	              if count == 2 {
	                  break 'counting_up;
	              }
	              remaining -= 1;
	          }
	
	          count += 1;
	      }
	      println!("End count = {count}");
	  }
	  ```
~~~~~~~~

### `Latent Space/Pod`

~~~~~~~~markdown
- # The Latent Space Podcast
	- ## [What is Latent.Space?](https://www.latent.space/about)
	  id:: 67d943c3-9827-4274-9daa-b2069d73d4b0
		- ![The Latent Space Podcast](https://i.scdn.co/image/ab6765630000ba8ad1157ab13e3e7438bf374972){:height 208, :width 192}
		- > A technical **newsletter**, **podcast** ([top 10 in US Tech](https://twitter.com/swyx/status/1672306744884887553)! in audio podcast and YouTube video formats), and community (VERY active Discord online and in person) by and for the rising cl…
~~~~~~~~

### `yazi/Keyshort/Leader`

~~~~~~~~markdown
- [[Keyshort]] [[yazi]] [[yazi/Keyshort]]
	- **`<yazi-sort-leader>` for default sort chords** #card
	  card-last-interval:: 84.1
	  card-repeats:: 5
	  card-ease-factor:: 2.76
	  card-next-schedule:: 2026-09-16T08:24:02.595Z
	  card-last-reviewed:: 2026-06-24T06:24:02.595Z
	  card-last-score:: 5
		- Shortcut: `,` (comma), then a second key (two-key sequence)
		- Context: File list / manager pane (`[mgr]` layer, default keymap)
~~~~~~~~

### `yazi/Keyshort/Sort/Time/Modification`

~~~~~~~~markdown
- [[Keyshort]] [[yazi]] [[yazi/Keyshort]]
	- **Sort by modification time (normal order)** #card
	  card-last-interval:: 90.49
	  card-repeats:: 5
	  card-ease-factor:: 2.9
	  card-next-schedule:: 2026-09-22T18:17:14.100Z
	  card-last-reviewed:: 2026-06-24T07:17:14.100Z
	  card-last-score:: 5
		- Shortcut: `,` `m` (comma, then lowercase `m`)
		- Context: File list / manager pane (default keymap)
		- Description: Runs `sort mtime --reverse=no` and sets **linemode** to `mtime` so each row shows last modified time. Pair with `,` `M` for reversed order.
~~~~~~~~

### `Logseq/Flashcard/Review/Favorite/yazi`

~~~~~~~~markdown
- # Favorite Flashcards: [[yazi]]
	- {{cards [[yazi]] }}
	  favorite-deck:: [[yazi]]
~~~~~~~~

### `Logseq/Flashcard/Review/Favorite`

~~~~~~~~markdown
alias:: [[LFRF]]

- # Favorite Flashcards
	- foregrounds flashcard scopes for my [[Logseq/Favorite]]s ([[Logseq/Queries]]).
	- ## All Favorite Flashcards
		- {{cards (and (or [[LazyVim]] [[Obsidian]] [[Codex]] [[chezmoi]] [[DevContainer]] [[mise]] [[rg]] [[vim]] [[nvim]] [[fzf]] [[Zoxide]] [[yazi]] [[tmux]] [[Lazygit]]) (not [[oh-my-tmux]])) }}
		  favorite-deck:: aggregate
		  deck-exclude:: [[oh-my-tmux]]
			- Summary: 19 items, 19 review counts [[2026-06-26 Fri]]
				- Remembered:   14 (73%)
				- Forgotten :   5 (26%)
	- ## Background / Non-Favorite
	  collapsed:: true
		- {{cards (not (or [[LazyVim]] [[Obsidian]] [[Codex]] [[chezmoi]] [[DevContainer]] [[mise]] [[rg]] [[vim]] [[nvim]] [[fzf]] [[Zoxide]] [[yazi]] [[tmux]] [[Lazygit]])) }}
		  favorite-deck:: background
		  query-properties:: [:block]
	- ## By Favorite
		- [[Logseq/Flashcard/Review/Favorite/yazi]]
		- [[Logseq/Flashcard/Review/Favorite/tmux]]
~~~~~~~~

### `Prototype/Fixture/Headings`

~~~~~~~~markdown
- This page is a prototype fixture, not a page in the garden: one block per Markdown heading level, then body text. h1–h4 were measured on the published site; h5–h6 come from Logseq's block.css.
- # Heading 1 · 32px, weight 300, rule below
- ## Heading 2 · 24px, weight 300, rule below
- ### Heading 3 · 19.2px, weight 600
- #### Heading 4 · 16px, weight 600
- ##### Heading 5 · 0.83em, weight 600
- ###### Heading 6 · 0.75em, weight 600
- Body text · 16px, weight 400, line-height 1.6
~~~~~~~~

### `Prototype/Fixture/Queries`

~~~~~~~~markdown
- This page is a prototype fixture, not a page in the garden. Each block shows one query state; results are computed from the pages in this prototype.
- ## Simple query: blocks grouped by page
	- {{query [[tuicr]]}}
- ## Simple query in table view
	- {{query (and [[Keyshort]] [[yazi]])}}
	  query-table:: true
- ## Page query
	- {{query (page-tags [[Logseq/Slideshow]])}}
- ## Advanced query with a title
	- #+BEGIN_QUERY
	  {:title "Cards about yazi"
	   :query (and [[yazi]] [[card]])}
	  #+END_QUERY
- ## Advanced query, collapsed
	- #+BEGIN_QUERY
	  {:title "Blocks that mention tmux"
	   :query [[tmux]]
	   :collapsed? true}
	  #+END_QUERY
- ## No results
	- {{query (and [[yazi]] [[Rust]])}}
- ## Slow query (auto refresh disabled)
	- {{query (or [[nvim]] [[vim]])}}
	  prototype-slow:: 120
- ## Failed query
	- #+BEGIN_QUERY
	  {:title "Broken query"
	   :query (and [[tmux]]}
	  #+END_QUERY
- ## Flashcards query
	- {{cards [[yazi]]}}
- ## Flashcards query with no matches
	- {{cards [[Rust]]}}
~~~~~~~~

