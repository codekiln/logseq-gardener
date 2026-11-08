## MODIFIED Requirements

### Requirement: View pages

Each view page SHALL have `id` (the lower-cased page name), `title`, `url` (relative to the output root), `kind` (`page`, `journal`, `namespace` or `reference`) and `properties`. Pages SHALL be sorted by `id`. Generated lists (All pages, the journals feed) SHALL be absent.

#### Scenario: Journal page

- **WHEN** a garden is published with `--journals`
- **THEN** each published journal appears once with kind `journal` and the URL of its generated file

#### Scenario: Reference-only page

- **WHEN** a published block links `Deno Deploy`, which no file holds
- **THEN** the view holds `Deno Deploy` once with kind `reference`, the URL of its generated file and empty `properties`

### Requirement: Selected publication view

The SDK SHALL provide one operation that builds a read-only view of a publication from a loaded garden, a filename format, a namespace selection and the publishing options. The view SHALL hold `apiVersion` `1`, a `revision`, a `pages` array and a `blocks` array. The publisher SHALL select, withhold and route pages through the same operation, so the view and the generated site agree on which pages are published.

#### Scenario: Same pages as the site

- **WHEN** a garden is published with build plugins
- **THEN** the view's page ids are the ids of the `page`, `journal`, `namespace` and `reference` entries in `lsg-site.json`, and each page's `title` and `url` match its entry

#### Scenario: Selection moved without output changes

- **WHEN** the whole encode garden is published without plugins before and after the selection code moves into the SDK
- **THEN** both builds produce the same `lsg-site.json` revision
