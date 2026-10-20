## Purpose

Define local CSS customization for selected static publications through the SDK and CLI.

## Requirements

### Requirement: Optional stylesheet publication

The SDK SHALL offer defaultable site options and an options-aware publishing function while preserving the existing `publish_site` API and default output. When a stylesheet is supplied, it SHALL copy the contents unchanged to `custom.css` and include an escaped relative stylesheet link after the built-in style in the index and every published page. Graph asset counts and publication selection SHALL remain unchanged.

#### Scenario: Styled selected publication

- **WHEN** a caller publishes selected pages with a local stylesheet
- **THEN** every generated HTML document loads `custom.css` after built-in CSS, the copied contents match the input, excluded and private pages stay withheld, and source files remain unchanged

#### Scenario: Default compatibility

- **WHEN** callers publish with the existing API or default options
- **THEN** their generated files and reports match and no custom stylesheet is emitted

### Requirement: Stylesheet preflight

The publisher SHALL require a readable regular local file with a case-insensitive `.css` extension and reject symlink components. It SHALL validate and read this file before creating output. Invalid stylesheet input SHALL produce a path-specific error and leave the destination absent.

#### Scenario: Invalid stylesheet

- **WHEN** the stylesheet is missing, unreadable, a directory, has another extension, or includes a symlink component
- **THEN** publication fails before creating the destination and identifies the stylesheet problem

### Requirement: CLI stylesheet selection and guidance

`lsg publish` SHALL accept optional native-path `--stylesheet PATH` and delegate to SDK options. Fatal stylesheet errors SHALL exit one with empty stdout. Offline publishing help and the local-site guide SHALL describe the option, validation, stylesheet precedence, and relative-resource behavior. Help SHALL reflect implemented alias, block-reference, and HTTPS media support.

#### Scenario: CLI custom appearance

- **WHEN** a caller supplies a valid stylesheet flag
- **THEN** publishing succeeds with the selected stylesheet and preserves existing human and JSON report fields

#### Scenario: Discover stylesheet use offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the optional stylesheet flag and its behavior are available with current rendering limitations
