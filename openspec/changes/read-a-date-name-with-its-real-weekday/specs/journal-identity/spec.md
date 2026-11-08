## MODIFIED Requirements

### Requirement: Recognise journal references in Logseq's date formats

The SDK SHALL expose an operation that parses a page reference as a journal date when it matches the configured title format, `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd`, tried in that order. Month and weekday names SHALL match without regard to case. A weekday token SHALL match the full or the short weekday name, whatever its width. A weekday name that differs from the date's own weekday SHALL NOT make that format fail, as in Logseq: the date decides. The selected page lookup SHALL resolve a reference that matches no title or alias to the published journal with the parsed date.

#### Scenario: Default-format reference
- **WHEN** the encode garden's `Docker/For Mac/How to Install into Zsh` links `[[Feb 1st, 2025]]`
- **THEN** the reference parses as 2025-02-01 and resolves to the `2025-02-01 Sat` journal

#### Scenario: Configured-format reference
- **WHEN** a page links `[[2026-10-23 fri]]`
- **THEN** the reference resolves to the `2026-10-23 Fri` journal

#### Scenario: Weekday-first reference
- **WHEN** the title format is `E, MM/dd/yyyy` and pages link `[[Fri, 10/23/2026]]` and `[[friday, 10/23/2026]]`
- **THEN** both references resolve to the `Fri, 10/23/2026` journal

#### Scenario: Wrong weekday
- **WHEN** a page links `[[2026-10-23 Mon]]` and no page has that title
- **THEN** the reference parses as 2026-10-23 and resolves to the `2026-10-23 Fri` journal when the garden has one

#### Scenario: Word that is no weekday
- **WHEN** a page links `[[2026-10-23 Someday]]`
- **THEN** the reference parses as no date
