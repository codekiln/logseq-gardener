# Comparison harness: build the whole encode garden and compare it with Logseq's build

Every parity analysis needs the same two things: an `lsg` build of the whole encode garden, and a way to put an `lsg` page next to the same page in Logseq's build. Neither exists as a command today. A one-off run on 2026-10-24 shows both are practical: `lsg` publishes the garden in about a minute, and Logseq's page list can be read from its build without a browser.

The run found that every page `lsg` publishes also exists in Logseq's build. The pages Logseq has and `lsg` lacks fall into five groups, listed under [Gaps](#gaps). The largest groups belong to other parity issues (journals to #110, reference-only pages to #111 and #115); the harness only has to report them in separate groups so they do not hide smaller differences.

## What Logseq's build contains

Logseq's build is the `gh-pages` branch of `codekiln/logseq-encode-garden`. The garden's `.github/workflows/gh-pages.yml` runs `logseq/publish-spa` on every push to `main`, then moves the inline graph data into `static/js/db.gz` with `.github/scripts/compress_logseq_db.py`. The build measured here was deployed from garden commit `0ce74d6` on 2026-10-23.

- **Page set.** The garden sets `:publishing/all-pages-public? true`, so publish-spa uses `clean-export!` ([`deps/publishing/src/logseq/publishing/db.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/publishing/src/logseq/publishing/db.cljs)). Every page is public except a page whose page properties set `public:: false`; for that page the page and its blocks are removed. File entities (`:file/*`, `:block/file`) are removed from every page.
- **Hidden files.** The graph parser drops files matched by `:hidden` in `logseq/config.edn` before parsing (`remove-hidden-files` in [`deps/graph-parser/src/logseq/graph_parser/cli.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/graph-parser/src/logseq/graph_parser/cli.cljs)). The encode garden hides `CLAUDE.md`, `README.md`, `.rulesync` files and `worktrees`, among others.
- **Pages without files.** Logseq creates a page for every page reference, tag and property value that names a page, even when no file exists. Such a page shows its title, its references and its hierarchy.
- **Routes.** Pages are at `#/page/<name>` ([`src/main/frontend/routes.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/routes.cljs)), where `<name>` is the lower-cased page name, URL-encoded. Journals are on the home route `#/` and also at `#/page/<journal title>`; All pages is `#/all-pages`.
- **Graph data.** `db.gz` holds one JavaScript assignment, `window.logseq_db="..."`. Its string is a Transit-encoded DataScript database in which `&`, `<`, `>`, `"` and `'` are replaced by `logseq____&amp;`-style tokens ([`deps/publishing/src/logseq/publishing/html.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/publishing/src/logseq/publishing/html.cljs); reversed in [`src/main/frontend/publishing.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/publishing.cljs)). Decoding it gives every datom: page names from `:block/name` and `:block/original-name`, journals from `:block/journal?`, and pages with content from `:block/page`.
- **Rendering.** The app renders in the browser. A page title appears within a second or two of loading on a local server, but Linked References render only when scrolled into view (`ui/lazy-visible` in [`src/main/frontend/components/reference.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/reference.cljs)), so a full-page screenshot has to scroll the page first.

To view the build locally, follow the steps in [the Logseq UI reference](../logseq-reference/Reference.md): `git archive origin/gh-pages` into an empty directory and serve it with `python3 -m http.server --bind 127.0.0.1`.

### Measured counts (gh-pages build of 2026-10-23)

| Count | Value |
| --- | --- |
| Page entities | 17,396 |
| Journal entities | 533 (505 with content) |
| Pages with content (`:block/page` targets) | 7,492, including the 505 journals |
| Decoded graph size | 174 MB from a 15.8 MB `db.gz`; 1,390,757 datoms |

The reference boards quote 16,693 pages; the build has grown since they were measured.

## What lsg does today

`lsg publish` builds only the namespaces named with `--include` ([Generate a local HTML garden](../local-site.md)). There is no option that selects every page, so a whole-garden build needs one `--include` per top-level namespace: 1,042 of them for the encode garden on 2026-10-24.

A debug build run with all 1,042 roots and `--filename-format triple-lowbar` produced:

| Measure | Value |
| --- | --- |
| Wall time | 64 s (debug build) |
| Source pages published | 6,984 |
| Generated namespace pages | 3,266 |
| HTML files | 10,251 (each page plus `index.html`) |
| Withheld pages (`public:: false` or nesting) | 2 |
| Skipped journals | 505 |
| Copied assets | 105 |
| Diagnostics | 24,692 |

The largest diagnostic groups were unresolved page references (16,108), unsupported macros (4,189), footnotes (3,346), drawers (262) and math (106).

Other behaviour that matters for a comparison:

- **Routes.** Each page is written to `p-<16 hex digits>.html`, derived from its title. `lsg` writes no title-to-file list, so finding the file for a title means parsing the links in `index.html`.
- **Configuration.** `lsg` reads `:file/name-format` only through the required `--filename-format` flag. It does not read `:hidden`. Page properties appear only when the garden opts in with `:publishing/page-properties? true`, which the encode garden does not set, so the `lsg` build shows no Properties block where Logseq shows one (for example on `tmux`).
- **Fresh output.** `lsg` refuses an existing output directory, so a repeatable harness has to pick a new directory, or delete the previous one, on each run.

## Gaps

Each gap is a check the harness can run, or a difference the harness has to report in its own group.

- 1. **Whole-garden selection.** One command builds every public page of the encode garden. Today that needs 1,042 `--include` flags. Check: the build publishes every page Logseq's build has content for, apart from the groups below.
- 2. **Journals.** Logseq publishes 505 journals with content; `lsg` skips them. The harness reports them as one group until #110 adds them.
- 3. **Reference-only pages.** About 6,600 Logseq pages exist only because something references them. `lsg` publishes no page for them and reports 16,108 unresolved references. The harness reports them as one group, separate from pages with content (see #111 and #115).
- 4. **Titles with `%2F`.** Three pages whose file names encode `/` as `%2F` are in Logseq's build and missing from `lsg`'s, for example `Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?`. The decoded title has a segment with a leading space, which `lsg`'s namespace rules reject. Check: these pages appear in both builds under the same title.
- 5. **Hidden files.** `lsg` does not apply `:hidden`. Check: a file matched by `:hidden` (such as `pages/CLAUDE.md`) is absent from both builds.
- 6. **Unexplained pages with content.** Three more Logseq pages with content are missing from `lsg`: `Contents` (Logseq's built-in sidebar page; the garden's `pages/contents.md` is empty), `parameters`, and a page parsed from a Markdown file under `assets/`. The harness lists such pages by name so each can be explained or fixed.
- 7. **Title-to-file list.** The comparison needs to open the same page in both builds. Check: for any title, the harness finds the `lsg` file and the Logseq route without parsing HTML by hand.
- 8. **Page list from Logseq's build.** The comparison reads Logseq's page list from `db.gz` rather than from the app. Check: the harness reports page, journal and content-page counts matching the table above for the 2026-10-23 build.
- 9. **Missing-page report.** Check: one command lists pages present in one build and missing from the other, grouped as in gaps 2 to 6. On 2026-10-24 the `lsg`-only list is empty.
- 10. **Side-by-side screenshots.** Check: one command saves a full-page screenshot of each fixture page from both builds, after scrolling so that lazily rendered sections are drawn, using the same viewport and colour scheme for both.
- 11. **Same `lsg` settings each run.** The harness names the `lsg` options it uses, including whether page properties are on, so later comparisons are not affected by a changed default.

## Fixture pages

Use these pages for the screenshot set. Each exercises features other parity issues compare.

| Page | Why |
| --- | --- |
| `tmux` | Page properties, headings, Linked References (lsg: 114 pages and 176 referring contexts; Logseq counts journal references too), hierarchy |
| `Foam` | Page embed |
| `Logseq/Block/Id` | Block embeds and block references |
| `Logseq/Queries` | Queries |
| `Lazygit` | Flashcards (`#card`) |
| `rw-cli` | Collapsed blocks |
| `ff` | Fenced code |
| `Contrando` | `{{video}}` embeds |
| `AI/E/WS/25/d1` | `{{youtube}}` embeds |
| `GitP/A/Session/24/11/19-Tue` | Remote artwork and audio |
| The journal for 2026-10-23 | Journal page (Logseq only until #110) |
| Home route and All pages | Journals home and All pages table (Logseq only until #110 and #117) |

## Out of scope

- Making `lsg` match Logseq for any of the groups above. Journals, reference-only pages, properties and the rest belong to the feature issues under #106.
- Pixel-level screenshot diffing. The harness saves images for a person to review; automatic scoring is a later decision.
- Rebuilding Logseq's site. The harness compares against the existing `gh-pages` build and does not run publish-spa.
- Block-level differences inside a page, which the feature analyses cover.
