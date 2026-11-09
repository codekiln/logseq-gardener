## Context

The [Reference-only pages analysis](../../../../docs/parity/reference-only-pages.md) measured `lsg` 0.0.38 on the encode garden at `66b81e9543` against Logseq's publish-spa build of 2026-10-25. Of the 3,872 unresolved names, 3,494 have no file, 376 already have a generated namespace page and 2 are withheld pages.

The SDK already knows most of what the publisher needs. `references.rs` records, per block, the names that resolve to no page (`GraphRefs::unresolved`), their namespace parents and unresolved `tags::` names; `graph.rs` turns them into `REFERENCE_ONLY` nodes; `title_spellings.rs` applies Logseq's first-reference spelling to alias names; `Publication::garden_names` tells a name with a file from a name without one. What is missing is a set of pages built from those names, routes for them, and a page-name index that resolves them and the generated namespace pages. The publisher's `page_link` and `tag` in `site/render.rs` mark every name outside the index `unresolved`.

Logseq pays for these pages by sending every reader the whole graph: 15.8 MB of `db.gz` and about 35 MB of JavaScript before the first page shows. [Issue #106 Match Logseq's look and behaviour, beat its performance](https://github.com/codekiln/logseq-gardener/issues/106) asks for Logseq's look and behaviour with per-page loading, incremental search and bounded phone memory, so this design does the graph work at build time and sends each reader one small static page.

## Goals / Non-Goals

**Goals:**

- Every link, tag and page-valued property that opens a page in Logseq's build opens a published page in `lsg`'s build, and the unresolved diagnostics name only withheld, unselected or ambiguous names.
- A reference-only page's first paint costs the same whatever the garden's size, and no reader downloads data about pages they did not open.
- The CLI and the language server get the same answers as the publisher: which pages exist, their titles and what refers to them.

**Non-Goals:**

- Pages for property names such as `date` or `created-by`, which Logseq makes when `:property-pages/enabled?` is on.
- Logseq's built-in pages, such as `TODO`, `card` and `Favorites`, that exist whether or not anything links them.
- Search by title for generated namespace pages.
- An `lsg` command that lists reference-only pages and their references.
- Page embeds of reference-only pages.

## Decisions

### Where the work lives

The SDK answers which pages exist, what they are called and what refers to them; the publisher renders those answers.

| Part | SDK (`garden-core`) | Publisher (`site/`) |
| --- | --- | --- |
| Which names are pages | A new `reference_only` module collects the unresolved names and parents that `references.rs` records for published blocks, drops names with a file and names only withheld or unselected blocks write, and returns the set with each page's key, title and journal date. | — |
| Titles | The same module applies `title_spellings.rs`'s first-reference rule in Logseq's parse order, boundary-slash removal, journal titles and built-in spellings. | — |
| Name resolution | `publication.rs` fills the `PageNameIndex` with reference-only pages and generated namespace pages, marked by kind, so links, tags and properties resolve through one lookup. Alias, embed, block reference and homepage lookups keep using the source pages only. | `page_link` and `tag` render a link for every resolved name and keep `unresolved` for the rest. |
| Routes | `publication.rs` routes reference-only pages beside `namespace_destinations` and turns a collision into a diagnostic. | Writes each page at its route. |
| References | `ReferenceIndex` answers linked references, tagged pages and unlinked mentions for reference-only keys; `NamespaceIndex` gives hierarchy rows. | Writes the reference sections and fragments; limits Unlinked References to one file for reference-only pages. |
| Other views | `page_list.rs` adds kind `Reference`; `graph.rs` gives published reference-only nodes routes; `search.rs` adds page records; the publication view adds kind `reference`. | All pages rows, the journals feed, `graph.js` opening nodes with a route, search computing routes. |
| Updates | The module updates the set when the reference index replaces one document's contribution. | Rewrites only the files whose content changed. |

### Which names become pages

A name becomes a reference-only page when a published block writes it as a page reference, a tag, a `tags::` value or another page-valued property value, or when it is a namespace parent of such a name, and no published page, journal, alias or generated namespace page answers to it and no page file has it as its title. Task markers and priorities add no page. This matches Logseq's `with-page-refs` for written names, leaves Logseq's built-in pages to a later change, and keeps withheld and unselected names unresolved, as [Issue #351](https://github.com/codekiln/logseq-gardener/issues/351) asks.

A namespace parent that is only a prefix of reference-only names becomes a reference-only page. A name that is a prefix of a published title stays a generated namespace page with its Namespace notice, as [Issue #91](https://github.com/codekiln/logseq-gardener/issues/91) set.

### The page

A reference-only page shows its title, then Pages tagged with, Linked References, Hierarchy and Unlinked References in the existing order and markup. The template is the generated namespace page's without the Namespace notice. Links to it use the ordinary page link markup. The Deno Deploy page in the rough build was 9.1 KB, 3.3 KB gzip, almost all of it the shared header and sidebar.

### Unlinked References

The SDK computes unlinked mentions for reference-only pages in the same one pass over block text as for other pages. For a reference-only page, the publisher writes only the first fragment file, about 50 pages. When the page has more mentions, the heading still counts every block and the list ends with `Showing the first N of M pages`. In the rough build, precomputing every group for these pages wrote 31.4 MB, and the largest 100 names, such as `To`, `)` and `About`, wrote 72% of it. Because the cost sits in so few names, a one-file limit shortens the lists of the pages with the most mentions and leaves the rest whole.

Answering the section in the browser from the search shards, as Logseq scans on open, was the other option. Search records hold plain text without link brackets or rendered markup, so the browser could not tell an unlinked mention from a link or render Logseq's groups, and names made of punctuation, such as `)`, have no words to look up.

### Search title index

A title entry carries its page's route only when the route differs from the one its title gives under the build's URL scheme, which the manifest names. The browser computes a digest route as All pages already does for its list, and a readable route by percent-encoding each segment of the title as the SDK's `readable_url` does. In the analysis this takes the encode garden's title index from 244 KB to 120 KB gzip today and to about 150 KB with the new titles, within the 300 KB budget. This is search format version 4; a page published before the change asks the reader to reload, as version 3 does.

Putting reference-only titles in a second file loaded on the first keystroke was the other option. It keeps every file under its budget but sends the same data to every reader who searches.

### Budgets and measurement

| Measure | Budget | Checked by |
| --- | --- | --- |
| Reference-only page HTML | 11 KB, 4 KB gzip | `parity:budgets` on the fixture garden in CI; `parity:perf` on `Deno Deploy` |
| First paint of `Deno Deploy` | `[desktop]` and `[phone]`: 100 KB gzip of data and 16 KB of script before the title, title within 300 ms on desktop and 1,500 ms on the phone profile | `parity:perf` |
| Peak JS heap on the phone profile | 10 MB on `Deno Deploy`; 20 MB after opening its Linked References | `parity:perf` |
| Other pages when the fixture garden gains 2,000 reference-only names | unchanged files | `parity:budgets` in CI |
| Added script | no new script file; the search engine grows by at most 0.5 KB minified for route computation; `[references]` `script_kb` stays 7 | `parity:budgets` in CI |
| Search title index | 300 KB gzip and 1.5 MB decoded, expected about 150 KB gzip | `parity:budgets` on the fixture garden in CI; `parity:perf --search-site` on the encode garden |
| Search keystroke, p95 | warm 50 ms desktop and 100 ms phone; cold 150 ms and 1,000 ms; heap growth 30 MB | `parity:perf --search-site` |
| Unlinked References of reference-only pages | one fragment file each, at most 100 KB as stored; 15 MB together | `parity:budgets` per file in CI; total recorded in `verification.md` |
| Linked References of reference-only pages | 10 MB together; the rough build added 7.3 MB | recorded in `verification.md` |
| Page HTML, `_refs/`, whole site | 260 MB, 115 MB, 600 MB | `parity:publish-bench --check` on the pinned encode garden commit in CI |
| Whole encode garden build | at most 10% slower than the build before this change | `parity:perf --baseline-lsg`; `parity:publish-bench --check` against recorded rows in CI |

The page HTML budget follows from about 3,800 pages of about 10 KB each: 205.4 MB today plus 42.9 MB in the rough build, with room for the garden to grow. The `_refs/` budget adds 7.3 MB of Linked References and at most 15 MB of Unlinked References to today's 87.2 MB. The site budget adds those and 0.7 MB of Page graph files to today's 514.5 MB. GitHub Pages serves at most 1 GB.

To measure on the whole encode garden against Logseq's build, the Apply step publishes the garden with `mise run parity:build` (every top-level namespace, journals and search) with and without this change, and:

- runs `parity:compare` against Logseq's 2026-10-25 publish-spa build and records the reference-only group;
- runs `parity:perf` on `Deno Deploy` and a search for `deno deploy` in both builds, on the desktop and phone profiles, and commits the rows to `docs/parity/performance.md`;
- runs `parity:perf --baseline-lsg` for the build time and `parity:publish-bench --record` for the output's size, file counts and parts, and commits the new rows to `docs/parity/publication-speed.json`, so CI's `parity:publish-bench --check` compares later builds with them.

`scripts/parity/budgets.toml` gains a `[reference_only]` table with `page_kb`, `page_gzip_kb`, `unlinked_files_per_page`, `unlinked_total_mb`, `linked_total_mb`, `search_engine_growth_kb` and `build_slowdown_pct`, and `[site]` takes the new values.

### Updates after an edit

The reference-only set is a function of the reference index's contributions, so replacing one document's contribution adds or removes the names that document writes. The publisher already rewrites only changed pages and search files after an edit ([PR #537 rewrite only the changed pages and search files after an edit](https://github.com/codekiln/logseq-gardener/pull/537)); a new reference-only page adds its own page, fragments and Page graph file, and changes All pages, the graph data and the search title index.

## Risks / Trade-offs

- [Names that look like accidents, such as `)` and `10977)`, get pages and All pages rows] → Logseq publishes the same pages; the Unlinked References limit keeps their cost small, and a garden author can fix the source text.
- [Search format version 4 breaks hosts that read the title index] → The loader asks for a reload on older data, and the Static Search Data reference page documents the entry shape.
- [The encode garden's page HTML is already over its 200 MB budget] → CI checks a pinned, older garden commit; the new budgets are set from today's garden plus this change.
- [Titles depend on Logseq's parse order] → The Apply step compares all 23 names with more than one spelling against Logseq's build.
- [A reference-only page whose route collides is left out] → The diagnostic names both, and the rough build, with and without `--readable-urls`, met no collision.
- [`not published` now marks fewer graph nodes] → Only nodes without a route, such as task markers and withheld names, keep it.

## Resolved Questions

### 1 - Do reference-only pages follow `--include`?

Decided by the agent without codekiln: no. A reference-only page is published whenever a published block names it, whatever namespaces the selection includes, because it has no file for `--include` to select. Publishing `--include Deno` publishes `Deno Deploy`, which matches what a reader of the `Deno` pages expects to open.

### 2 - How are Unlinked References answered for reference-only pages?

Decided by the agent without codekiln: at build time, like every other page, with the published list limited to the first fragment file, about 50 pages, and a closing line naming how many pages it shows. Precomputing every group costs 31 MB, mostly for names like `To` and `)`, and answering in the browser from the search shards cannot tell links from mentions or render Logseq's groups.

### 3 - How does the search title index stay within its 300 KB budget?

Decided by the agent without codekiln: title entries leave out a route the browser can compute from the title, as All pages already does, in search format version 4. This halves the index for every reader who searches, while a second file loaded on the first keystroke would send the same data.

### 4 - What are the new site budgets?

Decided by the agent without codekiln: page HTML 260 MB, `_refs/` 115 MB and the whole site 600 MB, with the build at most 10% slower. These are today's measured sizes plus the rough build's growth for pages, Linked References and Page graph files, at most 15 MB of Unlinked References, and room for the garden to grow, well under GitHub Pages' 1 GB.

### 5 - Do namespace parents of reference-only names get generated namespace pages or reference-only pages?

Decided by the agent without codekiln: reference-only pages, unless the parent is also a prefix of a published title. The Namespace notice says to browse the pages below, but the Hierarchy section lists only pages with files, so a parent of reference-only names would point at an empty list. A reference-only page shows the blocks that name its descendants in its Linked References, as Logseq shows them.

### 6 - Do task markers and priorities create reference-only pages?

Decided by the agent without codekiln: no. Logseq's `TODO`, `DOING` and priority pages are built-in pages that exist whether or not anything links them, which the analysis left to a later change. A priority `[#A]` links to `A` when `A` is published for another reason, as today.

### 7 - What happens when a reference-only page's address collides with another page or a reserved file?

Decided by the agent without codekiln: the page is left out with a diagnostic naming both, references to it stay unresolved, and the build completes. A collision between generated pages is not the author's error, so it should not stop the build the way a collision between two files does.

### 8 - Does the journals feed list reference-only journals?

Decided by the agent without codekiln: yes, those dated on or before the build date, as Logseq's journals view lists every journal page up to today, each with its title, no blocks and its collapsed Linked References.

### 9 - What does a reference-only page show where Logseq shows its `Click here to edit...` bullet?

Decided by the agent without codekiln: nothing. The bullet is an editing control, and `lsg` already leaves out Logseq's editing controls on All pages.

### 10 - Which kind do reference-only pages have in the page list, the publication view and `lsg-site.json`?

Decided by the agent without codekiln: `Reference` in the page list and `reference` in the publication view and `lsg-site.json`, and `Journal`/`journal` for a date name. A plugin or tool can then tell a page with no file and no properties from a page with a file, as it already can for generated namespace pages.

### 11 - Does `lsg publish` report reference-only pages?

Decided by the agent without codekiln: yes, as a separate count in the human and JSON reports, with the page count unchanged. On the encode garden about 3,800 new pages appear, and the report should say where they came from.

### 12 - Do links to generated namespace pages in page text keep the Namespace link style?

Decided by the agent without codekiln: yes, the dotted underline and accessible Namespace name that issue #91 set for every link to a generated namespace page. Links to reference-only pages use the ordinary style, as Logseq draws every page link the same way.

### 13 - Are reference-only titles searchable while generated namespace titles stay out?

Decided by the agent without codekiln: yes. The analysis found that Logseq's search finds reference-only pages by title, and the issue asks for them; adding generated namespace titles changes an existing search decision and is listed as a non-goal.
