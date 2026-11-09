## MODIFIED Requirements

### Requirement: Show the journals feed

When journals are published, at least one published journal is dated on or before the build date, and `:feature/enable-journals?` is not `false`, the SDK SHALL write a journals feed. The build date SHALL be the UTC date of `SOURCE_DATE_EPOCH` when set, otherwise of the system clock. The feed SHALL list those journals and the reference-only journals dated on or before the build date, newest first: three in the first batch file and seven in each later batch file `journals-<n>.html`. The first batch SHALL be `index.html`, or `journals.html` when a homepage is selected. Each journal SHALL appear as an article with an `h1` linking to its page, its block tree with authored collapse state, and, when it has Linked References, a collapsed section summarising their count with a link to the journal page's Linked References. Each batch with a later batch SHALL end with an Earlier link to it. Journals dated after the build date SHALL keep their pages and stay out of the feed.

#### Scenario: First batch
- **WHEN** the encode garden at commit `09cf7fb` is published with journals and a build date of 2026-10-25
- **THEN** `index.html` lists `2026-10-24 Sat`, `2026-10-23 Fri` and `2026-10-22 Thu` in that order, and its Earlier link opens `journals-2.html`

#### Scenario: Reading without JavaScript
- **WHEN** a reader with JavaScript off follows Earlier from `index.html`
- **THEN** `journals-2.html` lists `2026-10-21 Wed` through `2026-10-15 Thu`

#### Scenario: Future journal
- **WHEN** a journal is dated after the build date
- **THEN** its page is published and no feed batch lists it

#### Scenario: Journals disabled
- **WHEN** `:feature/enable-journals?` is `false`
- **THEN** journal pages are published, no feed is written, and `index.html` is the complete page index

#### Scenario: Reference-only journal
- **WHEN** a published block links `[[Nov 18th, 2024]]`, no journal file has that date, and the garden is published with journals after that date
- **THEN** the feed lists `Nov 18th, 2024` in date order as an article with its `h1` link, no blocks and its collapsed Linked References section
