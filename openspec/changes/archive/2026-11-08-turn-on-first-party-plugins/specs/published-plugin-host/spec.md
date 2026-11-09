# Spec Delta

## MODIFIED Requirements

### Requirement: No plugin cost without plugins

When neither `--plugin` nor the garden's `:lsg/plugins` key turns on a plugin with a `module`, generated pages SHALL contain no page metadata element and no reference to the host module, and SHALL make no request for plugin or host code.

#### Scenario: Plain publication

- **WHEN** a garden whose `logseq/config.edn` has no `:lsg/plugins` key is published without `--plugin`
- **THEN** every generated HTML file is identical to the same build before this change, and a reader's page load requests no host or plugin module

#### Scenario: Build-only plugins

- **WHEN** a garden is published with only build-only plugins
- **THEN** no page has a page metadata element or a host reference, and a reader's page load requests no host or plugin module

#### Scenario: First-party plugin turned on

- **WHEN** a garden turns on `sequence-header` with `:lsg/plugins` and is published without `--plugin`
- **THEN** every page except the graph view carries a page metadata element and loads the host module after first paint
