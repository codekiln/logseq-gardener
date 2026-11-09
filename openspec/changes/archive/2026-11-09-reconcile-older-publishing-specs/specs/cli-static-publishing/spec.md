## MODIFIED Requirements

### Requirement: Explicit SDK-backed publishing

lsg publish SHALL accept required --graph and --output native filesystem paths, required --filename-format legacy|triple-lowbar, and repeated UTF-8 --include and --exclude namespace roots. It SHALL accept an optional UTF-8 --homepage logical page selector, an optional --journals flag and an optional --include-all flag, and pass them to the SDK; the requirement "Publish every page and journal" defines --include-all. It SHALL delegate selection and publication to the SDK, preserving its exclusion and fresh-output protections.

#### Scenario: Publish selected content

- **WHEN** a caller publishes a garden containing selected links and assets plus excluded page, block and asset sentinels
- **THEN** the output contains selected navigation and assets, excludes sentinel content, and all source bytes remain unchanged

#### Scenario: Native filesystem arguments

- **WHEN** a caller supplies a native OS path that is not UTF-8
- **THEN** publication uses that path successfully and reports a display representation

#### Scenario: Publish a selected homepage
- **WHEN** a caller supplies --homepage identifying a unique selected visible page title or alias
- **THEN** the SDK generates that page as index.html with a separate page index and retains ordinary routes

#### Scenario: Publish journals
- **WHEN** a caller supplies --journals
- **THEN** the SDK publishes the garden's journals and their feed according to the published-journals capability

