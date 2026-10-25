## Context

The [journals analysis](../../../docs/parity/journals.md) compared Logseq's `gh-pages` build of the encode garden (commit `09cf7fb`) with an `lsg` build from `ac32058`. Logseq's `#/` shows three journals, then seven more per scroll batch, each with its blocks and a collapsed Linked References section. Every journal is a page titled with `:journal/page-title-format` (`yyyy-MM-dd EEE` in the encode garden). `lsg` skips every file under `journals/` (`publish_site_with_options` in `crates/logseq-gardener-sdk/src/site/mod.rs`), and `page_title` returns `UnsupportedJournal` for them.

Logseq can show its first journal only after downloading 15.8 MB of `db.gz` and about 35 MB of script. The feed is where copying Logseq's design would cost most, so the feed is built as static batch files.

Two consumers constrain the change. Gitpa (Codex's work) runs `lsg publish` with namespace selectors and must keep its output. The [published-app-shell](../../specs/published-app-shell/spec.md) spec reserved the first sidebar position for Journals.

## Goals / Non-Goals

**Goals:**

- Readers of the encode garden see Logseq's journals feed on the home page and can open every journal as a page.
- Links to journals resolve in every date format Logseq accepts.
- The home page's size and first paint stay the same as the garden gains journals.
- The CLI and the future language server get journal dates, titles and date-reference parsing from the same SDK operations the publisher uses.

**Non-Goals:**

- Publishing the 28 dates that only links mention as reference-only journal pages. They belong with the 6,616 other reference-only pages that [Issue #115 — Match Logseq's linked references, filters, unlinked references and tagged pages](https://github.com/codekiln/logseq-gardener/issues/115) covers.
- Today's NOW/NEXT queries and SCHEDULED AND DEADLINE section, which depend on the reader's date and on [Issue #124 — Render Logseq queries at build time](https://github.com/codekiln/logseq-gardener/issues/124).

## Decisions

**Where the work lives.** The SDK gains a public `journals` module:

- `DateFormat::parse(pattern)` accepts the tokens `yyyy`, `MM`, `M`, `dd`, `d`, `do`, `MMM`, `MMMM`, `EEE`, `EEEE`, quoted literals and other literal characters, and returns an error naming the first unsupported token. `format(date)` writes a date; `parse(text)` reads one, matching month and weekday names without regard to case and rejecting impossible dates (`2025_02_30`) and a weekday that contradicts the date.
- `JournalDate` holds year, month and day and computes the weekday with a civil-date calculation. The module adds no dependency.
- `JournalFormats::from_settings(&GardenSettings)` holds the title format (default `MMM do, yyyy`) and file-name format (default `yyyy_MM_dd`). `GardenSettings::read` gains `journal_title_format` and `journal_file_name_format`; an unparseable pattern falls back to the default with a diagnostic, as other malformed settings do.
- `journal_identity(document, formats)` returns the date from the file stem and the title: the leading `title::` property when present, otherwise the formatted date. A file under `journals/` whose stem does not parse returns an error carrying its path.
- `parse_journal_reference(name, formats)` tries the configured title format, then `MMM do, yyyy`, `yyyy-MM-dd` and `yyyy_MM_dd`, as Logseq's `date_time_util.cljs` does.
- `journal_list(garden, formats)` returns every journal's date, title and document, newest first, with errors for unparseable stems.

`PageNameIndex` gains `insert_journal(route, date, title, document)`. `candidates(name)` first tries titles and aliases, then a journal whose date `parse_journal_reference` returns, so every link, embed, favorite and homepage selector resolves dates the same way. `page_title` keeps its current contract for page files.

The publisher renders journal pages, the feed and its batch files, the sidebar item and `g j`. Linked References for journals come from the existing `site/backlinks.rs` index; moving that index into the SDK is part of #115.

**Selecting journals.** `SiteOptions.journals: bool` and `lsg publish --journals` turn journal publishing on. Each journal then passes through the same `public:: false` check as pages; namespace selectors do not apply because journal titles have no namespace. A file that fails `journal_identity` is skipped with a diagnostic and counted in `skipped_journals`. Journal and page titles share one lookup, so a page file whose title equals a journal title fails with the existing ambiguity error before output is written; the encode garden has none. Journal routes use the existing `p-<digest>.html` scheme.

**Journal pages.** A journal page uses the ordinary page template with the journal title as its `h1`, page properties, aliases and Linked References, and no namespace hierarchy section. The complete page index lists journal pages with other pages, sorted by title. Journals join the search index as pages.

**The feed.** The build date is the UTC date of `SOURCE_DATE_EPOCH` when set, otherwise of the system clock. The feed lists published journals dated on or before the build date, newest first; later-dated journals keep their pages. The feed is written when journals are published, at least one is in the feed, and `:feature/enable-journals?` is not `false`.

- Batch 1 (three journals) is `index.html`, or `journals.html` when `--homepage` holds `index.html`. Batch *n* ≥ 2 (seven journals) is `journals-<n>.html`. Every batch file is a complete page with the shell.
- Each journal is `<article class="lsg-journal">` holding `<h1><a class="journal-title" href="<route>">title</a></h1>`, the expanded block tree, and, when the journal has references, `<details class="linked-references" data-lsg-journal-refs="<route>">` with the summary `N Linked References` and a link to `<route>#linked-references`. Collapsed blocks keep their authored state, as on journal pages.
- Below the last article, `<a class="lsg-journals-earlier" href="journals-<n+1>.html">Earlier</a>` appears when a later batch exists.
- `journals.js` (written once to the site root, loaded with `defer` from feed pages only) watches the Earlier link with an `IntersectionObserver` with a 500px root margin. When it intersects, the script fetches the next batch, parses it with `DOMParser`, appends its articles, and moves the Earlier link to the new batch's target. On the first `toggle` of a journal's Linked References, it fetches the journal page and copies its Linked References content in, then initialises filters with the existing `backlinks.js` functions. A failed fetch leaves the Earlier link and the reference link working.

**Page index and sidebar.** When the feed holds `index.html`, the complete page index moves to `pages.html`, the same address `--homepage` uses, and the shell's All pages item follows it. Journals is the sidebar's first item, links to the feed's first batch, carries `aria-current="page"` on every feed batch, and uses Tabler's calendar icon. `g j` opens the feed's first batch. Neither appears when no feed is written.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md) on the encode garden, desktop and phone profiles:

| Measure | Budget |
| --- | --- |
| `journals.js`, uncompressed | 4 KB |
| Data before the title is visible on `index.html` (gzip) | 100 KB (existing budget) |
| Script before the title is visible on `index.html` | 16 KB (existing budget) |
| Peak JavaScript heap on `index.html` | 10 MB (existing budget) |
| Peak JavaScript heap after loading ten batches (73 journals), phone | 20 MB |
| `index.html` size change when 1,000 older journals are added | none |
| Whole-garden build time with `--journals` | recorded in `docs/parity/performance.md`; at most 20% above the build without journals |

Search latency keeps its existing budgets with the 505 journals added to the corpus. `parity:budgets` in CI gains the `journals.js` size check and the 1,000-journal size check on the fixture garden. `parity:perf --record` adds the feed and scroll rows.

## Risks / Trade-offs

- [The build date differs from the reader's date, so the feed's newest journal can be a day off Logseq's] → `SOURCE_DATE_EPOCH` lets a build pin the date; a build from the current commit matches Logseq's build from the same commit.
- [A heavily referenced journal makes its feed article large] → Reference content loads only when opened; the summary carries just the count.
- [Appending batches grows the page's DOM on long scrolls] → The ten-batch heap budget bounds it; batch files keep pages separate for readers who use Earlier.
- [Gitpa replaces `<main>` content] → The feed's markup sits inside `<main>`, and `journals.js` uses event delegation and re-finds the Earlier link on each intersection, so swapped content needs no reinitialisation.
- [Publishing journals exposes notes a garden owner meant to keep private] → Journals publish only with `--journals`, and `public:: false` withholds a journal.

## Resolved Questions

### 1 - How are journals selected for publishing?

Decided by the agent without codekiln: with an explicit `--journals` flag (`SiteOptions.journals`), and then every journal passes the `public:: false` check. Logseq publishes all journals, but `lsg`'s selection starts from nothing and adds what the caller names; publishing journals by default would add 505 pages of daily notes to Gitpa's site and any other existing build. `parity:build` passes the flag so the encode garden matches Logseq.

### 2 - Where does the page list go when the feed takes `index.html`?

Decided by the agent without codekiln: to `pages.html`, the address `--homepage` already uses, so All pages and `g a` keep one target. #117 replaces it with Logseq's All pages table at the same address.

### 3 - Does this change publish journals that only links mention?

Decided by the agent without codekiln: no. They are 28 of the 6,616 reference-only pages `lsg` does not publish, and #115 decides how reference-only pages publish. Until then, a link to such a date renders with the existing unresolved-reference fallback, so no link leads to a missing file.

### 4 - Does the feed show today's NOW/NEXT queries and SCHEDULED AND DEADLINE?

Decided by the agent without codekiln: no. A static build cannot know the reader's date, and both sections need query rendering from #124. Showing them for the build date would show stale tasks as today's.

### 5 - What date decides which journals the feed lists?

Decided by the agent without codekiln: the build date, from `SOURCE_DATE_EPOCH` when set and the system clock otherwise, in UTC. Logseq uses the reader's date, which a static page cannot apply without loading the journal list in the browser; a pinned date also makes builds reproducible.

### 6 - How does the feed load later journals?

Decided by the agent without codekiln: from static batch files of seven journals each, appended when the reader nears the bottom, with an Earlier link that opens the same file without JavaScript. This keeps Logseq's three-then-seven pacing, keeps `index.html` independent of the number of journals, and gives every batch a plain URL.

### 7 - Do Linked References under feed journals render with the journal?

Decided by the agent without codekiln: the count renders with the journal; the content loads from the journal's page when the reader opens the section. The encode garden collapses every Linked References section (`:ref/linked-references-collapsed-threshold 0`), so readers see only the summary until they open it.

### 8 - How does a journals file whose name is not a date publish?

Decided by the agent without codekiln: it is skipped with a diagnostic and counted in `skipped_journals`. Logseq would publish it as an ordinary page, but `lsg` would then have to apply namespace selection to a file outside `pages/`; the encode garden has no such file.

### 9 - Does the journal date code add a date library?

Decided by the agent without codekiln: no. The format tokens and the weekday calculation are a small module in the SDK, which keeps the dependency and licence list unchanged.

### 10 - Does the Journals sidebar item appear with `--homepage`?

Decided by the agent without codekiln: yes, linking to `journals.html`. Logseq sends Journals to `#/all-journals` when a default home is set; `journals.html` is the static equivalent and costs nothing extra.

## Open Questions

None.
