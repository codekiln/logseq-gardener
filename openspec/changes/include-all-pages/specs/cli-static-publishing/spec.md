## ADDED Requirements

### Requirement: Publish every page and journal

lsg publish SHALL accept an optional --include-all flag. With it, the SDK SHALL select every page of the garden, less the --exclude roots. Journals SHALL stay governed by --journals. --include roots given beside it SHALL be validated as usual and SHALL change nothing. Pages and journals withheld by the public-page and private-property rules SHALL stay withheld and counted.

#### Scenario: Whole garden with one flag

- **WHEN** a caller publishes a garden holding the pages `Notes`, `Projects/Alpha` and `SolveSpace` (from `pages/Solvespace.md` with `alias:: [[SolveSpace]]`) and one journal using --include-all --journals and no --include
- **THEN** the report shows three published pages and one published journal

#### Scenario: Exclusion under --include-all

- **WHEN** a caller publishes that garden using --include-all --journals --exclude Projects
- **THEN** `Projects/Alpha` is not published and `Notes`, `SolveSpace` and the journal are

#### Scenario: Journals still need --journals

- **WHEN** a caller publishes that garden using --include-all without --journals
- **THEN** the report shows three published pages, no published journals and one skipped journal

#### Scenario: Withheld pages stay withheld

- **WHEN** a caller publishes, using --include-all, a garden that holds a page marked `public:: false`
- **THEN** that page is withheld and counted in withheld_pages, and its content appears in no output file

#### Scenario: Invalid include beside --include-all

- **WHEN** a caller supplies --include-all with an --include root of `My//AI`
- **THEN** lsg exits 2 with the usual invalid-namespace error and creates no output
