## 1. SDK

- [ ] 1.1 Record each block's and page's property names in `Contribution` and keep them through `ReferenceIndex::replace`; unit-test that linked references are unchanged
- [ ] 1.2 Read `:graph/settings`, `:graph/forcesettings` and `:shortcuts {:go/graph-view …}` in `garden_settings`; unit-test defaults, set values and a malformed value with its diagnostic
- [ ] 1.3 Add the `graph` module: nodes with flags, degree and optional route; distinct edges; the `links` count; the nested-namespace rule; property-name edges; Logseq's filters; privacy from published pages only; unit-test every scenario in the garden-graph spec
- [ ] 1.4 Add `neighbourhood(page, journals)` following Logseq's `build-page-graph`, through aliases; unit-test `Foam`-shaped and journal fixtures
- [ ] 1.5 Update the graph from a replaced contribution; unit-test the one-file-changes scenario

## 2. Layout and data files

- [ ] 2.1 Add `site/graph_layout.rs`: seeded Barnes–Hut layout with Logseq's forces and velocity decay, cached by a hash of nodes, edges and forces; test determinism and the cache hit
- [ ] 2.2 Write `_graph/default.bin` and `_graph/journals.bin` in the documented format, with a format version; test round-trip decoding in Rust and in `tests/` against the JavaScript reader
- [ ] 2.3 Write `_graph/p/<hash>.json` for neighbourhoods at most 100 KB gzip, with journal nodes flagged; test the limit with a large fixture

## 3. Graph page and scripts

- [ ] 3.1 Add `site/graph.rs` writing `graph.html` in the page shell with the canvas, the no-JavaScript message and the panel markup
- [ ] 3.2 Add `site/lsg-graph.js`: binary reader into typed arrays, WebGL renderer with Canvas 2D fallback, Logseq's sizes and colours (port Logseq's namespace colour hash), label threshold, hover, theme following, pan and zoom by mouse, touch and keys, click, shift-click, long press, drag, N hops
- [ ] 3.3 Add the settings panel behaviour: sections, switches, counts, `Reset Graph`, title filters and the matching node list, sliders, `Reset Forces`, `as PNG`, session storage
- [ ] 3.4 Add `site/lsg-graph-worker.js` with the same forces, loaded on the first force change or node drag; stop on settle and on pause
- [ ] 3.5 Add `site/graph.css` with Logseq's panel values and the theme tokens

## 4. Entry points and Page graph

- [ ] 4.1 Add the header Graph link (`graph.html#/graph`, `cp__header`), the `Graph view` sidebar item (`graph-view-nav`, `g g` hint) and `g g` to `site/shell.rs` and `lsg-shell.js`; update the shell tests; check the encode garden's custom.css hides both
- [ ] 4.2 After `match-logseq-right-sidebar` is applied, add the `Page graph` top-bar button and the `graph` card type: fetch the neighbourhood file or fall back to the global file, worker layout, `Show Journals`, following navigation, today's journal on routes without a page

## 5. Harness and budgets

- [ ] 5.1 Add the `[graph]` table to `scripts/parity/budgets.toml`; make `parity:budgets` check `data_gzip_kb`, `script_kb`, `worker_kb`, `page_graph_fragment_gzip_kb` and the shell frame, and `parity:perf` measure the first frame, heap, pan frame time, Page graph open times and build slowdown for `lsg` and Logseq

## 6. Verification

- [ ] 6.1 Build the encode garden with `parity:build`; compare `graph.html` with the `gh-pages` build's `#/graph` at 1280×800 and 390×844 in both themes: node and edge counts after removing pages `lsg` does not publish, `tmux`'s and `Person`'s degree and size, the `Logseq/Entity` colour, panel text, Journals count, N hops on `tmux`, the `tmux` filter, a force change and PNG export; record values and screenshots in `verification.md`
- [ ] 6.2 Compare the Page graph card on `Contrando`, `Foam`, `tmux`, `Person` and `Logseq` with Logseq's, with Show Journals off and on, and record the counts
- [ ] 6.3 Measure the phone heap and frame time; if they miss the budget, apply the reference-only fallback from the design and record the measurement
- [ ] 6.4 Run `parity:perf --record` and commit the graph rows in `docs/parity/performance.md`
- [ ] 6.5 Sync this change's `published-app-shell` delta only after `match-logseq-right-sidebar` is archived
- [ ] 6.6 Run `mise run ci`
