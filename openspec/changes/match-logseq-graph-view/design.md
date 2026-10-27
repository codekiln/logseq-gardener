## Context

The [Graph view analysis](../../../docs/parity/graph-view.md) measured Logseq's `gh-pages` build of the encode garden (garden commit `0ce74d6`) and an `lsg` build from `1e72969`. Logseq's global graph has 19,105 nodes and 78,010 drawn edges with its defaults; it downloads the whole database, builds the graph in the browser and lays it out with a live d3-force simulation. On the desktop the first frame took 38 minutes and the JS heap peaked at 2,340 MB. Its Page graph card took 19.7 s on `Foam` and 205.8 s on `Person`.

`lsg` has no graph output. `ReferenceIndex` in `crates/logseq-gardener-sdk/src/references.rs` holds each block's references with namespace ancestors added, each page's `tags::`, namespace ancestors and referring blocks; it serves the CLI, the language server and the publisher. It does not record property names as references. The right sidebar that holds the Page graph card is proposed in `match-logseq-right-sidebar` ([Issue #119](https://github.com/codekiln/logseq-gardener/issues/119)), whose card API this change uses.

## Goals / Non-Goals

**Goals:**

- Readers get Logseq's global graph and Page graph: the same nodes, edges, look, interaction and settings panel.
- The global graph's first frame arrives within a second on a desktop and three seconds on a phone, with a phone heap of at most 50 MB; no other page loads any graph data or script.
- The CLI and the language server can ask for a page's neighbours with the same answers the publisher draws.

**Non-Goals:**

- The block reference graph Logseq draws in the Page graph card on a zoomed block route. The card shows the block's page there until a per-block answer is worth its files.
- Opening the graph's search in the static search dialog from [Issue #118](https://github.com/codekiln/logseq-gardener/issues/118); the panel uses its own title filter until that dialog offers a graph mode.

## Decisions

**Where the work lives.** The SDK answers what the graph holds; the publisher lays it out and renders it.

| Piece | Lives in | Notes |
| --- | --- | --- |
| `graph(index, titles, settings) -> Graph` | SDK, new `graph` module | Nodes with title, optional route, degree and flags; distinct edges; `links` count with duplicates, for `N links`. Applies the nested-namespace rule, property-name references and Logseq's filters. |
| `neighbourhood(index, page, journals) -> Graph` | SDK | Logseq's `build-page-graph`: through aliases, referring pages, tags, edges among the nodes, namespace edges. |
| Property names per block | SDK, `Contribution` | Recorded beside references; only `graph` reads them, so linked references stay as they are. |
| `:graph/settings`, `:graph/forcesettings`, `:shortcuts {:go/graph-view …}` | SDK, `garden_settings` | Read by the existing bounded EDN reader; unknown values give a diagnostic and the defaults. |
| Incremental updates | SDK | `ReferenceIndex::replace` already swaps one file's contribution; the graph's edges for that page are recomputed from it. |
| Force layout and its cache | Publisher, `site/graph_layout.rs` | Presentation only. |
| `graph.html`, data files, neighbourhood files, renderer, panel, worker, Page graph card, entry points | Publisher | Rendering only. |

**Node set and privacy.** The graph starts from the published pages only, so a page that is private or outside the selected namespaces, and anything only such pages refer to, never appears. Reference-only pages (named by a published page but without a file) are nodes without a route, as Logseq's are. Logseq's counts include pages `lsg` does not publish; the verification compares `lsg`'s counts with Logseq's after removing those, using `parity:compare`'s list.

**Layout at build time.** `site/graph_layout.rs` runs a Barnes–Hut force layout (a quadtree with theta 0.5) with Logseq's link, charge, collision and centring forces from a seeded start, for a fixed number of ticks with Logseq's velocity decay 0.5. It is written in the crate, about 300 lines, so no layout dependency enters the notice process. One layout covers every node of a journals setting, including orphan, built-in and excluded nodes, so toggling those settings hides nodes without moving the rest. The result is cached in the build cache under a hash of the nodes, edges and forces. The journals-on graph gets its own layout.

**Data files.** `_graph/default.bin` and `_graph/journals.bin` are little-endian binary: a header with counts and the format version; node titles as one UTF-8 block of lengths and bytes; route hashes (the hash part of `p-<hash>.html`, empty for reference-only nodes); one flags byte and a degree per node; positions as two 16-bit numbers scaled to the layout's bounds; edges sorted by source then target, delta-coded as variable-length integers. The analysis estimates about 420 KB gzip for the default view; the budget is 600 KB. The script reads the file into typed arrays and never builds one object per node.

**Renderer.** `lsg-graph.js` draws with WebGL from the typed arrays, with Canvas 2D when WebGL is unavailable; it hit-tests with a uniform spatial grid. Labels are drawn for hovered and selected nodes and for nodes whose drawn size reaches 12px, so zooming in shows them all; Logseq draws every label, which at 19,105 nodes is a grey mass and costs frames on a phone. `N hops` is a breadth-first search over the edge arrays. The renderer follows the site's theme attribute, so `t t` recolours without reloading.

**Simulation on demand.** `lsg-graph-worker.js` holds the same forces as the build-time layout, written in JavaScript. It loads only when a reader changes a force slider or drags a node, receives the positions and edges as transferable buffers, and posts positions back each tick until the simulation settles or is paused. The page graph also uses it to lay out a neighbourhood before its first frame.

**Settings.** Defaults come from `config.edn`. Changes are stored in `sessionStorage` under `logseq-gardener-graph`, so they last while the reader moves between pages in the tab, as Logseq's last until reload. The Orphan pages switch starts off, matching the graph Logseq draws (see question 4).

**Entry points.** The header Graph link points at `graph.html#/graph` inside a header with the class `cp__header`, and the left sidebar item carries `graph-view-nav`, so the garden's two `custom.css` rules hide them when a site loads that stylesheet. Without it, both show, because `lsg`'s graph opens quickly. `graph.html` ignores the `#/graph` hash.

**Page graph.** The publisher writes `_graph/p/<hash>.json` for each published page whose neighbourhood is at most 100 KB gzip: compact arrays of titles, route hashes, flags and edges, with journal nodes flagged so the `Show Journals` switch needs no second file. The card fetches it when it opens or the route changes and lays it out in the worker. For the few hubs above the limit (`Logseq`, `Microfreak`), no file is written and the card draws the neighbourhood from the global file at the global positions. The card registers with the right sidebar as type `graph` and adds the `Page graph` top-bar button.

**Budgets.** New `[graph]` rows in `scripts/parity/budgets.toml`, measured on the whole encode garden against Logseq's build:

| Budget | Value |
| --- | ---: |
| `data_gzip_kb` (`_graph/default.bin`) | 600 |
| `script_kb` (`lsg-graph.js` and `graph.css`, minified gzip, worker separate) | 30 |
| `worker_kb` (`lsg-graph-worker.js`, minified gzip) | 8 |
| `first_frame_ms_desktop` / `first_frame_ms_phone` | 1,000 / 3,000 |
| `heap_mb` (phone, after first frame, a pan and a zoom) | 50 |
| `frame_ms_phone` (p95 while panning) | 50 |
| `page_graph_fragment_gzip_kb` | 100 |
| `page_graph_open_ms_desktop` / `page_graph_open_ms_phone` | 300 / 1,500 |
| `build_slowdown_pct` (against `--baseline-lsg`) | 25 |

`parity:budgets` checks the sizes in CI, and that the header link and sidebar item keep every page within `[shell] frame_kb`; `parity:perf` marks the times, heap, frame time and build time in `docs/parity/performance.md` beside Logseq's values.

## Risks / Trade-offs

- [A phone misses the heap or frame budget at 19,105 nodes] → The Apply step measures first. If it fails, the phone profile starts with reference-only nodes hidden (8,543 nodes), as Logseq starts with journals hidden, a `Reference-only pages` switch in Nodes turns them on, and `verification.md` records the measurement.
- [The build-time layout differs from Logseq's live layout] → Both use the same forces; the positions differ in detail, as two runs of Logseq's own simulation do. The verification compares structure (counts, degrees, colours) and judges the picture by screenshots.
- [About 10,000 neighbourhood files] → Each is small and sits beside the `_refs/` fragments; the build report keeps counting source pages.
- [This change and `match-logseq-right-sidebar` both modify the header and shortcut requirements] → The delta here is written on top of the right sidebar change's version; sync this change after that one is archived.
- [Property names become graph edges] → `prev`, `next`, `up` and `see-also` become large reference-only nodes, as in Logseq; the Built-in pages filter does not cover them, matching Logseq.

## Resolved Questions

### 1 - Does `lsg` show the graph entry points that the garden's custom.css hides?

Decided by the agent without codekiln: yes, and with Logseq's classes and `href` shape so the garden's rules still hide them when a build loads `--stylesheet logseq/custom.css`. The garden hides them because Logseq's graph crashes; `lsg`'s does not have that cost, and parity with the same stylesheet gives the same result as Logseq.

### 2 - Does `g g` follow `config.edn`?

Decided by the agent without codekiln: yes. `:shortcuts {:go/graph-view false}` turns it off, as in Logseq, so the encode garden has no `g g` and the sidebar item shows no hint.

### 3 - Are reference-only pages graph nodes?

Decided by the agent without codekiln: yes, without a route; clicking one shows its title as `not published`. They are more than half of Logseq's nodes and carry many of its edges, so leaving them out would change the picture. They gain routes when `lsg` publishes them ([Issue #111](https://github.com/codekiln/logseq-gardener/issues/111), [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115)).

### 4 - Does the Orphan pages switch match Logseq's visible state or its behaviour?

Decided by the agent without codekiln: its behaviour. Logseq's switch shows on while its graph leaves orphans out; `lsg` leaves them out and shows the switch off. The node count matches Logseq's 19,105, and the switch tells the truth about what is drawn.

### 5 - Do property names count as references?

Decided by the agent without codekiln: in the graph only. Logseq counts them in file graphs, and `up`, `prev`, `next`, `see-also` and `created-by` are among its ten largest nodes. Linked references stay as they are, because changing them belongs with [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115).

### 6 - Where is the layout computed?

Decided by the agent without codekiln: at build time in Rust, in the publisher, with Logseq's default forces and a fixed seed, cached by content. Issue #106 asks for graph work at build time; the layout is presentation, so it lives in the publisher beside the renderer.

### 7 - Does every node show its label?

Decided by the agent without codekiln: labels show for hovered and selected nodes and for nodes drawn at 12px or larger, so zooming in shows them all. Logseq's all-labels view is unreadable at the default zoom and would cost phone frame time.

### 8 - How long do settings changes last?

Decided by the agent without codekiln: for the tab's session, in `sessionStorage`. Logseq keeps them until reload while the reader moves between pages of its single-page app; `lsg` pages are separate documents, so session storage gives readers the same experience.

### 9 - What does `Click to search` open?

Decided by the agent without codekiln: a text field inside the panel that adds a title filter and lists up to 20 matching nodes. Logseq opens its search dialog in a graph mode, which the static search from Issue #118 does not offer; the panel's list also gives keyboard readers a path to any node.

### 10 - How does the page graph get its data?

Decided by the agent without codekiln: from a per-page neighbourhood file written at build time and laid out in the worker, with the global file as the source for the few neighbourhoods above 100 KB gzip. Per-page files keep the card's cost tied to the neighbourhood; building every page's layout at build time would cost build time for cards most readers never open.

### 11 - What does the Page graph card show on a zoomed block route?

Decided by the agent without codekiln: the block's page. Logseq draws the block's own reference graph there; a per-block answer would need a file per block, and it is listed as a non-goal until readers need it.

### 12 - Which renderer draws the graph?

Decided by the agent without codekiln: a small WebGL renderer written for `lsg`, with Canvas 2D as fallback. Logseq's `pixi-graph-fork` and d3 would exceed the 30 KB script budget and build one object per node.

## Open Questions

None.
