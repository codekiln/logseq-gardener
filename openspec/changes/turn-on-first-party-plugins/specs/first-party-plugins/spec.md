# Spec Delta

## Purpose

The person publishing a garden can turn on plugins that ship inside `lsg`, such as a podcast player, with one entry in the garden's `logseq/config.edn`. Every garden that turns a plugin on runs the same code at the same `lsg` version, and a garden that turns none on gets the same site as before.

## ADDED Requirements

### Requirement: Plugins shipped inside lsg

`lsg` SHALL ship the first-party plugins `podcast-episode` and `sequence-header`. Each SHALL have a manifest that passes the checks a `--plugin` manifest passes, with its module and stylesheets built into the `lsg` binary. A turned-on plugin's files SHALL be written to `plugins/<id>/` and listed in `lsg-site.json` as a manifest plugin's are, with the `lsg` version as the plugin's version. A build SHALL read no file outside the garden for a first-party plugin.

#### Scenario: Files written for a turned-on plugin

- **WHEN** a garden turns on `podcast-episode` and is published by `lsg` 0.0.40
- **THEN** the output holds the plugin's module and stylesheet under `plugins/podcast-episode/`, and `lsg-site.json` lists the plugin with version `0.0.40`

#### Scenario: Files absent for a plugin left off

- **WHEN** a garden turns on `sequence-header` only
- **THEN** the output holds no `plugins/podcast-episode/` folder and `lsg-site.json` lists only `sequence-header`

### Requirement: Turn plugins on in config.edn

The publisher SHALL read the top-level `:lsg/plugins` key of the garden's `logseq/config.edn` as a vector of keywords, each naming a first-party plugin to turn on. An absent key or an empty vector SHALL turn on no plugin. The same key inside a nested map SHALL be ignored. The key SHALL be the only way to turn on a first-party plugin, for the CLI and for SDK callers alike.

#### Scenario: Two plugins

- **WHEN** `logseq/config.edn` holds `:lsg/plugins [:podcast-episode :sequence-header]`
- **THEN** the build runs both plugins and writes both plugins' files

#### Scenario: No key

- **WHEN** a garden's `logseq/config.edn` has no `:lsg/plugins` key, or holds `:lsg/plugins []`
- **THEN** every generated file is identical to the same build before this change

#### Scenario: Logseq ignores the key

- **WHEN** the fixture garden with `:lsg/plugins [:podcast-episode :sequence-header]` is exported by Logseq 0.10.15 through `parity:logseq-export`
- **THEN** Logseq shows no configuration error and its export equals the export of the same garden without the key

### Requirement: Stop on an unknown or malformed entry

Before it creates the output directory, the publisher SHALL stop with an error when `:lsg/plugins` is not a vector, holds an entry that is not a keyword, names a plugin this `lsg` does not ship, or names a plugin twice. The error SHALL name `logseq/config.edn`, the entry, the `lsg` version and the names of the plugins it ships.

#### Scenario: Misspelled name

- **WHEN** `logseq/config.edn` holds `:lsg/plugins [:podcast-epsiode]`
- **THEN** the build stops before the output directory exists, and the error names `:podcast-epsiode`, the `lsg` version, `:podcast-episode` and `:sequence-header`

#### Scenario: String entry

- **WHEN** `logseq/config.edn` holds `:lsg/plugins ["podcast-episode"]`
- **THEN** the build stops before the output directory exists, and the error says the entry must be a keyword

#### Scenario: Repeated name

- **WHEN** `logseq/config.edn` holds `:lsg/plugins [:sequence-header :sequence-header]`
- **THEN** the build stops before the output directory exists, and the error names `:sequence-header`

### Requirement: One id namespace with manifest plugins

A turned-on first-party plugin SHALL share one id namespace with `--plugin` manifests and in-process build plugins. A build in which two of them have the same id SHALL stop before the output directory exists, with an error that names the id, `logseq/config.edn` and the other plugin's manifest or caller. All plugins SHALL be ordered by id.

#### Scenario: Manifest with a first-party id

- **WHEN** a garden turns on `sequence-header` and the build also passes `--plugin` with a manifest whose `id` is `sequence-header`
- **THEN** the build stops before the output directory exists and names the id, `logseq/config.edn` and the manifest

#### Scenario: Manifest and first-party plugin together

- **WHEN** a garden turns on `podcast-episode` and the build passes `--plugin tests/fixtures/plugins/recorder/lsg-plugin.json`
- **THEN** both plugins run, and the output equals the build with the same plugins given in any order

