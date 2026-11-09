## MODIFIED Requirements

### Requirement: Build the whole garden with lsg

`mise run parity:build` SHALL publish every page and journal of a garden with `lsg publish --include-all --journals`. It SHALL default to the encode garden resolved with `ghq` and accept another garden path. It SHALL write into a new directory for each run and record, in a settings file beside the output, the garden commit, the `lsg` version and every `lsg` option it passed. It SHALL leave the garden's files unchanged.

#### Scenario: Whole-garden build
- **WHEN** a contributor runs `mise run parity:build` with the encode garden checked out
- **THEN** a new output directory holds the `lsg` site for every page and journal, and the settings file names the garden commit, the `lsg` version and the options used

#### Scenario: Repeat run
- **WHEN** the task runs a second time
- **THEN** it writes a second new directory and leaves the first run's output in place

#### Scenario: Title taken from an alias
- **WHEN** the garden holds `pages/Solvespace.md` with `alias:: [[SolveSpace]]`
- **THEN** the build publishes the page as `SolveSpace`

#### Scenario: Title outside lsg's namespace rules
- **WHEN** a page's title has an empty or whitespace-only segment or a control character
- **THEN** `lsg` leaves that page unselected and the build finishes the rest
