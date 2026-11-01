## Purpose

Let readers of a published garden open Logseq's global graph from a file computed at build time, so the view draws in about a second and stays light on a phone.

## Requirements

### Requirement: Publish the graph view page

The publisher SHALL write `graph.html` inside the normal page shell, with a canvas filling the main column at `calc(100vh - 100px)` high and the settings panel at its top right. The page SHALL draw the default view: journals, built-in pages and excluded pages off and orphan pages left out, unless `:graph/settings` in `config.edn` sets other values. Without JavaScript the page SHALL show a message that the graph needs JavaScript and a link to All pages.

#### Scenario: Default view on the encode garden

- **WHEN** a reader opens `graph.html` on the encode garden
- **THEN** the canvas draws every node and edge of the SDK's default graph and the panel's Nodes section reads `N pages` with that node count

#### Scenario: No JavaScript

- **WHEN** a reader opens `graph.html` with JavaScript disabled
- **THEN** the page says the graph needs JavaScript and links to All pages

### Requirement: Lay out the graph at build time

The publisher SHALL compute node positions at build time with a Barnes–Hut force layout using Logseq's default forces (link distance 70, charge −600 with range 1 to 600 and theta 0.5, collision radius 26, centring 0.02) or the values `:graph/forcesettings` sets, from a fixed seed, so the same garden gives the same positions. It SHALL cache the positions by the graph's nodes, edges and forces and reuse them when those are unchanged. One layout SHALL cover every node of a journals setting, including orphan, built-in and excluded nodes, so that toggling those settings hides or shows nodes without moving the others.

#### Scenario: Stable layout

- **WHEN** the same garden commit is built twice
- **THEN** both builds write identical graph data files

#### Scenario: Cached layout

- **WHEN** the garden is built again after a change that adds no node or edge
- **THEN** the build reuses the cached positions instead of running the layout

### Requirement: Load graph data only on the graph page

The publisher SHALL write the journals-off graph to `_graph/default.bin` and the journals-on graph to `_graph/journals.bin`, each one compact binary file holding node titles, routes, flags, degrees, quantised positions and delta-coded edges. Only `graph.html` SHALL fetch them: `default.bin` when the page loads and `journals.bin` the first time a reader turns on Journals. No other page SHALL reference either file.

#### Scenario: Journals on

- **WHEN** a reader on `graph.html` turns on Journals
- **THEN** the page fetches `_graph/journals.bin` once and draws the journals-on graph

#### Scenario: Other pages

- **WHEN** a reader opens `tmux`
- **THEN** the browser fetches no file under `_graph/` and no graph script

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

### Requirement: Draw nodes and edges in Logseq's look

Each node SHALL be a circle of size `8 × max(1, ∛n)`, where n is its degree, coloured `#999` (dark theme `#93a1a1`), green for a page used as a tag (dark theme orange), and for a namespace parent one of Logseq's ten colours chosen by Logseq's hash of its name. Labels SHALL be the page's title at 12px in `rgba(0,0,0,0.8)` (dark theme `rgba(255,255,255,0.8)`). Edges SHALL be 1px `#cccccc` (dark theme the accent colour or `#094b5a`). Hovering a node SHALL draw it in the accent colour (`#6366F1` when none is set), its label on `rgba(238,238,238,1)` in `#333333`, and its edges in `#A5B4FC`. Labels SHALL be drawn for the hovered and selected nodes and for every node whose drawn size reaches 12px at the current zoom, so zooming in shows every label. The drawing SHALL follow the site's theme when the reader toggles it.

#### Scenario: Hub sizes

- **WHEN** `Person` has 1,025 edges and `Contrando` one
- **THEN** `Person` is drawn at size 80 and `Contrando` at size 8

#### Scenario: Namespace parent colour

- **WHEN** `Logseq/Entity` is a namespace parent
- **THEN** it is drawn in the colour Logseq's hash of `Logseq/Entity` picks from the ten, in both themes

#### Scenario: Theme toggle

- **WHEN** a reader on `graph.html` presses `t t`
- **THEN** nodes, labels and edges switch to the other theme's colours without reloading the data

### Requirement: Pan, zoom, open and select nodes

Dragging the background SHALL pan, and the wheel and pinch SHALL zoom about the pointer. Clicking a node with a route SHALL open its page; clicking a reference-only node SHALL show its title as `not published` and open nothing. Shift-click or dragging a node SHALL select it, mark it, colour its neighbours and their edges, and show the `N hops from selected nodes` slider (1 to 10), which keeps only nodes within N hops of the selection. The canvas SHALL be focusable, arrow keys SHALL pan and `+` and `-` SHALL zoom, and the Search section's matching node list SHALL be the keyboard path to opening and selecting nodes. Touch SHALL pan, pinch-zoom, open with a tap and select with a long press.

#### Scenario: Open a page

