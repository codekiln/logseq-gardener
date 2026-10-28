## Why

Logseq renders `{{query …}}` and `#+BEGIN_QUERY` blocks as result boxes: a header with a title, a result count and view buttons, then the matching blocks grouped by page, or a table. `lsg` prints the query's source as inline code or a plain `<aside>` ([Queries analysis](../../../docs/parity/queries.md), [Issue #124 — Render Logseq queries at build time](https://github.com/codekiln/logseq-gardener/issues/124)).

The encode garden has only one `{{query}}`, and it is blank, but its 29 `{{cards}}` decks select cards with the same simple query language. [Issue #123 — Publish Logseq's flashcards](https://github.com/codekiln/logseq-gardener/issues/123) needs one evaluator for both.

Logseq runs every query in the reader's browser after loading the whole graph: 19.8 s to the title of `Logseq/Queries` on desktop and 185 s on a phone. `lsg` can evaluate each query during the build and write the results into the page, so the browser runs no query and loads no graph.

## What Changes

- The SDK gains a `query` module. It parses the simple query language and the `#+BEGIN_QUERY` map, with errors that carry source locations, and evaluates simple queries over the reference index: page references through path references and aliases, `and`, `or`, `not`, `"text"`, `task`/`todo`, `priority`, `between` with relative dates, `property`, `page-property`, `page-tags`, `all-page-tags`, `namespace`, `page`, `sort-by` and `sample`. It returns ordered blocks or pages, leaving out the query's own block and results whose parent is also a result.
- "Today" is the build date that the journals feed already uses, from `SOURCE_DATE_EPOCH` when set.
- `{{query …}}` renders Logseq's `.custom-query` box: the `Live query` or `Live query for pages` header with `N results`, the parsed query as a read-only clause row, then the results grouped by page (journals newest first), each block with its breadcrumb and children, or `No matched result`.
- `#+BEGIN_QUERY` renders the same box with its `:title` as text or markup. A `:query` in the simple language is evaluated. A Datalog `:query` renders `Query failed:` with a message saying the build evaluates simple queries only, and reports a diagnostic.
- A table renders for `query-table:: true`, `:table-view? true`, a simple query ending in `table`, and every page result, with Logseq's columns, `query-properties` order and `query-sort-by` order. Column titles sort the rows in the browser. The table and list toggle loads the other view from a fragment on click.
- `:collapsed? true` and `collapsed:: true` queries start closed with no count; their results load from a fragment when opened. Results larger than the in-page part load the rest from fragments as the reader scrolls, as Linked References do.
- Three error forms match Logseq: `Query Error:` for a simple query that does not parse, `Invalid query:` for a map that does not read, and `Query failed:` for a query that cannot run. Each is also a build diagnostic.
- `[[page]]` text inside `#+BEGIN_QUERY` stops counting as a reference, as in Logseq.
- `:default-queries :journals` with a simple-language `:query` render under the build date's journal on the Journals feed and on that journal's page, when that journal exists and the query has results. Datalog default queries are skipped with one diagnostic each.
- A new `[queries]` budget table caps the query script, styles, the in-page part, each fragment, and the time to open a collapsed query.

## Capabilities

### New Capabilities

- `garden-queries`: the SDK operation that parses and evaluates Logseq's simple query language and the advanced-query map, with the build date as "today", shared by the publisher, the CLI, the language server and the flashcard decks.
- `published-queries`: the query box, header, clause row, grouped list, table, collapsed and empty states, the three error forms, default journal queries, fragments, the small script and budgets.

### Modified Capabilities

- `published-inline-markup`: `query` leaves the list of macros that keep their literal rendering.
- `reference-index`: text inside a `#+BEGIN_QUERY` block supplies no references.

## Impact

- SDK: new `query` module (parser, evaluator, result ordering) over `ReferenceIndex`, outline task markers and priorities, block and page properties, and journal dates; `references.rs` skips query blocks when it collects references. The CLI and language server can call the same operation.
- Publisher: the `query` arm in `macro_fallback` and the `Block::Custom` `query` arm in `site/render.rs` call new `site/queries.rs`; new `site/queries.css` and `site/queries.js` load only on pages with a query; fragments reuse `site/fragments.js`; the journals feed renders default queries.
- Parity harness: a `[queries]` table in `scripts/parity/budgets.toml`, `parity:budgets` and `parity:perf` rows for a fixture page with the 603-block tmux query, and a `Queries` page in the parity fixture garden holding the analysis's test queries.
- [Issue #123](https://github.com/codekiln/logseq-gardener/issues/123) builds its deck selection on `garden-queries`; its Apply step starts after this change's Apply merges.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
