## Context

`graph.js` serves two views. `globalGraph` runs on `graph.html` and fetches `_graph/default.bin`, or `_graph/journals.bin` once a reader turns on Journals. `pageGraph` runs in the right sidebar's Page graph card: it fetches the page's neighbourhood file `_graph/p/<hash>.json` and lays it out in the graph worker, or, when that file is missing for a hub page, fetches the global file and cuts the neighbourhood out of it. `sidebar.js` loads `lsg-graph.js` when the first Page graph card opens and calls `window.logseqGardenerGraph.page(body)`.

Today both views write `Loading the graph…` or `Loading the page graph…` into a small status line at the bottom left of the canvas, with no spinner and no way to stop. A failed first load on `graph.html` says to reload the page. The Journals switch has no failure path: a failed `journals.bin` leaves `Loading the graph…` on screen and an unhandled rejection. The data cache keeps a rejected promise, so a second attempt would fail without a request. In the card, any failure of the neighbourhood fetch, including a network failure, falls back to downloading the much larger global file. If `lsg-graph.js` fails to load, the card is empty.

Logseq's graph view shows `Preparing` centred over the graph area at 14px and 75% opacity, and on failure `Graph failed to load.` in the secondary text colour with an outlined small `Retry` button below, 12px apart ([`graph.cljs` lines 385–392 and 731–754](https://github.com/logseq/logseq/blob/63b76c5db4/src/main/frontend/components/graph.cljs#L385-L392), [`graph.css` lines 454–486](https://github.com/logseq/logseq/blob/63b76c5db4/src/main/frontend/extensions/graph.css#L454-L486)). Its general loading indicator is a 20px spinning ring, a 25%-opacity track with a 75%-opacity arc, beside the text with 8px between them ([`ui.cljs` lines 622–628](https://github.com/logseq/logseq/blob/63b76c5db4/src/main/frontend/ui.cljs#L622-L628), [`svg.cljs` lines 15–25](https://github.com/logseq/logseq/blob/63b76c5db4/src/main/frontend/components/svg.cljs#L15-L25)). An outlined small button is 28px high with 12px side padding, a 1px border, a 4px radius, 14px medium text and the background colour ([`button.tsx` lines 8–37](https://github.com/logseq/logseq/blob/63b76c5db4/packages/ui/@/components/ui/button.tsx#L8-L37)). Logseq offers no Cancel.

## Goals / Non-Goals

**Goals:**

- Show readers of both views that the graph is loading, let them stop it, and let them load it again after a cancel or a failure, without reloading the page.

**Non-Goals:**

- Stop the Page graph's download when the reader closes or collapses the card. The download finishes in the background and draws into a detached card, as it does today.

## Decisions

### One loading overlay for both views

`graph.js` gains `progress(graph, what, task)`, which appends `div.lsg-graph-loading` to the graph's box and returns the function that starts a load: `fetchGraph` in the graph view, `fetchPage` in the card. The overlay holds a `p[role=status]` containing a spinner and a text span, then one `button`. It stays in the page while empty, so the live region exists before any message and screen readers announce each change. The box carries `data-lsg-load` set to `loading`, `cancelled` or `failed`, or no attribute when idle; CSS shows the spinner only while loading and dims the canvas and its labels while the attribute is present. `globalGraph` passes `graph` and `pageGraph` passes `page graph`, which produce every message and button label.

The old status line at the bottom left stays for messages about a drawn graph, such as `<title> — not published` and `This page has no page graph.`. Loading and failure messages move to the overlay.

### Delay, then show at once

Each load starts a 500 ms timer before showing `Loading the …` and `Cancel`. A load that ends first clears the timer, so nothing appears. When the overlay already shows a message, because the reader pressed its button or flipped a Journals switch after a cancel or failure, the loading state replaces it at once: hiding a visible message for 500 ms and showing it again would be the flash the delay avoids.

### Abort through one controller per load

Each load creates an `AbortController`, aborts the previous load's controller, and passes the signal to the task. `fetch` takes the signal, so Cancel stops the network request and the body stream. The load races the task against the signal's abort, so Cancel takes effect at once even during the worker's layout or a decode, and the task checks the signal before drawing so a late result never replaces the current graph. A load whose controller is no longer current changes nothing on screen.

