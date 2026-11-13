## Purpose

Define the command-line publishing inputs, reports, and offline help.

## Requirements

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

### Requirement: Publication reports and diagnostics

Successful human and JSON output SHALL report pages, published journals, reference-only pages, assets, withheld pages, skipped journals, diagnostic count, and an index location. JSON SHALL contain format_version 1 and command_path [publish]. Unsupported selected constructs SHALL produce local stderr diagnostics and success. Fatal SDK failures SHALL return one with empty stdout and an actionable stderr error; invalid selectors SHALL return two.

#### Scenario: Unsupported content

- **WHEN** generation completes with unresolved references
- **THEN** the success report remains on stdout, diagnostics stay on stderr, and the exit is zero

#### Scenario: Fatal generation errors

- **WHEN** the source is missing, output exists or lies inside the garden, or selected titles are ambiguous
- **THEN** publication returns one with empty stdout and explains the SDK failure on stderr

#### Scenario: Journal counts
- **WHEN** a caller publishes a garden with three journals using --journals, one of them private
- **THEN** the report shows two published journals and one withheld page

#### Scenario: Reference-only page count
- **WHEN** a caller publishes a garden whose published pages link two names that no file holds
- **THEN** the human report shows two reference-only pages, the JSON report holds `reference_only_pages` 2, and the page count is unchanged

### Requirement: Offline publishing help

The root help SHALL discover publish, and publish SHALL provide equivalent help, --help and -h plus outline, section and programmatic guidance without requiring paths or reading the garden. Help SHALL describe permissions, fresh output, selector semantics, filename format, homepage selection and page-index navigation, output/exit codes and SDK limitations.

#### Scenario: Discover publishing offline

- **WHEN** a caller recursively traverses JSON help with closed stdin
- **THEN** publish documentation is discoverable and equivalent through each help alias

### Requirement: Plugin selection from the CLI

`lsg publish` SHALL accept a repeatable native-path `--plugin PATH` naming a plugin manifest and pass the manifests to the SDK's site options. A manifest error, a failing build plugin or a rejected build contribution SHALL exit one with empty stdout, a message on stderr naming the manifest or plugin and the problem, and no output directory. Publishing help SHALL describe the option, the manifest fields including the `build` section and the `plugins/<id>/` output folder, and SHALL name the `lsg/Ref/Plugin Host API` and `lsg/Ref/Build Plugin Protocol` pages with their documentation site URLs for the lifecycle and the build protocol.

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
- **THEN** the help describes `--plugin` and its `build` section and names the `lsg/Ref/Plugin Host API` and `lsg/Ref/Build Plugin Protocol` pages with their documentation site URLs

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

### Requirement: Readable URL option

`lsg publish` SHALL accept an optional `--readable-urls` flag and pass the readable layout to the SDK's site options. An unsafe or colliding title SHALL exit one with empty stdout, a message naming the pages on stderr and no output directory. Publishing help SHALL describe the flag, the folder layout, what stays at the output root and how unsafe and colliding titles fail, and SHALL name the `lsg/Ref/Publish Options` page with its documentation site URL for the layout.

#### Scenario: Publish readable URLs

- **WHEN** a caller runs `lsg publish` with `--readable-urls`
- **THEN** publishing succeeds, the report fields and index path are unchanged, and each page is written to its namespace folder

#### Scenario: Colliding titles

- **WHEN** a caller runs `lsg publish --readable-urls` on a garden whose selected titles differ only in letter case
- **THEN** `lsg publish` exits one with empty stdout, names both pages on stderr and creates no output directory

#### Scenario: Discover readable URLs offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `--readable-urls` and the layout it writes and names the `lsg/Ref/Publish Options` page with its documentation site URL

### Requirement: Old URL redirect option

`lsg publish` SHALL accept an optional `--redirect-old-urls` flag and pass it to the SDK's site options. A redirect that would replace another generated file SHALL exit one with empty stdout, a message naming the page and the file on stderr and no output directory. Publishing help SHALL describe the flag, the old addresses it redirects, and SHALL name the `lsg/Ref/Publish Options` page with its documentation site URL.

#### Scenario: Publish with redirects

- **WHEN** a caller runs `lsg publish --readable-urls --redirect-old-urls`
- **THEN** publishing succeeds, the report fields and index path are unchanged, and the output has a redirect at each page's old digest file

#### Scenario: Discover redirects offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `--redirect-old-urls` and names the `lsg/Ref/Publish Options` page with its documentation site URL

### Requirement: First-party plugins from the garden's configuration

`lsg publish` SHALL turn on the first-party plugins the garden's `:lsg/plugins` key names, together with any `--plugin` manifests. An unknown or malformed entry, or an id shared with a manifest, SHALL exit one with empty stdout, a message on stderr naming `logseq/config.edn` and the entry, and no output directory. Publishing help SHALL describe the key, list each first-party plugin with one line on what it does, and name the `lsg/Ref/First-Party Plugins` page with its documentation site URL.

#### Scenario: Turned on in config.edn

- **WHEN** a caller runs `lsg publish` on a garden whose `logseq/config.edn` holds `:lsg/plugins [:podcast-episode]`
- **THEN** publishing succeeds, the report fields are unchanged, and the output contains `plugins/podcast-episode/` and the plugin's entry in `lsg-site.json`

#### Scenario: Unknown name

- **WHEN** a caller runs `lsg publish` on a garden whose `logseq/config.edn` holds `:lsg/plugins [:podcast-epsiode]`
- **THEN** `lsg publish` exits one with empty stdout, names `logseq/config.edn`, `:podcast-epsiode` and the shipped plugin names on stderr, and creates no output directory

#### Scenario: Discover first-party plugins offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `:lsg/plugins`, lists `ableton-set`, `podcast-episode` and `sequence-header`, and names the `lsg/Ref/First-Party Plugins` page with its documentation site URL

### Requirement: Publish every page and journal

lsg publish SHALL accept an optional --include-all flag. With it, the SDK SHALL select every page of the garden, less the --exclude roots. Journals SHALL stay governed by --journals. --include roots given beside it SHALL be validated as usual and SHALL change nothing. Pages and journals withheld by the public-page and private-property rules SHALL stay withheld and counted.

#### Scenario: Whole garden with one flag

- **WHEN** a caller publishes a garden holding the pages `Notes`, `Projects/Alpha` and `SolveSpace` (from `pages/Solvespace.md` with `alias:: [[SolveSpace]]`) and one journal using --include-all --journals and no --include
- **THEN** the report shows three published pages and one published journal

#### Scenario: Exclusion under --include-all

- **WHEN** a caller publishes that garden using --include-all --journals --exclude Projects
- **THEN** `Projects/Alpha` is not published and `Notes`, `SolveSpace` and the journal are

#### Scenario: Journals still need --journals

- **WHEN** a caller publishes that garden using --include-all without --journals
- **THEN** the report shows three published pages, no published journals and one skipped journal

#### Scenario: Withheld pages stay withheld

- **WHEN** a caller publishes, using --include-all, a garden that holds a page marked `public:: false`
- **THEN** that page is withheld and counted in withheld_pages, and its content appears in no output file

#### Scenario: Invalid include beside --include-all

- **WHEN** a caller supplies --include-all with an --include root of `My//AI`
- **THEN** lsg exits 2 with the usual invalid-namespace error and creates no output
