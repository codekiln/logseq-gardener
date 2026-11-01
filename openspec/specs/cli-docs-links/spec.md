## Purpose

`lsg help` points readers at pages on the documentation site by page title and URL. Each URL comes from one base URL and the SDK's readable route rule, and a test fails when help links a page that the built site does not have.

## Requirements

### Requirement: One documentation base URL

The CLI SHALL define the documentation base URL `https://codekiln.github.io/logseq-gardener/` once and SHALL build every documentation link in help text from it.

#### Scenario: Every link starts at the base URL

- **WHEN** a test reads the help of every command in human and JSON output
- **THEN** every link to the documentation site starts with `https://codekiln.github.io/logseq-gardener/`, and the CLI source holds that string in one place

### Requirement: Help names pages by title and URL

Help text SHALL point a reader at documentation by naming the page's title and printing its URL. The CLI SHALL build a page's URL from the base URL and the route that the SDK's readable route function gives the page's title, the same function the publisher uses, so the CLI and the publisher apply one rule. Help text SHALL name no file under `docs/`.

#### Scenario: Link to the search data page

- **WHEN** a caller reads `lsg publish help section behavior`
- **THEN** the section names `lsg/Ref/Static Search Data` and prints `https://codekiln.github.io/logseq-gardener/lsg/Ref/Static%20Search%20Data/`

#### Scenario: No repository file paths

- **WHEN** a test reads the help of every command recursively in JSON
- **THEN** no section content contains `docs/`

### Requirement: Help links work offline

Help SHALL print documentation URLs without fetching them, and help output SHALL stay the same with or without network access.

#### Scenario: Help without network

- **WHEN** a caller runs `lsg publish help` with network access blocked
- **THEN** help prints the same text, URLs included, and exits zero

### Requirement: Every help link names a built page

A test SHALL collect every documentation URL from the help of every command, in human and JSON output, and SHALL fail when one of them, with the base URL removed, equals no page `url` in the built site's `lsg-site.json`. The base URL by itself SHALL count as the homepage, `index.html`. `mise run docs:check` SHALL run the test against the site it builds, so CI runs it.

#### Scenario: Help links a missing page

- **WHEN** help links a title that has no page in the docs graph
- **THEN** `mise run docs:check` fails and names the URL

#### Scenario: Renamed page

- **WHEN** a pull request renames a page that help links and leaves help unchanged
- **THEN** `mise run docs:check` fails and names the old URL
