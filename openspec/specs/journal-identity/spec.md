## Purpose

Define how the SDK identifies journals: their date formats, dates, titles, references and order.

## Requirements

### Requirement: Read journal date formats from the garden

The SDK SHALL read `:journal/page-title-format` and `:journal/file-name-format` from `logseq/config.edn` with the existing bounded EDN reader, defaulting to `MMM do, yyyy` and `yyyy_MM_dd`. A date format SHALL support the tokens `yyyy`, `MM`, `M`, `dd`, `d`, `do`, `MMM`, `MMMM`, `E`, `EEE` and `EEEE`, quoted literals and other literal characters. `E` and `EEE` SHALL write the short weekday name and `EEEE` the full name. A setting that is not a string, or a pattern with an unsupported token, SHALL fall back to the default and produce a diagnostic naming the setting.

#### Scenario: Encode garden title format
- **WHEN** `config.edn` sets `:journal/page-title-format "yyyy-MM-dd EEE"`
- **THEN** the title format writes 23 October 2026 as `2026-10-23 Fri`

#### Scenario: Weekday title format from Logseq's settings
- **WHEN** `config.edn` sets `:journal/page-title-format "E, MM/dd/yyyy"`
- **THEN** no diagnostic is produced and the title format writes 23 October 2026 as `Fri, 10/23/2026`

#### Scenario: Default title format
- **WHEN** `config.edn` sets no journal formats
- **THEN** the title format writes 23 October 2026 as `Oct 23rd, 2026` and the file-name format reads `2026_10_23`

#### Scenario: Unsupported token
- **WHEN** `:journal/page-title-format` is `"yyyy QQQ"` or `"EE yyyy"`
- **THEN** the title format is `MMM do, yyyy` and a diagnostic names `:journal/page-title-format`

### Requirement: Derive a journal's date and title

The SDK SHALL expose an operation that returns a journal document's date, parsed from its file stem with the file-name format, and its title: the leading page-properties block's last case-insensitive `title` property when present, otherwise the date written with the title format. A stem that does not parse as a real calendar date SHALL produce an error carrying the relative path. The operation SHALL leave the document unchanged and perform no filesystem access.

#### Scenario: Journal title from the file name
- **WHEN** the encode garden's `journals/2026_10_23.md` is supplied
- **THEN** the operation returns the date 2026-10-23 and the title `2026-10-23 Fri`

#### Scenario: Title property wins
- **WHEN** a journal's leading properties block has `title:: Launch day`
- **THEN** the title is `Launch day` and the date still comes from the file name

#### Scenario: Impossible date
- **WHEN** `journals/2025_02_30.md` is supplied
- **THEN** the operation returns an error carrying that path

### Requirement: Recognise journal references in Logseq's date formats

The SDK SHALL expose an operation that parses a page reference as a journal date when it matches the configured title format, `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd`, tried in that order. Month and weekday names SHALL match without regard to case. A weekday token SHALL match the full or the short weekday name, whatever its width. A weekday that contradicts the date SHALL make that format fail. The selected page lookup SHALL resolve a reference that matches no title or alias to the published journal with the parsed date.

#### Scenario: Default-format reference
- **WHEN** the encode garden's `Docker/For Mac/How to Install into Zsh` links `[[Feb 1st, 2025]]`
- **THEN** the reference parses as 2025-02-01 and resolves to the `2025-02-01 Sat` journal

#### Scenario: Configured-format reference
- **WHEN** a page links `[[2026-10-23 fri]]`
- **THEN** the reference resolves to the `2026-10-23 Fri` journal

#### Scenario: Weekday-first reference
- **WHEN** the title format is `E, MM/dd/yyyy` and pages link `[[Fri, 10/23/2026]]` and `[[friday, 10/23/2026]]`
- **THEN** both references resolve to the `Fri, 10/23/2026` journal

#### Scenario: Contradicting weekday
- **WHEN** a page links `[[2026-10-23 Mon]]` and no page has that title
- **THEN** the reference parses as no date and stays unresolved

### Requirement: List journals newest first

The SDK SHALL expose an operation that returns every journal document in a loaded garden with its date and title, ordered by date from newest to oldest, together with an error for each journal whose stem does not parse.

#### Scenario: Ordered journals
- **WHEN** a garden has journals for 2026-10-21, 2026-10-23 and 2026-10-22
- **THEN** the list is 2026-10-23, 2026-10-22, 2026-10-21
