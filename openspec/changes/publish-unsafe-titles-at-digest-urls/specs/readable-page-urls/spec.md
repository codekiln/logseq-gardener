## MODIFIED Requirements

### Requirement: Readable page layout

When the caller selects readable URLs, the publisher SHALL write each published page, journal and generated namespace page to `<title path>/index.html`, where the title's slash-separated segments name the folders, and SHALL use `<encoded path>/` as the page's route, link target and `lsg-site.json` `url`. Each segment of the encoded path SHALL keep the RFC 3986 unreserved characters `A`–`Z`, `a`–`z`, `0`–`9`, `-`, `.`, `_` and `~` and percent-encode every other UTF-8 byte with upper-case hexadecimal digits. A page whose title cannot form a safe path, as "Safe and distinct page paths" defines, SHALL instead keep the `p-<digest>.html` file it has in the default layout, at the output root, and SHALL use that file name as its route, link target and `lsg-site.json` `url`. The home page, All pages files, journals feed files and `graph.html` SHALL stay at the output root under the names they have in the default layout. A homepage page SHALL be written both to `index.html` and to its own folder. Without readable URLs, the publisher SHALL produce the same files with the same contents as before this capability existed.

#### Scenario: Nested page folder

- **WHEN** a garden with the page `GitP/A/Session` is published with readable URLs
- **THEN** the output contains `GitP/A/Session/index.html`, links to that page use `GitP/A/Session/`, and no `p-*.html` page file is written

#### Scenario: Encoded segments

- **WHEN** a page titled `Notes/Q&A: 2026` is published with readable URLs
- **THEN** its file is `Notes/Q&A: 2026/index.html` and its route is `Notes/Q%26A%3A%202026/`

#### Scenario: Title that keeps its digest file

- **WHEN** a garden with the pages `Notes/Other` and `Notes/Q/Rename with /rename`, the second stored as `Notes___Q___Rename with %2Frename.md`, is published with readable URLs
- **THEN** `Notes/Other` is at `Notes/Other/index.html`, `Notes/Q/Rename with /rename` and the namespace page `Notes/Q/Rename with ` are at their `p-<digest>.html` files at the output root, and links and `lsg-site.json` use those file names

#### Scenario: Lists stay at the root

- **WHEN** a garden is published with readable URLs and `--journals`
- **THEN** the journals feed is `index.html`, All pages is `pages.html`, the graph view is `graph.html`, and each journal is written to the folder named by its title

#### Scenario: Default layout unchanged

- **WHEN** a garden is published without readable URLs by this release and by the previous release, with the same options and `SOURCE_DATE_EPOCH`
- **THEN** the two outputs contain the same files with identical contents

### Requirement: Safe and distinct page paths

With readable URLs, a published title cannot form a safe path when it is empty, or has a segment that is empty, `.` or `..`, starts or ends with whitespace, contains a backslash or a control character, or is longer than 255 bytes in UTF-8. The publisher SHALL publish such a page at its `p-<digest>.html` file and SHALL report one diagnostic for it that names the page's source file, its title, the problem and the file name; a generated namespace page with such a title SHALL get the same file without a diagnostic of its own. The publisher SHALL fail before creating the output directory when two pages that get folders have paths that are equal after Unicode NFC normalization and lower-casing, when their folders share such a path with different spellings, when a page's folder is another page's file, or when a page's first segment matches a file or folder the build writes at the output root. Each failure message SHALL name every page involved.

#### Scenario: Unsafe title

- **WHEN** a selected page title contains a backslash, a `..` segment, or a segment that ends with a space, and readable URLs are requested
- **THEN** publication succeeds, the page is at its `p-<digest>.html` file, and a diagnostic names the page, the problem and the file

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

With readable URLs, search page routes SHALL be the routes the layout gives the pages, folder routes or `p-<digest>.html` files, the published search engine SHALL accept both, and page and block results SHALL open the page's route, keeping a block's fragment. Shift-click on a link to a page folder SHALL open the page in the right sidebar. A graph node SHALL open its page's route, and the page graph SHALL find the current page and its neighbourhood file from the page's title on a page in a folder and from the file name on a page that kept its digest file.

#### Scenario: Search result opens the folder

- **WHEN** a reader searches a readable build and activates a block result on the page `A/B`
- **THEN** the browser opens `A/B/` at the block's fragment, from the home page and from a nested page

#### Scenario: Search result opens a digest file

- **WHEN** a reader searches a readable build and activates a result on a page that kept its `p-<digest>.html` file
- **THEN** the browser opens that file at the result's fragment

#### Scenario: Graph node opens the folder

- **WHEN** a reader activates the node of the page `A/B` in the graph view of a readable build
- **THEN** the browser opens `A/B/`

#### Scenario: Page graph on a digest file

- **WHEN** a reader opens the page graph on a page of a readable build that kept its `p-<digest>.html` file
- **THEN** the page graph loads the neighbourhood file named by that digest and draws the page

### Requirement: Plugin host navigation in readable builds

With readable URLs and a plugin with a browser module, the host SHALL treat links to page folders and page files of the site as same-site pages, SHALL keep the document's base at the site root across same-document navigation, and SHALL leave the next page's `<base>` out of the current document. Page metadata `url` values SHALL be the routes the layout gives the pages.

#### Scenario: Navigate between nested pages

- **WHEN** a reader of a readable build with a plugin clicks from `A/B/` to `C/` and then to an asset link on `C/`
- **THEN** the host adopts `C/` in the same document, history shows `C/`, and the asset link resolves under the site root
