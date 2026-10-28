## Context

The [queries analysis](../../../docs/parity/queries.md) measured Logseq's `gh-pages` build of the encode garden (`8132c95`) and an `lsg` build from `48fcc54`. Because the garden has no query that returns results, it also measured thirteen test queries on a patched copy of that build. Logseq evaluates `{{query}}` and `#+BEGIN_QUERY` in the browser against the whole Datascript database. `lsg` writes `{{query}}` as literal `<code>` (`macro_fallback` in `site/render.rs`) and `#+BEGIN_QUERY` as an `<aside>` that also counts its `[[page]]` text as references.

The SDK already has what an evaluator needs: `ReferenceIndex::path_refs` (a block's page plus the references of the block and its ancestors), aliases, task markers and priorities in `outline.rs`, journal dates in `journals.rs`, and the build date in `site/journals.rs`. Linked References already split large sections into an in-page part and fragments loaded by `site/fragments.js`.

[Issue #106](https://github.com/codekiln/logseq-gardener/issues/106) asks for Logseq's look and behaviour without its cost, and for relationship logic to live in the SDK, shared by the CLI, the language server and the publisher.

## Goals / Non-Goals

**Goals:**

- Every simple query in the garden, and every test query in the analysis that uses the simple language, shows Logseq's result count and results.
- `{{cards}}` decks for [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123) can select cards with the same evaluator.
- A page with a query runs no query in the browser, and its first paint stays within the `[desktop]` and `[phone]` budgets whatever the result size.

**Non-Goals:**

- A Datalog subset over the SDK's index, enough for the garden's two `:default-queries` and `:inputs` such as `:current-page`.
- An `lsg query` command that prints the same results in the terminal.
- Saving the reader's table toggle and column sort between visits.

## Decisions

### One SDK operation for parsing and evaluation

A new SDK module, `query`, provides:

- `parse_simple(text) -> Result<Query, QueryError>` for the simple language, after expanding `<% today %>`, `<% yesterday %>` and `<% tomorrow %>` against the build date. An unbalanced or unknown form is a `QueryError` with a byte offset.
- `parse_advanced(edn) -> Result<AdvancedQuery, QueryError>` for the `#+BEGIN_QUERY` map: `:title`, `:query`, `:collapsed?`, `:group-by-page?`, `:remove-block-children?`, `:table-view?`, `:breadcrumb-show?`, and the presence of `:inputs`, `:rules`, `:view` and `:result-transform`. A `:query` that is a vector with `:find` is `Datalog`; a string or list is the simple language.
- `evaluate(query, context) -> QueryResults`, where the context is the reference index, the query's block id and the build date. Results are `Blocks(Vec<BlockId>)` or `Pages(Vec<PageId>)`, in Logseq's order, after the own-block exclusion and, unless `:remove-block-children? false`, the top-level-only rule.

The forms and their meaning follow `query_dsl.cljs`:

- `[[page]]` and `#tag` match blocks whose path references include the page or one of its aliases.
- `and`, `or`, `not` combine; a query of only `not` forms matches every block except those.
- `"text"` matches block source, case-sensitive. A bare word matches nothing, as in a published Logseq site.
- `(task …)` and `(todo …)` match task markers; `(priority …)` priorities.
- `(between A B)` matches blocks on journals dated from A to B inclusive, where A and B are `today`, `yesterday`, `tomorrow`, a journal `[[page]]`, or ±N `d`, `w`, `m` or `y` from the build date.
- `(property k v)` and `(property k)` match block properties; `(page-property k v)`, `(page-tags …)`, `(all-page-tags)` and `(namespace ns)` return pages; `(page name)` matches blocks on a page.
- `(sort-by k asc|desc)` orders results; `(sample n)` keeps n results chosen by a generator seeded with the query's block id, so two builds of one commit agree.
- A query whose forms all return pages returns pages.

Any other form, such as `(between created-at …)`, is a `QueryError::Unsupported` naming the form. The publisher renders it as `Query failed:`, never as empty or wrong results.

The SDK holds no HTML. [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123) calls `evaluate` with `(and <deck query> [[card]])`.

### "Today" is the build date

The evaluator takes the same build date as the journals feed: the UTC date of `SOURCE_DATE_EPOCH` when set, otherwise of the system clock. Two builds of the same commit with the same `SOURCE_DATE_EPOCH` give the same results. Results with relative dates are as fresh as the last build, which the encode garden runs on every push.

### The box, header and clause row

```html
<div class="custom-query" data-lsg-query="q3">
  <div class="th">
    <div class="title"><i class="ti ti-search"></i> Live query</div>
    <div class="flex"><span class="results-count">18 results</span>
      <button class="lsg-query-view" aria-label="Switch to table view">…</button></div>
  </div>
  <div class="query-builder">( AND [[yazi]] [[card]] )</div>
  <div class="custom-query-results">…</div>
</div>
```

The header uses Logseq's measured styles: 12px, opacity 0.9, margin `4px 2px`, padding `16px 24px`, background and ring from the theme tokens, radius 8px. `Live query for pages` replaces `Live query` for page results. An advanced query's `:title` string renders as 16px text; a hiccup vector such as `[:h3 "…"]` renders as that element with escaped text, limited to the elements Logseq's inline renderer allows. The count shows only above zero. The clause row shows the parsed query as Logseq's builder does, without the `+` button, because no edit can be kept.

`Setting properties` and the refresh button are left out: the dialog's choices cannot be kept, and the refresh button appears only for queries that run slowly in the browser, which none do here.

### Results grouped by page

Block results are grouped by page unless `:group-by-page? false`, `(by-page false)` or `:result-transform` is present. Journal groups come first, newest first, then the other pages in page-name order. Each group is `.custom-query-page-result` with the fold arrow and page title link, then each result's breadcrumb and the block with its children, rendered as embeds are. A `{{query}}` inside a result renders as its literal text, as Logseq does. An ungrouped result is a flat list of blocks with breadcrumbs.

### Tables

The table renders when the block has `query-table:: true`, the map has `:table-view? true`, a simple query ends with the word `table`, or the results are pages. Columns follow `query_table.cljs`: for blocks, `block`, `page`, then every non-built-in property on the results in first-seen order; for pages, `page`, the page properties, `created-at` and `updated-at`. `query-properties` picks and orders columns. Rows sort by `query-sort-by` (default `updated-at`) and `query-sort-desc` (default descending). `created-at` and `updated-at` are the page's file dates the SDK already supplies to All pages, formatted `yyyy-MM-dd HH:mm` in UTC. Table styles are the All pages table styles.

`queries.js` sorts rows when a reader clicks a column title, flipping the direction on a second click. The table and list toggle is drawn when Logseq draws it; on click it fetches the other view's fragment once and swaps it in.

### Large and collapsed results in fragments

The page holds the first part of each result: whole groups, or table rows, up to 30 KB gzip. The rest is written to fragments of at most 100 KB gzip, which `fragments.js` loads when the reader scrolls near the end, as for Linked References. A collapsed query writes its header without a count and all its results to fragments; opening it loads them and shows the count. The browser runs no query in either case.

### Errors

- A simple query that does not parse replaces the box with Logseq's block error: `h5` `Query Error:`, and the query text in a code box. `report issue` is left out, since it links to Logseq's tracker.
- An advanced map that does not read shows the same box headed `Invalid query:` with the block text.
- A query that parses but cannot run keeps the header and shows `.warning` `Query failed:` with the message. This covers unsupported forms, Datalog `:query` values, `:inputs` and `:rules`. A `:view` shows `Custom view failed:` and a `:result-transform` is ignored with the results untransformed, as when Logseq's function fails.

Each error is also a build diagnostic with the page and line.

### Query text is not a reference

`ReferenceIndex` skips `#+BEGIN_QUERY` blocks when it collects a block's references, as Logseq's parser does. `{{query}}` arguments already supply none.

### Default journal queries

`:default-queries :journals` entries whose `:query` is in the simple language render, in a `.lsg-default-queries` list, under the build date's journal on the Journals feed and on that journal's page, when that journal exists and the query has results. They run while collapsed and show their count, as in Logseq. Datalog entries, including both of the encode garden's, are skipped, with one diagnostic each per build.

### Code loads only where needed

`queries.css` and `queries.js` are written at the site root and linked only from pages with a query. Pages without a query are byte-for-byte unchanged.

## Risks / Trade-offs

- Results with relative dates go stale between builds. → They match Logseq on the build date, and the garden builds on every push. A reader-clock design would need the journal data in the browser.
- Datalog queries show `Query failed:` instead of results. → The garden's pages contain none; only its two default queries are Datalog, and Logseq does not show those to most visitors either. A Datalog subset is listed as a non-goal for a later change.
- `sample` differs from Logseq's random choice on every visit. → A seeded choice keeps builds reproducible; the sample is still a sample.
- Groups of non-journal pages are in name order where Logseq uses database order. → Logseq's order cannot be reproduced outside its database; name order is stable and readable.

## Performance budgets

New `[queries]` table in `scripts/parity/budgets.toml`:

- `script_kb = 2`: `queries.js`, uncompressed: column sort and the view toggle.
- `styles_gzip_kb = 1`: `queries.css`.
- `in_page_gzip_kb = 30`: each query's in-page part.
- `fragment_gzip_kb = 100`: each query fragment.
- `open_ms_desktop = 300` and `open_ms_phone = 1500`: opening the collapsed tmux query to its first group.
- `open_heap_mb = 20`: phone JS heap after the tmux query has loaded all its fragments.
- `build_slowdown_pct = 10`: a whole-garden build against the `--baseline-lsg` binary.

A fixture page with the 603-block tmux query, open and collapsed, stays within the `[desktop]` and `[phone]` first-paint budgets (100 KB data and 16 KB script before the title) and does not grow when the fixture garden gains 2,000 pages. Search latency is unaffected: query results join no search index.

## Resolved Questions

### 1 - Which forms of the simple query language does the first change support?

Decided by the agent without codekiln: every form in the analysis's table except full-text bare words (which a published Logseq site does not run either) and `(between created-at …)`, whose dates in a published Logseq build are only file-parse times. The garden's decks need only page references, `and`, `or` and `not`, but each other form is a small match over data the SDK already holds, and supporting them makes the analysis's test queries checkable against Logseq.

### 2 - How are Datalog advanced queries handled?

Decided by the agent without codekiln: they render `Query failed:` with a message that the build evaluates simple queries only, and report a diagnostic. A Datalog engine is a large piece of work for no query on the garden's pages; recording it as a non-goal keeps it visible for a later change.

### 3 - What do `:view` and `:result-transform` do?

Decided by the agent without codekiln: a `:view` shows Logseq's own `Custom view failed:` warning, and a `:result-transform` is ignored with the untransformed results shown, which is how Logseq behaves when those functions fail. Running ClojureScript at build time would need an interpreter; Rust equivalents of the garden's `:pprint` and `:sort-by-priority` would serve only the Datalog default queries, which this change skips.

### 4 - What does "today" mean?

Decided by the agent without codekiln: the build date the journals feed already uses, from `SOURCE_DATE_EPOCH` when set. It keeps builds reproducible and matches Logseq for readers on the build date.

### 5 - Where do default journal queries render?

Decided by the agent without codekiln: under the build date's journal, on the feed and on that journal's page, only when that journal exists and the query has results. That is what a Logseq reader on the build date sees. The garden's two default queries are Datalog, so they are skipped with a diagnostic.

### 6 - Which controls does the published box keep?

Decided by the agent without codekiln: the table and list toggle (loading the other view from a fragment on click) and column sorting, which work on results already built. The clause row is shown read-only, without `+`. `Setting properties` and the refresh button are left out: the dialog's choices cannot be kept, and the refresh button only appears for slow browser queries, which never run here.

### 7 - How are large results kept out of first paint?

Decided by the agent without codekiln: the Linked References pattern, an in-page part up to 30 KB gzip and fragments of up to 100 KB gzip loaded on scroll. Collapsed queries put all their results in fragments loaded on open. Logseq draws all 603 rows of a table at once; `lsg` keeps first paint independent of result size.

### 8 - In what order are non-journal groups listed?

Decided by the agent without codekiln: by page name, after journals newest first. Logseq's order for non-journal pages is the order of its database, which no build can reproduce.

### 9 - How does `sample` choose results?

Decided by the agent without codekiln: with a generator seeded by the query's block id, so the same commit always publishes the same sample.

### 10 - What lives in the SDK and what in the publisher?

Decided by the agent without codekiln: the SDK parses both query forms, evaluates them over the reference index with the build date, orders results, applies the own-block and top-level rules, and skips query text when collecting references. The publisher renders the box, header, clause row, groups, tables, errors, fragments, styles and script. The flashcard decks, the CLI and the language server use the same `evaluate`, as the shared-engine comment on #106 asks.
