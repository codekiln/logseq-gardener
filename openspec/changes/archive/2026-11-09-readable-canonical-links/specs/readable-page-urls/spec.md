## ADDED Requirements

### Requirement: Canonical page links

With readable URLs, each published page, journal, generated namespace page and reference-only page SHALL carry one `<link rel="canonical">` in its head, after its `<base>`, whose `href` is the page's route relative to the output root. The `--homepage` page SHALL name the site root as `./` in `index.html` and in its folder. The lists at the output root SHALL carry no canonical link. Without readable URLs, no page SHALL carry one.

#### Scenario: Page in a folder

- **WHEN** a garden with the page `Docs/Guide/Start` is published with readable URLs
- **THEN** `Docs/Guide/Start/index.html` carries `<link rel="canonical" href="Docs/Guide/Start/">` after `<base href="../../../">`, which resolves to that folder under any server path

#### Scenario: Page that keeps its digest file

- **WHEN** a page whose title cannot form a safe path is published with readable URLs
- **THEN** its `p-<digest>.html` file names `p-<digest>.html` as canonical

#### Scenario: Home page

- **WHEN** a garden is published with readable URLs and `--homepage Docs/Guide/Start`
- **THEN** `index.html` and `Docs/Guide/Start/index.html` both carry `<link rel="canonical" href="./">`

#### Scenario: Lists

- **WHEN** a garden is published with readable URLs and `--journals`
- **THEN** the journals feed, All pages and `graph.html` carry no canonical link

#### Scenario: Default layout

- **WHEN** a garden is published without readable URLs
- **THEN** no generated page carries a canonical link

### Requirement: Canonical link across plugin host navigation

With readable URLs and a plugin with a browser module, when the host adopts the next page in the same document, the document SHALL carry exactly the next page's canonical link, or none when the next page has none.

#### Scenario: Navigate between nested pages

- **WHEN** a reader of a readable build with a plugin clicks from `A/B/` to `C/`
- **THEN** the document has one canonical link, and its `href` is `C/`
