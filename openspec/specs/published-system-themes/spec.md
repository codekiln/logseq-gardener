## Purpose

Let readers follow system appearance or choose Light or Dark on published sites, with safe browser preferences and accessible controls.

## Requirements

### Requirement: Follow system appearance and visitor overrides

Each generated document SHALL offer Light, Dark and System appearance. Without a valid saved Light or Dark override, it SHALL follow the system color preference and react to changes while open. A valid explicit choice SHALL take precedence over system changes until the visitor chooses System. The initial resolved appearance SHALL be applied before body content paints where JavaScript is available. Without JavaScript, shared CSS SHALL follow the system preference. If media queries are unavailable, the default SHALL be Light.

#### Scenario: Automatic appearance
- **WHEN** no valid saved override exists and the system requests dark colors
- **THEN** the document starts dark and changes to light when the system preference changes

#### Scenario: Explicit appearance
- **WHEN** a visitor chooses Light while the system requests dark colors
- **THEN** the document stays light through subsequent system changes until the visitor changes the choice

#### Scenario: Return to System
- **WHEN** a visitor with an explicit override chooses System
- **THEN** the current system appearance applies immediately and subsequent system changes apply

#### Scenario: No JavaScript
- **WHEN** JavaScript is disabled
- **THEN** shared CSS follows the system preference and the inactive appearance selector stays hidden

### Requirement: Persist safe browser preferences

The shared controller SHALL persist explicit Light and Dark choices in browser storage and remove the override for System. A missing or invalid value SHALL resolve to System. Reading or writing unavailable storage SHALL not interrupt content, controls or navigation; the current document SHALL retain its selected choice in memory. Theme selection SHALL not modify graph content, publication selection, queries or media playback.

#### Scenario: Reload and page navigation
- **WHEN** a visitor chooses Dark and opens another generated page or reloads
- **THEN** the page uses the saved Dark choice and its selector reflects that choice

#### Scenario: Invalid saved value
- **WHEN** saved theme data is unrecognized
- **THEN** the page follows System without interpreting the value as code or markup

#### Scenario: Storage unavailable
- **WHEN** browser storage throws while a visitor changes appearance
- **THEN** the current page applies the choice and remains usable, including internal navigation and media playback

### Requirement: Provide accessible shared controls and styling

Generated sites SHALL provide a labelled native Appearance selector with System, Light and Dark options, readable light/dark default palettes and visible keyboard focus. Shared palette variables SHALL let site styles supply their own colors. The shared script SHALL initialize normal loads and expose idempotent `window.logseqGardenerInitTheme(root)` for host-inserted controls. Reinitialization SHALL reflect the active choice without duplicate handlers, resetting appearance, or rebuilding page content. Shared hierarchy, disclosures, backlink filters and native media controls SHALL remain readable in both appearances and at mobile widths.

#### Scenario: Keyboard appearance selection
- **WHEN** a reader reaches Appearance using the keyboard
- **THEN** all three choices are available through native select controls and focus remains visible

#### Scenario: Replaced host navigation
- **WHEN** a host inserts new navigation and initializes its appearance controls
- **THEN** the controls show the existing choice and repeated initialization leaves one working change handler

#### Scenario: Shared content and site colors
- **WHEN** either appearance is active on a narrow page with hierarchy, disclosures and backlinks
- **THEN** shared text and controls remain readable without horizontal overflow and site palette variables can supply matching colors
