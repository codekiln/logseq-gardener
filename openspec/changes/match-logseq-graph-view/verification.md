# Verification

Measured on 2026-10-27 with an `lsg` release build of this change (`parity:build`, garden commit cc1bb2eb, 8,697 pages, with `--journals --search`), served from localhost and driven in Chromium at 1280×800 and 390×844 in both themes. Logseq's values come from the [Graph view analysis](../../../docs/parity/graph-view.md), measured on the `gh-pages` build of garden commit `0ce74d6`. Logseq's global graph took 38 minutes to draw on the desktop, so it was not opened again; its screenshots for this change are the analysis's description of the live view.

## Global graph

| Check | Logseq (analysis) | lsg |
| --- | --- | --- |
| Nodes, default view | 19,105: 8,543 pages with content, 10,562 reference-only | 17,541: 12,930 published pages, including generated namespace pages, 4,611 reference-only |
| Drawn edges, default view | 78,010 | 75,333 |
| `N links` | 81,394 | 86,066 |
| Journals on: nodes, edges, `N links` | 19,732, 85,691, 88,691 | 18,176, 83,652, 94,186 |
| `Person` | 1,025 edges, size 80 | 1,025 drawn edges, degree 1,087 with repeats, size 82 |
| `tmux` | listed as a hub | 142 drawn edges, degree 155, size 42; a namespace parent |
| Largest nodes | `up`, `prev`, `next`, `see-also` and `created-by` among the ten largest | `logseq-entity` (4,925), `up`, `prev`, `next`, `Person`, `see-also` among the ten largest |
| `Logseq/Entity` colour | one of the ten colours by hash | `#7f7f7f` in both themes; `tests/graph.test.cjs` checks the hash against Clojure's `(hash s)` computed with an independent Murmur3 library |
| Canvas | `calc(100vh - 100px)` | 700px at 1280×800, 744px at 390×844 |
| Panel text | Nodes, Search, Forces, Export, closed; `19105 pages`; Journals, Orphan pages (shown on), Built-in pages, Excluded pages; `Reset Graph`; `Click to search`; `N links`, `Pause simulation`, Link Distance 70, Charge Strength −600, Charge Range 600, `Reset Forces`; `as PNG` | the same labels and order, all closed; `17541 pages`; Orphan pages off as decided; `86066 links` |
| Journals switch | adds journals | fetched `_graph/journals.bin` once; `18176 pages`; kept after opening `index.html` and going back |
| N hops on `tmux` | keeps `tmux` and its neighbours | Shift-Enter on the `tmux` result selected it; N hops 1 left `tmux` and its neighbours, coloured with the accent, `tmux` in the selection colour |
| `tmux` filter | keeps titles containing `tmux` | filter chip `tmux`; only matching titles drawn; `Clear All` restores the view |
| Force change | nodes move | Link Distance 120 loaded `lsg-graph-worker.js` and spread the graph; the longest input delay during the simulation was 5 ms |
| PNG export | saves the canvas | `graph.png` with the theme background and labels |
| Open a page | opens the page | Enter on the `tmux` result opened `p-424036ef4353ec8f.html`; a reference-only node (`prev`) shows `prev — not published` |
| `t t` | — | nodes, labels and edges recolour without reloading data |

Logseq's counts include pages that `lsg` does not draw as reference-only nodes: Logseq makes a page for every property value that names a page in a file graph, and for every namespace parent, which `lsg` publishes as generated namespace pages and therefore counts among its published pages. `lsg` draws one edge per page pair and drops an ancestor edge only when the page does not link the ancestor itself, as the spec requires; Logseq drops it in both cases. `lsg` counts a pair that is both a reference and a tag twice in `N links`, as Logseq does, and also counts the namespace relations of its generated pages, which is why its `N links` is higher. The analysis's comparison after removing every page `lsg` does not publish needs Logseq's reference data per page, which the harness's `db.gz` reader does not decode yet; the table compares the totals and the named nodes instead.

Screenshots of `graph.html` at both sizes and themes, with the panel open, the `tmux` filter, N hops on `tmux`, Link Distance 120, the PNG export and the Page graph cards were reviewed by hand; they are not committed.

## Page graph

Counts are nodes and edges, with Show Journals off and then on.

