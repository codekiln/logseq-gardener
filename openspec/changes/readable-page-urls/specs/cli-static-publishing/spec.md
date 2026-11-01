## ADDED Requirements

### Requirement: Readable URL option

`lsg publish` SHALL accept an optional `--readable-urls` flag and pass the readable layout to the SDK's site options. An unsafe or colliding title SHALL exit one with empty stdout, a message naming the pages on stderr and no output directory. Publishing help SHALL describe the flag, the folder layout, what stays at the output root, how unsafe and colliding titles fail, and where the publishing guide describes the layout.

#### Scenario: Publish readable URLs

- **WHEN** a caller runs `lsg publish` with `--readable-urls`
- **THEN** publishing succeeds, the report fields and index path are unchanged, and each page is written to its namespace folder

#### Scenario: Colliding titles

- **WHEN** a caller runs `lsg publish --readable-urls` on a garden whose selected titles differ only in letter case
- **THEN** `lsg publish` exits one with empty stdout, names both pages on stderr and creates no output directory

#### Scenario: Discover readable URLs offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `--readable-urls` and the layout it writes
