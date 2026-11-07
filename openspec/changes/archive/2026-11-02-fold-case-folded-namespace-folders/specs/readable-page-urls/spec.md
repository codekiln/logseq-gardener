## ADDED Requirements

### Requirement: One spelling per folder

With readable URLs, when the titles of pages that get folders spell one folder in more than one way that compare equal after Unicode NFC normalization and lower-casing, the publisher SHALL give that folder one spelling: the last segment of the title of the page whose own path is that folder, when such a page exists, and otherwise the spelling of that segment that sorts first. Each folder SHALL sit inside its parent's spelling. Every page inside such a folder SHALL use that spelling in its file path, route, link target and `lsg-site.json` `url`, and SHALL keep its title as authored. The graph view SHALL open such a page at that route.

#### Scenario: Folder named by a page

- **WHEN** a garden with the pages `CLAUDE` and `Claude/Artifact` is published with readable URLs
- **THEN** the output contains `CLAUDE/index.html` and `CLAUDE/Artifact/index.html`, links to `Claude/Artifact` use `CLAUDE/Artifact/`, and its title stays `Claude/Artifact`

#### Scenario: Folder without a page

- **WHEN** a garden with the pages `tmux/Session/Picker` and `tmux/session/Window` and no page `tmux/session` is published with readable URLs
- **THEN** both pages and the generated namespace page are under `tmux/Session/`

#### Scenario: Graph node in a respelled folder

- **WHEN** a reader activates the node of `Claude/Artifact` in the graph view of that build
- **THEN** the browser opens `CLAUDE/Artifact/`

## MODIFIED Requirements

### Requirement: Safe and distinct page paths

With readable URLs, a published title cannot form a safe path when it is empty, or has a segment that is empty, `.` or `..`, starts or ends with whitespace, contains a backslash or a control character, or is longer than 255 bytes in UTF-8. The publisher SHALL publish such a page at its `p-<digest>.html` file and SHALL report one diagnostic for it that names the page's source file, its title, the problem and the file name; a generated namespace page with such a title SHALL get the same file without a diagnostic of its own. The publisher SHALL fail before creating the output directory when two pages that get folders have paths that are equal after Unicode NFC normalization and lower-casing, when a page's folder is another page's file, or when a page's first segment matches a file or folder the build writes at the output root. Each failure message SHALL name every page involved.

#### Scenario: Unsafe title

- **WHEN** a selected page title contains a backslash, a `..` segment, or a segment that ends with a space, and readable URLs are requested
- **THEN** publication succeeds, the page is at its `p-<digest>.html` file, and a diagnostic names the page, the problem and the file

#### Scenario: Titles differing in letter case

- **WHEN** a garden publishes the pages `Tools/Vim` and `tools/vim` with readable URLs
- **THEN** publication fails before output is created and the message names both pages

#### Scenario: Reserved root name

- **WHEN** a garden with copied assets publishes a page titled `assets` with readable URLs
- **THEN** publication fails before output is created and the message names the page and `assets`
