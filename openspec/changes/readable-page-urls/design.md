## Context

`lsg publish` writes every page, journal and namespace page to `p-<digest of the title>.html` at the output root, next to `index.html`, the All pages files, the journals feed, `graph.html`, the shared scripts and the `assets/`, `_refs/` and `_graph/` folders. Every link the publisher writes is relative to the output root: page links, asset paths, script and stylesheet tags, search result routes, the All pages list, the right sidebar's card routes and the plugin host's page URLs. Markup also moves between documents: the right sidebar shows other pages' content, reference and hierarchy fragments under `_refs/` are copied into pages, and the plugin host swaps one page's `<main>` into another document. Those moves work because every document resolves links against the same folder.

Gitpa turns the hashed files into readable URLs after each build ([gitpa scripts/build_site.py](https://github.com/codekiln/gitpa/blob/main/scripts/build_site.py) `package_pages`). It copies each page to `<title path>/index.html` and rewrites `href` and `src` attributes to absolute paths under its site prefix `/gitpa/`. That needs a known prefix, fails when the site is opened from disk, and misses URLs inside data files and scripts.

Gitpa pins an older `lsg` release, and its build edits the default output, so the default layout and its files stay exactly as they are.

## Goals / Non-Goals

**Goals:**

- An opt-in layout that writes each page to `<namespace path>/index.html` and links to it as `<namespace path>/`.
- Every generated URL works from nested folders, from disk and under any path prefix.
- Unsafe titles and colliding paths fail before the output directory is created, naming the pages.
- The default output stays byte for byte the same.

**Non-Goals:**

- Keep `p-<digest>.html` copies beside the readable folders, so links into an earlier build keep working. Gitpa writes such copies today and can keep doing so in its own build.
- Link `lsg help` and the README to pages of the documentation site, in [Issue #334 Link CLI help to pages on the docs site](https://github.com/codekiln/logseq-gardener/issues/334).

## Decisions

### `--readable-urls` and `PageUrls::Readable`

The CLI flag is `--readable-urls`, a plain switch like `--search` and `--journals`. Rust callers set `SiteOptions::page_urls` to `PageUrls::Readable`; `PageUrls::Digest` is the default. The `page_urls` module holds the layout, `readable_url(title)` for tools that link to a page, and the checks below. `publication::select_with_urls` assigns routes in either layout, and `publication::select` keeps assigning digest routes.

### Layout

A page titled `A/B C` is written to `A/B C/index.html`, and its route, link and `lsg-site.json` `url` are `A/B%20C/`. Each segment keeps the unreserved characters of RFC 3986 (`A`–`Z`, `a`–`z`, `0`–`9`, `-`, `.`, `_`, `~`) and percent-encodes every other UTF-8 byte with upper-case hex digits, as Python's `quote(segment, safe='')` does in Gitpa. The title keeps its authored spelling. Journals follow the same rule with their titles, so `2026-10-23 Fri` becomes `2026-10-23%20Fri/`. Generated namespace pages get folders like any page. A homepage page is written to `index.html` and to its own folder.

Generated lists stay at the output root under their current names: the home page `index.html`, All pages at `index.html` or `pages.html` and `pages-N.html`, the journals feed at `index.html` or `journals.html` and `journals-N.html`, and `graph.html`. Links to them keep their file names. Shared scripts, `custom.css`, `lsg-site.json`, search files and the `assets/`, `_refs/`, `_graph/`, `flashcards/` and `plugins/` folders stay where they are. Reference, hierarchy and query fragment files and graph neighbourhood files keep the names they have in the default layout, derived from the title digest, so a fragment's `<base href="../">` still names the site root and the page graph finds a page's file from its title.

### Links stay relative to the site root, and nested pages carry `<base>`

Each page in a folder gets `<base href="../../">`, with one `../` per segment, right after `<meta charset>`. Every link the publisher writes keeps resolving against the output root, so markup copied between documents keeps working, and no HTML, JSON or script needs rewriting. A link from `GitP/A/Session/` to `assets/a-1.png` or `Tools/` resolves to the site root's `assets/a-1.png` and `Tools/`, under `file:///…/site/` and under `https://host/logseq-gardener/` alike.

Rewriting each link relative to its own document was the alternative. It breaks markup that moves: a sidebar card, a reference fragment or a page swapped in by the plugin host would carry links relative to the wrong folder.

### Fragment-only links

Under a `<base>`, `href="#"` and `href="#x"` resolve against the site root instead of the page. Readable pages carry a short inline script, `data-lsg-readable`, that keeps a plain click on such a link in the current page: it sets the location's fragment for `#x` and does nothing for `#`, which the bullet and Flashcards scripts handle. The left sidebar's graph icon draws its own path instead of `<use href="#lsg-i-graph">`. The outline script's zoom breadcrumbs link to the page's full address.

### Opening the site from disk

A browser shows a folder listing for a `file://` URL that ends in `/`. When the site is read from disk, the `data-lsg-readable` script adds `index.html` to a link that ends in `/` as the reader clicks it, middle-clicks it or opens its context menu, and the journals feed's bullets do the same when they open a journal. Fetch-based features (search, the right sidebar, the graph) already stay off or fall back from disk. A reader with JavaScript turned off who opens the site from disk sees the folder listing and can open `index.html` from it.

### Unsafe titles and collisions

`readable_url` rejects a title that is empty or has a segment that is empty, `.` or `..`, starts or ends with whitespace, contains a backslash or a control character, or is longer than 255 bytes in UTF-8, the file-name limit of common file systems.

Two pages collide when their paths are equal after Unicode NFC normalization and lower-casing, since macOS and Windows file systems fold case. Two pages also collide when their folders share such a path with different spellings, such as `Tools/A` and `tools/B`, because a case-insensitive file system merges the folders and a case-sensitive host then misses one spelling. A page whose title ends in an `index.html` segment collides with its parent's page file.

These checks run during page selection, before rendering. A page whose first segment matches a file or folder the build writes at the output root, such as `assets`, `graph.html`, `lsg-site.json` or `_refs`, fails after rendering and before the output directory is created, because only then is every generated name known. Each message names the pages involved.

### Scripts in readable builds

Readable builds serve variants of the outline, shell, sidebar, graph, search engine and plugin host scripts, made by replacing a few exact lines of the shared scripts:

- outline: the zoom breadcrumb links to the page's full address, and a journal's bullet in the feed opens that journal's folder.
- shell and sidebar: Shift-click treats a link to a folder of the site as a page link.
- graph: a node opens its page's folder, and the page graph finds the current page and its neighbourhood file from the page's title.
- search engine: it accepts readable page routes, so results resolve beside the manifest with no resolver.
- plugin host: links to folders of the site are local pages, the next page's `<base>` is ignored, and the host sets `<base>` to the absolute site root before its first navigation, so the base stays fixed when history changes the address.

Changing the shared scripts to handle both layouts would change every default build's script files and `revision`. A unit test checks that each replaced line occurs exactly once, so a change to a shared script fails the test instead of silently dropping a readable variant.

The All pages list already carries a route for each row whose route differs from the digest name, so every readable row carries its route and the All pages script needs no variant.

## Risks / Trade-offs

- [A future feature writes a fragment-only link or a page-relative URL] → the readable build test checks every `href` and `src` in every page against the files in the output, and fails on a URL that resolves outside them.
- [A shared script changes and its readable variant no longer applies] → the exactly-once test fails.
- [Long or non-ASCII titles produce long URLs and paths] → percent-encoding keeps the URL valid, and the 255-byte segment check fails early. Deep namespaces on Windows can still pass its path-length limit; the build then fails with the path that could not be written.
- [Reading from disk without JavaScript shows folder listings] → the publishing guide says so.
- [Folder-per-page writing is slower] → only readable builds create folders; the default layout and its budgets are unchanged, and the pull request reports a readable build's time on the encode garden.

## Migration Plan

Gitpa keeps its pinned release. After upgrading, it can pass `--readable-urls`, delete `package_pages` and read readable URLs from `lsg-site.json`. Its legacy `#/page/` redirects and RSS links already use the same URLs.

## Resolved Questions

### 1 - What are the flag and the SDK option called?

Decided by the agent without codekiln: `--readable-urls`, matching the plain switches `--search` and `--journals`, and `SiteOptions::page_urls` with `PageUrls::Digest` (default) and `PageUrls::Readable`. An enum names the layout in Rust code, where a bare `true` would not.

### 2 - How are page links written in nested folders?

Decided by the agent without codekiln: relative to the site root, as in the default layout, with `<base>` on each nested page. The publisher, the right sidebar, reference fragments and the plugin host all move markup between documents, and a shared base keeps those links valid without rewriting.

### 3 - Where do home, lists, namespace pages and journals go?

Decided by the agent without codekiln: home, All pages, the journals feed and the graph view stay at the output root with their current names. Journals and namespace pages get folders from their titles like any page. A homepage page also keeps its folder.

### 4 - How does a site opened from disk follow links to folders?

Decided by the agent without codekiln: the inline readable script adds `index.html` to a folder link as the reader follows it. The links themselves stay `<namespace path>/`, the URL a static server shows.

### 5 - Which titles fail, and do titles that differ only in letter case collide?

Decided by the agent without codekiln: Gitpa's rules (empty segments, `.`, `..`, surrounding whitespace, backslashes, control characters) plus segments longer than 255 bytes. Paths compare after NFC normalization and lower-casing, so titles and folders that differ only in letter case collide.

### 6 - What does `lsg-site.json` list as a readable page's `url`?

Decided by the agent without codekiln: the page's folder URL, such as `GitP/A/Session/`, relative to the output root. Its file is that folder's `index.html`.

### 7 - Does the search index change?

Decided by the agent without codekiln: its page routes become readable URLs, and readable builds serve a search engine variant that accepts them. Results then resolve beside the manifest without a host resolver.

## Open Questions

None.
