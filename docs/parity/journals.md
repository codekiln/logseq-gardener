# Journals and the Journals home page

Logseq's published site opens on a Journals feed: the encode garden's day pages, newest first, three at a time and seven more each time the reader nears the bottom, each with its blocks and a collapsed Linked References section. Every journal is also an ordinary page at `#/page/<title>`, titled with `:journal/page-title-format` (`yyyy-MM-dd EEE` in the encode garden, so `journals/2026_10_23.md` is "2026-10-23 Fri"). `lsg` publishes none of this. It skips all 505 journal files, its home page is a 1.2 MB list of every page, and the 778 links to journal titles in published pages render as unresolved text.

The work splits in two. Journal identity (file name to date to title, and parsing `[[Feb 1st, 2025]]`-style links back to a date) is graph logic that belongs in the SDK next to page titles and aliases. The feed is a publisher view, and it is the one place where copying Logseq's architecture would be expensive: Logseq can show three journals only after it has downloaded and inflated the whole graph, while `lsg` can write the first three journals into `index.html` and load later batches as small static files.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `09cf7fb` (506 journal files; served locally) and an `lsg` build of the whole garden from `ac32058` (`mise run parity:build`, then `parity:compare`).

## What Logseq's published site shows

The local `logseq/og` checkout is `1.0.0-12-g6e7afa8eb`, the file-graph continuation of 0.10; its journal code matches 0.10's structure. Links below point at that commit (`frontend` = `src/main/frontend`). The live build agrees with the source except where noted.

### Configuration in the encode garden

From `logseq/config.edn`:

- `:journal/page-title-format "yyyy-MM-dd EEE"`.
- `:default-home` is unset, so the home route shows journals.
- `:feature/enable-journals?` is unset, which means enabled.
- `:ref/linked-references-collapsed-threshold 0`, so every Linked References section starts collapsed.
- `:default-queries {:journals [NOW, NEXT]}`, shown only on today's journal.

### The home route

