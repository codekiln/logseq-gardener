## MODIFIED Requirements

### Requirement: Generate selected standalone HTML pages

The SDK SHALL combine loaded Markdown, explicit filename interpretation, the existing namespace selection policy and the garden's public-page rule to generate a navigable index and standalone HTML pages. It SHALL count and skip journals unless the caller requests them, and SHALL then publish them according to the published-journals capability. It SHALL withhold any document containing a parsed `public:: false` property, and let exclusion override explicit public intent. It SHALL use deterministic routes and reject selected lowercase-title ambiguity and route collisions. It SHALL add safe namespace navigation destinations according to the published-namespace-hierarchy capability and include them in the complete page index. Generated namespace destinations SHALL contain navigation derived from selected visible titles; source-page reporting and source lookup SHALL retain their existing meaning.

The public-page rule SHALL follow Logseq's publishing export. When `logseq/config.edn` sets `:publishing/all-pages-public?` or `:all-pages-public?` to `true`, every page SHALL be public. Otherwise a page SHALL be public when its leading properties set `public:: true`, or when it names such a page with `alias::` or such a page names it. A selected page that is not public SHALL be withheld, counted as withheld and reported with a diagnostic, and its title SHALL be treated like a withheld title in namespace navigation. A setting value other than `true`, `false` or `nil` SHALL count as `false` and produce a diagnostic. A garden without `logseq/config.edn` SHALL treat every page as public.

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

#### Scenario: Setting absent or false
- **WHEN** a garden's `config.edn` leaves out `:publishing/all-pages-public?`, or sets it or `:all-pages-public?` to `false`, and include `Notes` selects `Notes/Marked` with `public:: true`, `Notes/Unmarked` with no `public::` property, and `Notes/Late` whose `public:: true` is on a later block
- **THEN** only `Notes/Marked` is published, and `Notes/Unmarked` and `Notes/Late` are absent and counted as withheld

#### Scenario: Alias of a marked page
- **WHEN** the setting is off and an unmarked selected page names a `public:: true` page with `alias::`
- **THEN** both pages are published

#### Scenario: Setting true
- **WHEN** a garden's `config.edn` sets `:publishing/all-pages-public? true`, or sets `:all-pages-public? true` while the newer key is absent or `false`
- **THEN** every selected page without `public:: false` is published

#### Scenario: Marked page outside the selection
- **WHEN** a page marked `public:: true` lies outside every included namespace
- **THEN** it is absent from the output

#### Scenario: Garden without a config file
- **WHEN** a garden has no `logseq/config.edn`
- **THEN** every selected page without `public:: false` is published
