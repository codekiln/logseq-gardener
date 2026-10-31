## ADDED Requirements

### Requirement: Show the Page graph's loading state

The Page graph card SHALL show the graph view's loading, not-loaded and failure states, with the same delay, controls, focus and look, using `Loading the page graph…`, `The page graph was not loaded.` with `Load the page graph`, and `The page graph could not be loaded.` with `Retry`. The states SHALL cover downloading the page's neighbourhood file, downloading the global file it falls back to, and the worker's layout before the first frame. Turning Show Journals on or off SHALL replace any load in progress. The card SHALL fall back to the global file only when the server answers that the neighbourhood file is missing; a network failure or a cancelled download SHALL show its message instead. If `lsg-graph.js` fails to load, the card SHALL say `The page graph could not be loaded.` with a `Retry` button that loads the script again.

#### Scenario: Slow neighbourhood file

- **WHEN** a reader opens the Page graph card on `Foam` and `Foam`'s neighbourhood file has not arrived after 500 ms
- **THEN** the card's graph area shows the spinner, `Loading the page graph…` and `Cancel`

#### Scenario: Cancel in the card

- **WHEN** a reader presses `Cancel` in the card, then `Load the page graph`
- **THEN** the download stops without fetching the global file, the card says `The page graph was not loaded.`, and the second press downloads the neighbourhood file again and draws it

#### Scenario: Failure and retry in the card

- **WHEN** the neighbourhood file cannot be downloaded because the network fails, and the reader presses `Retry`
- **THEN** the card says `The page graph could not be loaded.` without fetching the global file, and `Retry` downloads the neighbourhood file again and draws it

#### Scenario: Script fails to load

- **WHEN** a reader opens the Page graph card and `lsg-graph.js` fails to load
- **THEN** the card says `The page graph could not be loaded.`, and `Retry` loads the script again and draws the page graph
