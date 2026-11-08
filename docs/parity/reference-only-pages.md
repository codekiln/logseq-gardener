# Publish pages that exist only as link targets

Logseq makes a page for every name a block links to or tags, whether or not a file holds it, and its published site lets a reader open that page and see the blocks that name it. `lsg` v0.0.38 still renders those links as plain text: on the encode garden, 3,872 page names stay unresolved, among them `[[Deno Deploy]]`. This change publishes a small static page for each such name, with its Linked References, and makes every link and tag to it a real link. A rough build of that output adds about 4,200 pages and 84 MB to the 515 MB site; most of the growth is precomputed Unlinked References, which the plan below avoids. This document is step 1, Analyze, of [issue #351 Publish pages that exist only as link targets, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/351). It records Logseq's behaviour, what `lsg` does now, and the gaps a test can check.

**Status, 2026-11-08:** step 3, Apply, publishes these pages. On the encode garden the unresolved-reference diagnostics fall from 9,366 to the 2 withheld `GitP/.../wav` names, and the build publishes 4,656 reference-only pages. The [verification record](../../openspec/changes/publish-reference-only-pages/verification.md) holds the measurements; the sections below describe `lsg` 0.0.38 as analysed.

The work follows four steps, one pull request each: Analyze (this document), Propose (an OpenSpec change), Apply (code and tests), and Sync and archive. The reasons are in [Why four steps](#why-four-steps).

Measured on 2026-11-08 with `lsg` 0.0.38 built from logseq-gardener `b1773f9`, the encode garden at `66b81e9543`, Logseq's source at `og` `6e7afa8eb` and Logseq's docs at `628cdde`. The encode garden's `gh-pages` branch now holds `lsg`'s own build, so the Logseq comparison uses the Logseq 0.10 publish-spa build of the garden that `parity:compare` downloaded on 2026-10-25 (garden `083106dc`, `gh-pages` `8132c95b`), read with [`scripts/parity/logseq_db.py`](../../scripts/parity/logseq_db.py).

## The issue is still open

`mise run parity:build` on today's garden, with every top-level namespace, journals and search, publishes 9,202 pages and reports:

| Diagnostic | Count | Distinct names |
| --- | --- | --- |
| `Unresolved or excluded page reference` | 8,136 | 3,717 |
| `Unresolved or excluded tag` | 1,230 | 277 |
| Either, ignoring letter case | 9,366 | 3,872 |

The issue counted 19,809 and 1,918 diagnostics against v0.0.7; the number of names is about the same. `pages/Deno.md` and `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?` still render `<span class="page-reference unresolved"><span>Deno Deploy</span></span>`. 294 of the diagnostics repeat a page's line for its flashcard deck.

The 3,872 names fall into three groups:

- **376 names already have a generated namespace page but their links stay unresolved.** `[[AI]]`, `[[Ableton/Live]]` and `[[Person/Cal Newport]]` are prefixes of published titles, so `lsg` writes a page for each, and the Hierarchy section links to it. A link in a block's text does not: those pages are only in the namespace table, not in the page-name index that links use. They account for 1,881 of the diagnostics. The issue assumed these links already worked.
- **2 names are withheld pages.** `GitP/A/Session/26/10/22-Thu/Asset/Audio/Full/wav` and `.../Trimmed/wav` carry `public:: false`; journals `2026_10_22` and `2026_10_23` link to them. These stay unresolved.
- **3,494 names have no file at all.** These need new pages. 2,620 of them are named only once, the median is 1 occurrence and the 90th percentile is 4.

## What Logseq does

1. **A reference creates a page.** The parser collects every `[[name]]`, every tag and the block's marker and priority, plus every namespace parent of each name, and makes a page entity for each one that does not exist yet ([`with-page-refs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/block.cljs#L337-L372), [`page-name->map`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/block.cljs#L287-L335)). A tag must contain no `#`, space or line break ([`tag-valid?`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/util.cljs#L61-L64)), so `(pytest issue #10977)` makes the page `10977)`. A slash at either end of a name is dropped ([`remove-boundary-slashes`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/util.cljs#L106-L113)), so `(C#/D♭)` makes `D♭)`. Logseq's docs say opening a link to a page that does not exist creates it ([Backlink](https://github.com/logseq/docs/blob/628cdde/pages/term___backlink.md)).
2. **The title is the first spelling Logseq reads.** A page that already exists keeps its `:block/original-name`, so later spellings do not change it. `lsg` already models this rule for alias names and graph nodes in [`title_spellings.rs`](../../crates/logseq-gardener-sdk/src/title_spellings.rs). Today's garden writes `Typescript` 28 times and `TypeScript` 15 times; Logseq's build titles the page `Typescript`, and titles `MacOS`/`macOS` as `MacOS`.
3. **publish-spa keeps these pages.** The encode garden sets `:publishing/all-pages-public? true`, so the export removes only pages marked `public:: false` and their blocks, and keeps every other page entity, including those without a file ([`clean-export!`](https://github.com/logseq/og/blob/6e7afa8eb/deps/publishing/src/logseq/publishing/db.cljs#L111-L126), chosen in [`build-html`](https://github.com/logseq/og/blob/6e7afa8eb/deps/publishing/src/logseq/publishing/html.cljs#L135-L144)). The 2026-10-25 build holds 19,911 pages, 10,810 of them non-journal pages with no blocks.
4. **The page shows its title and the sections around the blocks.** With no blocks, the page body is the empty `Click here to edit...` bullet, which has no publishing check ([`page-blocks-cp`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L167-L204), [`dummy-block`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L113-L149)). Below it come Pages tagged with, Linked References, Hierarchy and Unlinked References, as on any page ([`page.cljs` L508-L524](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L508-L524)). In the browser, `VSCode/Terminal` (referenced, no file) showed exactly that ([Right Sidebar compatibility](../pages/lsg___Ref___Logseq%20Compatibility___Right%20Sidebar.md)).
5. **Every other view treats them as pages.** All pages lists every page entity ([`get-all-pages`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L124-L130)); the [All pages compatibility page](../pages/lsg___Ref___Logseq%20Compatibility___All%20Pages.md) counted 7,540 more rows than `lsg` on 2026-10-26. Search finds them by title, and the graph view draws them as ordinary nodes that open their page ([Graph View compatibility](../pages/lsg___Ref___Logseq%20Compatibility___Graph%20View.md)).
6. **Unlinked References scan every block when opened.** The query runs a regular expression over every block's text in the in-memory database ([`get-page-unlinked-references`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1325-L1350)).
7. **The cost is paid up front.** None of this costs Logseq's reader anything extra, because the reader already downloads the whole graph: a 15.8 MB `db.gz` and about 35 MB of JavaScript before the first page shows. [#106 Match Logseq's look and behaviour, beat its performance](https://github.com/codekiln/logseq-gardener/issues/106) rules that design out for `lsg`.

The Logseq build also has pages that this change does not add; see [Out of scope](#out-of-scope).

## What `lsg` does now

- **Links.** The publisher's `page_link` and `tag` look the name up in the page-name index of published pages and aliases. Without a single match, they write `page-reference unresolved` or a plain `tag` span and a diagnostic ([`site/render.rs` L1346-L1436](../../crates/logseq-gardener-sdk/src/site/render.rs)).
- **Generated namespace pages.** [`namespace_destinations`](../../crates/logseq-gardener-sdk/src/publication.rs) (L382-L417) creates a page for every `/` prefix of a published title that is not a published, withheld or alias name. The page shows a `Namespace — browse the pages below.` notice, Linked References, Hierarchy and Unlinked References, and links to it carry a dotted underline ([`site/page_status.rs`](../../crates/logseq-gardener-sdk/src/site/page_status.rs)). Those choices come from [issue #91](https://github.com/codekiln/logseq-gardener/issues/91), recorded in the [Hierarchy compatibility page](../pages/lsg___Ref___Logseq%20Compatibility___Hierarchy.md). These pages are not in the page-name index, so text links to them stay unresolved.
- **The SDK already knows the names.** [`references.rs`](../../crates/logseq-gardener-sdk/src/references.rs) records, per block, the names that resolve to no page (`GraphRefs::unresolved`), their namespace parents (`unresolved_parents`) and unresolved `tags::` names. [`graph.rs`](../../crates/logseq-gardener-sdk/src/graph.rs) turns them into `REFERENCE_ONLY` graph nodes. `Publication::garden_names` tells a name with a file, published or not, from a name with none.
- **The browser says "not published".** Clicking a reference-only graph node shows `<name> — not published` ([`site/graph.js`](../../crates/logseq-gardener-sdk/src/site/graph.js)), and the right sidebar shows `This page is not published.` ([`site/sidebar.js`](../../crates/logseq-gardener-sdk/src/site/sidebar.js)).
- **Hidden files.** `lsg` does not load files that `:hidden` in `logseq/config.edn` hides, as Logseq does not, so a link to such a file's page is a name with no file in both.

## Gaps

Each gap is a test. Fixture pages are in the encode garden unless named otherwise; see [Fixture pages](#fixture-pages).

1. **Link to generated namespace pages.** `[[AI]]`, `[[Ableton/Live]]` and `[[Person/Cal Newport]]` (`created-by::` on `Person/Cal Newport/Pod`) link to their generated namespace pages, and tags to such names link too. The 1,881 diagnostics for those 376 names go away.
2. **Publish a page for each name with no file.** `[[Deno Deploy]]` on `Deno` and on `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?` links to a page titled `Deno Deploy` whose Linked References list those two blocks.
3. **Tags make pages too.** `#10977)` on `Pytest/Warning/How To/Resolve AST-related Deprecation Warnings on Python 3.13` links to a page `10977)`. `company` in the `tags::` of `Sourcegraph` and 17 other pages gets a page whose Pages tagged with section lists all 18.
4. **Drop a slash at either end of a tag name.** `(C#/D♭)` on `Ableton/Live/UG/29 Audio Effect Reference/39 Tuner/04 Note Spellings` names the page `D♭)`, not `/D♭)`, and `/.NET` names `.NET`, as `[[a/b/]]` already does for page links.
5. **Title the page as Logseq does.** With two spellings, the page takes Logseq's: `Typescript`, not `TypeScript`; `MacOS`, not `macOS`; `LangGraph`, not `langgraph`. Today's garden has 23 names spelled more than one way, and the Apply step compares all 23 with Logseq's build. Links in either spelling open the one page.
6. **Give namespace parents of these names a page.** `1Password/Dev/ssh/Tutorial/Setup Walkthrough` links `1Password/Desktop/Settings/Developer/SSH Agent/Advanced`; Logseq also makes `1Password/Desktop/Settings`, `.../Developer` and `.../SSH Agent`. Each parent that is not already a page gets a generated namespace page. In the rough build below this added 348 namespace pages.
7. **Keep withheld names unresolved.** The journal `2026_10_22` keeps `GitP/A/Session/26/10/22-Thu/Asset/Audio/Full/wav` as plain text with its diagnostic, and no page of that name is written. The same holds for a page or an alias outside the `--include` selection: publishing with `--include Deno` only leaves a link to an existing, unselected page unresolved.
8. **Only published blocks create pages.** A name that only a withheld page, an unselected page or a skipped journal names gets no page, so `lsg` never publishes a name that a reader cannot already see in a published block. Its Linked References list only published blocks.
9. **Journal dates without a journal file.** `[[Nov 18th, 2024]]` on `Mac/OS/Q/How can I open a particular system setting from the command line?` opens a page titled with the garden's journal title format, as Logseq makes a journal page for it.
10. **Names with a space before a slash.** `[[MacOS /Applications Folder]]` on `Mac/OS/App` names the parent `MacOS ` with its trailing space, which Logseq keeps as a page. The page is published at its `p-<digest>.html` address, as `--readable-urls` already does for other titles with spaces at a segment end, with the existing diagnostic.
11. **Page content.** A reference-only page shows the title, then Pages tagged with when other pages tag it, Linked References, Hierarchy for a namespaced name, and Unlinked References, in Logseq's order. It shows no Namespace notice and no `Click here to edit...` bullet; see [Out of scope](#out-of-scope).
12. **Plain links.** A link or tag to a reference-only page looks like any page link: no `unresolved` class and no dotted underline. Links to generated namespace pages keep their issue-#91 style.
13. **Other views.** All pages lists reference-only pages with their backlink count and no dates. Search finds `Deno Deploy` by title. Clicking the `Deno Deploy` graph node opens its page, and the page graph card and right sidebar card open it as well. `not published` remains only for withheld names.
14. **Diagnostics.** On the encode garden with every namespace selected, the unresolved-reference diagnostics name only the two withheld pages.
15. **Route collisions do not stop the build.** A reference-only name whose address collides with another page's, or with a reserved root file, is skipped with a diagnostic instead of failing the build, unlike today's collision error for namespace pages. The rough build below, with and without `--readable-urls --redirect-old-urls`, met no collision.
16. **Incremental output.** After adding one block that names a new page, a rebuild changes the referencing page, the new page, its references fragment and the search files, and leaves the other pages unchanged, as [#537 rewrite only the changed pages and search files after an edit](https://github.com/codekiln/logseq-gardener/pull/537) requires for ordinary edits. Removing the last reference removes the page.
17. **Match Logseq.** `parity:compare` on the encode garden reports no reference-only page from the gaps above as missing from `lsg`. The remaining groups are in [Out of scope](#out-of-scope).

## Size and speed

### Rough build

To estimate the cost, a copy of today's garden was given one file per name holding only `title:: <name>` and published with the same options as `parity:build`. This overstates the cost in two ways: each stub file is a real page, so `lsg` precomputed Unlinked References for it, and each page carries an empty property row. Both builds use `p-<digest>.html` addresses and leave out the few Markdown files outside `pages/` and `journals/`.

| Part | Today | With 3,872 names | Change | Budget in [`budgets.toml`](../../scripts/parity/budgets.toml) |
| --- | --- | --- | --- | --- |
| Whole site | 514.5 MB, 50,565 files | 598.9 MB, 67,066 files | +84.4 MB | 550 MB |
| Page HTML at the site root | 205.4 MB, 14,720 files | 248.3 MB, 19,045 files | +42.9 MB | 200 MB |
| `_refs/` | 87.2 MB | 126.3 MB | +39.1 MB | 100 MB |
| `_refs/` Linked References | 37.7 MB | 45.0 MB | +7.3 MB | |
| `_refs/` Unlinked References | 43.7 MB | 75.1 MB | +31.4 MB | |
| `_graph/` | 20.0 MB | 20.7 MB | +0.7 MB | |
| Search title index | 244 KB gzip | 319 KB gzip | +75 KB | 300 KB gzip |
| All pages list | 146 KB gzip | 178 KB gzip | +32 KB | 300 KB gzip |
| Build, one warm run | 18.4 s | 20.9 s | +14% | 10% slower |

- The page HTML grows by about 10 KB per page: the `Deno Deploy` page is 9.1 KB, 3.3 KB gzip, almost all of it the shared header and sidebar every page carries. Today's generated namespace pages are 10.3 KB, 3.6 KB gzip.
- Unlinked References grow most, and few names cause it: the stubs for `To`, `)`, `` ` ``, `1`, `2`, `About`, `Work` and `Video` each wrote 0.4 to 2.9 MB, and the largest 100 names wrote 72% of the 31.4 MB.
- Today's garden is already above the page-HTML budget (205.4 MB against 200 MB) before this change. CI checks the budgets on a pinned, older garden commit, so CI passes today.
- With `--readable-urls --redirect-old-urls`, as the garden's `mise run site:build` publishes, the build also succeeded with all 13,071 pages and no collision.

### What a reader downloads

A reader who opens `Deno Deploy` downloads its 3.3 KB gzip page, the shared stylesheets and scripts that are cached after the first page, and a 0.9 KB gzip Linked References fragment when the section opens. A first visit to the same page in Logseq's build downloads the whole 15.8 MB graph and about 35 MB of JavaScript first. No page's first paint grows with the number of reference-only pages, so the page-load budgets in [`budgets.toml`](../../scripts/parity/budgets.toml) (100 KB gzip of data and 16 KB of script before the title, title within 300 ms on desktop and 1,500 ms on a phone, 10 MB peak heap) apply unchanged.

### Suggested budgets for the proposal

1. A reference-only page's HTML is at most 11 KB, 4 KB gzip. The page-load budgets above apply to `Deno Deploy` as a new fixture page.
2. Unlinked References add at most 10 MB to `_refs/`. Precomputing them for every reference-only page costs 31 MB. The proposal chooses between answering the section in the browser from the search shards a reader already fetches, which also matches Logseq's scan-on-open, and precomputing only within that limit.
3. The search title index stays within its 300 KB gzip budget. With these titles it reaches 319 KB. Dropping the address from each title entry, which the reader can derive from the title, reduces the index from 244 KB to 120 KB today and to 150 KB with these titles. Moving reference-only titles to a second file loaded on the first keystroke is the other option.
4. The All pages list stays within its 300 KB gzip budget; the measured 178 KB needs no change.
5. Linked References fragments keep their 100 KB limit, and Linked References add at most 10 MB to `_refs/`; the rough build added 7.3 MB.
6. The page-HTML and whole-site budgets need new values, because about 43 MB of page HTML is the price of one static page per name. The proposal records new `page_html_mb` and `site_mb` values and their reasons; about 260 MB and 600 MB fit the estimates above with Unlinked References computed in the browser.
7. A whole encode garden build stays within `[site]` `build_slowdown_pct` (10%), measured with `parity:perf --baseline-lsg`. The one run above, with Unlinked References precomputed, was 14% slower.

## SDK and publisher

Following the [shared-engine comment on #106](https://github.com/codekiln/logseq-gardener/issues/106), the SDK answers which pages exist and what references them; the publisher only writes HTML for those answers.

- **SDK** (shared with the CLI and the future language server):
  - The set of reference-only pages: names that published blocks link, tag or list in `tags::`, with their namespace parents, that no page, alias, withheld page or unselected page answers to. `references.rs` already finds these names per block.
  - Each page's title by Logseq's first-spelling rule, from `title_spellings.rs`.
  - The page-name index (`page_names.rs`, filled in `publication.rs`) resolving these names and the generated namespace pages, so every caller resolves `[[Deno Deploy]]` and `[[AI]]` the same way. This fixes gap 1 without rendering code.
  - Routes and collision handling for the new pages in `publication.rs`, beside `namespace_destinations`.
  - A page kind for reference-only pages in `page_list.rs` and in the plugin view (`publication.rs`), routes for `REFERENCE_ONLY` graph nodes in `graph.rs`, and their titles in the search title index (`search.rs`).
  - Linked References, Pages tagged with and Hierarchy for these pages, from the existing reference and namespace indexes. `lsg` commands and the language server can then report that `Deno Deploy` is a page with two references.
- **Publisher** (`site/`, rendering only):
  - The page template: a generated page's sections without the Namespace notice.
  - Removing the `unresolved` markup and diagnostic for names the SDK now resolves, in `site/render.rs`.
  - All pages rows, the graph's click handler (`graph.js`) and the right sidebar card (`sidebar.js`) opening the page.
  - Unlinked References in the form the proposal chooses.

## Interactions

- **Withheld and private pages.** A name whose page is withheld by `public:: false`, by the all-pages-public rule or by selection stays unresolved and gets no page (gaps 7 and 8). Logseq's export keeps the link text and drops the page; `lsg` keeps the name as plain text, as the issue asks.
- **Generated namespace pages.** When a name is both a namespace prefix and a link target, it stays one generated namespace page with its notice, and links resolve to it. The Hierarchy section keeps leaving out rows for pages with no file, as issue #91 asked, so reference-only pages appear inside their descendants' rows only as linked prefixes.
- **Selection.** A reference-only page is published when a published block names it, whatever its top-level namespace, because no file exists for `--include` to select. Publishing `--include Deno` publishes `Deno Deploy`. The proposal should record this as a decision.
- **Excluded from the graph view.** `exclude-from-graph-view::` needs a file, so it does not apply to these pages.

## Why four steps

The change touches the SDK's page set, name resolution, titles, routes, the reference index, the graph, search, All pages and the publisher. It needs written decisions on four questions: whether reference-only pages follow `--include`, how Unlinked References are answered, how the title index stays within budget, and the new site budgets. It also changes CI budgets. The OpenSpec change records those decisions before code, and each pull request stays small enough to review.

## Fixture pages

Encode garden, by gap:

- `Deno`, `Deno/Q/Under what circumstances would I need or want deno installed globally on my machine?` → `Deno Deploy` (gaps 2, 13).
- `Person/Cal Newport/Pod` (`created-by:: [[Person/Cal Newport]]`), any page linking `[[AI]]` or `[[Ableton/Live]]` (gap 1).
- `Pytest/Warning/How To/Resolve AST-related Deprecation Warnings on Python 3.13` → `10977)`; `Sourcegraph` → `company` (gap 3).
- `Ableton/Live/UG/29 Audio Effect Reference/39 Tuner/04 Note Spellings` → `D♭)` (gap 4).
- The pages naming `Typescript`/`TypeScript`, `MacOS`/`macOS`, `langgraph`/`LangGraph` (gap 5).
- `1Password/Dev/ssh/Tutorial/Setup Walkthrough` (gap 6).
- Journals `2026_10_22` and `2026_10_23` with the withheld `GitP/A/Session/26/10/22-Thu/Asset/Audio/Full/wav` and `.../Trimmed/wav` (gap 7).
- `Mac/OS/Q/How can I open a particular system setting from the command line?` → `Nov 18th, 2024` (gap 9).
- `Mac/OS/App` → `MacOS ` (gap 10).

Added in Apply to `tests/fixtures/parity-garden`: a page linking a name with no file, a tag with a boundary slash, a name in two spellings, a link to a `public:: false` page, a name only a private page links, a file hidden by `:hidden`, and a nested name whose parents have no file.

## Out of scope

- **Pages for property names.** Logseq makes a page for each property key such as `date`, `banner` or `created-by` (`:property-pages/enabled?`); about 106 such names are in Logseq's build and not in `lsg`. That is a separate change.
- **Logseq's built-in pages.** `TODO`, `DOING`, `A`, `card`, `Favorites` and 9 others exist in every Logseq graph whether or not anything links them.
- **The `Click here to edit...` bullet.** It is an editing control; `lsg` already leaves out Logseq's in-memory editing controls on All pages.
- **Rows for pages with no file in the Hierarchy section.** Issue #91's rule stays.
- **Embeds of reference-only pages** (`{{embed [[Deno Deploy]]}}`), which keep today's fallback. They were not counted.
- **Aliases.** Logseq stores each alias as its own page entity, which is why its build has about 1,366 more names; `lsg` resolves an alias to its page, so these are not gaps. About 930 further names in Logseq's 2026-10-25 build were not classified; the samples checked were aliases written in a form the count missed and names from pages edited since that build.