| Page | Logseq (analysis) | lsg | Fetched |
| --- | --- | --- | --- |
| `Contrando` | 2, 1; 3, 2 | 2, 1; 3, 2 | `_graph/p/13c09278c3095993.json` |
| `Foam` | 17, 51; 18, 58 | 17, 43; 18, 50 | `_graph/p/077e10850d78a1dc.json` |
| `tmux` | 224, 1,305; 274, 1,714 | 225, 1,198; 275, 1,591 | `_graph/p/424036ef4353ec8f.json` |
| `Person` | 1,431, 5,367; 1,673, 7,384 | 1,487, 4,648; 1,730, 6,705 | `_graph/p/40d76f1f51639ec0.json` |
| `Logseq` | 4,964, 60,392 | from the global file at the global positions | `_graph/default.bin` |

Node counts match or differ by the same reference-only pages as the global graph. `lsg` counts each pair once; Logseq lists a directed edge for each direction in which one page names the other, so its edge counts are higher. Only `Logseq` and `Logseq/Entity` have neighbourhoods above 100 KB gzip and no file. The card follows navigation, sits between Contents and Help in the top bar, carries the `hierarchy` icon and is kept in the sidebar's saved state.

## Entry points

On `tmux` the header holds Toggle left sidebar, Home, Graph (`graph.html#/graph` inside `header.cp__header`), Search, More and Toggle right sidebar, and the left sidebar holds Journals, Graph view (`li.graph-view-nav`) and All pages. With the garden's `logseq/custom.css` loaded, both entry points compute to `display: none`; `graph.html` still opens when typed. The encode garden sets `:shortcuts {:go/graph-view false}`, so its build shows no `g g` hint and `g g` does nothing; `tests/shell.test.cjs` and `crates/logseq-gardener-sdk/tests/site_shell.rs` cover both settings.

## Budgets

Measured with `parity:perf --graph --baseline-lsg` on the whole encode garden and recorded in [Parity performance](../../../docs/parity/performance.md). The phone profile is Lighthouse's: 412×823 at 1.75×, CPU slowed four times, slow 4G.

| Budget | Value | lsg |
| --- | ---: | ---: |
| `data_gzip_kb` (`_graph/default.bin`, stored gzip) | 600 | 409 |
| `script_kb` (`lsg-graph.js` and `lsg-graph.css`, gzip) | 30 | 11.3 |
| `worker_kb` | 8 | 2.7 |
| `first_frame_ms_desktop` / `_phone` | 1,000 / 3,000 | 76 / 2,871 |
| `heap_mb` (phone, after first frame, pan and zoom) | 50 | 10.2 |
| `frame_ms_phone` (p95 while panning) | 50 | 10 |
| `page_graph_fragment_gzip_kb` (largest file) | 100 | 85.0 |
| `page_graph_open_ms_desktop` (`Foam`, `tmux`, `Person`) | 300 | 221, 153, 164 |
| `page_graph_open_ms_phone` (same) | 1,500 | 435, 578, 821 |
| `build_slowdown_pct` | 25 | +19% (15.3 s against 12.9 s, fastest of three) |

Logseq on the same desktop: 2,291,000 ms (38 minutes) to the first frame of its global graph with a 2,340 MB peak heap, and 19.7 s, 47.9 s and 205.8 s to open the Page graph card on `Foam`, `tmux` and `Person`.

No other page loads graph data or the graph script: `parity:budgets` checks every fixture page, and the measured runs fetched nothing under `_graph/` before `graph.html` or the card asked for it. The worker did not load on `graph.html` until a force changed or a node was dragged.

### Task 6.3: the phone budgets and the fallback

Headless Chromium draws WebGL on the CPU (SwiftShader) unless it is given the GPU. With SwiftShader, the phone run's pan frame time was 83 ms at the 95th percentile (desktop 77 ms) and its heap 10.5 MB; with the GPU, 10 ms and 10.2 MB. A phone draws WebGL on its GPU, so the harness now launches Chromium with the GPU for the graph rows and records the renderer, and the reference-only fallback was not applied: the heap is a fifth of its budget either way, and the frame time is within budget with a GPU.

## Differences from the plan

- `lsg publish` does not expose the layout cache; the SDK's `SiteOptions::layout_cache` does, and its tests cover the cache hit. Every CLI build lays the graph out, on two background threads while pages render.
- The data files are stored gzip-compressed and decompressed with `DecompressionStream`, so they transfer at their gzip size from hosts that do not compress `.bin` files.
- The data files carry no route hashes: a node's page is `p-<FNV-1a 64 of its title>.html`, which the script computes when a reader opens a node.
- The Page graph card draws its first frame after at most 100 ms of layout in the worker, then keeps moving until the layout settles, as Logseq's live simulation does.
- `parity:perf` cannot wait 38 minutes for Logseq's global graph, and a timed-out Logseq page stopped answering the browser driver; its row comes from the analysis.
