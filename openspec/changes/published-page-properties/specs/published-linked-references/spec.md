## MODIFIED Requirements

### Requirement: Show grouped context and explicit counts

Each generated source or namespace page SHALL have a bottom linked-reference section grouped by originating published page. Its disclosure heading SHALL show the distinct source-page count even while collapsed, including zero. It SHALL show a separate referring-context count when the context and page totals differ, with a short explanation inside the disclosure that a context is a referring block or page properties. Unfiltered equal totals SHALL use the page count alone. Context SHALL retain escaped useful source content, working source-page navigation and available block navigation, bounded descendant rendering, and independent authored disclosures. Copies SHALL not duplicate original anchor IDs. Existing hierarchy, source content, and fragment navigation SHALL remain intact. Property source navigation SHALL target the visible Properties summary only when its safe displayed rows refer to the viewed target; otherwise its source link SHALL identify a page destination.

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
