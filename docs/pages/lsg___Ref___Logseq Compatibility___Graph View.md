tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/Ref/Logseq Published Site/Graph View]], [[lsg/Ref/Site Navigation]], [[lsg/Ref/Logseq Compatibility/Right Sidebar]]

- # Graph View Compatibility
	- `lsg` publishes Logseq's global graph at `graph.html` and its Page graph card in the right sidebar, with Logseq's node rules, sizes, colours, labels and hover, its entry points and their classes, and its settings panel with Nodes, Search, Forces and Export. It computes the graph and its layout when the site is built, so the global graph draws in under a second on desktop, where Logseq's took 38 minutes on the encode garden. The Orphan pages switch starts off to match what is drawn, labels show only for nodes large enough to read, reference-only nodes open nothing, Click to search filters inside the panel, settings last for the tab's session, and a zoomed block shows its page's graph.
	  id:: 8ad7a1b4-2fae-4870-af8d-c266233cf883
	- Measured on 2026-10-26 against the `gh-pages` build of garden commit `0ce74d6` (served locally, light theme, 1280×800) and an `lsg` build of the whole garden at commit `0ce74d6` from `lsg` `1e72969` (`mise run parity:build`). Node and edge counts come from decoding the build's `static/js/db.gz` and applying Logseq's graph rules from the source below; the live build's settings panel confirms the global counts (`19105 pages`, `81394 links`). Times and heap come from a Playwright Chromium run on an Apple silicon Mac with no CPU slowdown; the phone profile was not run, because the desktop run alone took 38 minutes. The OpenSpec change [match-logseq-graph-view](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-28-match-logseq-graph-view) then built the graph view and the Page graph card, [show-graph-loading](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-31-show-graph-loading) added their loading states, and [published-graph-view](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-graph-view/spec.md), [published-page-graph](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-page-graph/spec.md) and [garden-graph](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/garden-graph/spec.md) hold their requirements.
	- ## What Logseq's published site shows
		- Logseq's published site has two graph views. The global graph at `#/graph` draws every page as a node and every page reference, tag and namespace link as an edge, with a settings panel in the top-right corner. The page graph is a card in the right sidebar that draws the current page with the pages it references, the pages that reference it, and its tags. On the encode garden the global graph has 19,105 nodes and 78,010 edges with Logseq's default settings. Logseq builds both graphs in the browser from the whole database and lays them out with a live force simulation. On the encode garden that takes 38 minutes on a desktop before the first frame, with the JS heap peaking at 2,340 MB.
		- The garden hides both entry points to the global graph. `logseq/custom.css` sets `display: none` on the header `Graph` link and on the left sidebar `Graph view` item, with the comment "it crashes on large graphs", and `config.edn` turns off the `g g` shortcut (garden commit `f2c2ca3`, 2026-06-28). `#/graph` still opens when typed, and the right sidebar still offers Page graph.
		- Source links point at the local `logseq/og` checkout, `6e7afa8eb` (`frontend` = `src/main/frontend`). Measured live values come first and win where they differ from the checkout or from [[lsg/Ref/Logseq Published Site/Graph View]]. The board's 280px panel, rounded border, `Show Journals` row in the global panel and blue hub node are illustrations; the live panel is described below.
		- ### Entry points
			- **Header `Graph` link**, shown only when publishing, linking to `#/graph` ([`header.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/header.cljs#L278-L280)). Hidden in the encode garden by `.cp__header a.button[href$="#/graph"] { display: none !important; }`.
			- **Left sidebar `Graph view`** with the `hierarchy` icon and the `g g` hint ([`container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L457-L463)). Hidden in the encode garden by `.graph-view-nav { display: none !important; }`. The live sidebar still reads `Graph view gg` in its markup.
			- **`g g`** ([`shortcut/config.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/modules/shortcut/config.cljs#L448-L449)), turned off by `:shortcuts {:go/graph-view false}`.
			- **Route** `#/graph` ([`routes.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/routes.cljs#L60-L62)).
			- **Page graph**: the right sidebar's top row has `Contents`, `Page graph` and `Help` buttons; `Page graph` adds a card with the `hierarchy` icon ([`right_sidebar.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L125-L127), [L442-L449](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/right_sidebar.cljs#L442-L449)). The right sidebar opens from the header button or `t r`.
		- ### Which nodes and edges the global graph holds
			- [`build-global-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L84-L121) builds the graph from three relations:
				- **References.** One edge from page A to page B when a block on A has B in its `:block/refs` ([`get-pages-relation`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1178-L1199)). Because Logseq's parser also adds every namespace ancestor of a referenced page to a block's references, Logseq drops the edge from A to a namespace page when A also links one of its children ([`remove-nested-namespaces-link`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1157-L1176)). Property names are references in file graphs, so `up`, `prev`, `next`, `see-also` and `created-by` are among the ten largest nodes. With journals off, references from journal pages are left out.
				- **Tags.** One edge from each page to each page in its `tags::`.
				- **Namespaces.** One edge from each namespaced page to its parent.
			- Nodes are all pages, then filtered by the settings:
			- | Setting (`:graph/settings` key) | Default | Effect |
			  | --- | --- | --- |
			  | Journals (`:journal?`) | off | Leaves out journal pages and the references they make. |
			  | Orphan pages (`:orphan-pages?`) | toggle shows on | Off leaves out pages with no edge. See the note below. |
			  | Built-in pages (`:builtin-pages?`) | off | Leaves out `NOW`, `LATER`, `TODO`, `DOING`, `DONE`, `WAIT`, `WAITING`, `CANCELED`, `CANCELLED`, `STARTED`, `IN-PROGRESS`, `A`, `B`, `C`, `Favorites`, `Contents`, `card` ([`default.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/db/src/logseq/db/default.cljs#L12-L16)). |
			  | Excluded pages (`:excluded-pages?`) | toggle shows on | Off leaves out pages with `exclude-from-graph-view:: true`. No encode-garden page sets it. |
			- Pages whose names look like UUIDs or asset paths are always left out. Reference-only pages (pages that exist only because something links them) are nodes like any other.
			- **Defaults differ between the panel and the graph.** The garden leaves `:graph/settings` commented out in `logseq/config.edn`. The panel's toggles start from their own values ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L548-L559): orphans on, excluded on), but `build-global-graph` reads the missing settings as false, so it leaves out orphans and excluded pages while the toggles show them as on. Live: the panel shows Orphan pages and Excluded pages on, Journals and Built-in pages off, and `19105 pages`, the count with orphans left out.
			- Counts on the encode garden (decoded from `db.gz`):
			- | Settings | Nodes | Edges drawn | `N links` in the panel |
			  | --- | ---: | ---: | ---: |
			  | Defaults (journals off, orphans left out) | 19,105 | 78,010 | 81,394 |
			  | Orphans shown | 19,202 | 78,010 | 81,394 |
			  | Journals on | 19,732 | 85,691 | 88,691 |
			- Of the 19,105 default nodes, 8,543 are pages with content and 10,562 are reference-only pages. The panel's link count includes duplicates (a pair that is both a reference and a tag counts twice) and edges with one end outside the node set; only distinct edges with both ends present are drawn ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L803-L808), [`pixi.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph/pixi.cljs#L207-L215)).
		- ### How a node looks
			- [`build-nodes`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L17-L44) and [`default-style`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph/pixi.cljs#L30-L56):
			- **Size**: `8 × max(1, ∛n)` where n counts the edges that touch the page. `Person` (1,025 edges) is 80; a page with one edge is 8.
			- **Colour**: `#999` (dark `#93a1a1`); the current page `#045591` (dark `#ffffff`); a page used as a tag green (dark orange). A page that is a namespace parent takes one of ten colours (`#1f77b4`, `#ff7f0e`, `#2ca02c`, `#d62728`, `#9467bd`, `#8c564b`, `#e377c2`, `#7f7f7f`, `#bcbd22`, `#17becf`) chosen by a hash of its name.
			- **Label**: the page's original name, 12px, `rgba(0,0,0,0.8)` (dark `rgba(255,255,255,0.8)`), padding 4px.
			- **Edge**: 1px `#cccccc` (dark: the accent colour or `#094b5a`).
			- **Hover**: node in the accent colour (`#6366F1` when none is set), label on `rgba(238,238,238,1)` in `#333333`, edges `#A5B4FC`.
		- ### Layout and interaction
			- **Layout** is a d3-force simulation that runs in the page and keeps moving after the first frame: link distance 70, many-body charge −600 with range 1–600 and theta 0.5, collision radius 26, x and y pull 0.02 towards the centre, velocity decay 0.5 ([`layout!`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph/pixi.cljs#L58-L99)). The renderer is `pixi-graph-fork` on WebGL; it pans and zooms with the mouse.
			- **Canvas** fills the main column: `calc(100vh - 100px)` high ([`graph.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph.css#L12-L18)).
			- **Click** a node to open its page (through its alias redirect). **Shift-click** or **drag** a node to select it: Logseq marks it, colours its neighbours and edges, and shows the `N hops from selected nodes` slider (1–10), which keeps only nodes within N hops of the selection ([`graph.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/graph.cljs#L10-L47), [`n-hops`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L217-L237)). Dragging a node reheats the simulation for two seconds.
		- ### Settings panel
			- [`graph-filters`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L576-L768) renders `div.graph-filters` at `top: 16px; right: 16px`, 200px wide on `--ls-secondary-background-color` with a large shadow and small radius ([`page.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.css#L209-L221)). It holds four foldable sections, all closed at first; each header is a full-width button with the title in medium weight and a caret. Opening a section expands it over 700ms to at most 400px.
			- **Nodes**: `N pages` (the node count), then the toggles `Journals`, `Orphan pages`, `Built-in pages`, `Excluded pages`; `N hops from selected nodes` while a node is selected; and a `Reset Graph` link that clears the selection, N hops and search filters.
			- **Search**: `Click to search` opens the search dialog in graph mode. Choosing a result adds the typed text as a filter instead of opening a page ([`cmdk.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/cmdk.cljs#L475-L490)). With filters set, the section opens by itself and lists them, each with a close button, and `Clear All`. A filter keeps nodes whose name contains the text, ignoring case ([`filter-graph-nodes`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L825-L830)).
			- **Forces**: `N links`, a `Pause simulation` toggle, sliders `Link Distance` (10–180 in steps of 10, default 70), `Charge Strength` (−1000 to 1000 in steps of 100, default −600) and `Charge Range` (500–4000 in steps of 100, default 600), each showing its value in a tooltip, and `Reset Forces`.
			- **Export**: `as PNG` saves the canvas.
			- Toggles and sliders call `set-config!` to write `:graph/settings` and `:graph/forcesettings`; a published site has no file to write, so changes last until reload. The garden's `config.edn` comment lists a default link distance of 180; the code uses 70.
			- Live, with the garden's theme: the panel is 200px wide on `rgb(248,249,250)` with an 8px radius and a hard `4px 4px 0` black shadow from the dev theme; the switches use the indigo accent. With every section open it is taller than the window and its top scrolls out of view. The canvas fills 1280×752 at 1280×800. Every node draws its label, so at the default zoom the labels form a grey mass.
		- ### Page graph
			- [`page-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L852-L889) and [`build-page-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L123-L177):
			- **Which page.** The page in the current route, re-read whenever the route changes, so the card follows navigation. On any other route (such as the journals home) it uses today's journal and shows nothing when that page does not exist. On a zoomed block route it draws the block's reference graph instead ([`build-block-graph`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/graph.cljs#L179-L215)).
			- **Nodes.** The page, the pages it references (through its aliases), the pages that reference it, and its `tags::`. Referring journal pages appear only when the card's `Show Journals` toggle is on (off by default, reset on reload).
			- **Edges.** Page to each referenced page and tag, each referring page to the page, every reference between two of the other nodes, and the namespace edges among them.
			- **Look.** The same node and edge style, labels on every node and click behaviour as the global graph, without the settings panel or forces controls. The card body is the `Show Journals` row with a switch at the right, then the graph. The code asks for 600×600, but `.graph { height: 500px }` and the card width clip it: live, the canvas is 448×500 in a 512px sidebar at 1280px. On `tmux` the 224 labels overlap into an unreadable block.
			- Counts on the encode garden, with Show Journals off and on:
			- | Page | Nodes | Edges | With journals |
			  | --- | ---: | ---: | --- |
			  | `Contrando` | 2 | 1 | 3 nodes, 2 edges |
			  | `Foam` | 17 | 51 | 18, 58 |
			  | `2026-10-23 Fri` | 48 | 228 | 48, 228 |
			  | `mise` | 197 | 947 | 242, 1,256 |
			  | `tmux` | 224 | 1,305 | 274, 1,714 |
			  | `Person` | 1,431 | 5,367 | 1,673, 7,384 |
			  | `Microfreak` | 951 | 29,354 | 962, 29,481 |
			  | `Logseq` | 4,964 | 60,392 | 5,068, 62,955 |
			- The live build opens the right sidebar with a Contents card; clicking `Page graph` adds the graph card above it.
		- ### Performance in Logseq
			- Logseq downloads the whole database before drawing anything (`static/js/db.gz`, 19.8 MB, 223 MB inflated in this build) and builds the graph from it in the browser. The step that drops nested namespace edges compares every reference pair with every other one: about 125,000 pairs, so about 1.6 × 10¹⁰ comparisons with journals off.
			- Live on the desktop, the main thread stayed busy for 38 minutes (2,291 s from opening `#/graph` to the first canvas, after 17.6 s to load the app); the JS heap rose from 1,741 MB to a peak of 2,340 MB, then fell to about 800 MB while the layout started. The simulation then keeps the main thread busy as it moves 19,105 nodes each tick.
			- The page graph runs one database query per neighbour, so its cost grows with the neighbourhood. Time from clicking `Page graph` to the canvas, after the page itself took 21–30 seconds to appear: `Foam` 19.7 s, `tmux` 47.9 s, `Person` 205.8 s; the JS heap peaked at 2,019, 2,289 and 2,408 MB. These ran beside the global graph measurement in a second browser, so they may be slightly slow.
	- ## What lsg publishes
		- [[lsg/Ref/Site Navigation]] describes the graph view and the Page graph card for readers.
		- ### Entry points
			- The header's Graph link is an `a.button` linking to `graph.html#/graph`, between Home and More, inside a header with the class `cp__header`. The left sidebar's Graph view item, between Flashcards and All pages, links to `graph.html`, carries the `hierarchy` icon and the class `graph-view-nav`, and shows the `g g` hint when that shortcut is on.
			- `g g` opens `graph.html` unless `logseq/config.edn` sets `:shortcuts {:go/graph-view false}`, as the encode garden does.
			- Because the link and the item carry Logseq's classes and `href` shape, the encode garden's `custom.css` rules hide both when a site is built with `--stylesheet logseq/custom.css`, and `graph.html` still opens when typed. Without that stylesheet both show.
		- ### Nodes and edges
			- The SDK's `graph` operation returns the graph from the same reference index that answers linked references, so the CLI, the language server and the publisher share one answer. It starts from the published pages only: a private or unselected page, and anything only such pages refer to, never appears.
			- Edges are distinct page pairs from references, `tags::` and namespace parents. The SDK drops the edge to a namespace ancestor when the page links one of its children and not the ancestor itself, and counts property names as references in the graph only, so `up`, `prev`, `next` and `see-also` are large nodes, as in Logseq; linked references leave property names out.
			- `:graph/settings` in `logseq/config.edn` sets the defaults; without it the graph leaves out journals, built-in pages, pages with `exclude-from-graph-view:: true` and orphan pages, as the graph Logseq draws does.
			- Reference-only pages are nodes without a route.
		- ### Look and interaction
			- Each node is a circle of size `8 × max(1, ∛n)`, coloured `#999` (dark `#93a1a1`), green for a page used as a tag (dark orange), and, for a namespace parent, one of Logseq's ten colours chosen by Logseq's hash of its name: `Logseq/Entity` is `#7f7f7f` in both themes. Labels are 12px, edges 1px `#cccccc` (dark the accent colour or `#094b5a`), and hover uses Logseq's colours.
			- Labels show for the hovered and selected nodes and for every node drawn at 12px or larger at the current zoom, so zooming in shows every label. `t t` recolours the drawing without reloading the data.
			- `graph.html` sits inside the normal page shell, with the canvas filling the main column at `calc(100vh - 100px)` high (700px at 1280×800, 744px at 390×844).
			- Dragging the background pans; the wheel and pinch zoom about the pointer. Clicking a node opens its page; clicking a reference-only node shows its title as `not published`, such as `prev — not published`, and opens nothing. Shift-click or dragging a node selects it, colours its neighbours and their edges, and shows `N hops from selected nodes` (1 to 10).
			- The canvas is focusable: arrow keys pan and `+` and `-` zoom, and the Search section's list of matching nodes is the keyboard path to opening and selecting a node. Touch pans, pinch-zooms, opens with a tap and selects with a long press.
			- Without JavaScript the page says the graph needs JavaScript and links to All pages.
		- ### Settings panel
			- `div.graph-filters` sits 16px from the top and right of the canvas, 200px wide, with Nodes, Search, Forces and Export in Logseq's order, all closed at first, each header a full-width button with `aria-expanded`.
			- Nodes shows `N pages`, the switches Journals, Orphan pages, Built-in pages and Excluded pages, `N hops from selected nodes` while a node is selected, and `Reset Graph`. Each switch starts in the state the drawn graph uses, so Orphan pages starts off.
			- Search's `Click to search` opens a text field inside the panel. Entering text adds a filter that keeps nodes whose title contains it, ignoring case; the section lists each filter with a remove button and `Clear All`, and opens by itself while filters are set. While the field has text, it lists up to 20 matching nodes as buttons that open the page on Enter or click and select the node on Shift+Enter or Shift-click.
			- Forces shows `N links`, `Pause simulation`, the sliders Link Distance (10 to 180, step 10), Charge Strength (−1000 to 1000, step 100) and Charge Range (500 to 4000, step 100), each showing its value, and `Reset Forces`. Export offers `as PNG`, which saves the canvas as drawn at the device's pixel ratio, with the theme's background, as `graph.png`.
			- Settings changes are kept in `sessionStorage` under `logseq-gardener-graph`, so they last while the reader moves between pages in the tab, and the published files never change.
		- ### Layout and data files
			- The publisher lays the graph out at build time with a Barnes–Hut force layout in `site/graph_layout.rs`, with Logseq's default forces or the values `:graph/forcesettings` sets, from a fixed seed, so two builds of one commit write identical graph files. It caches the positions by the graph's nodes, edges and forces. One layout covers every node of a journals setting, so toggling orphans, built-in or excluded pages hides or shows nodes without moving the others.
			- `_graph/default.bin` holds the journals-off graph and `_graph/journals.bin` the journals-on graph: node titles, routes, flags, degrees, quantised positions and delta-coded edges in one compact binary file each. Only `graph.html` fetches them: `default.bin` when the page loads and `journals.bin` the first time a reader turns on Journals. No other page references a file under `_graph/` or the graph script.
			- The first frame uses the build-time positions. A force simulation starts in `lsg-graph-worker.js` only when a reader changes a force slider or drags a node, a drag reheating it for two seconds; it stops when it settles or when `Pause simulation` is on, and `Reset Forces` restores the default forces and the build-time positions.
			- `lsg-graph.js` draws with WebGL from typed arrays, with Canvas 2D when WebGL is unavailable, and hit-tests with a uniform spatial grid.
			- While a graph file downloads, decodes or draws for more than 500 ms, the graph area shows a spinner, `Loading the graph…` in a live region and `Cancel`. `Cancel` leaves `The graph was not loaded.` with `Load the graph`, and a failed download leaves `The graph could not be loaded.` with `Retry`. A graph already drawn stays visible, dimmed, behind a message.
		- ### Page graph
			- The right sidebar's top bar holds `Page graph`, with the `hierarchy` icon, between Contents and Help. It adds a Page graph card at the top, or moves the existing one there, and the card is kept in the sidebar's saved state like other cards.
			- The card draws the SDK's neighbourhood of the page in the current route, through its aliases, with the global graph's look, hover, pan, zoom and click, without the settings panel. Its body is a `Show Journals` row with a switch, off at first and kept for the tab's session, then a canvas 500px high at the card's width. It follows navigation; with no page in the route it shows today's journal's neighbourhood when that journal is published, and on a zoomed block it shows the block's page.
			- The publisher writes each published page's neighbourhood, with journal nodes flagged, to `_graph/p/<hash>.json` when it is at most 100 KB gzip. The card fetches it when it opens or the route changes and lays it out in the graph worker. For `Logseq` and `Logseq/Entity`, whose neighbourhoods exceed 100 KB gzip, no file is written and the card draws the neighbourhood from the global file at the global positions.
			- The card shows the same loading, not-loaded and failure states as the graph view, with `Loading the page graph…`, `The page graph was not loaded.` with `Load the page graph`, and `The page graph could not be loaded.` with `Retry`.
		- ### Counts on the encode garden
			- Measured on 2026-10-27 with the change's release build, garden commit `cc1bb2e`, against Logseq's counts above:
			- | Measure | Logseq | `lsg` |
			  | --- | --- | --- |
			  | Nodes, default view | 19,105: 8,543 pages with content, 10,562 reference-only | 17,541: 12,930 published pages, including generated namespace pages, 4,611 reference-only |
			  | Drawn edges, default view | 78,010 | 75,333 |
			  | `N links` | 81,394 | 86,066 |
			  | Journals on: nodes, edges, `N links` | 19,732, 85,691, 88,691 | 18,176, 83,652, 94,186 |
			  | `Person` | 1,025 edges, size 80 | 1,025 drawn edges, degree 1,087 with repeats, size 82 |
			  | `tmux` | a hub | 142 drawn edges, degree 155, size 42; a namespace parent |
			- | Page graph | Logseq, journals off; on | `lsg`, journals off; on |
			  | --- | --- | --- |
			  | `Contrando` | 2, 1; 3, 2 | 2, 1; 3, 2 |
			  | `Foam` | 17, 51; 18, 58 | 17, 43; 18, 50 |
			  | `tmux` | 224, 1,305; 274, 1,714 | 225, 1,198; 275, 1,591 |
			  | `Person` | 1,431, 5,367; 1,673, 7,384 | 1,487, 4,648; 1,730, 6,705 |
			  | `Logseq` | 4,964, 60,392 | from the global file at the global positions |
			- The counts in each table are nodes, then edges. Node counts differ by pages that Logseq makes and `lsg` does not draw as reference-only nodes: Logseq makes a page for every property value that names a page and for every namespace parent, and `lsg` publishes namespace parents as generated namespace pages. `lsg` counts each page pair once, where Logseq lists an edge for each direction in which one page names the other. `lsg`'s `N links` also counts the namespace relations of its generated pages.
	- ## Performance
		- The `[graph]` table of [`scripts/parity/budgets.toml`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/budgets.toml) sets the budgets, measured on the whole encode garden:
		- | Budget | Value | What it limits |
		  | --- | ---: | --- |
		  | `data_gzip_kb` | 600 | `_graph/default.bin` |
		  | `script_kb` | 30 | `lsg-graph.js` with its styles, minified gzip, loaded only by `graph.html` and the Page graph card |
		  | `worker_kb` | 8 | `lsg-graph-worker.js`, minified gzip, loaded only when the simulation starts |
		  | `first_frame_ms_desktop` / `first_frame_ms_phone` | 1,000 / 3,000 | From navigation to `graph.html` to the first frame with every node drawn |
		  | `heap_mb` | 50 | Peak JS heap on the phone profile after the first frame, a pan and a zoom |
		  | `frame_ms_phone` | 50 | 95th-percentile frame time while panning on the phone profile |
		  | `page_graph_fragment_gzip_kb` | 100 | Largest neighbourhood file; larger neighbourhoods use the global file |
		  | `page_graph_open_ms_desktop` / `page_graph_open_ms_phone` | 300 / 1,500 | From clicking Page graph to the first frame |
		  | `build_slowdown_pct` | 25 | Whole-garden build with graph data and layout against the `--baseline-lsg` binary |
		- `parity:budgets` checks the sizes in CI, and that the header link and sidebar item keep every page within `[shell] frame_kb`. `parity:perf --graph` marks the times, heap and build values.
		- [[lsg/Ref/Logseq Compatibility/Performance]] records the measurement of the graph view change on `73d9c2f` with garden commit `cc1bb2e`: the first frame in 76 ms on desktop and 2,871 ms on the phone profile, against Logseq's 2,291,000 ms on desktop; a 95th-percentile pan frame time of 9 ms and 10 ms; a peak heap of 9.9 MB and 10.2 MB, against Logseq's 2,340 MB; 409 KB gzip of graph data, against Logseq's 19,800 KB `db.gz`; and no worker loaded. `lsg-graph.js` with its styles was 11.3 KB gzip, the worker 2.7 KB gzip, and the 13,467 neighbourhood files at most 85.0 KB gzip.
		- The Page graph opened in 221 ms on `Foam`, 153 ms on `tmux` and 164 ms on `Person` on desktop, and 435, 578 and 821 ms on the phone profile in a 1280px window. A whole-garden build took 15.3 seconds with the graph and 12.9 seconds with the baseline binary, 19% slower against a 25% budget.
		- Headless Chromium draws WebGL on the CPU unless it is given the GPU, so `parity:perf --graph` launches Chromium with the GPU and records the renderer. With the CPU renderer, the phone run's pan frame time was 83 ms at the 95th percentile; with the GPU, 10 ms.
	- ## Differences from Logseq
		- The graph and its layout are computed at build time, and the browser runs a force simulation only when the reader changes a force or drags a node. Logseq rebuilds the graph from the whole database and simulates it on every visit, which took 38 minutes on the encode garden. The layout uses Logseq's forces, so positions differ only in detail, as two runs of Logseq's own simulation do.
		- The Orphan pages switch starts off. Logseq's switch shows on while its graph leaves orphans out; `lsg` draws the same graph and shows the switch in the state it uses.
		- Labels show only for hovered and selected nodes and for nodes drawn at 12px or larger. Logseq draws every label, which at 19,105 nodes is a grey mass at the default zoom and costs frames on a phone.
		- Reference-only nodes have no page to open; clicking one shows its title as `not published`. They gain routes when `lsg` publishes reference-only pages.
		- `Click to search` opens a field inside the panel that adds a title filter and lists up to 20 matching nodes. Logseq opens its search dialog in a graph mode, which `lsg`'s search does not offer.
		- Settings last for the tab's session, kept in `sessionStorage`. Logseq keeps them until reload, while the reader moves between pages of its single-page app.
		- On a zoomed block route, the Page graph card shows the block's page. Logseq draws the block's own reference graph there; a per-block answer would need a file per block.
		- The entry points show unless the garden's stylesheet hides them, since `lsg`'s graph opens quickly. With `--stylesheet logseq/custom.css`, the encode garden's rules hide them as on Logseq's build.
		- `lsg`'s renderer is a small WebGL renderer written for `lsg`. Logseq's `pixi-graph-fork` and d3 would exceed the 30 KB script budget and build one object per node.
		- Whiteboards and the desktop app's graph features that the published site does not show have no counterpart.
	- ## Before the change
		- The `lsg` build measured on 2026-10-26, from `1e72969`, had no graph view, no page graph, no right sidebar and no graph data, checked against `crates/logseq-gardener-sdk/src/site/shell.rs`, `references.rs` and the whole-garden build.
		- **Entry points.** The header held Toggle left sidebar, Search, Home and More; the left sidebar held Journals, All pages and the favorites. No Graph link, no Graph view item, no `g g`, no right sidebar and no `t r`. The [published app shell spec](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/published-app-shell/spec.md) requires that no header control links to a missing file, so the Graph link waited for the graph view.
		- **Graph output.** None. No file in the build listed pages with their edges or positions.
		- **What the SDK knew.** `crates/logseq-gardener-sdk/src/references.rs` built a `ReferenceIndex` shared by the CLI, the language server and the publisher. It held each block's references with namespace ancestors added, each page's `tags::`, each page's namespace ancestors, and the referring blocks of every page. Page-to-page edges, tag edges and namespace edges could be derived from it, but no operation returned them, and the index did not count property names as references.
		- **Which pages existed.** The build published 10,756 pages. Logseq's database holds 19,911, of which 7,540 are reference-only pages that `lsg` does not publish.
	- ## Fixture pages
		- | Page | Why | Logseq page graph | `lsg` on 2026-10-26, before the change |
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
