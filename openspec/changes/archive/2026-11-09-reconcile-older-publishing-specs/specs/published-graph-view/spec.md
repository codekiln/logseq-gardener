## MODIFIED Requirements

### Requirement: Load graph data only on the graph page

The publisher SHALL write the journals-off graph to `_graph/default.bin` and the journals-on graph to `_graph/journals.bin`, each one compact binary file holding node titles, routes, flags, degrees, quantised positions and delta-coded edges. `graph.html` SHALL fetch `default.bin` when the page loads and `journals.bin` the first time a reader turns on Journals. On any other page, only an open Page graph card SHALL fetch either file, as the published-page-graph requirement "Load neighbourhoods per page" defines; such a page SHALL fetch no graph file and no graph script while no Page graph card is open.

#### Scenario: Journals on

- **WHEN** a reader on `graph.html` turns on Journals
- **THEN** the page fetches `_graph/journals.bin` once and draws the journals-on graph

#### Scenario: Other pages

- **WHEN** a reader opens `tmux` with no Page graph card open
- **THEN** the browser fetches no file under `_graph/` and no graph script

