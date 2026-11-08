## MODIFIED Requirements

### Requirement: Selected-source search corpus

The publisher SHALL provide opt-in deterministic search artifacts built from selected visible original source. Stored titles, reference-only page titles and uniquely resolved aliases SHALL participate; generated namespace titles and property-content search SHALL be outside version1. Unresolved or ambiguous reference targets, hidden or excluded pages, macros, query programs, raw HTML, asset metadata and copied/transcluded text MUST contribute no terms or snippets. Canonical-title body exclusions SHALL retain title results and remove body records.

#### Scenario: Public source refers to private content

- **WHEN** a selected public page references an excluded or private page and embeds a selected target
- **THEN** the index contains only safe original referring text and the target's own original content, without unavailable reference names or copied embed content

#### Scenario: Host excludes a generated-content slot page

- **WHEN** a host excludes a selected page body
- **THEN** its title remains searchable and its content records are absent

#### Scenario: Reference-only title

- **WHEN** a published block links `[[Deno Deploy]]`, which no file holds
- **THEN** the title index holds `Deno Deploy` with no block records, and the referring block's record holds the words `Deno Deploy`

### Requirement: Versioned deterministic publication

Manifest, title index, shards and record chunks SHALL declare format version 4. The manifest SHALL be written at the fixed address `lsg-search-manifest.json`, SHALL carry the publication revision, the page URL scheme (`digest` or `readable`) and each page's number of block records in title index order, and SHALL list the title index, every shard and every chunk by content digest. Every generated page SHALL name the manifest's fixed address, without a revision, and SHALL load no search data eagerly. The pure validator SHALL reject unsupported versions, page counts that do not add up to the block total or disagree with the title index, digest mismatch and malformed record shape. A title index entry SHALL carry its page's route only when that route differs from the route its title gives under the manifest's URL scheme; the browser SHALL compute the route otherwise. Hosts SHALL be able to adapt routes using page title and source fragment without rewriting the index.

#### Scenario: Cached corpus from another publication

- **WHEN** a title index, shard or chunk's digest disagrees with the manifest
- **THEN** validation fails and no query results are accepted

#### Scenario: Version 2 data

- **WHEN** the controller receives a manifest that declares format version 2
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Version 3 data

- **WHEN** the controller receives a manifest that declares format version 3
- **THEN** validation fails with the reload message and no results are shown

#### Scenario: Pages name the fixed manifest

- **WHEN** a garden is published with search
- **THEN** every page links `lsg-search-manifest.json` with no revision in its markup, and the manifest's `revision` field identifies the publication

#### Scenario: Computable routes

- **WHEN** a garden is published with the default page URLs and search
- **THEN** the entry for `Deno Deploy` carries no route, search opens `p-<digest of "Deno Deploy">.html` for it, and an entry whose page was published at another route carries that route
