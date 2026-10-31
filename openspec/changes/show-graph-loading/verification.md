# Verification

## Tests

`node --test tests/graph.test.cjs tests/sidebar.test.cjs` with the new tests and main's `graph.js` and `sidebar.js`: 11 of the 11 new tests failed, because main has no overlay, Cancel or Retry. With this change all 30 tests in the two files pass.

The new `graph.js` tests drive the script in a VM with a fake clock, a fetch stub and a fake worker:

| View | Cases |
| --- | --- |
| Graph view | A load within 500 ms shows nothing; at 500 ms the spinner, `Loading the graph…` and `Cancel` appear in the live region that was already in the page; Cancel aborts the request and `Load the graph` downloads again; a lost connection and an HTTP 500 both offer Retry; the Journals switch loads through the same states, keeps the default graph drawn behind a cancel, draws it again without a request, refetches a cancelled file, and drops a load replaced by a newer one; focus on the button moves to the canvas when the graph draws. |
| Page graph card | A fast load; the loading state, Cancel and `Load the page graph` without a fallback request; a lost connection offers Retry without a fallback request; Cancel during the worker layout draws nothing when the layout arrives; a hub's missing file (404) falls back to the global file, Show Journals replaces that download, and a failed journals file offers Retry. |
| Right sidebar | `lsg-graph.js` failing to load shows `The page graph could not be loaded.` and Retry, which keeps focus on the card header, loads the script again and draws the card. |

## Browser

The parity fixture garden published from this branch (`lsg publish --graph tests/fixtures/parity-garden --include Tools --journals`), served by a local server that could hold graph files for 30 seconds or answer them with 503. Chromium at 1280×800, light and dark system themes.

- `graph.html`: the spinner, `Loading the graph…` and Cancel appeared after half a second; Cancel left `The graph was not loaded.` with focus on `Load the graph`; Enter on it loaded and drew the graph, showed the panel and moved focus to the canvas. Turning on Journals with the graph drawn showed the same states over the dimmed graph; a 503 showed `The graph could not be loaded.` with Retry; Retry drew the journals graph with 15 pages.
- Page graph card on `Tools/tmux`: the same states in the card; after Cancel the browser had requested only the neighbourhood file; `Load the page graph` drew the neighbourhood.
- Measured styles: the button is 28px high with a 1px `--lsg-border` border, 4px radius, `--lsg-background` and 14px weight-500 text; the message is 14px in `--lsg-muted`; the canvas is at 0.25 opacity while a message shows.

## Checks

`mise run ci` passed locally: format, Clippy, the workspace tests and the 132 browser-script tests, release build, `parity:budgets` (the fixture site's graph script, styles and sidebar script within their budgets), OpenSpec validation, parser checks, RuleSync, notices, docs, clilint and release checks.
