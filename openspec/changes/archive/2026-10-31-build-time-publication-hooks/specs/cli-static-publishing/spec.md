## MODIFIED Requirements

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
