## MODIFIED Requirements

### Requirement: CLI stylesheet selection and guidance

`lsg publish` SHALL accept optional native-path `--stylesheet PATH` and delegate to SDK options. Fatal stylesheet errors SHALL exit one with empty stdout. Offline publishing help and the `lsg/How To/Customize a Published Site` page SHALL describe the option, validation, stylesheet precedence, and relative-resource behavior. Help SHALL reflect implemented alias, block-reference, and HTTPS media support.

#### Scenario: CLI custom appearance

- **WHEN** a caller supplies a valid stylesheet flag
- **THEN** publishing succeeds with the selected stylesheet and preserves existing human and JSON report fields

#### Scenario: Discover stylesheet use offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the optional stylesheet flag and its behavior are available with current rendering limitations
