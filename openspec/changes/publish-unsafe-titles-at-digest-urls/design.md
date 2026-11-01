## Context

`publication::select_with_urls` gives every published page and generated namespace page its route. With readable URLs it called `page_urls::readable_url` and returned its error, so one unsafe title stopped the build. A route that ends in `/` is a folder route and any other route is a file at the output root; `page_urls::is_readable` tells the two apart, and the site already handles both kinds: `output_path`, `stem` and `<base>` work per route, and `check_reserved` skips file routes. Two scripts in a readable build compute routes in the browser: the graph script turns a title into the route a graph node opens and finds the current page's neighbourhood file from the address, and the search engine checks each route in the search data against a pattern.

A namespace prefix of an unsafe title is often unsafe too. The encode garden page `Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?` produces the namespace page `Claude/Code/Q/If I rename a CLI session with `.

## Goals / Non-Goals

**Goals:**

- A readable build of a garden with unsafe titles completes, publishes every other page in its folder, and reports each unsafe page once.
- Links, search, the right sidebar, the page graph, the graph view and the plugin host reach a page that kept its digest file.

**Non-Goals:**

- Accept these titles in `--include` and `--exclude` selectors, so a garden can leave such a page out.

## Decisions

- `route_for` returns the route together with the reason a title cannot have a folder. For a page, `select_with_urls` adds a `Notice` with the page's source file and the message `page "<title>" cannot have a readable URL: <problem>; it is published at p-<digest>.html`. The site turns notices into diagnostics, which the command line prints to standard error, so the exit code stays 0.
- A generated namespace page that falls back gets no notice. Each segment of the prefix is a segment of a published title, so the page that produced the prefix already has a notice.
- Every unsafe problem falls back: an empty, `.` or `..` segment, whitespace at either end of a segment, a backslash, a control character and a segment longer than 255 bytes. Any of them stops the title from naming folders, and the digest file works for any title, so one rule covers all of them.
- `check_collisions` receives only titles whose route ends in `/`. A digest file has no folder, so its title cannot collide with a folder. A page titled like a digest file, such as `p-0123456789abcdef.html`, is still caught: `check_reserved` treats every file route as a name the build writes at the root.
- The readable graph script decides in JavaScript whether a title can name folders, with the same rules as `readable_url`, and otherwise returns `p-<digest>.html`. JavaScript's `\s` matches Rust's `White_Space` set plus U+FEFF, so the script excludes U+FEFF; U+0085, the one `White_Space` character `\s` lacks, is a control character, which the script refuses first. The graph data carries titles, and the default build's script already computes routes from titles, so the script needs no new data file.
- The page graph treats an address that ends in `p-<16 hex digits>.html` as a page and reads the neighbourhood file from the digest in the address.
- The readable search engine's route pattern accepts `p-<16 hex digits>.html` or a folder route.

## Risks / Trade-offs

- [The Rust and JavaScript rules for "can name folders" differ for some title] → a graph node for that title would open a missing page. `scripts/readable/browser.py` checks a graph route for a fallback title, and the comment in `readable-graph.js` names `readable_url` as the rule it mirrors.
- [A reader expects every page of a readable build at its namespace address] → the diagnostic names the page and its file, and lsg/Ref/Publish Options says how to give the page a folder.

## Resolved Questions

### 1 - Should lsg fall back to the digest file, or should the garden rename its three pages?

Decided by the agent without codekiln: fall back in lsg. Issue #363 offered both and its title asks for the fallback. Renaming fixes three pages today, but any later question page with a slash in its title would stop the build again, and the garden cannot exclude such a page because `--exclude` refuses the same titles.

### 2 - Which unsafe titles fall back, and which still stop the build?

Decided by the agent without codekiln: every title that `readable_url` refuses falls back. Path collisions between pages, and a page whose first segment is a reserved root name, still stop the build, because both pages in a collision have valid folders and lsg cannot choose which one keeps its folder.

### 3 - Does a generated namespace page get its own diagnostic?

Decided by the agent without codekiln: no. Its unsafe segment always comes from a published page that already has a diagnostic, and a second line for the same problem adds nothing the reader can act on.

## Open Questions

None.
