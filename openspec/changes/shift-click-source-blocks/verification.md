# Verification: shift-click-source-blocks

Shift-click on a crumb or a bullet in Linked References adds a right sidebar card for its block, on a Gitpa build whose pages are served at readable routes.

Checked on 2026-11-01 with a Gitpa preview built from [codekiln/gitpa `fb98a28` build: install Logseq Gardener v0.0.5](https://github.com/codekiln/gitpa/commit/fb98a283f) by `python3 scripts/build_site.py --renderer <this branch's lsg> --output …/gitpa`, served at `http://localhost:8794/gitpa/` and opened in a 1280×800 browser tab with an empty `logseq-gardener-right-sidebar` entry.

| Action | Result |
| --- | --- |
| On `/gitpa/Microfreak/`, expand Linked References and Shift-click the crumb `Can I record two tracks in Ableton/Arrangement View while I play and record` | Block references card showing that block and its children; saved as `refs`, route `Ableton/Arrangement%20View/Q/Can%20I%20record%20in%20Arrangement%20View%20while%20playing%20Session%20View%20clips%3F/`, block `source-block-58`; URL unchanged |
| Shift-click the bullet of `Recording a Microfreak on a MIDI track…` below that crumb | Block card whose title is the breadcrumb `Ableton/Arrangement View/Q/Can I record… › Can I record two tracks…` and whose body is that block; saved as `block`, block `source-block-174`; URL unchanged |
| Open `/gitpa/Ableton/` | Both cards restored with the same titles and content |
| On `/gitpa/Microfreak/`, plain click on the same crumb | Browser opens the source page at `#source-block-58`, which is that block |

The crumb was clicked with the browser pane's own Shift-click. The bullet was clicked with a Shift-click `MouseEvent` dispatched on it from the page, because the pane could not place a click on the 14-pixel bullet after the sidebar opened. Both go through the same `lsg-shell.js` listener.

Before this change, on the live Gitpa site, Shift-click on the same crumb opened a new browser window, and Shift-click on the bullet was cancelled with no card, as [Issue #317](https://github.com/codekiln/logseq-gardener/issues/317) reports.

`tests/shell.test.cjs` and `tests/sidebar.test.cjs` cover the same forms with a fixture host at `/gitpa/`. Both new tests fail against the scripts on `main` and pass here.
