## Purpose

Define the command-line publishing inputs, reports, and offline help.

## Requirements

### Requirement: Explicit SDK-backed publishing

lsg publish SHALL accept required --graph and --output native filesystem paths, required --filename-format legacy|triple-lowbar, and repeated UTF-8 --include and --exclude namespace roots. It SHALL accept an optional UTF-8 --homepage logical page selector and an optional --journals flag and pass them to the SDK. It SHALL delegate selection and publication to the SDK, preserving its exclusion and fresh-output protections.

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

### Requirement: Publication reports and diagnostics

Successful human and JSON output SHALL report pages, published journals, assets, withheld pages, skipped journals, diagnostic count, and an index location. JSON SHALL contain format_version 1 and command_path [publish]. Unsupported selected constructs SHALL produce local stderr diagnostics and success. Fatal SDK failures SHALL return one with empty stdout and an actionable stderr error; invalid selectors SHALL return two.

#### Scenario: Unsupported content

- **WHEN** generation completes with unresolved references
- **THEN** the success report remains on stdout, diagnostics stay on stderr, and the exit is zero

#### Scenario: Fatal generation errors

- **WHEN** the source is missing, output exists or lies inside the garden, or selected titles are ambiguous
- **THEN** publication returns one with empty stdout and explains the SDK failure on stderr

#### Scenario: Journal counts
- **WHEN** a caller publishes a garden with three journals using --journals, one of them private
- **THEN** the report shows two published journals and one withheld page

### Requirement: Offline publishing help

The root help SHALL discover publish, and publish SHALL provide equivalent help, --help and -h plus outline, section and programmatic guidance without requiring paths or reading the garden. Help SHALL describe permissions, fresh output, selector semantics, filename format, homepage selection and page-index navigation, output/exit codes and SDK limitations.

#### Scenario: Discover publishing offline

- **WHEN** a caller recursively traverses JSON help with closed stdin
- **THEN** publish documentation is discoverable and equivalent through each help alias

### Requirement: Plugin selection from the CLI

`lsg publish` SHALL accept a repeatable native-path `--plugin PATH` naming a plugin manifest and pass the manifests to the SDK's site options. A manifest error, a failing build plugin or a rejected build contribution SHALL exit one with empty stdout, a message on stderr naming the manifest or plugin and the problem, and no output directory. Publishing help SHALL describe the option, the manifest fields including the `build` section, the `plugins/<id>/` output folder and where the lifecycle and the build protocol are documented.

#### Scenario: Publish with a plugin

- **WHEN** a caller runs `lsg publish` with a valid `--plugin player/lsg-plugin.json`
- **THEN** publishing succeeds, the report fields are unchanged, and the output contains `plugins/<id>/` and the plugin's entry in `lsg-site.json`

#### Scenario: Invalid plugin manifest

- **WHEN** a caller passes a manifest with an unsupported `apiVersion`
- **THEN** `lsg publish` exits one with empty stdout, names the manifest on stderr and creates no output directory

#### Scenario: Failing build plugin

- **WHEN** a caller passes a manifest whose build command exits with status 1
- **THEN** `lsg publish` exits one with empty stdout, names the plugin and the exit status on stderr and creates no output directory

#### Scenario: Discover plugins offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `--plugin` and its `build` section and links the publishing guide's plugin section

### Requirement: Optional publication timings

lsg publish SHALL accept an optional --timings flag. With it, human output SHALL keep its stdout report and add one final stderr line, after any diagnostics, that names each publication phase from the SDK in order with its time in milliseconds and the total. JSON output SHALL add `timings`, a list of objects with `phase` and `ms`, and `total_ms`. The timings SHALL contain no page title, path or content. Without --timings, stdout and stderr SHALL be unchanged, and with or without it the published files SHALL be the same.

#### Scenario: Human timings
- **WHEN** a caller publishes with --timings
- **THEN** stdout matches a publication without the flag, and stderr ends with one line that starts with `Timings in milliseconds: read` and lists every phase and the total

#### Scenario: JSON timings
- **WHEN** a caller publishes with --timings and --format json
- **THEN** the report lists the phases read, parse, index, plugins, render, history, search, graph and write in that order, and `total_ms` equals their sum to within rounding

#### Scenario: Published files are unchanged
- **WHEN** a caller publishes the same garden twice, once with --timings and once without
- **THEN** both sites contain the same files with the same contents, and a JSON report without --timings has no `timings` or `total_ms`
