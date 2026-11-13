## MODIFIED Requirements

### Requirement: Plugins shipped inside lsg

`lsg` SHALL ship the first-party plugins `ableton-set`, `podcast-episode` and `sequence-header`. Each SHALL have a manifest that passes the checks a `--plugin` manifest passes, with its module and stylesheets built into the `lsg` binary. A turned-on plugin's files SHALL be written to `plugins/<id>/` and listed in `lsg-site.json` as a manifest plugin's are, with the `lsg` version as the plugin's version. A build SHALL read no file outside the garden for a first-party plugin.

#### Scenario: Files written for a turned-on plugin

- **WHEN** a garden turns on `podcast-episode` and is published by `lsg` 0.0.40
- **THEN** the output holds the plugin's module and stylesheet under `plugins/podcast-episode/`, and `lsg-site.json` lists the plugin with version `0.0.40`

#### Scenario: Files absent for a plugin left off

- **WHEN** a garden turns on `sequence-header` only
- **THEN** the output holds no `plugins/podcast-episode/` folder and `lsg-site.json` lists only `sequence-header`

### Requirement: Stop on an unknown or malformed entry

Before it creates the output directory, the publisher SHALL stop with an error when `:lsg/plugins` is not a vector, holds an entry that is not a keyword, names a plugin this `lsg` does not ship, or names a plugin twice. The error SHALL name `logseq/config.edn`, the entry, the `lsg` version and the names of the plugins it ships.

#### Scenario: Misspelled name

- **WHEN** `logseq/config.edn` holds `:lsg/plugins [:podcast-epsiode]`
- **THEN** the build stops before the output directory exists, and the error names `:podcast-epsiode`, the `lsg` version, `:ableton-set`, `:podcast-episode` and `:sequence-header`

#### Scenario: String entry

- **WHEN** `logseq/config.edn` holds `:lsg/plugins ["podcast-episode"]`
- **THEN** the build stops before the output directory exists, and the error says the entry must be a keyword

#### Scenario: Repeated name

- **WHEN** `logseq/config.edn` holds `:lsg/plugins [:sequence-header :sequence-header]`
- **THEN** the build stops before the output directory exists, and the error names `:sequence-header`
