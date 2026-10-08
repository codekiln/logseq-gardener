## ADDED Requirements

### Requirement: Explicit SDK-backed publishing

lsg publish SHALL accept required --graph and --output native filesystem paths, required --filename-format legacy|triple-lowbar, and repeated UTF-8 --include and --exclude namespace roots. It SHALL delegate selection and publication to the SDK, preserving its exclusion and fresh-output protections.

#### Scenario: Publish selected content

- **WHEN** a caller publishes a garden containing selected links and assets plus excluded page, block and asset sentinels
- **THEN** the output contains selected navigation and assets, excludes sentinel content, and all source bytes remain unchanged

#### Scenario: Native filesystem arguments

- **WHEN** a caller supplies a native OS path that is not UTF-8
- **THEN** publication uses that path successfully and reports a display representation

### Requirement: Publication reports and diagnostics

Successful human and JSON output SHALL report pages, assets, withheld pages, skipped journals, diagnostic count, and an index location. JSON SHALL contain format_version 1 and command_path [publish]. Unsupported selected constructs SHALL produce local stderr diagnostics and success. Fatal SDK failures SHALL return one with empty stdout and an actionable stderr error; invalid selectors SHALL return two.

#### Scenario: Unsupported content

- **WHEN** generation completes with unresolved references
- **THEN** the success report remains on stdout, diagnostics stay on stderr, and the exit is zero

#### Scenario: Fatal generation errors

- **WHEN** the source is missing, output exists or lies inside the garden, or selected titles are ambiguous
- **THEN** publication returns one with empty stdout and explains the SDK failure on stderr

### Requirement: Offline publishing help

The root help SHALL discover publish, and publish SHALL provide equivalent help, --help and -h plus outline, section and programmatic guidance without requiring paths or reading the garden. Help SHALL describe permissions, fresh output, selector semantics, filename format, output/exit codes and SDK limitations.

#### Scenario: Discover publishing offline

- **WHEN** a caller recursively traverses JSON help with closed stdin
- **THEN** publish documentation is discoverable and equivalent through each help alias
