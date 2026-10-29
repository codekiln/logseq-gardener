## ADDED Requirements

### Requirement: Plugin selection from the CLI

`lsg publish` SHALL accept a repeatable native-path `--plugin PATH` naming a plugin manifest and pass the manifests to the SDK's site options. A manifest error SHALL exit one with empty stdout, a path-specific message on stderr and no output directory. Publishing help SHALL describe the option, the manifest fields, the `plugins/<id>/` output folder and where the lifecycle is documented.

#### Scenario: Publish with a plugin

- **WHEN** a caller runs `lsg publish` with a valid `--plugin player/lsg-plugin.json`
- **THEN** publishing succeeds, the report fields are unchanged, and the output contains `plugins/<id>/` and the plugin's entry in `lsg-site.json`

#### Scenario: Invalid plugin manifest

- **WHEN** a caller passes a manifest with an unsupported `apiVersion`
- **THEN** `lsg publish` exits one with empty stdout, names the manifest on stderr and creates no output directory

#### Scenario: Discover plugins offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `--plugin` and links the publishing guide's plugin section
