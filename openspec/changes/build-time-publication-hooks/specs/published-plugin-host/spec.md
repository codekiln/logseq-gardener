## MODIFIED Requirements

### Requirement: Plugin manifest

A plugin SHALL be described by a JSON manifest file with `id` (lower-case letters, digits and hyphens), `apiVersion` (the host API major version, currently `1`), `version` (the plugin's own version string), `module` (a path to an ES module relative to the manifest), optional `styles` (paths to CSS files relative to the manifest), `activation` (`after-load` or `on-demand`), optional `pageProperties` (names of page properties the plugin may read) and optional `build` (an object with `command`, a non-empty array of strings; optional `blocks`, a boolean; and optional `timeoutSeconds`, an integer from 1 to 600). `module` and `activation` SHALL be required when `build` is absent and optional when it is present; `styles` SHALL require `module`. The SDK SHALL parse and validate manifests through one operation shared by the CLI and other callers.

#### Scenario: Valid manifest accepted

- **WHEN** a caller publishes with a manifest whose fields are well formed and whose module and stylesheets are regular files inside the manifest's folder
- **THEN** publication succeeds and copies the module and stylesheets under `plugins/<id>/`

#### Scenario: Build-only manifest accepted

- **WHEN** a caller publishes with a manifest that has `build` and no `module`
- **THEN** publication runs the build command, copies no plugin files and adds no plugin entry to `lsg-site.json`

### Requirement: Plugin manifest validation before output

The publisher SHALL check every plugin manifest before creating the output directory. It SHALL reject, with an error naming the manifest path and the problem: an unsupported `apiVersion`, a malformed or duplicate `id`, a missing field, a manifest with neither `module` nor `build`, an unknown `activation`, a malformed `build` section, a module or stylesheet path that is absolute, contains `..`, passes through a symlink, or is not a regular file, a build command whose first element contains `/` and does not name a regular file inside the manifest's folder, and a copied path that collides with another generated file. Plugins SHALL be ordered by `id`, so output does not depend on argument order.

#### Scenario: Unsupported API version

- **WHEN** a manifest declares `apiVersion` `2`
- **THEN** publication fails before the output directory exists and the error names the manifest and the supported version

#### Scenario: Duplicate plugin id

- **WHEN** two manifests declare the same `id`
- **THEN** publication fails before the output directory exists and the error names both manifest paths

#### Scenario: Escaping asset path

- **WHEN** a manifest's `module` is `../player.js`
- **THEN** publication fails before the output directory exists and the error names the manifest and the path

#### Scenario: Empty manifest

- **WHEN** a manifest has neither `module` nor `build`
- **THEN** publication fails before the output directory exists and the error names the manifest

#### Scenario: Escaping build command

- **WHEN** a manifest's `build.command` is `["../tools/build.mjs"]`
- **THEN** publication fails before the output directory exists and the error names the manifest and the path

#### Scenario: Argument order

- **WHEN** the same plugins are passed in two different orders
- **THEN** both outputs are byte-identical

### Requirement: Per-page plugin metadata

With at least one plugin that has a `module`, each generated page SHALL include a `<script type="application/json" data-lsg-page>` element holding the page's `id`, `title`, `url`, the site `revision`, and a `plugins` object keyed by plugin id. A plugin's entry SHALL hold the values of the page properties named in its `pageProperties`, taken only from properties the page publishes visibly, and the values its build contributions add for that page; a page with none of those SHALL have no entry for that plugin. The site manifest's plugin list (`plugins` with `id`, `version`, `module`, `styles` and `activation`, using output-relative URLs) SHALL be added to `lsg-site.json` and SHALL list the plugins that have a `module`.

#### Scenario: Recording metadata

- **WHEN** a plugin names `recording` in `pageProperties` and a published page has `recording:: https://example.com/a.mp3`
- **THEN** that page's metadata holds `{"recording": "https://example.com/a.mp3"}` under the plugin's id, and a page without the property has no entry for the plugin

#### Scenario: Contributed metadata

- **WHEN** a plugin with a module contributes `{"chapters": […]}` for a page without any of its page properties
- **THEN** that page's metadata holds `{"chapters": […]}` under the plugin's id

#### Scenario: Hidden properties stay hidden

- **WHEN** a plugin names a property that the publisher hides from readers, or a page is withheld
- **THEN** no generated file contains that property's value or that page's metadata

### Requirement: No plugin cost without plugins

When no plugin with a `module` is configured, generated pages SHALL contain no page metadata element and no reference to the host module, and SHALL make no request for plugin or host code.

#### Scenario: Plain publication

- **WHEN** a garden is published without `--plugin`
- **THEN** every generated HTML file is identical to the same build before this change, and a reader's page load requests no host or plugin module

#### Scenario: Build-only plugins

- **WHEN** a garden is published with only build-only plugins
- **THEN** no page has a page metadata element or a host reference, and a reader's page load requests no host or plugin module

### Requirement: Host loading and plugin activation

With at least one plugin that has a `module`, each page SHALL load the host module after first paint. The host SHALL import an `after-load` plugin on every page, and an `on-demand` plugin only on the first page whose metadata has an entry for it. Each plugin module SHALL be imported at most once per document, its stylesheets SHALL be added once, and its default export's `setup(host)` SHALL be called once, after the host checks that the export's `id` and `apiVersion` match its manifest.

#### Scenario: On-demand plugin

- **WHEN** a reader opens a page with no metadata for an `on-demand` plugin and then navigates to a page with metadata for it
- **THEN** the plugin module is requested only after the second navigation, once

#### Scenario: Mismatched export

- **WHEN** a plugin module exports an `id` different from its manifest
- **THEN** the host skips that plugin's `setup`, logs a diagnostic naming the plugin, and the page keeps working with ordinary links and native media
