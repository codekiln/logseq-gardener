## Why

When a reader opens the graph view, the page shows only its title until `lsg-graph.js` has downloaded and decoded `_graph/default.bin` and drawn the first frame. On a slow connection or a large garden the reader cannot tell whether anything is happening and has no way to stop the download. The Page graph card in the right sidebar has the same gap, and a failed download leaves either a one-line message that asks for a page reload or, if `lsg-graph.js` itself fails, an empty card ([Issue #294 — Show that the graph view is loading, and let the reader cancel](https://github.com/codekiln/logseq-gardener/issues/294)).

## What Changes

- While the graph view or the Page graph card downloads, decodes or lays out its data, the graph area shows a spinner, a status line that screen readers announce, and a Cancel button. The indicator appears only after the load has taken 500 ms, so a fast load shows nothing.
- Cancel stops the download. The graph area then says the graph was not loaded and offers a button that loads it.
- A failed download says the graph could not be loaded and offers Retry, which loads it again without reloading the page.
- Turning on Journals in the graph view or Show Journals in the card loads a different data file and shows the same indicator, controls and messages. A newer load replaces an older one, so turning a switch back off never draws the data the reader turned away from.
- The look follows Logseq's graph loading and error states: a centred spinner and 14px muted text, and an outlined small button. Both themes use the site's colour tokens.
- If `lsg-graph.js` fails to load in the right sidebar, the Page graph card says so and offers Retry instead of staying empty.

## Capabilities

### New Capabilities

### Modified Capabilities

- `published-graph-view`: the graph view shows a delayed loading indicator with Cancel, a not-loaded message with a load button, and a failure message with Retry, for the first load and for the Journals switch.
- `published-page-graph`: the Page graph card shows the same states for its neighbourhood file, the global file it falls back to, its layout, and the Show Journals switch, and recovers from a failed script load.

## Impact

- `crates/logseq-gardener-sdk/src/site/graph.js` and `graph.css`: a loading overlay shared by both views, fetches that accept an abort signal, and a cache that forgets failed or cancelled downloads.
- `crates/logseq-gardener-sdk/src/site/sidebar.js` and `sidebar.css`: a Retry message in the Page graph card when the script fails to load.
- `tests/graph.test.cjs` gains a browser-script harness for `graph.js` with a fake clock, fetch and worker; `tests/sidebar.test.cjs` covers the script failure; `tests/mini-dom.cjs` learns the `$=` attribute selector `graph.js` uses.
- `docs/local-site.md` describes the loading states in its Graph view section.
- `lsg-graph.js` grows by about 1 KB gzip, within the 30 KB `[graph]` script budget. No published file changes other than the graph script and styles and the sidebar script and styles.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify/Create Uniform Interfaces](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Create%20Uniform%20Interfaces.md)
- [My/Principle/Simplify/Minimize Surface Area](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Minimize%20Surface%20Area.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
