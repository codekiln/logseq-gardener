## MODIFIED Requirements

### Requirement: Show grouped context and explicit counts

Each generated source or namespace page SHALL have a bottom linked-reference section grouped by originating published page. Its disclosure heading SHALL show the distinct source-page count even while collapsed, including zero. It SHALL show a separate referring-context count when the context and page totals differ, with a short explanation inside the disclosure that a context is a referring block or page properties. Unfiltered equal totals SHALL use the page count alone. Context SHALL retain escaped useful source content, working source-page navigation and available block navigation, bounded descendant rendering, and independent authored disclosures. Copies SHALL not duplicate original anchor IDs. Existing hierarchy, source content, and fragment navigation SHALL remain intact.

Groups SHALL use a compact source-page card with one page heading and linked ancestor breadcrumbs. Each source block SHALL appear once across overlapping referring outlines within its group, while genuine context identities, counts and filter predicates remain available. Safe displayed property references SHALL show the relevant validated property rows and a link to the original property section.

#### Scenario: Overlapping referring outlines
- **WHEN** a referring parent and its descendant both link to the viewed page
- **THEN** their shared outline appears once with two context identities, working original-source links and the existing related-reference matching behavior

#### Scenario: Safe property context
- **WHEN** a visible property row links to the viewed page
- **THEN** its group shows that row and a property-section destination while unrelated or omitted rows remain absent

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

