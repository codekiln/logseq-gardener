## Purpose

`lsg publish --redirect-old-urls` keeps links to a garden's earlier page addresses working after the garden moves to readable URLs. It redirects the `p-<digest>.html` files of earlier lsg builds and the `#/page/<name>` links of Logseq's own published site to the pages the build publishes.

## Requirements

### Requirement: Redirect option

The publisher SHALL write redirects only when the caller selects them with `SiteOptions::old_url_redirects` or `--redirect-old-urls`. Without the option, the publisher SHALL produce the same files with the same contents as before this capability existed.

#### Scenario: Default output unchanged

- **WHEN** a garden is published without the option by this release and by the previous release, with the same options and `SOURCE_DATE_EPOCH`
- **THEN** the two outputs contain the same files with identical contents

### Requirement: Digest file redirects

With readable URLs and the option, the publisher SHALL write, for each published page, journal and generated namespace page whose route is not `p-<digest of its title>.html`, a file at that name in the output root. Opening the file SHALL take the reader to the page's route and keep the URL's fragment. Without readable URLs, the publisher SHALL write no digest file redirects.

#### Scenario: Old digest link opens the readable page

- **WHEN** a garden with the page `GitP/A/Session` is published with readable URLs and the option, and a reader opens `p-<digest of "GitP/A/Session">.html#ls-block-1`
- **THEN** the browser opens `GitP/A/Session/#ls-block-1`

#### Scenario: Title that keeps its digest file

- **WHEN** a page whose title cannot form a safe path is published with readable URLs and the option
- **THEN** its `p-<digest>.html` file is the page itself, not a redirect

#### Scenario: Digest layout

- **WHEN** a garden is published with the option and without readable URLs
- **THEN** the output has no redirect files, since every page is at its digest file

### Requirement: Redirects from an earlier title spelling

With readable URLs and the option, a page that the publisher titles in another spelling than its file name or `title::` property gives, because the page refers to itself in that spelling as Logseq reads it, SHALL also get a digest file redirect at `p-<digest of the title its file name or title:: gives>.html`. Earlier releases named the page's file after that title.

#### Scenario: Page titled by its own alias

- **WHEN** `pages/Solvespace.md` with `alias:: [[SolveSpace]]` is published with readable URLs and the option
- **THEN** both `p-<digest of "SolveSpace">.html` and `p-<digest of "Solvespace">.html` open `SolveSpace/`

### Requirement: Redirect file contents

Each digest file redirect SHALL redirect with JavaScript turned off, link to the page by its title, name the page's route as its canonical URL and ask search engines not to index it.

#### Scenario: JavaScript off

- **WHEN** a reader with JavaScript turned off opens a digest file redirect
- **THEN** the browser opens the page's route, without its fragment

#### Scenario: Search engines

- **WHEN** a crawler reads a digest file redirect
- **THEN** the file carries `noindex` and a canonical link to the page's route

### Requirement: Logseq hash link redirects

With the option, opening `index.html` with the fragment `#/page/<name>` SHALL take the reader to the published page that `<name>` names by title, journal title or an alias of one published page. The name SHALL be percent-decoded, cut at its first `?`, and compared as page names are compared, ignoring letter case and surrounding slashes. An unknown name SHALL leave the reader on `index.html`. A fragment change on the open `index.html` SHALL be handled the same way.

#### Scenario: Page by title

- **WHEN** a reader opens `index.html#/page/gitp%2Fa%2Fsession` on a site that publishes `GitP/A/Session`
- **THEN** the browser opens that page's route

#### Scenario: Page by alias

- **WHEN** a reader opens `index.html#/page/<alias>` for an alias of one published page
- **THEN** the browser opens that page's route

#### Scenario: Unknown name

- **WHEN** a reader opens `index.html#/page/missing`
- **THEN** the browser stays on `index.html`

#### Scenario: Fragment changes on the open page

- **WHEN** a reader on `index.html` follows a link to `#/page/gitp%2Fa%2Fsession`
- **THEN** the browser opens that page's route

### Requirement: Logseq list link redirects

With the option, opening `index.html` with the fragment `#/all-pages` SHALL open All pages, and with `#/graph` SHALL open the graph view.

#### Scenario: Lists

- **WHEN** a reader opens `index.html#/all-pages` or `index.html#/graph`
- **THEN** the browser opens All pages or `graph.html`

### Requirement: Redirect files checked before output

The publisher SHALL fail before it creates the output directory when a digest file redirect would share its name with another generated file, or when two pages would share one digest file redirect, and the error SHALL name the page and the file.

#### Scenario: Name already generated

- **WHEN** a redirect's `p-<digest>.html` name is already the route of a page that keeps its digest file
- **THEN** publication fails, names the page and the file, and no output directory exists
