## MODIFIED Requirements

### Requirement: Generate selected standalone HTML pages

The SDK SHALL combine loaded Markdown, explicit filename interpretation, and the existing namespace selection policy to generate a navigable index and standalone HTML pages. It SHALL skip journals, withhold any document containing a parsed `public:: false` property, and let exclusion override explicit public intent. It SHALL use deterministic routes and reject selected lowercase-title ambiguity and route collisions. It SHALL add safe namespace navigation destinations according to the published-namespace-hierarchy capability and include them in the complete page index. Generated namespace destinations SHALL contain navigation derived from selected visible titles; source-page reporting and source lookup SHALL retain their existing meaning.

#### Scenario: Select and exclude namespaces
- **WHEN** a garden contains `Notes/Start`, `Notes/Other`, and `Notes/Private/Hidden`, with include `Notes` and exclude `Notes/Private`
- **THEN** Start and Other are the only source pages in the index and generated output, accompanied by their safe namespace navigation destinations

#### Scenario: Private nested property withholds the page
- **WHEN** an included page contains a nested parsed `public:: false` property
- **THEN** the entire page is withheld, including its text and referenced assets

#### Scenario: Ambiguous selected title
- **WHEN** selected files derive titles that compare equal after Unicode lowercase conversion
- **THEN** generation fails before creating output

