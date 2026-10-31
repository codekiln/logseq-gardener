## MODIFIED Requirements

### Requirement: Publish journal pages

When the caller requests journals, the SDK SHALL publish every journal whose identity derives, that is public under the public-page rule of the local-static-publishing capability, and that contains no parsed `public:: false` property, as a page at its deterministic route, titled with its journal title. A journal that is not public SHALL be withheld and counted as withheld. A journal page SHALL show the journal's blocks, page properties and Linked References as other pages do, SHALL accept its aliases as link targets, and SHALL omit the namespace hierarchy section. Journal titles SHALL share the selected title lookup with pages, so a page whose title equals a journal title SHALL fail with the existing ambiguity error before output is created. A journal whose identity does not derive SHALL be skipped with a diagnostic and counted as a skipped journal. Journal pages SHALL appear in the complete page index and the search index.

#### Scenario: Ordinary journal page
- **WHEN** the encode garden is published with journals
- **THEN** `2026-10-23 Fri` shows its Filed and Updated blocks and 1 Linked Reference from the LangSmith question page

#### Scenario: Journal with properties and an alias
- **WHEN** the encode garden is published with journals
- **THEN** `2019-08-07 Wed` shows its page properties and a link to `1987` opens it

#### Scenario: Private journal
- **WHEN** a journal contains a nested `public:: false` property
- **THEN** the journal, its text and its referenced assets are absent from the output and the report counts it as withheld

#### Scenario: Unmarked journal in a garden that publishes marked pages only
- **WHEN** journals are requested, the garden's all-pages-public setting is off, and one journal is marked `public:: true` while another has no `public::` property
- **THEN** only the marked journal is published and the report counts the other as withheld
