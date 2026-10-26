# Graph view and page graph

Logseq's published site has two graph views. The global graph at `#/graph` draws every page as a node and every page reference, tag and namespace link as an edge, with a settings panel in the top-right corner. The page graph is a card in the right sidebar that draws the current page with the pages it references, the pages that reference it, and its tags. On the encode garden the global graph has 19,105 nodes and 78,010 edges with Logseq's default settings. Logseq builds both graphs in the browser from the whole database and lays them out with a live force simulation. On the encode garden that takes 38 minutes on a desktop before the first frame, with the JS heap peaking at 2,340 MB.

The garden hides both entry points to the global graph. `logseq/custom.css` sets `display: none` on the header `Graph` link and on the left sidebar `Graph view` item, with the comment "it crashes on large graphs", and `config.edn` turns off the `g g` shortcut (garden commit `f2c2ca3`, 2026-06-28). `#/graph` still opens when typed, and the right sidebar still offers Page graph.

`lsg` has no graph view, no page graph, no right sidebar and no graph data: no header link, no sidebar item and no file that lists edges. The SDK's reference index already holds most of what the edges need. `lsg` should compute the nodes and edges in the SDK, compute the layout at build time, and write one compact graph file that the browser loads only when a reader opens the graph view, plus a small neighbourhood file per page for the page graph. A static build that never has to run Logseq's in-browser work can show the entry points again; whether it does is a decision for the proposal (gap 1).

Measured on 2026-10-26 against the `gh-pages` build of garden commit `0ce74d6` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden at commit `0ce74d6` from `lsg` `1e72969` (`mise run parity:build`). Node and edge counts come from decoding the build's `static/js/db.gz` and applying Logseq's graph rules from the source below; the live build's settings panel confirms the global counts (`19105 pages`, `81394 links`). Times and heap come from a Playwright Chromium run on an Apple silicon Mac with no CPU slowdown; the phone profile was not run, because the desktop run alone took 38 minutes.

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where they differ from the checkout or from `docs/logseq-reference/Graph.md`. The board's 280px panel, rounded border, `Show Journals` row in the global panel and blue hub node are illustrations; the live panel is described below.

### Entry points

