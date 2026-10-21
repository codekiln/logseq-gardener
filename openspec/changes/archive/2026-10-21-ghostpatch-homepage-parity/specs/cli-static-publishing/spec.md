## MODIFIED Requirements

### Requirement: Explicit SDK-backed publishing

lsg publish SHALL accept required --graph and --output native filesystem paths, required --filename-format legacy|triple-lowbar, and repeated UTF-8 --include and --exclude namespace roots. It SHALL accept an optional UTF-8 --homepage logical page selector and pass it to the SDK. It SHALL delegate selection and publication to the SDK, preserving its exclusion and fresh-output protections.

#### Scenario: Publish selected content

- **WHEN** a caller publishes a garden containing selected links and assets plus excluded page, block and asset sentinels
- **THEN** the output contains selected navigation and assets, excludes sentinel content, and all source bytes remain unchanged

#### Scenario: Native filesystem arguments

- **WHEN** a caller supplies a native OS path that is not UTF-8
- **THEN** publication uses that path successfully and reports a display representation

#### Scenario: Publish a selected homepage
- **WHEN** a caller supplies --homepage identifying a unique selected visible page title or alias
- **THEN** the SDK generates that page as index.html with a separate page index and retains ordinary routes

### Requirement: Offline publishing help

The root help SHALL discover publish, and publish SHALL provide equivalent help, --help and -h plus outline, section and programmatic guidance without requiring paths or reading the garden. Help SHALL describe permissions, fresh output, selector semantics, filename format, homepage selection and page-index navigation, output/exit codes and SDK limitations.

#### Scenario: Discover publishing offline

- **WHEN** a caller recursively traverses JSON help with closed stdin
- **THEN** publish documentation is discoverable and equivalent through each help alias
