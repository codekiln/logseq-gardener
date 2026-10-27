## Why

Logseq's published site has a global graph at `#/graph` and a Page graph card in the right sidebar. `lsg` has neither: no Graph link, no graph page and no file that lists pages with their edges ([Graph view analysis](../../../docs/parity/graph-view.md), [Issue #120 — Add Logseq's graph view and page graph](https://github.com/codekiln/logseq-gardener/issues/120)).

Logseq builds both graphs in the browser from the whole database. On the encode garden its global graph took 38 minutes to draw its first frame on a desktop, with the JS heap peaking at 2,340 MB, and the garden's `custom.css` hides the graph's entry points because the graph crashes. `lsg` can compute the graph and its layout at build time and send the browser one compact file, so the same view opens in about a second and stays light on a phone.

## What Changes

- The SDK gains a garden graph operation shared by the CLI, the language server and the publisher. It returns Logseq's nodes and edges for given settings (journals, orphans, built-in pages, excluded pages): reference edges with Logseq's nested-namespace rule, property-name references, tag edges and namespace edges, with each node's flags and degree. A neighbourhood operation returns one page's page graph with and without journals.
- The publisher writes `graph.html`, Logseq's global graph view: a canvas filling the main column with Logseq's node sizes, colours, labels and edges in both themes; pan, zoom, click to open, shift-click or drag to select, and `N hops from selected nodes`; the `div.graph-filters` panel with Nodes, Search, Forces and Export.
- The layout is computed at build time in Rust with Logseq's default forces and a fixed seed, and cached by the graph's content. The browser draws the first frame from it. A force simulation runs in a worker only after a reader changes a force or drags a node.
- The graph data loads only on `graph.html`: one binary file for the default view and a second file for the journals-on view, fetched when a reader turns on Journals.
- The header gains a `Graph` link and the left sidebar a `Graph view` item, with Logseq's classes, so the garden's `custom.css` hides them when a site loads it. `g g` opens the graph unless `config.edn` turns `:go/graph-view` off.
- The right sidebar's top bar gains a `Page graph` button that adds a Page graph card: the current page's neighbourhood, Logseq's look, a `Show Journals` toggle, following navigation. Each page's neighbourhood is a small file fetched when the card opens.
- Pages that exist only because something links them are graph nodes, as in Logseq, and open nothing until `lsg` publishes them.
- New `[graph]` budgets cap the graph file, the scripts, the first frame on desktop and phone, the phone heap, frame time while panning, the neighbourhood files, the Page graph open time and the build-time cost.

## Capabilities

### New Capabilities

- `garden-graph`: the SDK operations that return the global graph and a page's neighbourhood with Logseq's node and edge rules.
- `published-graph-view`: `graph.html`, its data files, layout, rendering, interaction, settings panel, export and budgets.
- `published-page-graph`: the right sidebar's Page graph button and card, its neighbourhood files and budgets.

### Modified Capabilities

- `published-app-shell`: the header gains the `Graph` link, the left sidebar the `Graph view` item, and the shortcuts `g g`.

## Impact

- SDK: a new `graph` module beside `references` and `page_names`; `ReferenceIndex` contributions gain each block's property names so the graph can count them as Logseq does; `garden_settings` reads `:graph/settings`, `:graph/forcesettings` and `:shortcuts {:go/graph-view …}`.
- Publisher: new `site/graph.rs` (page, data files, neighbourhood files), `site/graph_layout.rs` (Barnes–Hut force layout and its cache), `site/graph.css`, `site/lsg-graph.js` (renderer and panel) and `site/lsg-graph-worker.js` (simulation); `site/shell.rs` gains the entry points; the right sidebar from `match-logseq-right-sidebar` gains the Page graph card.
- Parity harness: `scripts/parity/budgets.toml` gains a `[graph]` table checked by `parity:budgets` and marked by `parity:perf`.
- Build output: about 10,000 small neighbourhood files under `_graph/p/`, beside the existing `_refs/` fragments.
- Depends on [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119) for the Page graph card; the global graph does not.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