- **Header `Graph` link**, shown only when publishing, linking to `#/graph` ([`header.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/header.cljs#L278-L280)). Hidden in the encode garden by `.cp__header a.button[href$="#/graph"] { display: none !important; }`.
- **Left sidebar `Graph view`** with the `hierarchy` icon and the `g g` hint ([`container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L457-L463)). Hidden in the encode garden by `.graph-view-nav { display: none !important; }`. The live sidebar still reads `Graph view gg` in its markup.
- **`g g`** ([`shortcut/config.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/modules/shortcut/config.cljs#L448-L449)), turned off by `:shortcuts {:go/graph-view false}`.
- **Route** `#/graph` ([`routes.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/routes.cljs#L60-L62)).
- **Page graph**: the right sidebar's top row has `Contents`, `Page graph` and `Help` buttons; `Page graph` adds a card with the `hierarchy` icon ([`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L125-L127), [L442-L449](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L442-L449)). The right sidebar opens from the header button or `t r`.

### Which nodes and edges the global graph holds

[`build-global-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L84-L121) builds the graph from three relations:

- **References.** One edge from page A to page B when a block on A has B in its `:block/refs` ([`get-pages-relation`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1178-L1199)). Because Logseq's parser also adds every namespace ancestor of a referenced page to a block's references, Logseq drops the edge from A to a namespace page when A also links one of its children ([`remove-nested-namespaces-link`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1157-L1176)). Property names are references in file graphs, so `up`, `prev`, `next`, `see-also` and `created-by` are among the ten largest nodes. With journals off, references from journal pages are left out.
- **Tags.** One edge from each page to each page in its `tags::`.
- **Namespaces.** One edge from each namespaced page to its parent.

Nodes are all pages, then filtered by the settings:

| Setting (`:graph/settings` key) | Default | Effect |
| --- | --- | --- |
| Journals (`:journal?`) | off | Leaves out journal pages and the references they make. |
| Orphan pages (`:orphan-pages?`) | toggle shows on | Off leaves out pages with no edge. See the note below. |
| Built-in pages (`:builtin-pages?`) | off | Leaves out `NOW`, `LATER`, `TODO`, `DOING`, `DONE`, `WAIT`, `WAITING`, `CANCELED`, `CANCELLED`, `STARTED`, `IN-PROGRESS`, `A`, `B`, `C`, `Favorites`, `Contents`, `card` ([`default.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/db/src/logseq/db/default.cljs#L12-L16)). |
| Excluded pages (`:excluded-pages?`) | toggle shows on | Off leaves out pages with `exclude-from-graph-view:: true`. No encode-garden page sets it. |

Pages whose names look like UUIDs or asset paths are always left out. Reference-only pages (pages that exist only because something links them) are nodes like any other.

**Defaults differ between the panel and the graph.** The garden leaves `:graph/settings` commented out in `logseq/config.edn`. The panel's toggles start from their own values ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L548-L559): orphans on, excluded on), but `build-global-graph` reads the missing settings as false, so it leaves out orphans and excluded pages while the toggles show them as on. Live: the panel shows Orphan pages and Excluded pages on, Journals and Built-in pages off, and `19105 pages`, the count with orphans left out.

Counts on the encode garden (decoded from `db.gz`):

| Settings | Nodes | Edges drawn | `N links` in the panel |
| --- | ---: | ---: | ---: |
| Defaults (journals off, orphans left out) | 19,105 | 78,010 | 81,394 |
| Orphans shown | 19,202 | 78,010 | 81,394 |
| Journals on | 19,732 | 85,691 | 88,691 |

Of the 19,105 default nodes, 8,543 are pages with content and 10,562 are reference-only pages. The panel's link count includes duplicates (a pair that is both a reference and a tag counts twice) and edges with one end outside the node set; only distinct edges with both ends present are drawn ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L803-L808), [`pixi.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph/pixi.cljs#L207-L215)).

### How a node looks

[`build-nodes`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L17-L44) and [`default-style`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph/pixi.cljs#L30-L56):

- **Size**: `8 × max(1, ∛n)` where n counts the edges that touch the page. `Person` (1,025 edges) is 80; a page with one edge is 8.
- **Colour**: `#999` (dark `#93a1a1`); the current page `#045591` (dark `#ffffff`); a page used as a tag green (dark orange). A page that is a namespace parent takes one of ten colours (`#1f77b4`, `#ff7f0e`, `#2ca02c`, `#d62728`, `#9467bd`, `#8c564b`, `#e377c2`, `#7f7f7f`, `#bcbd22`, `#17becf`) chosen by a hash of its name.
- **Label**: the page's original name, 12px, `rgba(0,0,0,0.8)` (dark `rgba(255,255,255,0.8)`), padding 4px.
- **Edge**: 1px `#cccccc` (dark: the accent colour or `#094b5a`).
- **Hover**: node in the accent colour (`#6366F1` when none is set), label on `rgba(238,238,238,1)` in `#333333`, edges `#A5B4FC`.

### Layout and interaction

- **Layout** is a d3-force simulation that runs in the page and keeps moving after the first frame: link distance 70, many-body charge −600 with range 1–600 and theta 0.5, collision radius 26, x and y pull 0.02 towards the centre, velocity decay 0.5 ([`layout!`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph/pixi.cljs#L58-L99)). The renderer is `pixi-graph-fork` on WebGL; it pans and zooms with the mouse.
- **Canvas** fills the main column: `calc(100vh - 100px)` high ([`graph.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph.css#L12-L18)).
- **Click** a node to open its page (through its alias redirect). **Shift-click** or **drag** a node to select it: Logseq marks it, colours its neighbours and edges, and shows the `N hops from selected nodes` slider (1–10), which keeps only nodes within N hops of the selection ([`graph.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph.cljs#L10-L47), [`n-hops`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L217-L237)). Dragging a node reheats the simulation for two seconds.

### Settings panel

[`graph-filters`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L576-L768) renders `div.graph-filters` at `top: 16px; right: 16px`, 200px wide on `--ls-secondary-background-color` with a large shadow and small radius ([`page.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.css#L209-L221)). It holds four foldable sections, all closed at first; each header is a full-width button with the title in medium weight and a caret. Opening a section expands it over 700ms to at most 400px.

- 1. **Nodes**: `N pages` (the node count), then the toggles `Journals`, `Orphan pages`, `Built-in pages`, `Excluded pages`; `N hops from selected nodes` while a node is selected; and a `Reset Graph` link that clears the selection, N hops and search filters.
- 2. **Search**: `Click to search` opens the search dialog in graph mode. Choosing a result adds the typed text as a filter instead of opening a page ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L475-L490)). With filters set, the section opens by itself and lists them, each with a close button, and `Clear All`. A filter keeps nodes whose name contains the text, ignoring case ([`filter-graph-nodes`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L825-L830)).
- 3. **Forces**: `N links`, a `Pause simulation` toggle, sliders `Link Distance` (10–180 in steps of 10, default 70), `Charge Strength` (−1000 to 1000 in steps of 100, default −600) and `Charge Range` (500–4000 in steps of 100, default 600), each showing its value in a tooltip, and `Reset Forces`.
- 4. **Export**: `as PNG` saves the canvas.

Toggles and sliders call `set-config!` to write `:graph/settings` and `:graph/forcesettings`; a published site has no file to write, so changes last until reload. The garden's `config.edn` comment lists a default link distance of 180; the code uses 70.

Live, with the garden's theme: the panel is 200px wide on `rgb(248,249,250)` with an 8px radius and a hard `4px 4px 0` black shadow from the dev theme; the switches use the indigo accent. With every section open it is taller than the window and its top scrolls out of view. The canvas fills 1280×752 at 1280×800. Every node draws its label, so at the default zoom the labels form a grey mass.

### Page graph

[`page-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L852-L889) and [`build-page-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L123-L177):

- **Which page.** The page in the current route, re-read whenever the route changes, so the card follows navigation. On any other route (such as the journals home) it uses today's journal and shows nothing when that page does not exist. On a zoomed block route it draws the block's reference graph instead ([`build-block-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L179-L215)).
- **Nodes.** The page, the pages it references (through its aliases), the pages that reference it, and its `tags::`. Referring journal pages appear only when the card's `Show Journals` toggle is on (off by default, reset on reload).
- **Edges.** Page to each referenced page and tag, each referring page to the page, every reference between two of the other nodes, and the namespace edges among them.
- **Look.** The same node and edge style, labels on every node and click behaviour as the global graph, without the settings panel or forces controls. The card body is the `Show Journals` row with a switch at the right, then the graph. The code asks for 600×600, but `.graph { height: 500px }` and the card width clip it: live, the canvas is 448×500 in a 512px sidebar at 1280px. On `tmux` the 224 labels overlap into an unreadable block.

Counts on the encode garden, with Show Journals off and on:

| Page | Nodes | Edges | With journals |
| --- | ---: | ---: | --- |
| `Contrando` | 2 | 1 | 3 nodes, 2 edges |
| `Foam` | 17 | 51 | 18, 58 |
| `2026-10-23 Fri` | 48 | 228 | 48, 228 |
| `mise` | 197 | 947 | 242, 1,256 |
| `tmux` | 224 | 1,305 | 274, 1,714 |
| `Person` | 1,431 | 5,367 | 1,673, 7,384 |
| `Microfreak` | 951 | 29,354 | 962, 29,481 |
| `Logseq` | 4,964 | 60,392 | 5,068, 62,955 |

The live build opens the right sidebar with a Contents card; clicking `Page graph` adds the graph card above it.

### Performance in Logseq

Logseq downloads the whole database before drawing anything (`static/js/db.gz`, 19.8 MB, 223 MB inflated in this build) and builds the graph from it in the browser. The step that drops nested namespace edges compares every reference pair with every other one: about 125,000 pairs, so about 1.6 × 10¹⁰ comparisons with journals off. Live on the desktop, the main thread stayed busy for 38 minutes (2,291 s from opening `#/graph` to the first canvas, after 17.6 s to load the app); the JS heap rose from 1,741 MB to a peak of 2,340 MB, then fell to about 800 MB while the layout started. The simulation then keeps the main thread busy as it moves 19,105 nodes each tick. The page graph runs one database query per neighbour, so its cost grows with the neighbourhood. Time from clicking `Page graph` to the canvas, after the page itself took 21–30 seconds to appear: `Foam` 19.7 s, `tmux` 47.9 s, `Person` 205.8 s; the JS heap peaked at 2,019, 2,289 and 2,408 MB. These ran beside the global graph measurement in a second browser, so they may be slightly slow.

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/shell.rs`, `references.rs` and the whole-garden build:

- **Entry points.** The header holds Toggle left sidebar, Search, Home and More; the left sidebar holds Journals, All pages and the favorites. No Graph link, no Graph view item, no `g g`, no right sidebar and no `t r`. The [published app shell spec](../../openspec/specs/published-app-shell/spec.md) requires that no header control links to a missing file, so the Graph link waits for this issue.
- **Graph output.** None. No file in the build lists pages with their edges or positions.
- **What the SDK already knows.** `crates/logseq-gardener-sdk/src/references.rs` builds a `ReferenceIndex` shared by the CLI, the language server and the publisher. It holds each block's references with namespace ancestors added, each page's `tags::`, each page's namespace ancestors, and the referring blocks of every page. Page-to-page edges, tag edges and namespace edges can be derived from it, but no operation returns them, and the index does not count property names as references.
- **Which pages exist.** The build publishes 10,756 pages. Logseq's database holds 19,911, of which 7,540 are reference-only pages that `lsg` does not publish ([Issue #111](https://github.com/codekiln/logseq-gardener/issues/111), [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115)). Private and unselected pages are not published and must not appear in any graph file.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Entry points.** The header `Graph` link (publishing only, between Home and More), the left sidebar `Graph view` item with the `hierarchy` icon between Flashcards and All pages, and `g g` open the graph view, a `graph.html` page. `g g` stays off when `config.edn` sets `:shortcuts {:go/graph-view false}`, as the encode garden does. The garden's custom.css hides Logseq's entry points because Logseq's graph crashes; the proposal decides whether `lsg` shows them anyway, and if it keeps the garden's choice, gives its elements the same classes (`graph-view-nav`, and a header link whose `href` the garden's selector matches) so `--stylesheet logseq/custom.css` hides them. Check: on `tmux` the header and sidebar hold the Graph controls in Logseq's order and both reach `graph.html`; with the garden's custom.css loaded, the decision's expected state holds.
- 2. **Node set.** Pages, minus journals, built-in pages, pages with `exclude-from-graph-view:: true`, UUID and asset names, and (per toggle) orphans, as Logseq does. The proposal decides how to treat Logseq's reference-only pages, which are more than half of Logseq's nodes and have no `lsg` page: as nodes that open nothing, or left out. Private and unselected pages never appear. Check: the default node count is 19,105 with reference-only pages or 8,543 without, and a fixture page marked `exclude-from-graph-view:: true` is absent.
- 3. **Edges.** Reference edges with the nested-namespace rule, tag edges and namespace edges; edges with an end outside the node set are not drawn. Property-name pages (`up`, `prev`, `next`) are nodes only if the SDK counts property names as references, which the proposal decides with [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115). Check: 78,010 drawn edges and `N links` 81,394 with reference-only pages (39,167 among pages with content), and `tmux`'s degree matches Logseq's.
- 4. **Node and edge look.** Size `8 × max(1, ∛n)`, the colour rules (default, current page, tag, ten namespace colours by name hash), 12px labels, 1px edges, hover colours, and the dark-theme values. Check: computed sizes and colours of `Person`, `tmux` and a namespace parent such as `Logseq/Entity` in light and dark themes.
- 5. **Canvas and interaction.** The graph fills the main column (`100vh − 100px`); drag pans, wheel and pinch zoom; click opens the page; shift-click or drag selects a node and highlights its neighbours and edges; with a selection, `N hops from selected nodes` (1–10) keeps nodes within N hops. All of this also works from the keyboard and with touch (a node list or focusable search result as the keyboard path to a node). Check: clicking `tmux` opens its page; shift-click on `tmux` then N hops 1 keeps `tmux`'s neighbours.
- 6. **Settings panel.** `div.graph-filters` top-right with Nodes, Search, Forces and Export, closed at first, with Logseq's labels, order, counts (`N pages`, `N links`), defaults and the `Reset Graph`, `Clear All` and `Reset Forces` links. Defaults come from `:graph/settings` and `:graph/forcesettings` in `config.edn`; changes last for the visit. The proposal decides whether the orphan toggle matches Logseq's visible state or its behaviour (see "Defaults differ between the panel and the graph"). Check: the panel text on the encode garden matches the live build's, and turning on Journals changes `N pages` to the journals-on count.
- 7. **Search filter.** `Click to search` lets the reader add text filters that keep nodes whose title contains the text, ignoring case, with each filter removable and `Clear All`. It uses the static search from [Issue #118](https://github.com/codekiln/logseq-gardener/issues/118) for suggestions, or a plain text field when search is unavailable. Check: filter `tmux` leaves only nodes whose titles contain `tmux`.
- 8. **Forces.** `Pause simulation` and the three sliders change the layout; `Reset Forces` restores the defaults. The first frame uses the layout computed at build time with Logseq's default forces; the simulation runs in the browser only after a reader changes a force or drags a node, off the main thread, and stops when it settles. Check: changing Link Distance moves nodes while the page keeps answering input within 100ms.
- 9. **Export as PNG.** Saves the drawn canvas. Check: the file opens and shows the current view.
- 10. **Page graph.** A `Page graph` card in the right sidebar with the `hierarchy` icon, the `Show Journals` toggle, the current page's neighbourhood with Logseq's node and edge rules, 500px high at the card's width, following navigation; empty on the journals home unless today's journal exists; the block reference graph on a zoomed block. Depends on [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119) for the sidebar, its top-row buttons, `t r` and its cards. Check: `Foam` shows 17 nodes and 51 edges, `tmux` 224 and 1,305, and turning on Show Journals on `tmux` shows 274 nodes.
- 11. **Performance.** The graph view, the page graph and the entry points meet the budgets below. Check: `parity:budgets` passes and `parity:perf` reports the graph rows within budget.

## Performance and budgets

Parity here is Logseq's look and behaviour, not its architecture. Logseq's graph view is slow and, by the garden author's account, crashes, because the browser rebuilds a 19,000-node graph from the whole database on every visit and lays it out from scratch.

### What lsg should do instead

- **Compute the graph at build time.** The SDK returns nodes (id, title, flags for journal, tag, namespace parent, built-in, reference-only) and edges (reference, tag, namespace) from the same index that answers references and hierarchy, updated incrementally when a file changes.
- **Compute the layout at build time.** Run a force layout with Logseq's default forces (link distance 70, charge −600, range 600, collision 26, centring 0.02) in Rust with a fixed seed, so rebuilds give the same picture, and cache it by the graph's content so an unchanged graph costs nothing. A Barnes–Hut layout of about 19,000 nodes and 78,000 edges should take seconds, not minutes; the budget below bounds it.
- **Ship one compact graph file, loaded only by `graph.html`.** Node titles once, then node flags and degree, quantised positions and edges as sorted index pairs, in a binary or delta-coded JSON form. Estimates for the default view: titles 142 KB gzip, edges about 200 KB gzip as 32-bit pairs (less with delta coding), positions 76 KB before compression as two 16-bit numbers per node. No other page loads it, and the header link adds only markup within the `[shell]` budget.
- **One file per journals setting.** Journals on adds 627 nodes and about 7,700 edges; ship it as a second file, or as an addition that the browser merges, loaded only when a reader turns on Journals.
- **Per-page neighbourhood files for the page graph.** The SDK computes each page's neighbourhood with and without journals; the publisher writes it beside the page's reference fragments and the page graph card fetches it when opened. A few hubs have huge neighbourhoods (`Logseq` 4,964 nodes and 60,392 edges, `Microfreak` 29,354 edges); above a size limit the card loads the global graph file and filters it in the browser instead of shipping a large per-page file.
- **A renderer that stays responsive on a phone.** Draw with WebGL (Canvas 2D as fallback) from typed arrays, not one object per node. Draw labels only for nodes above a size or zoom threshold, as a reader cannot read 19,000 labels at once. Hit-test with a spatial grid. Load no third-party graph or physics library on first frame; the force simulation for gap 8 loads in a worker only when used.
- **If a phone cannot stay within budget at 19,000 nodes,** the documented reduction is to start with reference-only pages left out (8,543 nodes, 39,167 edges) and let the reader add them, the same way Logseq starts with journals off. The proposal records which it chooses.

### Proposed budgets

New `[graph]` rows in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml), measured by `parity:perf` and recorded in [Parity performance](performance.md). KB is 1,000 bytes, as in that file.

| Budget | Value | Why |
| --- | ---: | --- |
| `data_gzip_kb` | 600 | Graph file for the default view on the encode garden; the estimate above is about 420 KB. |
| `script_kb` | 30 | Renderer and settings panel, loaded only by `graph.html`; the force worker is separate and loads on use. |
| `first_frame_ms_desktop` | 1,000 | From navigation to `graph.html` to the first frame with every node drawn. |
| `first_frame_ms_phone` | 3,000 | Same, on the Lighthouse mobile profile. |
| `heap_mb` | 50 | Peak JS heap on the phone profile after the first frame and a pan and zoom; Logseq's page loads already peak near 2,000 MB. |
| `frame_ms_phone` | 50 | 95th-percentile frame time while panning on the phone profile (about 20 frames per second). |
| `page_graph_fragment_gzip_kb` | 100 | Largest per-page neighbourhood file, matching `[references] fragment_gzip_kb`; larger neighbourhoods use the global file. |
| `page_graph_open_ms_desktop` / `_phone` | 300 / 1,500 | From clicking Page graph to the first frame, matching `[references]` open times. |
| `build_slowdown_pct` | 25 | Whole-garden build with graph data and layout against the `--baseline-lsg` binary. |

Every other page keeps its `[desktop]` and `[phone]` first-paint sizes: the graph adds a header link and a sidebar item, nothing else.

## Where the work belongs

- **SDK (`garden-core`):** a graph operation that the CLI, the language server and the publisher share. Given options (journals, orphans, built-in pages, excluded pages, reference-only pages), it returns nodes with their flags and degrees and typed edges, applying the nested-namespace rule. A neighbourhood operation returns a page's page graph (through aliases, with or without journals) and a block's reference graph. Both derive from `ReferenceIndex` and update with it; the CLI could print a page's neighbours from the same answer.
- **Publisher:** `graph.html`, the header link and sidebar item, the layout computed from the SDK's graph and its cache, writing the graph and neighbourhood files, the renderer, the settings panel, the force worker, PNG export, and the Page graph card inside the right sidebar from #119.

## Fixture pages

| Page | Why | Logseq page graph | lsg today |
| --- | --- | --- | --- |
| Global graph (`#/graph`) | Whole-garden graph and settings panel | 19,105 nodes, 78,010 edges | none |
| `Contrando` | Smallest neighbourhood among the existing fixtures | 2 nodes, 1 edge | none |
| `Foam` | Small neighbourhood with edges between neighbours | 17 nodes, 51 edges | none |
| `2026-10-23 Fri` | Journal page; Show Journals changes nothing | 48 nodes, 228 edges | none |
| `tmux` | Medium neighbourhood; journals add 50 nodes; the board's example hub | 224 nodes, 1,305 edges | none |
| `mise` | Medium neighbourhood with tagged pages | 197 nodes, 947 edges | none |
| `Person` | Large hub, largest global node among real pages | 1,431 nodes, 5,367 edges | none |
| `Logseq` | Very large neighbourhood, tests the per-page file limit | 4,964 nodes, 60,392 edges | none |
| `Microfreak` | Dense neighbourhood: few nodes, many edges | 951 nodes, 29,354 edges | none |
| Home (`#/`) | No page in the route; today's journal or nothing | depends on the date | none |

## Out of scope

- The right sidebar itself (opening, closing, `t r`, cards, Contents and Help), which belongs to [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119). The page graph card depends on it.
- Publishing reference-only pages as pages, which belongs with [Issue #111](https://github.com/codekiln/logseq-gardener/issues/111) and [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115); gap 2 only decides whether they are graph nodes.
- The search dialog that `Click to search` opens, which belongs to [Issue #118](https://github.com/codekiln/logseq-gardener/issues/118).
- Loading the garden's `custom.css` by default, recorded as a separate gap on #106; gap 1 only decides how the graph's entry points react to it.
- Whiteboards and the Electron app's graph features that the published site does not show.