- **WHEN** a reader clicks the `tmux` node
- **THEN** the browser opens `tmux`'s page

#### Scenario: N hops

- **WHEN** a reader shift-clicks `tmux` and sets N hops to 1
- **THEN** only `tmux` and its neighbours remain drawn

#### Scenario: Keyboard

- **WHEN** a keyboard reader types `tmux` in the Search section and presses Enter on the `tmux` result
- **THEN** the browser opens `tmux`'s page

### Requirement: Offer Logseq's settings panel

`div.graph-filters` SHALL sit 16px from the top and right of the canvas, 200px wide, with the foldable sections Nodes, Search, Forces and Export in that order, all closed at first, each header a full-width button with `aria-expanded`. Nodes SHALL show `N pages` and the switches Journals, Orphan pages, Built-in pages and Excluded pages, `N hops from selected nodes` while a node is selected, and `Reset Graph`, which clears the selection, N hops and search filters. Each switch SHALL start in the state the drawn graph uses. Forces SHALL show `N links`, `Pause simulation`, and the sliders Link Distance (10 to 180, step 10), Charge Strength (−1000 to 1000, step 100) and Charge Range (500 to 4000, step 100), each showing its value, and `Reset Forces`. Export SHALL offer `as PNG`. Settings changes SHALL last for the tab's session on `graph.html` and SHALL leave the published files unchanged.

#### Scenario: Panel text

- **WHEN** a reader opens every section on the encode garden's `graph.html`
- **THEN** the labels, order, defaults and counts match Logseq's panel, except that Orphan pages starts off

#### Scenario: Journals count

- **WHEN** a reader turns on Journals
- **THEN** `N pages` changes to the node count of the journals-on graph

#### Scenario: Session settings

- **WHEN** a reader turns on Journals, opens `tmux` and goes back
- **THEN** `graph.html` shows the journals-on graph with Journals on

### Requirement: Filter nodes by title

The Search section's `Click to search` SHALL open a text field inside the panel. Entering text SHALL add a filter that keeps nodes whose title contains it, ignoring case; the section SHALL list each filter with a remove button and `Clear All`, and SHALL open by itself while filters are set. While the field has text, the section SHALL list up to 20 matching nodes as buttons that open the node's page on Enter or click and select it on Shift+Enter or Shift-click.

#### Scenario: Filter

- **WHEN** a reader adds the filter `tmux`
- **THEN** only nodes whose titles contain `tmux` remain drawn

#### Scenario: Clear All

- **WHEN** a reader with two filters presses `Clear All`
- **THEN** every node of the current settings is drawn again

### Requirement: Simulate forces only on demand

The first frame SHALL use the build-time positions. A force simulation SHALL start, in a Web Worker, only when a reader changes a force slider or drags a node; a node drag SHALL reheat it for two seconds. The simulation SHALL stop when it settles or when `Pause simulation` is on, and `Reset Forces` SHALL restore the default forces and the build-time positions. The worker script SHALL load only when the simulation first starts.

#### Scenario: Change a force

- **WHEN** a reader moves Link Distance to 120
- **THEN** the worker loads, nodes move apart, and the page keeps answering input within 100 ms

#### Scenario: First frame

- **WHEN** a reader opens `graph.html` and changes nothing
- **THEN** the browser loads no worker script

### Requirement: Export the view as PNG

`as PNG` SHALL save the canvas as drawn, at the device's pixel ratio, with the theme's background, as `graph.png`.

#### Scenario: Export

- **WHEN** a reader pans to `tmux` and chooses `as PNG`
- **THEN** the saved image shows the same view

### Requirement: Keep the graph view within budgets

The graph view SHALL meet the `[graph]` budgets in `scripts/parity/budgets.toml`, measured by the comparison harness on the whole encode garden: `_graph/default.bin` at most 600 KB gzip; `lsg-graph.js` with its styles at most 30 KB minified gzip, with the worker separate; the first frame with every node drawn within 1,000 ms on the desktop profile and 3,000 ms on the phone profile; a peak JS heap of at most 50 MB on the phone profile after the first frame, a pan and a zoom; a 95th-percentile frame time of at most 50 ms while panning on the phone profile; and a whole-garden build at most 25% slower than the `--baseline-lsg` binary's. `parity:budgets` SHALL check the sizes in CI and `parity:perf` SHALL mark the times, heap and build values.

#### Scenario: Budgets in CI

- **WHEN** `mise run ci` runs `parity:budgets` on the parity fixture build
- **THEN** it fails if the graph file or script exceeds its budget

#### Scenario: Measured against Logseq

- **WHEN** `parity:perf --record` runs on the whole encode garden
- **THEN** the `lsg/Ref/Logseq Compatibility/Performance` page gains graph rows for `lsg` and Logseq with the first frame, bytes, heap and frame time
