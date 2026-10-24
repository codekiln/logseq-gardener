## MODIFIED Requirements

### Requirement: Follow system appearance and visitor overrides

Each generated document SHALL offer Light, Dark and System appearance through the More menu and the `t t` shortcut. Toggle theme SHALL switch between light and dark and save the result as an explicit choice; when the current appearance comes from the system, it SHALL save the opposite of the system appearance. Use system theme SHALL appear in the More menu while an explicit choice is saved and SHALL remove it. Without a valid saved Light or Dark override, the document SHALL follow the system color preference and react to changes while open. A valid explicit choice SHALL take precedence over system changes until the visitor chooses Use system theme. The initial resolved appearance SHALL be applied before body content paints where JavaScript is available. Without JavaScript, shared CSS SHALL follow the system preference. If media queries are unavailable, the default SHALL be Light.

#### Scenario: Automatic appearance
- **WHEN** no valid saved override exists and the system requests dark colors
- **THEN** the document starts dark and changes to light when the system preference changes

#### Scenario: Explicit appearance
- **WHEN** a visitor whose system requests dark colors chooses Toggle theme
- **THEN** the document turns light and stays light through subsequent system changes until the visitor changes the choice

#### Scenario: Return to System
- **WHEN** a visitor with an explicit override chooses Use system theme
- **THEN** the current system appearance applies immediately, subsequent system changes apply, and Use system theme leaves the menu

#### Scenario: No JavaScript
- **WHEN** JavaScript is disabled
- **THEN** shared CSS follows the system preference and the inactive More menu stays hidden

### Requirement: Provide accessible shared controls and styling

Generated sites SHALL provide appearance through labelled More menu items, readable light/dark default palettes taken from Logseq's indigo theme, and visible keyboard focus. Every default text and control color pair SHALL have at least 4.5:1 contrast. Shared palette variables SHALL keep their `--lsg-*` names so site styles can supply their own colors. The shared script SHALL initialize normal loads and expose idempotent `window.logseqGardenerInitTheme(root)`, which SHALL initialize any host-inserted labelled native Appearance selector with System, Light and Dark options. Reinitialization SHALL reflect the active choice without duplicate handlers, resetting appearance, or rebuilding page content. Shared hierarchy, disclosures, backlink filters and native media controls SHALL remain readable in both appearances and at mobile widths.

#### Scenario: Keyboard appearance selection
- **WHEN** a reader opens More using the keyboard
- **THEN** Toggle theme, and Use system theme when a choice is saved, are reachable and focus remains visible

#### Scenario: Replaced host navigation
- **WHEN** a host inserts new navigation with a native Appearance selector and initializes it
- **THEN** the selector shows the existing choice and repeated initialization leaves one working change handler

#### Scenario: Shared content and site colors
- **WHEN** either appearance is active on a narrow page with hierarchy, disclosures and backlinks
- **THEN** shared text and controls remain readable without horizontal overflow and site palette variables can supply matching colors