`globalGraph` caches each data file's promise so turning Journals off draws the default graph without a request. The cache now deletes an entry whose promise rejects, so Retry and `Load the graph` send a new request.

### Fall back to the global file only for a missing file

The fetch helper `load` marks an error from a non-OK HTTP response with its status. The card falls back to the global file only for such an error, which is how a hub page's missing neighbourhood file arrives; the status differs by host (404 on GitHub Pages, 403 on some object stores), so any non-OK status counts. A network failure or a cancel rethrows, so the card shows its message instead of starting a larger download. The same rule governs the card's fallback from `journals.bin` to `default.bin` for a site built without journals.

### Panel and focus

The graph view's settings panel appears with the first drawn graph instead of before the download. Every panel control acts on the drawn graph, so after a cancelled first load there is nothing for them to act on. `Reset Forces` resets the drawn graph instead of fetching the file for the current switch, so a press during a Journals load cannot draw the other graph.

The overlay reuses one button element for `Cancel`, `Load the …` and `Retry`, so focus stays on it as the state changes. When the graph draws while focus is inside the overlay, focus moves to the canvas, which is the next control and announces its label. A click whose `detail` is above 1 is ignored, so a double-click on `Load the graph` does not cancel the load it just started.

### Look

The overlay follows Logseq's graph states: a column centred over the canvas with 12px gaps, the 20px ring and the message in a row 8px apart at 14px in `--lsg-muted`, and the button with Logseq's outlined small size, `--lsg-border`, `--lsg-background`, `--lsg-text`, and `--lsg-surface` on hover. The ring is two CSS borders on pseudo-elements at Logseq's 25% and 75% opacity, rotating once a second; CSS draws it with fewer lines than building an SVG in script. Every colour comes from `theme.css`, so both themes and a garden's custom stylesheet apply. The overlay sits below the settings panel.

### Script failure in the sidebar

`sidebar.js` already finishes waiting for a script when it fails to load. When the wait ends without `window.logseqGardenerGraph`, the Page graph card shows `The page graph could not be loaded.` and a `Retry` button, and forgets the cached script promise. `Retry` moves focus to the card header and refills the card, which appends a new script element. The button uses the sidebar's existing outlined button style under its own class, because the existing class closes the card.

### Tests

`tests/graph.test.cjs` runs `graph.js` in a VM with `tests/mini-dom.cjs`, a fake clock for the 500 ms timer, a fetch stub that holds each request until the test answers or the signal aborts, and a fake worker that returns positions. It covers the fast load, the delayed indicator, Cancel and reload, failure and Retry, and the Journals switch on `graph.html`, and the same cases in the Page graph card, plus Cancel during its layout and the hub fallback rule. `tests/sidebar.test.cjs` covers the script failure and Retry. `mini-dom.cjs` learns the `$=` attribute selector that `graph.js` uses to find its stylesheet.

## Risks / Trade-offs

[A load that ends just after 500 ms shows the indicator briefly] → The delay removes the flash for loads under half a second, which covers the Page graph's 300 ms desktop budget. A minimum display time would remove the rest but adds a second timer; this change does without it.

[Cancel during decode cannot stop the decode] → Decoding runs on the main thread in one step after the download. The race makes the overlay answer at once and the late result is dropped.

[A cancelled card download keeps the worker running] → The worker stops on its own when the layout settles, and the card ignores its messages once the signal aborts.

## Resolved Questions

### 1 - How long should the delay be?

500 ms. Loads within the Page graph's desktop budget (300 ms) never show the indicator, and a reader on a slow connection sees it within half a second, well inside the one second after which a wait feels interrupted.

### 2 - Should the card offer Cancel while `lsg-graph.js` downloads?

No. The script is small and the sidebar already shows its `Loading…` status for every card while content loads. The data files are the large downloads, and Cancel covers them.

### 3 - Which words should the messages use?

The site's existing sentences, `Loading the graph…` and `The graph could not be loaded.`, with Logseq's `Retry`. Logseq's `Preparing` and `Graph failed to load.` say less about what is happening, and Logseq has no Cancel or not-loaded message to copy.

### 4 - What does a reader see behind a message when a graph is already drawn?

The drawn graph at 25% opacity. Clearing it would lose the reader's place, and drawing it at full opacity would make the message hard to read and suggest the graph matches the switch.

## Open Questions

None.
