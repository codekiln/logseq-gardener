tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Logseq Compatibility]], [[lsg/How To/Compare a Build with Logseq]], [[lsg/Ref/Parity Tasks]]

- # Comparison Harness
	- The comparisons on the feature pages use two builds of the whole encode garden: Logseq's own build, the `gh-pages` branch that `logseq/publish-spa` deploys, and an `lsg` build of every page `lsg` can select. The harness reads Logseq's page list from the build's compressed graph data, matches each title to its `lsg` file and its Logseq route, lists the pages that one build has and the other lacks in separate groups, and captures and measures the same pages in both builds.
	  id:: e41c3897-775b-4b25-8307-ca7e1632886b
	- A one-off run on 2026-10-24 measured both builds by hand, against the `gh-pages` build deployed from garden commit `0ce74d6` on 2026-10-23. The OpenSpec change [comparison-harness](https://github.com/codekiln/logseq-gardener/tree/main/openspec/changes/archive/2026-10-24-comparison-harness) then built the `parity:` tasks, and [parity-comparison-harness](https://github.com/codekiln/logseq-gardener/blob/main/openspec/specs/parity-comparison-harness/spec.md) holds their requirements. [[lsg/How To/Compare a Build with Logseq]] shows how to run them, and [[lsg/Ref/Parity Tasks]] lists what each one writes.
	- ## What Logseq's build contains
		- Logseq's build is the `gh-pages` branch of `codekiln/logseq-encode-garden`. The garden's `.github/workflows/gh-pages.yml` runs `logseq/publish-spa` on every push to `main`, then moves the inline graph data into `static/js/db.gz` with `.github/scripts/compress_logseq_db.py`. The build measured here was deployed from garden commit `0ce74d6` on 2026-10-23.
		- **Page set.** The garden sets `:publishing/all-pages-public? true`, so publish-spa uses `clean-export!` ([`deps/publishing/src/logseq/publishing/db.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/publishing/src/logseq/publishing/db.cljs)). Every page is public except a page whose page properties set `public:: false`; for that page the page and its blocks are removed. File entities (`:file/*`, `:block/file`) are removed from every page.
		- **Hidden files.** The graph parser drops files matched by `:hidden` in `logseq/config.edn` before parsing (`remove-hidden-files` in [`deps/graph-parser/src/logseq/graph_parser/cli.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/graph-parser/src/logseq/graph_parser/cli.cljs)). A pattern matches when `/` plus the file's graph-relative path starts with it (a leading `/` is added to the pattern when missing; `hidden?` in [`deps/common/src/logseq/common/config.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/common/src/logseq/common/config.cljs)). The encode garden's entries, such as `CLAUDE.md`, `README.md` and `worktrees`, therefore hide files at the garden root and match no file in `pages/` or `journals/`.
		- **Pages without files.** Logseq creates a page for every page reference, tag and property value that names a page, even when no file exists. Such a page shows its title, its references and its hierarchy.
		- **Routes.** Pages are at `#/page/<name>` ([`src/main/frontend/routes.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/routes.cljs)), where `<name>` is the lower-cased page name, URL-encoded. Journals are on the home route `#/` and also at `#/page/<journal title>`; All pages is `#/all-pages`.
		- **Graph data.** `db.gz` holds one JavaScript assignment, `window.logseq_db="..."`. Its string is a Transit-encoded DataScript database in which `&`, `<`, `>`, `"` and `'` are replaced by `logseq____&amp;`-style tokens ([`deps/publishing/src/logseq/publishing/html.cljs`](https://github.com/logseq/logseq/blob/0.10.9/deps/publishing/src/logseq/publishing/html.cljs); reversed in [`src/main/frontend/publishing.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/publishing.cljs)). Decoding it gives every datom: page names from `:block/name` and `:block/original-name`, journals from `:block/journal?`, and pages with content from `:block/page`.
		- **Rendering.** The app renders in the browser. A page title appears within a second or two of loading on a local server, but Linked References render only when scrolled into view (`ui/lazy-visible` in [`src/main/frontend/components/reference.cljs`](https://github.com/logseq/logseq/blob/0.10.9/src/main/frontend/components/reference.cljs)), so a full-page screenshot has to scroll the page first.
		- To view the build locally, follow the steps on [[lsg/Ref/Logseq Published Site/Reference Sheet]]: `git archive origin/gh-pages` into an empty directory and serve it with `python3 -m http.server --bind 127.0.0.1`.
		- ### Measured counts (gh-pages build of 2026-10-23)
			- | Count | Value |
			  | --- | --- |
			  | Page entities | 17,396 |
			  | Journal entities | 533 (505 with content) |
			  | Pages with content (`:block/page` targets) | 7,492, including the 505 journals |
			  | Decoded graph size | 174 MB from a 15.8 MB `db.gz`; 1,390,757 datoms |
			- The reference boards quote 16,693 pages; the build has grown since they were measured.
	- ## How the harness compares the builds
		- **Whole-garden build.** `mise run parity:build` publishes every page whose top-level namespace `lsg` can select, with one `--include` per top-level namespace found among the garden's page files, `--journals` and `--search`. Each run writes a new `target/parity/<UTC time>/` directory, and its `settings.json` records the garden commit, the `lsg` version and commit, every option passed, and the files whose namespace `lsg` cannot select.
		- **Logseq's page list.** `parity:compare` extracts the `gh-pages` branch with `git archive`, so the garden checkout stays on its branch, and decodes `static/js/db.gz` into page records with each page's name, original title, journal flag and content flag. It records the `gh-pages` commit it read.
		- **Titles in both builds.** The harness maps each title to its `lsg` file, read from the All pages screen and its list data file, and to its Logseq route `#/page/<name>`. Titles match after lower-casing and Unicode NFC normalization, as Logseq compares page names.
		- **Missing-page report.** `parity:compare` writes `report.md` and `report.json`, listing pages present in one build and missing from the other. It places each Logseq-only page in one group, checked in this order: journals, reference-only pages without content blocks, pages whose file `:hidden` matches, titles `lsg` cannot select, and other pages. `lsg`-only pages have their own group. Every page in the other-pages and `lsg`-only groups is named, so each can be explained or fixed.
		- **Screenshots.** `parity:screenshots` serves both builds on 127.0.0.1 and captures each fixture page from both at the same viewport and colour scheme, after scrolling so Logseq draws its Linked References. Its index shows the two captures side by side and names the build that lacks a page.
		- **Measurements.** `parity:perf` loads the fixture pages from both builds with an empty cache on a desktop and a phone profile and checks `lsg`'s values against `scripts/parity/budgets.toml`; [[lsg/Ref/Logseq Compatibility/Performance]] holds the recorded results.
		- `lsg` now loads what Logseq loads: it applies `:hidden` with Logseq's prefix rule in every folder, reads `.md` files from every folder of the garden, and publishes the pages whose file names encode `/` as `%2F`, whose titles have segments that start or end with a space. Pages that exist only because something references them have no `lsg` page, and the report keeps them in their own group.
		- The screenshots are for a person to review; the harness does not score them. These tasks compare against the existing `gh-pages` build. The feature pages under [[lsg/Ref/Logseq Compatibility]] cover the differences inside a page.
		- **Page text.** `parity:logseq-export` builds Logseq 0.10.15's own export of a garden, the docs graph by default, with the code publish-spa wraps, and an `lsg` site of the same garden. It compares the words of every page's blocks and the rows of its properties box, and names the pages that differ. [[lsg/Ref/Parity Tasks]] describes how it builds Logseq's export and what it leaves out.
	- ## Before the harness
		- On 2026-10-24, `lsg publish` built only the namespaces named with `--include`. No option selected every page, so a whole-garden build needed one `--include` per top-level namespace: 1,042 of them for the encode garden.
		- A debug build run with all 1,042 roots and `--filename-format triple-lowbar` produced:
			- | Measure | Value |
			  | --- | --- |
			  | Wall time | 64 s (debug build) |
			  | Source pages published | 6,984 |
			  | Generated namespace pages | 3,266 |
			  | HTML files | 10,251 (each page plus `index.html`) |
			  | Withheld pages (`public:: false` or nesting) | 2 |
			  | Skipped journals | 505 |
			  | Copied assets | 105 |
			  | Diagnostics | 24,692 |
			- The largest diagnostic groups were unresolved page references (16,108), unsupported macros (4,189), footnotes (3,346), drawers (262) and math (106).
		- **Routes.** Each page was written to `p-<16 hex digits>.html`, derived from its title, and `lsg` wrote no title-to-file list, so finding the file for a title meant parsing the links in `index.html`.
		- **Configuration.** `lsg` read `:file/name-format` only through the required `--filename-format` flag and did not read `:hidden`. Page properties appeared only when the garden opted in with `:publishing/page-properties? true`, which the encode garden does not set, so the `lsg` build showed no Properties block where Logseq shows one, for example on `tmux`.
		- **Fresh output.** `lsg` refuses an existing output directory, so the harness picks a new directory on each run.
		- Every page `lsg` published also existed in Logseq's build. The pages Logseq had and `lsg` lacked fell into these groups:
			- **Journals.** Logseq published 505 journals with content; `lsg` skipped them.
			- **Reference-only pages.** About 6,600 Logseq pages existed only because something referenced them. `lsg` published no page for them and reported 16,108 unresolved references.
			- **Titles with `%2F`.** Three pages whose file names encode `/` as `%2F`, for example `Claude/Code/Q/If I rename a CLI session with /rename, can I resume it in Claude Desktop's Code tab?`. The decoded title has a segment with a leading space, which `lsg`'s namespace rules rejected.
			- **Hidden files.** No entry in the encode garden's `:hidden` list hid a page file, so the two builds agreed.
			- **Unexplained pages with content.** `Contents` (Logseq's built-in sidebar page; the garden's `pages/contents.md` is empty), `parameters`, and a page parsed from a Markdown file under `assets/`.
	- ## Fixture pages
		- [`scripts/parity/fixtures.txt`](https://github.com/codekiln/logseq-gardener/blob/main/scripts/parity/fixtures.txt) lists the pages that `parity:screenshots` and `parity:perf` use. The harness started with the pages below, each exercising features the feature pages compare; each feature page names the fixture pages it added.
		- | Page | Why |
		  | --- | --- |
		  | `tmux` | Page properties, headings, Linked References (on 2026-10-24, `lsg`: 114 pages and 176 referring contexts; Logseq counts journal references too), hierarchy |
		  | `Foam` | Page embed |
		  | `Logseq/Block/Id` | Block embeds and block references |
		  | `Logseq/Queries` | Queries |
		  | `Lazygit` | Flashcards (`#card`) |
		  | `rw-cli` | Collapsed blocks |
		  | `ff` | Fenced code |
		  | `Contrando` | `{{video}}` embeds |
		  | `AI/E/WS/25/d1` | `{{youtube}}` embeds |
		  | `GitP/A/Session/24/11/19-Tue` | Remote artwork and audio |
		  | The journal for 2026-10-23 | Journal page |
		  | Home route and All pages | Journals home and All pages table |
