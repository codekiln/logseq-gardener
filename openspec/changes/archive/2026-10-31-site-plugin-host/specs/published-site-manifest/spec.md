## ADDED Requirements

### Requirement: Site manifest file

Every publication SHALL write `lsg-site.json` at the output root. The file SHALL be a JSON object with `apiVersion` set to `1`, a `revision` string that changes whenever any generated file changes, a `pages` array and an `assets` array. Each page entry SHALL have `id` (the page's lower-cased name, as Logseq identifies pages), `title` (the displayed title), `url` (the generated file's path relative to the output root) and `kind` (`page`, `journal`, `namespace`, `home` or `list`). Each asset entry SHALL be the copied asset's path relative to the output root. Both arrays SHALL be sorted by `url` or path so identical input produces identical bytes.

#### Scenario: Pages and journals listed

- **WHEN** a garden is published with `--journals`
- **THEN** `lsg-site.json` lists every generated page and journal file once with its id, title, URL and kind, and every URL names a file that exists in the output

#### Scenario: Withheld content absent

- **WHEN** a page is excluded by namespace selection or withheld by `public:: false`
- **THEN** its id, title and URL are absent from `lsg-site.json`, and assets referenced only by it are absent from the asset list

#### Scenario: Reproducible manifest

- **WHEN** the same garden is published twice with the same options and `SOURCE_DATE_EPOCH`
- **THEN** both `lsg-site.json` files are byte-identical

### Requirement: Manifest stays out of page loads

Generated pages SHALL NOT request `lsg-site.json`; the file exists for build tools and plugin hosts.

#### Scenario: No request on page load

- **WHEN** a reader opens any generated page of a publication with no plugins
- **THEN** the browser makes no request for `lsg-site.json`
