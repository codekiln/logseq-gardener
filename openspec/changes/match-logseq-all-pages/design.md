## Context

The [All pages analysis](../../../docs/parity/all-pages.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `0ce74d6`, `gh-pages` `8132c95`) with an `lsg` build from `1e72969`. Logseq's screen is a route in its single-page app: it downloads the whole graph (22.6 MB before the title), counts every page's references in the browser and shows 40 rows. `lsg` writes `pages.html` from `site/mod.rs` (the index list, lines 477–640) as one `<ul>` of 10,776 links, 1,266,786 bytes before its title.

`ReferenceIndex` in `crates/logseq-gardener-sdk/src/references.rs` already finds each page's linked references for the CLI, the language server and the publisher. Its `linked` answer leaves out blocks on the page itself, which matches Logseq's Linked References count but not its Backlinks column.

## Goals / Non-Goals

**Goals:**

- Readers see Logseq's All pages table: total, search, journals toggle, sortable columns, 40-row pages and Logseq's look.
- The first paint of All pages is independent of garden size, and the full list loads only when a reader sorts, searches or turns on journals.
- The CLI and the language server can list pages by backlinks or dates with the same answers as the publisher.

**Non-Goals:**

- Listing pages that only references create. `lsg` does not publish them yet ([Issue #111](https://github.com/codekiln/logseq-gardener/issues/111), [Issue #115](https://github.com/codekiln/logseq-gardener/issues/115)); the totals differ by the 7,540 pages `parity:compare` reports as reference-only.
- Shift-click to open a page in the right sidebar ([Issue #119](https://github.com/codekiln/logseq-gardener/issues/119)), the `All files` list and whiteboards.
- Measuring Logseq's dark values; the table uses the existing theme tokens.

## Decisions

**Where the work lives.** The SDK answers the list; the publisher renders it.

| Operation | Answer |
| --- | --- |
| `ReferenceIndex::backlink_count(page)` | The number of blocks whose references include the page, counting namespace children as `linked` does and also the page's own blocks. |
| `GitDates::read(garden_root)` | For each tracked file under `pages/` and `journals/`, the commit times of the first and last commits that touched it, from one `git log --format=%ct --name-only` pass that follows no renames. `None` when the garden is not a git repository or `git` is missing; a diagnostic says why. |
| `page_list(titles, references, dates) -> Vec<PageListEntry>` | One entry per published page, journal and generated namespace page: title, route, kind (`Page`, `Journal`, `Namespace`), `icon::` value, backlink count, lower-cased sort key, created and updated dates. |
| `PageListEntry::cmp_by(column, direction)` | Logseq's order for a column: names by sort key, numbers and dates with entries without a value last, ties broken by sort key. |

`git log` over the encode garden's 3,071 commits takes about 3.5 s on the measuring Mac. The build reads it once, on its own thread, while pages render.

**Default view.** Journals hidden, sorted by `Updated At` newest first, as Logseq. Without git dates, sorted by `Page name` ascending.

**Markup.**

```html
<main class="all-pages" data-list-src="_pages/list.json">
  <h1 class="title">All pages</h1>
  <p class="text-sm">Total 10241 pages</p>
  <div class="actions">
    <input type="search" placeholder="Search page names" aria-label="Search page names">
    <button class="journal-toggle" aria-pressed="false" title="Show journals">…</button>
    <nav class="paging" aria-label="Pages">1/257 <a href="pages-2.html" rel="next">Next ▸</a> <a href="pages-257.html">»</a></nav>
  </div>
  <div class="table-wrapper"><table class="table-auto all-pages-table">
    <thead><tr><th class="name"><a href="#" data-sort="name">Page name</a></th><th class="backlinks">…</th>…</tr></thead>
    <tbody><tr><td class="name"><a class="page-ref" href="p-….html">…</a></td><td class="backlinks">598</td><td class="created-at">2025-03-01 09:12</td><td class="updated-at">…</td></tr>…</tbody>
  </table></div>
  <nav class="paging" aria-label="Pages">…</nav>
</main>
```

Dates are written as `<time datetime="…">` in UTC and shown as `yyyy-MM-dd HH:mm`; the script rewrites them in the reader's time zone, as Logseq shows them. Without the script the UTC value stays.

**Static paging.** `pages-<n>.html` holds rows `40(n-1)+1` to `40n` of the default view, inside the normal page shell, so `Next`, `Prev`, `«` and `»` are ordinary links and the content swap already handles them. The encode garden needs about 257 such pages of about 6 KB gzip each.

**List data file.** `_pages/list.json` holds every entry, journals included, as compact arrays: `[title, route id, kind, backlinks, created, updated]`, routes as the hash part of `p-<hash>.html`. The script fetches it once, on the first sort, search or journals toggle, and keeps it for later views. Paging inside a script-made view re-renders from memory and scrolls to the top.

**Search.** On Enter, the script scores each name with Logseq's `score` from [`search.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/search.cljs#L30-L90): lower-case, drop `[ \/_]()`, reward consecutive matches, add the length-similarity term and a bonus for an exact substring, keep scores above 0, sort descending, take 20. Escape or the `×` link clears it. The ported function has its own tests with the analysis's `mise` example.

**Styles.** A new `site/all-pages.css` holds the measured light values from the analysis: heading 36px weight 300 with margin `8px 0 24px`; total 14px at opacity 0.7; header cells 14px weight 400 `rgb(104,112,118)` with `10px 8px` padding and a 2px bottom border; body cells 16px with `6px 8px` padding; stripes `rgb(251,252,253)` and `rgb(236,238,240)`, reusing the table tokens from `published-code-and-tables`. The toolbar sticks to the top. Below 640px wide, only the `Page name` column shows.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md) on the whole encode garden:

| Measure | Budget |
| --- | --- |
| All pages first paint, desktop and phone | within the existing `data_before_title_kb` of 100 KB and `script_before_title_kb` of 16 KB |
| Each static paging page, gzip | 10 KB |
| All pages growth when the garden gains 2,000 pages | 1 KB on the first page |
| `all-pages.js`, minified gzip | 3 KB, loaded only on the All pages screen |
| `_pages/list.json`, gzip | 300 KB for the encode garden, fetched only on the first sort, search or toggle |
| Peak phone heap after loading the list and sorting by Backlinks | 20 MB |
| Search on Enter, p95 | 50 ms desktop, 150 ms phone |
| Whole-garden build time | at most 10% above the build without this change |

`scripts/parity/budgets.toml` gains an `[all_pages]` table: `parity:budgets` checks the sizes and the grown-garden fixture in CI, and `parity:perf` marks the heap, search and build values in `docs/parity/performance.md`.

## Risks / Trade-offs

- [Git dates differ from what Logseq shows] → Logseq's values are parse times of one build and change on every build; git dates keep the same columns and order and add information. Decision 1 records the choice.
- [The list data file grows with the garden] → It loads only on interaction, never before the title, and has its own budget; paging in the default view never needs it.
- [A shallow clone, as CI checkouts often are, has short history] → Dates then reflect the history present; the build reports a diagnostic when the repository is shallow.
- [Static paging pages add about 257 files] → Each is small and shares the shell; the page count in the build report stays the count of source pages.
- [This change and `match-logseq-hierarchy` both modify "Distinguish namespace destinations"] → The delta here is written on top of the hierarchy change's version; sync this change after that one is archived.

## Resolved Questions

### 1 - What do Created At and Updated At show?

Decided by the agent without codekiln: the first and last git commit times of the page's file, and no date columns when the garden has no git history. Logseq's published values are the moments its build parsed each file, which carry no information and change on every build; git dates keep Logseq's columns and newest-first default and are stable for one commit.

### 2 - How is the default order chosen?

Decided by the agent without codekiln: `Updated At` newest first, as Logseq; `Page name` ascending when there are no dates. Generated namespace pages have no file and so no dates; they sort after dated pages, by name.

### 3 - Are Logseq's editing controls drawn?

Decided by the agent without codekiln: no. Row checkboxes, `Delete`, `Remove orphaned pages`, the whiteboard toggle and the `⋯` menu change only the reader's in-memory copy in Logseq's published site; drawing them in `lsg` would promise actions that do nothing.

### 4 - Do generated namespace pages appear, and with the Namespace badge?

Decided by the agent without codekiln: they appear, without the visible badge, keeping their link style and accessible Namespace name. Logseq lists namespace parent pages like any page, and the hierarchy change already removes the badge from hierarchy rows.

### 5 - Does the full list load up front?

Decided by the agent without codekiln: no. The page carries the first 40 rows, static pages carry the rest of the default view, and the list data file loads on the first sort, search or journals toggle. Issue #106 asks that first paint not depend on garden size.

### 6 - How does the phone layout decide it is a phone?

Decided by the agent without codekiln: by viewport width below 640px, not by user agent as Logseq does. Width is what makes the extra columns unreadable, and a CSS media query needs no script.

### 7 - Which time zone do dates use?

Decided by the agent without codekiln: the reader's, as in Logseq, set by the script; UTC without scripts, marked with `datetime` so the value is unambiguous.

### 8 - Does the page icon column appear?

Decided by the agent without codekiln: yes, before the name, from the `icon::` property, as the live Logseq build shows it. Two garden pages have one.
