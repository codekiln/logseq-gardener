## 1. SDK catalog

- [x] 1.1 Add `page_lookup` with `PageCatalog::build`, `PageCatalog::load`, `find`, `PageLocation`, `MatchedBy` and `CatalogDiagnostic`, reusing `PageNameIndex`
- [x] 1.2 Unit-test titles, title properties, aliases, shared aliases, private pages, journal titles and dates, and left-out files

## 2. Command

- [x] 2.1 Add `lsg lookup NAME --graph PATH --filename-format F` with human and JSON output, stderr diagnostics and exits 0, 1 and 2
- [x] 2.2 Add lookup help and list `lookup` in root help
- [x] 2.3 Integration-test paths, JSON fields, several matches, no match, missing inputs, a page named `help`, and unchanged sources

## 3. Documentation and checks

- [x] 3.1 Describe the command in `docs/cli.md`, `docs/architecture.md` and the README, and run the README assessments
- [x] 3.2 Measure a lookup on the whole encode garden and record it
- [x] 3.3 Reassess `tests/assessments/help.json` and pass `mise run ci`
