## 1. Test harness

- [x] 1.1 Teach `tests/mini-dom.cjs` the `$=` attribute selector.
- [x] 1.2 Add a `graph.js` harness to `tests/graph.test.cjs` with a fake clock, a fetch stub that answers or aborts on request, a fake worker and canvas contexts.

## 2. Loading overlay

- [x] 2.1 Add `progress()` to `graph.js`: the overlay with its live region and button, the 500 ms delay, Cancel, the not-loaded and failure states, and focus.
- [x] 2.2 Pass an abort signal through `load()` to `fetch`, mark HTTP errors with their status, and forget failed or cancelled downloads in the graph view's cache.
- [x] 2.3 Add the overlay, spinner, button and dimming styles to `graph.css` from `theme.css` tokens.

## 3. Graph view and Page graph card

- [x] 3.1 Load the graph view's first graph and the Journals switch through the overlay; show the panel with the first drawn graph; reset forces on the drawn graph.
- [x] 3.2 Load the Page graph card's neighbourhood, fallback file and layout through the overlay, falling back only on an HTTP error, and route Show Journals through it.
- [x] 3.3 Show `The page graph could not be loaded.` and Retry in the card when `lsg-graph.js` fails to load.

## 4. Tests and checks

- [x] 4.1 Cover the fast load, delayed indicator, Cancel and reload, failure and Retry, and the Journals switch for the graph view and the Page graph card; cover the script failure in `tests/sidebar.test.cjs`.
- [x] 4.2 Check the loading, not-loaded and failure states in a browser in both themes, run `mise run ci`, and record the results in `verification.md`.
