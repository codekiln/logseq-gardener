## MODIFIED Requirements

### Requirement: Selected-source search corpus

The publisher SHALL provide opt-in deterministic search artifacts built from selected visible original source. Stored titles, reference-only page titles and uniquely resolved aliases SHALL participate; generated namespace titles and property-content search SHALL be outside version1. Unresolved or ambiguous reference targets, hidden or excluded pages, macros, query programs, raw HTML, asset metadata and copied/transcluded text MUST contribute no terms or snippets, with one exception: a `{{youtube-timestamp}}` macro with a valid argument SHALL contribute its displayed time, as the published-media-embeds requirement "Show timestamps as text outside the page body" defines. Canonical-title body exclusions SHALL retain title results and remove body records.

#### Scenario: Public source refers to private content

- **WHEN** a selected public page references an excluded or private page and embeds a selected target
- **THEN** the index contains only safe original referring text and the target's own original content, without unavailable reference names or copied embed content

#### Scenario: Host excludes a generated-content slot page

- **WHEN** a host excludes a selected page body
- **THEN** its title remains searchable and its content records are absent

#### Scenario: Reference-only title

- **WHEN** a published block links `[[Deno Deploy]]`, which no file holds
- **THEN** the title index holds `Deno Deploy` with no block records, and the referring block's record holds the words `Deno Deploy`

#### Scenario: Macros in a block

- **WHEN** a published block reads `{{youtube-timestamp 620}} Agent skill {{tweet https://twitter.com/x/status/1}}`
- **THEN** the block's record holds `10:20 Agent skill` and no text from the `tweet` macro

