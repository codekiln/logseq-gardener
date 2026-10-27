## Purpose

Define accessible, lazy visitor search and persistent-host integration.

## Requirements

### Requirement: Accessible persistent search dialog

Search-enabled publication SHALL provide a labelled header button and Cmd/Ctrl+K shortcut. A native modal dialog SHALL provide a labelled input, useful result/status announcements, arrow/Enter navigation, Escape/Tab behavior and focus restoration. Results SHALL render safe text and real anchors preserving modified/new-tab navigation. The dialog SHALL persist outside main and repeated initialization SHALL install one set of listeners/controls. Without usable JavaScript/native dialog support the trigger SHALL remain hidden.

#### Scenario: Keyboard search and dismissal

- **WHEN** a visitor opens search with Cmd/Ctrl+K, types, navigates results and presses Escape until the dialog closes
- **THEN** the input/results are operable and focus returns to the invoking control

#### Scenario: Persistent host navigation

- **WHEN** a host replaces main, updates expected search metadata and calls the initializer repeatedly
- **THEN** one dialog remains usable with current publication identity and native result navigation

#### Scenario: Pointer dismissal

- **WHEN** a visitor clicks or taps outside the dialog
- **THEN** the dialog closes and focus returns to the invoking control

### Requirement: Lazy validated search loading

The controller SHALL fetch no search data or evaluator before the visitor opens search or points at or focuses the search button. It SHALL then fetch the manifest and title index, and on each query only the shards and record chunks its words need, validating same-origin paths, responses, version, revision, digests and shape. Fetched files SHALL be cached by publication identity. Close/retry/revision changes SHALL abort or invalidate stale work; loading, empty, no-results and error/retry states SHALL stay distinct, with no automatic reload.

#### Scenario: Closing a slow first load

- **WHEN** a visitor closes search before its fetch completes
- **THEN** the request is aborted or its result ignored and a later open can load successfully

#### Scenario: Publication metadata changes

- **WHEN** the current page advertises a new or missing expected revision
- **THEN** old pending/cached results are invalidated and unusable controls are hidden

#### Scenario: A keystroke fetches only its shards

- **WHEN** the title index is loaded and a visitor types `tm` and then `tmux`
- **THEN** the controller fetches the `tm` shard once, fetches record chunks only for block results it shows, and fetches no other shard

### Requirement: Host route and theme integration

Default result URLs SHALL resolve beside the advertised manifest. Hosts SHALL be able to resolve canonical page titles to readable routes, with original fragments retained. Normal result activation SHALL close before bubbling to delegated navigation. Shared appearance variables SHALL style the dialog on desktop/mobile and light/dark/system themes.

#### Scenario: Search from a nested hosted page

- **WHEN** a default result is activated from a nested page or a host supplies a readable-route resolver
- **THEN** the link reaches the source destination and the host can preserve player/theme state

### Requirement: Logseq search dialog look

The dialog SHALL use Logseq's published layout: 896px wide at most (90% of the viewport below that), 8px radius, no visible title or close button, a 20px borderless input with the placeholder `What are you looking for?` on a grey row with a bottom border, results filling at most 65% of the viewport height, and a footer with one tip and the keys for the highlighted row. The input SHALL keep an accessible name.

#### Scenario: Computed styles

- **WHEN** the dialog opens at 1280×800 in the light theme
- **THEN** its width is 896px, the input's font size is 20px with no border, and the placeholder reads `What are you looking for?`

### Requirement: Grouped results

Results SHALL appear in the groups Filters, Pages, Commands, Current page and Blocks, in that order, leaving out empty groups. Each group header SHALL show the group name and its match count, shown as `99+` from 100. Blocks on the page being viewed SHALL appear under Current page and not under Blocks.

#### Scenario: Pages before blocks

- **WHEN** a reader searches `tmux` on the home page
- **THEN** a Pages group with `99+` comes before a Blocks group with `99+`

#### Scenario: Current page blocks

- **WHEN** a reader searches a word that occurs in blocks of the page being viewed
- **THEN** those blocks appear under Current page and are absent from Blocks

### Requirement: Five rows and Show more

Each group SHALL show five rows unless it is expanded or a filter is set. The group holding the highlighted row SHALL show `Show more ⌘ ↓` or `Show less ⌘ ↑` in its header (`Ctrl` in place of `⌘` off Apple systems); `mod+Down`, `mod+Up` and a click on the group name SHALL expand or collapse it.

#### Scenario: Expanding Pages

- **WHEN** a reader searches `tmux` and presses `mod+Down` with a page row highlighted
- **THEN** the Pages group shows all its results and its header reads `Show less ⌘ ↑`

### Requirement: Result rows

Page rows SHALL show the page title and, for an alias match, `alias of <page>`. Block rows SHALL show a breadcrumb of the page title and up to three parents above the block text. Each query word SHALL be highlighted case-insensitively in titles, block text and breadcrumbs with Logseq's highlight colour in the light theme and a theme token in the dark theme.

#### Scenario: Alias and highlight

- **WHEN** a reader searches `tmux`
- **THEN** every `tmux` in the shown rows is highlighted, and `tmux.conf.local` shows `alias of tmux/Config/local`

#### Scenario: Block breadcrumb

- **WHEN** a block result is shown
- **THEN** its first line names its page

### Requirement: Slash filters

Typing `/` SHALL show a Filters group with `Search only pages`, `Search only current page`, `Search only blocks` and `Search only commands`, narrowed by the text after `/`. Choosing one SHALL remove the `/…` text, show a `Search only: <kind>` chip with a remove button, and list every result of that kind. Escape or Backspace on an empty input SHALL remove the filter.

#### Scenario: Pages filter

- **WHEN** a reader types `/`, chooses `Search only pages`, then types `tmux`
- **THEN** only the Pages group shows, with every matching page

### Requirement: Published commands

The Commands group SHALL list the published site's actions: toggle the theme, toggle the left sidebar, toggle wide mode, and go to the home page, All pages and journals when the site has them. Commands SHALL show on an empty input and when their names match the query. Enter SHALL run the highlighted command and close the dialog.

#### Scenario: Empty input

- **WHEN** a reader opens search without typing
- **THEN** the dialog shows the Commands group and no other group

### Requirement: Keyboard across groups

Up and Ctrl+P, Down and Ctrl+N SHALL move the highlight through every visible row across groups and wrap at the ends. Enter SHALL open the highlighted page or block, run a command or apply a filter. Escape SHALL clear the input, then remove the filter, then close the dialog.

#### Scenario: Wrapping

- **WHEN** the last visible row is highlighted and the reader presses Down
- **THEN** the first visible row is highlighted
