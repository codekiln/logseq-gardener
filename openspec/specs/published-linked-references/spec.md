## Purpose

Show published readers which pages refer to the current page, with safe source context, related-page filters, and configurable disclosure.

## Requirements

### Requirement: Index authored references from published sources

The publisher SHALL derive linked references only from selected visible original documents. It SHALL resolve page identities through existing selected-title and alias rules, deduplicate repeated links within a referring context, and omit self-page references. Parsed page links, tags, embed page targets, and authored page-reference property values SHALL supply references; literal code, comments, external links, generated navigation, and pure block UUID references SHALL not manufacture page backlinks. Missing or ambiguous identities SHALL supply no result or filter option. Safe generated namespace destinations MAY receive authored references without entering source alias, embed, or homepage lookup.

#### Scenario: Repeated references and aliases
- **WHEN** one source block links twice to a selected target and once to its unique alias
- **THEN** it contributes one reference context and one linking page for that target

#### Scenario: Several blocks on one page
- **WHEN** two blocks on one published page refer to a target
- **THEN** the target reports one distinct linking page and two reference contexts

#### Scenario: Tags and authored properties
- **WHEN** published source refers to a selected target through a tag or a page-reference property value
- **THEN** the appropriate source context supplies a backlink, and leading properties are labelled as page properties

#### Scenario: Literal and generated content
- **WHEN** a target appears only in literal code, comments, generated hierarchy links, or a pure block UUID reference
- **THEN** those occurrences supply no page backlink

#### Scenario: Withheld source and identities
- **WHEN** private or excluded sources contain unique text, references, or related-page names
- **THEN** none of their text, names, counts, filter options, or metadata enters backlink output or scripts

### Requirement: Show grouped context and explicit counts

Each generated source or namespace page SHALL have a bottom linked-reference section grouped by originating published page. Its disclosure heading SHALL show distinct linking-page and reference-context counts even while collapsed, including zero. Context SHALL retain escaped useful source content, working source-page navigation and available block navigation, bounded descendant rendering, and independent authored disclosures. Copies SHALL not duplicate original anchor IDs. Existing hierarchy, source content, and fragment navigation SHALL remain intact.

#### Scenario: Zero references
- **WHEN** a generated page has no backlinks
- **THEN** its linked-reference heading displays zero linking pages and zero references

#### Scenario: Referring block navigation
- **WHEN** a displayed reference came from an outline block
- **THEN** its source link opens that block in the original page, and fragment navigation opens only the containing disclosures

#### Scenario: Repeated context and media
- **WHEN** a source block is copied into linked-reference sections
- **THEN** its labels and source-relative media retain normal rendering while original anchor IDs remain unique on each output page

### Requirement: Filter by related published pages

Enhanced pages SHALL provide labelled related-page include/exclude controls, individual removal, reset, and option search. A reference SHALL match every included related page and no excluded related page, using current and ancestor reference identities including its source page. Related options SHALL derive only from published identities in matching context paths or descendants, excluding the viewed target. Search SHALL narrow options rather than replace these predicates. Filtering SHALL hide empty source groups and display filtered versus total linking-page and reference counts; all-count headings SHALL remain readable when collapsed. Controls SHALL support native keyboard and narrow-screen use. Without JavaScript, native disclosures and unfiltered content SHALL remain readable.

#### Scenario: Combined inclusion and exclusion
- **WHEN** two related pages are included and a third is excluded
- **THEN** only references containing both includes and no exclude remain visible

#### Scenario: Ancestor reference
- **WHEN** a matching block has a related-page link in its parent
- **THEN** the block can match an include filter for that related page

#### Scenario: Partial source group and reset
- **WHEN** filtering retains one of two references from a source page
- **THEN** that page counts once and only the retained reference counts, and reset restores every group and clears controls

#### Scenario: No filtered results
- **WHEN** no reference matches the active filters
- **THEN** the section shows zero filtered counts against the original totals and a readable no-results status

#### Scenario: Replaced host content
- **WHEN** a host replaces page content and initializes linked-reference controls again
- **THEN** the new section works and repeated initialization creates no duplicate handlers or player/navigation side effects

### Requirement: Honor graph reference disclosure settings

The publisher SHALL read integer `:ref/linked-references-collapsed-threshold` and `:ref/default-open-blocks-level` from the graph's top-level `logseq/config.edn` map, defaulting to 100 and 2. Linked-reference sections SHALL start collapsed when the reference-context count reaches the threshold; zero SHALL collapse every section while preserving its heading. Reference-context parents at or beyond the configured open depth SHALL start closed, and authored collapse SHALL also remain effective. Config reading SHALL be bounded, read-only, and ignore commented, discarded, string, or nested-map lookalikes. Missing or unusable settings SHALL retain defaults with useful diagnostics for unusable input.

#### Scenario: Threshold zero
- **WHEN** the graph sets the collapse threshold to zero
- **THEN** sections with zero or more references start closed and show their counts

#### Scenario: Open depth and authored collapse
- **WHEN** open depth is two and a reference contains nested parents plus an authored collapsed parent
- **THEN** parents at depth two or deeper and the authored collapsed parent start closed while manual disclosure and fragment reveal still work

#### Scenario: Defaults and misleading config text
- **WHEN** config is absent or contains setting names only in comments, strings, discarded forms, or nested maps
- **THEN** absent top-level integer settings retain their defaults
