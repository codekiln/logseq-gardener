## MODIFIED Requirements

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
