## Purpose

Define opt-in selected-source search artifacts and deterministic query evaluation.

## Requirements

### Requirement: Selected-source search corpus

The publisher SHALL provide opt-in deterministic search artifacts built from selected visible original source. Stored titles and uniquely resolved aliases SHALL participate; generated namespace titles and property-content search SHALL be outside version1. Unresolved or ambiguous reference targets, hidden or excluded pages, macros, query programs, raw HTML, asset metadata and copied/transcluded text MUST contribute no terms or snippets. Canonical-title body exclusions SHALL retain title results and remove body records.

#### Scenario: Public source refers to private content

- **WHEN** a selected public page references an excluded or private page and embeds a selected target
- **THEN** the index contains only safe original referring text and the target's own original content, without unavailable reference names or copied embed content

#### Scenario: Host excludes a generated-content slot page

- **WHEN** a host excludes a selected page body
- **THEN** its title remains searchable and its content records are absent

### Requirement: Stable original destinations

Every search result SHALL identify its selected source page and an output-relative route. Content records SHALL use an actually emitted original anchor, preferring globally unique UUIDs and otherwise source offsets. Existing fragment reveal behavior SHALL open collapsed ancestors.

#### Scenario: Searchable child under collapsed parent

- **WHEN** a matching original child is nested inside collapsed source
- **THEN** its result points to an emitted anchor that the fragment reveal helper can expose

### Requirement: Versioned deterministic publication

Manifest and corpus SHALL declare format version1 and the same deterministic revision. Every generated page SHALL advertise its expected manifest and revision without eagerly loading search data. The pure validator SHALL reject unsupported versions, revision mismatch and malformed record shape. Hosts SHALL be able to adapt routes using page title and source fragment without rewriting the corpus.

#### Scenario: Cached corpus from another publication

- **WHEN** manifest, corpus or page revisions disagree
- **THEN** validation fails and no query results are accepted

### Requirement: Bounded pure query evaluation

The standalone evaluator SHALL normalize lowercase NFC text and collapse whitespace, bound query length, term count, result count and plain-text snippet length, and return deterministic whole-result order. Exact title, exact unique alias, title/alias prefix, all-term title/alias, and all-term content SHALL form ordered ranking buckets; ties SHALL sort by normalized title, route and source position. Alias hits SHALL return one page result. Empty queries SHALL return no results; malformed and oversized queries SHALL report an explicit bounded error.

#### Scenario: Repeated equivalent Unicode queries

- **WHEN** equivalent composed or decomposed case variants are evaluated against the same corpus
- **THEN** the entire ordered result list is identical