[`components/container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L710-L747) `main-content`:

- 1. If `:default-home` names an existing page, `#/` redirects to it.
- 2. On a published site with no default home and no journals, `#/` redirects to All pages.
- 3. Otherwise `#/` renders the journals feed.

### The journals feed

[`components/journal.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/journal.cljs#L26-L87) and [`db/model.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L1090-L1111):

- **Which journals:** every page with a journal date on or before today, newest first. Future dates are left out. Nothing filters on content, so a date that exists only because a page links to it is listed too.
- **Batches:** 3 journals at first ([`state.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L56)), then 7 more each time the scroll comes within 500px of the bottom ([`handler/page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/page.cljs#L671-L679), [`ui.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/ui.cljs#L477-L511)). An "Earlier" link sits below the last journal. On the live build, scrolling the home page six times showed 24 journals, 2026-10-24 Sat back to 2026-10-01 Thu.
- **Each journal:** an `h1` title that links to the journal's page (`a.title.journal-title`, journal.cljs L42–56), then the whole block tree, expanded. Journals below the first render when they scroll into view.
- **Linked References** under every journal that has any, collapsed in the encode garden because of the threshold of 0 ([`reference.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/reference.cljs#L131-L162)).
- **Today's journal only:** the `:default-queries :journals` queries and a SCHEDULED AND DEADLINE section ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L211-L226), [`scheduled_deadlines.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/scheduled_deadlines.cljs#L13-L41)). On a published site "today" is the reader's date, so these appear only when the build happens to include that day.

The reference board `Prototype.md` shows four journals without references. The source and the live build show three, then batches of seven, each with Linked References.

### Journal titles, pages and links

- **Title from the file:** `journals/2026_10_23.md` is parsed as a date and re-rendered in `:journal/page-title-format` ([`extract.cljc`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/extract.cljc#L30-L63), [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/block.cljs#L273-L326)). A `title::` property would win, but no encode-garden journal has one. The default format, when the key is unset, is `MMM do, yyyy` ([`config.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/config.cljs#L70-L76)).
- **Links in other formats:** a page reference is treated as a journal if it parses with the configured format, `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd` ([`date_time_util.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/date_time_util.cljs#L15-L51)). So `[[Feb 1st, 2025]]` on `Docker/For Mac/How to Install into Zsh` opens 2025-02-01 Sat; the link text keeps the author's spelling.
- **URL:** `#/page/2026-10-23%20fri`, the lower-cased title.
- **Journal page:** the same layout as any page, with the title from the date and the `is-journals` class. Page properties show as on other pages (`2019-08-07 Wed` shows `logseq-entity` and `alias:: 1987`). Linked and unlinked references appear as on any page (`2026-10-23 Fri` has 1 Linked Reference, from the LangSmith question page). The namespace hierarchy section is skipped ([`page.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/page.cljs#L410-L540)).
- **Link-only journals:** Logseq's build has 534 journal pages: the 506 files plus 28 dates that only links mention (for example `2024-11-18 Mon`, `1932-01-24 Sun`). Each is a journal page whose only content is its Linked References.

### Left sidebar and shortcuts

- The sidebar's first item is Journals (calendar icon), active on the home route; it is hidden only when `:feature/enable-journals?` is `false` and `:default-home` is set ([`container.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/container.cljs#L418-L438)).
- `g j` goes to the journals feed and scrolls to the top ([`route.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/handler/route.cljs#L205-L212)); with a custom home it goes to `#/all-journals` instead.

## What lsg does today

- `publish_site_with_options` counts and skips every file under `journals/` (`site/mod.rs`, `skipped_journals: 505`). `page_title` returns `UnsupportedJournal` for journals because it has no date format.
- `GardenSettings` already reads `:feature/enable-journals?` but not `:journal/page-title-format`, `:journal/file-name-format` or `:default-home`.
- `index.html` is the full page list (1,230,364 bytes for the whole garden) unless `--homepage` names a page, in which case the list moves to `pages.html`.
- Links to journal titles render as `<span class="unresolved">` with a diagnostic: 778 of the 16,108 "Unresolved or excluded page reference" diagnostics name a `yyyy-MM-dd EEE` title.
- The app shell from #109 has no Journals sidebar item and no `g j`.

## Gaps

Each gap names a check the apply step can run against the fixture pages.

- 1. **Journal identity in the SDK.** Derive a journal's date from its file name (`yyyy_MM_dd`, or `:journal/file-name-format`) and its title from `:journal/page-title-format`, defaulting to `MMM do, yyyy`. Support the date tokens the garden and Logseq's defaults use (`yyyy`, `MM`, `M`, `dd`, `d`, `do`, `MMM`, `MMMM`, `EEE`, `EEEE`). Check: `journals/2026_10_23.md` is titled `2026-10-23 Fri`; the same file under the default format is `Oct 23rd, 2026`.
- 2. **Journal pages.** Publish every journal file as a page, with its blocks, page properties, aliases and Linked References, and without a namespace hierarchy section. Check: `2026-10-23 Fri` shows its Filed and Updated blocks and 1 Linked Reference; `2019-08-07 Wed` shows its properties and is reachable as `1987`.
- 3. **Journal links in any date format.** A reference that parses as a date in the configured format, `MMM do, yyyy`, `yyyy-MM-dd` or `yyyy_MM_dd` links to that journal, keeping the author's text. Check: `[[Feb 1st, 2025]]` on `Docker/For Mac/How to Install into Zsh` links to `2025-02-01 Sat`; the 778 journal-title diagnostics disappear for dates that have files.
- 4. **Selection.** `lsg publish` selects pages by namespace; journals have none. The proposal decides how journals are selected (for example, published whenever journals are enabled and the selection is not empty) and how `public:: false` applies. Check: the encode-garden build publishes all 505 journals and withholds a fixture journal with `public:: false`.
- 5. **Home page feed.** Without `--homepage`, `index.html` shows journals dated on or before the build date, newest first: three in the HTML, then seven more per batch as the reader nears the bottom, with an "Earlier" link that works without JavaScript. Each journal has an `h1` linking to its page, its expanded blocks and a collapsed Linked References section. Check: for garden commit `09cf7fb`, `index.html` lists `2026-10-24 Sat`, `2026-10-23 Fri` and `2026-10-22 Thu`, as Logseq's build does; scrolling loads `2026-10-21 Wed` through `2026-10-15 Thu`; with JavaScript off, Earlier leads to the next seven.
- 6. **Where the page list goes.** The home page currently holds the page list. The proposal decides where it moves (`pages.html`, as `--homepage` already does, until #117 replaces it with the All pages table). Check: the sidebar's All pages item opens the list from every page.
- 7. **Sidebar item and shortcut.** Add Journals as the first left-sidebar item, active on the home page, hidden when `:feature/enable-journals?` is `false`; add `g j`. Check: Journals is first in the sidebar on `tmux` and is marked current on the home page; `g j` opens the home page.
- 8. **Link-only journals.** Logseq lists 28 dates that only links mention, as journals with just their references. These belong with the reference-only pages (6,616 in `parity:compare`) that `lsg` does not publish today. The proposal decides whether this change publishes them or leaves them to the references issue (#115). Check: whichever is chosen, no link to `2024-11-18 Mon` is broken.
- 9. **Today's queries.** The NOW/NEXT default queries and SCHEDULED AND DEADLINE depend on the reader's date and on queries (#124). A static build cannot know the reader's date. The proposal decides whether to omit them (recommended until #124) or render them for the build date. Check: the home page shows no empty query sections.
- 10. **Performance.** Logseq shows the first journal only after downloading 15.8 MB of `db.gz` and ~35 MB of script. The feed's first paint must not grow with the number of journals: the home page holds three journals' HTML, each later batch is a separate small file, and nothing loads the whole journal set. Linked References under each journal should load with the journal, or on expand, not all up front. Check: `parity:perf` on the home page meets the first-paint byte and script budgets, and adding 1,000 journals leaves `index.html` the same size.

## Where the work belongs

- **SDK:** date-format parsing and formatting, journal identity (file name to date to title), recognising journal references in any accepted format, the ordered journal list with dates, and the per-journal reference sets. The CLI and the language server need the same answers, for example to resolve `[[Feb 1st, 2025]]` or list recent journals.
- **Publisher:** journal pages, the home feed and its batch files, the sidebar item and `g j`.

## Fixture pages

| Page | Why |
| --- | --- |
| Home route | The journals feed: first three journals, batch loading, Earlier |
| `2026-10-23 Fri` | An ordinary journal page with one Linked Reference |
| `2019-08-07 Wed` | A journal with page properties and the alias `1987` |
| `Docker/For Mac/How to Install into Zsh` | Links `[[Feb 1st, 2025]]` and `[[Feb 2nd, 2025]]` in the default date format |
| `LangSmith/Deployment/Q/Can a LangSmith Cloud deployment call Amazon Bedrock through an IAM role instead of stored AWS keys?` | Links `[[2026-10-23 Fri]]` in the configured format |

## Out of scope

- The `#/all-journals` route, which Logseq uses only when `:default-home` is set; the encode garden does not set it.
- Creating today's empty journal, which Logseq may do in the reader's browser. A static site cannot write pages.
- Shift-click to open a journal in the right sidebar (#119).
- The NOW/NEXT queries and SCHEDULED AND DEADLINE section beyond gap 9's decision (#124).
