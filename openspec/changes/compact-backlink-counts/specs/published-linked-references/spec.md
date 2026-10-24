## MODIFIED Requirements

### Requirement: Show grouped context and explicit counts

Each generated source or namespace page SHALL have a bottom linked-reference section grouped by originating published page. Its disclosure heading SHALL show the distinct source-page count even while collapsed, including zero. It SHALL show a separate referring-context count when the context and page totals differ, with a short explanation inside the disclosure that a context is a referring block or page properties. Unfiltered equal totals SHALL use the page count alone. Context SHALL retain escaped useful source content, working source-page navigation and available block navigation, bounded descendant rendering, and independent authored disclosures. Copies SHALL not duplicate original anchor IDs. Existing hierarchy, source content, and fragment navigation SHALL remain intact.

#### Scenario: Zero references
- **WHEN** a generated page has no backlinks
- **THEN** its linked-reference heading displays zero pages and omits redundant context detail

#### Scenario: Referring block navigation
- **WHEN** a displayed reference came from an outline block
- **THEN** its source link opens that block in the original page, and fragment navigation opens only the containing disclosures

#### Scenario: Repeated context and media
- **WHEN** a source block is copied into linked-reference sections
- **THEN** its labels and source-relative media retain normal rendering while original anchor IDs remain unique on each output page

#### Scenario: Equal and unequal totals
- **WHEN** one source page has two referring contexts
- **THEN** the heading shows one page and two referring contexts, while a page with one context shows only one page

### Requirement: Filter by related published pages

Enhanced pages SHALL provide labelled related-page include/exclude controls, individual removal, reset, and option search. A reference SHALL match every included related page and no excluded related page, using current and ancestor reference identities including its source page. Related options SHALL derive only from published identities in matching context paths or descendants, excluding the viewed target. Search SHALL narrow options rather than replace these predicates. Filtering SHALL hide empty source groups and display filtered versus total source-page counts. It SHALL also display filtered versus total referring-context counts when visible counts or original totals differ. Counts SHALL remain readable when collapsed. The filter instruction SHALL read “Include or exclude selected related pages.” Controls SHALL support native keyboard and narrow-screen use. Without JavaScript, native disclosures and unfiltered content SHALL remain readable.

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

#### Scenario: Filtered distinction and reset
- **WHEN** filtering retains one context from an original two-page, three-context result
- **THEN** the collapsed heading shows one of two pages and one of three referring contexts, and reset restores the original counts
