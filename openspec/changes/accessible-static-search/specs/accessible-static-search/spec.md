## ADDED Requirements

### Requirement: Accessible persistent search dialog

Search-enabled publication SHALL provide a labelled header button and Cmd/Ctrl+K shortcut. A native modal dialog SHALL provide a labelled input, useful result/status announcements, arrow/Enter navigation, Escape/Tab behavior and focus restoration. Results SHALL render safe text and real anchors preserving modified/new-tab navigation. The dialog SHALL persist outside main and repeated initialization SHALL install one set of listeners/controls. Without usable JavaScript/native dialog support the trigger SHALL remain hidden.

#### Scenario: Keyboard search and dismissal

- **WHEN** a visitor opens search with Cmd/Ctrl+K, types, navigates results and presses Escape
- **THEN** the input/results are operable and focus returns to the invoking control

#### Scenario: Persistent host navigation

- **WHEN** a host replaces main, updates expected search metadata and calls the initializer repeatedly
- **THEN** one dialog remains usable with current publication identity and native result navigation

### Requirement: Lazy validated search loading

The controller SHALL fetch no corpus/manifest or evaluator before first opening. It SHALL validate same-origin static artifact paths, successful responses, version/revision/shape and corpus UTF-8 length before accepting results. Prepared data SHALL cache by publication identity. Close/retry/revision changes SHALL abort or invalidate stale loading/query work. Loading, empty, no-results and actionable error/retry/reload states SHALL remain distinct, and the controller SHALL never reload automatically.

#### Scenario: Closing a slow first load

- **WHEN** a visitor closes search before its fetch completes
- **THEN** the request is aborted or its result ignored and a later open can load successfully

#### Scenario: Publication metadata changes

- **WHEN** the current page advertises a new or missing expected revision
- **THEN** old pending/cached results are invalidated and unusable controls are hidden

### Requirement: Host route and theme integration

Default result URLs SHALL resolve beside the advertised manifest. Hosts SHALL be able to resolve canonical page titles to readable routes, with original fragments retained. Normal result activation SHALL close before bubbling to delegated navigation. Shared appearance variables SHALL style the dialog on desktop/mobile and light/dark/system themes.

#### Scenario: Search from a nested hosted page

- **WHEN** a default result is activated from a nested page or a host supplies a readable-route resolver
- **THEN** the link reaches the source destination and the host can preserve player/theme state
