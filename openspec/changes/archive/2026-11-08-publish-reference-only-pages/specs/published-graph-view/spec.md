## MODIFIED Requirements

### Requirement: Pan, zoom, open and select nodes

Dragging the background SHALL pan, and the wheel and pinch SHALL zoom about the pointer. Clicking a node with a route SHALL open its page; clicking a node without a route SHALL show its title as `not published` and open nothing. Shift-click or dragging a node SHALL select it, mark it, colour its neighbours and their edges, and show the `N hops from selected nodes` slider (1 to 10), which keeps only nodes within N hops of the selection. The canvas SHALL be focusable, arrow keys SHALL pan and `+` and `-` SHALL zoom, and the Search section's matching node list SHALL be the keyboard path to opening and selecting nodes. Touch SHALL pan, pinch-zoom, open with a tap and select with a long press.

#### Scenario: Open a page

- **WHEN** a reader clicks the `tmux` node
- **THEN** the browser opens `tmux`'s page

#### Scenario: N hops

- **WHEN** a reader shift-clicks `tmux` and sets N hops to 1
- **THEN** only `tmux` and its neighbours remain drawn

#### Scenario: Keyboard

- **WHEN** a keyboard reader types `tmux` in the Search section and presses Enter on the `tmux` result
- **THEN** the browser opens `tmux`'s page

#### Scenario: Open a reference-only page

- **WHEN** a reader clicks the `Deno Deploy` node, a reference-only node with a route
- **THEN** the browser opens the `Deno Deploy` page

#### Scenario: Node without a page

- **WHEN** a reader clicks the `TODO` node, which has no route
- **THEN** the graph shows `TODO — not published` and the page stays open