### Requirement: Choose pages by type and properties

A first-party plugin SHALL choose its pages from the published pages by their authored page properties, including the pages a `logseq-entity::` value names, whether or not the garden hides the property from readers. It SHALL add metadata only to published pages, take every value it writes from published content, and name no garden page outside the entity types and properties its specification lists.

#### Scenario: Hidden entity property

- **WHEN** a garden hides `logseq-entity` with `:block-hidden-properties` and a published page carries `logseq-entity:: [[Logseq/Entity/Podcast/Episode]]` and a recording
- **THEN** the page gets the `podcast-episode` entry, and no generated file holds the hidden property's value

#### Scenario: Another namespace

- **WHEN** the fixture page `Tools/Plugins/Show/Episode One` carries `logseq-entity:: [[Logseq/Entity/Podcast/Episode]]` and a recording
- **THEN** it gets the same player behaviour as a `GitP/A/Session` page

#### Scenario: Withheld page

- **WHEN** a page of the episode type is withheld by `public:: false` or excluded by namespace selection
- **THEN** no generated file holds its recording, chapters or metadata

### Requirement: Plugin code only where used

Each first-party plugin with a module SHALL use `on-demand` activation and add an entry only to pages it applies to. A page without an entry for a plugin SHALL request none of that plugin's files unless a page visited earlier in the same document had an entry. No first-party plugin SHALL add a script that runs before first paint.

#### Scenario: Page without an episode

- **WHEN** a reader opens `GitP/A/Session` directly on a site with both plugins on
- **THEN** the browser requests no file under `plugins/podcast-episode/`

#### Scenario: Asset page reached from an episode

- **WHEN** a reader starts the recording on `GitP/A/Session/26/10/24-Sat` and follows the link to `GitP/A/Session/26/10/24-Sat/Asset/Audio/Full/mp3`
- **THEN** the recording keeps playing and the browser makes no second request for the plugin's module

### Requirement: Budget rows for every first-party plugin

`scripts/parity/budgets.toml` SHALL hold a `[plugins.<id>]` table for every first-party plugin with `script_styles_gzip_kb`, `page_growth_gzip_kb`, `phone_heap_growth_mb` and `build_ms_per_page`. A test SHALL fail when a shipped plugin has no table or a table lacks a key. `parity:budgets` and `parity:build-bench` SHALL check the rows they measure in CI.

#### Scenario: Plugin without a table

- **WHEN** a first-party plugin is added to `lsg` without a `[plugins.<id>]` table
- **THEN** `mise run test` fails and names the plugin

#### Scenario: Module over budget

- **WHEN** `podcast-episode`'s module and stylesheet grow past 8 KB gzip
- **THEN** `mise run parity:budgets` fails and names the plugin and the row

#### Scenario: Slow build step

- **WHEN** a plugin's build step adds more than 0.05 ms per published page on the pinned encode garden
- **THEN** `parity:build-bench` in CI fails and names the plugin

### Requirement: First paint independent of garden size

With first-party plugins on, a page's first paint SHALL stay within the `[desktop]` and `[phone]` budgets, a page without a plugin entry SHALL grow by at most 0.5 KB gzip over the build without plugins, and the size plugins add to a page SHALL stay the same when the garden gains 2,000 pages. Plugins SHALL add no search record, and the search files SHALL equal the build without plugins.

#### Scenario: Garden grows

- **WHEN** `parity:budgets` builds the fixture garden with both plugins on, then again with 2,000 more pages
- **THEN** the plugin-added size of the fixture episode page and of `Tools/Queries` is the same in both builds

#### Scenario: Search unchanged

- **WHEN** the fixture garden is built with `--search`, once with both plugins on and once without the key
- **THEN** the search files of the two builds are identical

### Requirement: Measured against Logseq on the whole encode garden

`parity:perf --first-party` SHALL build the whole encode garden with both plugins on and measure each fixture page in that build and in Logseq's build of the same garden commit: time to the title, data and script before the title, peak JavaScript heap on the phone profile, and the heap after playing a recording across 20 navigations. `--record` SHALL write the rows to the Performance page and mark each value over its budget.

#### Scenario: Recorded rows

- **WHEN** `mise run parity:perf --first-party --record` runs on the encode garden
- **THEN** the Performance page has a First-party plugins section with a row per fixture page, profile and build, and each `lsg` value within budget is unmarked
