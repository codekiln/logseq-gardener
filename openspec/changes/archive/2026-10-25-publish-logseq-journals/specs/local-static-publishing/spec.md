## MODIFIED Requirements

### Requirement: Generate selected standalone HTML pages

The SDK SHALL combine loaded Markdown, explicit filename interpretation, and the existing namespace selection policy to generate a navigable index and standalone HTML pages. It SHALL count and skip journals unless the caller requests them, and SHALL then publish them according to the published-journals capability. It SHALL withhold any document containing a parsed `public:: false` property, and let exclusion override explicit public intent. It SHALL use deterministic routes and reject selected lowercase-title ambiguity and route collisions. It SHALL add safe namespace navigation destinations according to the published-namespace-hierarchy capability and include them in the complete page index. Generated namespace destinations SHALL contain navigation derived from selected visible titles; source-page reporting and source lookup SHALL retain their existing meaning.

#### Scenario: Select and exclude namespaces
- **WHEN** a garden contains `Notes/Start`, `Notes/Other`, and `Notes/Private/Hidden`, with include `Notes` and exclude `Notes/Private`
- **THEN** Start and Other are the only source pages in the index and generated output, accompanied by their safe namespace navigation destinations

#### Scenario: Private nested property withholds the page
- **WHEN** an included page contains a nested parsed `public:: false` property
- **THEN** the entire page is withheld, including its text and referenced assets

#### Scenario: Ambiguous selected title
- **WHEN** selected files derive titles that compare equal after Unicode lowercase conversion
- **THEN** generation fails before creating output

#### Scenario: Journals without the request
- **WHEN** a garden with journals is published without requesting journals
- **THEN** no journal page or feed is generated and the report counts every journal as skipped

### Requirement: Select a published homepage

The SDK SHALL accept an optional homepage page selector resolved through selected visible titles and aliases. A unique selector SHALL render the selected page at index.html and retain its ordinary page route. The complete page index SHALL remain available at pages.html and generated pages SHALL expose navigation to it. An unavailable or ambiguous homepage selector SHALL fail before output is created. Without a selector, index.html SHALL hold the journals feed's first batch when the published-journals capability writes a feed, with the complete page index at pages.html, and SHALL otherwise hold the complete page index.

#### Scenario: Choose the homepage by alias
- **WHEN** the caller chooses a unique selected visible alias as the homepage
- **THEN** index.html contains the target page's content with functioning local links and media, its ordinary page route remains available, and pages.html lists selected pages

#### Scenario: Reject a withheld homepage
- **WHEN** the requested homepage resolves only to an excluded or private page, or its selected lookup is ambiguous
- **THEN** publication fails before creating output and introduces no withheld content

#### Scenario: Preserve default publication
- **WHEN** the caller supplies no homepage selector and requests no journals
- **THEN** index.html retains the generated selected-page index

#### Scenario: Journals feed as the home page
- **WHEN** the caller supplies no homepage selector and journals are published with a feed
- **THEN** index.html holds the feed's first batch and pages.html lists selected pages, including journal pages
