## Purpose

`lsg publish --readable-urls` writes each page to a folder named by its title and links to that folder, so the page `GitP/A/Session` is published at `GitP/A/Session/`. A reader can tell from the address which page it opens, and a person or a program can write the address by hand. Rust programs select the layout with `SiteOptions::page_urls` and get the address of a title from `page_urls::readable_url`. Links, search, the right sidebar, the page graph and plugins work from the nested folders, when the site is opened from disk and when a server serves it under a path prefix. The default layout names each page file after a digest of its title, such as `p-0e2769dd320a1844.html`.

## Requirements

### Requirement: Readable page layout

When the caller selects readable URLs, the publisher SHALL write each published page, journal and generated namespace page to `<title path>/index.html`, where the title's slash-separated segments name the folders, and SHALL use `<encoded path>/` as the page's route, link target and `lsg-site.json` `url`. Each segment of the encoded path SHALL keep the RFC 3986 unreserved characters `A`–`Z`, `a`–`z`, `0`–`9`, `-`, `.`, `_` and `~` and percent-encode every other UTF-8 byte with upper-case hexadecimal digits. The home page, All pages files, journals feed files and `graph.html` SHALL stay at the output root under the names they have in the default layout. A homepage page SHALL be written both to `index.html` and to its own folder. Without readable URLs, the publisher SHALL produce the same files with the same contents as before this capability existed.

#### Scenario: Nested page folder

- **WHEN** a garden with the page `GitP/A/Session` is published with readable URLs
- **THEN** the output contains `GitP/A/Session/index.html`, links to that page use `GitP/A/Session/`, and no `p-*.html` page file is written

#### Scenario: Encoded segments

- **WHEN** a page titled `Notes/Q&A: 2026` is published with readable URLs
- **THEN** its file is `Notes/Q&A: 2026/index.html` and its route is `Notes/Q%26A%3A%202026/`

#### Scenario: Lists stay at the root

- **WHEN** a garden is published with readable URLs and `--journals`
- **THEN** the journals feed is `index.html`, All pages is `pages.html`, the graph view is `graph.html`, and each journal is written to the folder named by its title

#### Scenario: Default layout unchanged

- **WHEN** a garden is published without readable URLs by this release and by the previous release, with the same options and `SOURCE_DATE_EPOCH`
- **THEN** the two outputs contain the same files with identical contents

### Requirement: Links work from nested folders

Every page in a folder SHALL carry a `<base>` element, placed before any element that names a URL, whose `href` climbs one folder per title segment to the output root. Every generated link, asset, stylesheet, script, search manifest, All pages row, journals feed link and reference fragment link SHALL resolve to a generated file when the site is served from any path prefix and when it is opened from disk. A plain click on a fragment-only link SHALL stay in the current page. When the site is opened from disk, following a link whose path ends in `/` SHALL open that folder's `index.html`.

#### Scenario: Every URL resolves to an output file

- **WHEN** a garden with nested pages, assets, journals, search and a custom stylesheet is published with readable URLs
- **THEN** every relative `href` and `src` in every generated page resolves, against the page's base, to a file in the output, with a URL ending in `/` naming that folder's `index.html`

#### Scenario: Served under a path prefix

- **WHEN** a static server serves the output at `/logseq-gardener/` and a reader follows a link from `/logseq-gardener/A/B/` to another page
- **THEN** the browser requests a URL under `/logseq-gardener/` and the page loads with its styles and scripts

#### Scenario: Opened from disk

- **WHEN** a reader opens the output's `index.html` from disk and follows a page link
- **THEN** the browser opens the linked folder's `index.html`

#### Scenario: Bullet click stays on the page

- **WHEN** a reader clicks a block's bullet on a nested page
- **THEN** the browser stays on that page and zooms into the block

### Requirement: Safe and distinct page paths

With readable URLs, the publisher SHALL fail before creating the output directory when a published title is empty, or has a segment that is empty, `.` or `..`, starts or ends with whitespace, contains a backslash or a control character, or is longer than 255 bytes in UTF-8. It SHALL also fail when two published pages have paths that are equal after Unicode NFC normalization and lower-casing, when their folders share such a path with different spellings, when a page's folder is another page's file, or when a page's first segment matches a file or folder the build writes at the output root. Each failure message SHALL name every page involved.

#### Scenario: Unsafe title

- **WHEN** a selected page title contains a backslash or a `..` segment and readable URLs are requested
- **THEN** publication fails, names the page and the problem, and no output directory exists

#### Scenario: Titles differing in letter case

- **WHEN** a garden publishes the pages `Tools/Vim` and `tools/vim` with readable URLs
- **THEN** publication fails before output is created and the message names both pages

#### Scenario: Folder spelling conflict

- **WHEN** a garden publishes the pages `Tools/A` and `tools/B` with readable URLs
- **THEN** publication fails before output is created and the message names both pages

#### Scenario: Reserved root name

- **WHEN** a garden with copied assets publishes a page titled `assets` with readable URLs
- **THEN** publication fails before output is created and the message names the page and `assets`

### Requirement: Readable routes in search, sidebar and graph

With readable URLs, search page routes SHALL be readable routes, the published search engine SHALL accept them, and page and block results SHALL open the page's folder, keeping a block's fragment. Shift-click on a link to a page folder SHALL open the page in the right sidebar. A graph node SHALL open its page's folder, and the page graph SHALL find the current page and its neighbourhood file from the page's title.

#### Scenario: Search result opens the folder

- **WHEN** a reader searches a readable build and activates a block result on the page `A/B`
- **THEN** the browser opens `A/B/` at the block's fragment, from the home page and from a nested page

#### Scenario: Graph node opens the folder

- **WHEN** a reader activates the node of the page `A/B` in the graph view of a readable build
- **THEN** the browser opens `A/B/`

### Requirement: Plugin host navigation in readable builds

With readable URLs and a plugin with a browser module, the host SHALL treat links to page folders of the site as same-site pages, SHALL keep the document's base at the site root across same-document navigation, and SHALL leave the next page's `<base>` out of the current document. Page metadata `url` values SHALL be readable routes.

#### Scenario: Navigate between nested pages

- **WHEN** a reader of a readable build with a plugin clicks from `A/B/` to `C/` and then to an asset link on `C/`
- **THEN** the host adopts `C/` in the same document, history shows `C/`, and the asset link resolves under the site root
