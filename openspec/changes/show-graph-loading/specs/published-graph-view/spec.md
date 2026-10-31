## ADDED Requirements

### Requirement: Show the graph's loading state

While `graph.html` downloads, decodes or draws a graph data file, the graph area SHALL show, once the load has taken 500 ms, a spinner, the status `Loading the graph…` and a `Cancel` button, centred over the canvas. A load that finishes within 500 ms SHALL show none of them. The status SHALL sit in a live region that is in the page before the load starts, so screen readers announce each message. `Cancel` SHALL abort the download and leave `The graph was not loaded.` with a `Load the graph` button. A failed download SHALL leave `The graph could not be loaded.` with a `Retry` button. Either button SHALL start the load again at once, showing the spinner and `Cancel` without the delay, and SHALL download the file again rather than reuse the failed or cancelled one.

Turning Journals on or off SHALL load through the same states. A newer load SHALL replace an older one, so the drawn graph always matches the Journals switch once loading ends. While a message covers a graph that is already drawn, that graph SHALL stay visible at reduced opacity. The settings panel SHALL appear with the first drawn graph. When a drawn graph replaces the messages while focus is on their button, focus SHALL move to the canvas.

The spinner, message and button SHALL follow Logseq's graph loading and error states: a 20px spinner beside the text, 14px text in the muted colour, and a 28px outlined button below, using the site's colour tokens in both themes.

#### Scenario: Fast load

- **WHEN** `_graph/default.bin` arrives and the first frame is drawn within 500 ms of opening `graph.html`
- **THEN** no spinner, status text or button appears

#### Scenario: Slow load

- **WHEN** `_graph/default.bin` has not arrived 500 ms after opening `graph.html`
- **THEN** the graph area shows the spinner, `Loading the graph…` and `Cancel`, and a screen reader announces `Loading the graph…`

#### Scenario: Cancel and load

- **WHEN** a reader presses `Cancel` while the graph loads, then presses `Load the graph`
- **THEN** the download stops, the graph area says `The graph was not loaded.`, and the second press downloads `_graph/default.bin` again and draws it

#### Scenario: Failure and retry

- **WHEN** the download of `_graph/default.bin` fails and the reader presses `Retry`
- **THEN** the graph area says `The graph could not be loaded.` until the press, then downloads the file again and draws it

#### Scenario: Cancel the journals graph

- **WHEN** a reader turns on Journals, cancels the download of `_graph/journals.bin`, then turns Journals off
- **THEN** the default graph stays drawn, dimmed, behind `The graph was not loaded.`, and turning Journals off draws it at full opacity without a new download
