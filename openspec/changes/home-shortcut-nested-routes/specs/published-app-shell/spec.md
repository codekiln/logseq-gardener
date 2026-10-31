## MODIFIED Requirements

### Requirement: Handle Logseq's shell shortcuts

Generated pages SHALL handle `t l` (toggle left sidebar), `t r` (toggle right sidebar, at 640px and wider), `t t` (toggle theme), `t w` (toggle wide mode), `g h` (open `index.html` in the site root, the directory `lsg-shell.js` loads from, from any page address), `g a` (open All pages), `g g` (open the graph view) unless `config.edn` sets `:shortcuts {:go/graph-view false}`, `alt+shift+c` (toggle the Contents card, at 640px and wider), `g f` and `t c` (open or close the flashcard review) unless flashcards are turned off, and, when a journals feed is written, `g j` (open the feed's first batch), with two-key shortcuts recognized when the second key follows the first within one second. Shortcuts SHALL be ignored while focus is in an input, textarea, select or editable element, or while Control or Meta is held, and the two-key shortcuts also while Alt is held.

#### Scenario: Shortcut outside a field
- **WHEN** a reader on `tmux` presses `g` then `a`
- **THEN** the browser opens the complete page index

#### Scenario: Home from a nested route
- **WHEN** a host serves a page from the directory route `/gitpa/GitP/A/Session/26/10/29-Thu/`, the page loads `/gitpa/lsg-shell.js`, and a reader presses `g` then `h`
- **THEN** the browser opens `/gitpa/index.html`

#### Scenario: Shortcut inside a field
- **WHEN** focus is in a backlink filter input and the reader types `t t`
- **THEN** the input receives the text and the appearance stays the same

#### Scenario: Open journals
- **WHEN** the encode garden is published with journals and a reader on `tmux` presses `g` then `j`
- **THEN** the browser opens `index.html` showing the feed

#### Scenario: Toggle the right sidebar
- **WHEN** a reader at 1280px on `tmux` presses `t` then `r`
- **THEN** the right sidebar opens

#### Scenario: Open the graph view
- **WHEN** a garden whose `config.edn` leaves `:go/graph-view` unset is published and a reader on `tmux` presses `g` then `g`
- **THEN** the browser opens `graph.html`

#### Scenario: Graph shortcut turned off
- **WHEN** the encode garden, whose `config.edn` sets `:shortcuts {:go/graph-view false}`, is published and a reader presses `g` then `g`
- **THEN** nothing happens and the Graph view item shows no `g g` hint

#### Scenario: Open the flashcard review
- **WHEN** a reader on `tmux` presses `g` then `f`
- **THEN** the review of all cards opens, and pressing `g` then `f` again closes it
