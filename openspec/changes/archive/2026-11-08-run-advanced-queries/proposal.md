# Proposal

## Why

Logseq's published site runs an advanced query and shows its result. `lsg` shows an error box in its place, so a page such as gitpa's homepage lists its episodes only after gitpa runs `nbb-logseq` and writes the rows into `index.html` after the build ([Issue #485 — Run advanced queries with :result-transform and :view at build time](https://github.com/codekiln/logseq-gardener/issues/485), part of [#484 — Render the GitP/A namespace the same on both sites](https://github.com/codekiln/logseq-gardener/issues/484); the [analysis](../../../docs/parity/advanced-queries.md) records Logseq's behaviour and the gaps).

Logseq runs such a query in the reader's browser after it loads the whole graph. `lsg` can run the same query while it builds the site and write the list into the page, so the reader's browser loads no script and no graph ([Issue #106 — Match Logseq's look and behaviour, beat its performance](https://github.com/codekiln/logseq-gardener/issues/106)).

## What Changes

- The SDK reads a small, closed subset of Datalog (`:find (pull ?p [*])` with data patterns and function clauses over pages and their properties) and of ClojureScript (`fn`, `sort-by`, `compare`, `reverse`, `get`, `get-in`, `into`, `map`, `str`, `clojure.string/replace`, keyword access and a few more) and runs both over the published pages.
- Page property values get Logseq's types: `true` and `false` are booleans, all-digit values are numbers, quoted values are strings, and values with page references are sets of page names.
- A `:result-transform` function reorders or filters the Datalog rows. A `:view` function returns Hiccup, which the publisher writes as escaped HTML inside the existing query box. A `#/page/<name>` link in that Hiccup becomes the page's published URL.
- A form outside the subset keeps Logseq's `Query failed:` or `Custom view failed:` box. The box names the form, and the build lists one diagnostic with the line.
- Every run has a step limit and an output size limit, so a query cannot loop or fill a page.
- A new `[advanced_queries]` budget table, a `parity:perf` section and committed benchmark rows measure the feature on a copy of the whole encode garden against Logseq's build.

## Capabilities

### New Capabilities

- `garden-advanced-queries`: the SDK operations that read and run the Datalog and ClojureScript subsets over pages, with typed property values, Hiccup output, step and size limits, and no garden-specific names.

### Modified Capabilities

- `garden-queries`: the map parser reads `:view` and `:result-transform` as functions in the supported subset; a Datalog `:query` inside the subset is evaluated.
- `published-queries`: the error requirement names the unsupported form and the new failure boxes; the box shows a view's result, resolves its page links, shows a transform's order, and stays within the new budgets.

## Impact

- SDK: new `query_datalog`, `query_script` and `query_hiccup` modules beside `query.rs`, which hands them the parsed forms; a typed-value reader for page properties. The CLI and the language server can call the same operations.
- Publisher: `site/queries.rs` renders the view's Hiccup and the new error boxes inside the existing box; no new script and no new stylesheet.
- Parity harness: an `[advanced_queries]` table in `scripts/parity/budgets.toml`, a `parity:perf` section recorded in the Performance page, rows in `docs/parity/publication-speed.json`, and fixture pages under `tests/fixtures/parity-garden`.
- Gitpa can later remove its `nbb-logseq` step ([gitpa #160](https://github.com/codekiln/gitpa/issues/160)); that work stays in gitpa.

## Budgets and placement

The design holds the full table. In short, on a page with the gitpa-form query:

- First-paint bytes: the page stays within the `[desktop]` and `[phone]` budgets (100 KB of data before the title) and grows by at most 3 KB gzip for 18 rows.
- Script bytes: zero new. The page loads no script and no stylesheet that a page without the query lacks.
- Per-keystroke search latency: unchanged, at most 50 ms on desktop and 100 ms on the phone profile, with the whole encode garden as the corpus.
- Peak phone memory: at most 10 MB of JavaScript heap, as for every page.
- Build time: a whole-garden build with the query pages stays within 10% of the baseline binary; the worst-case query on all 16,693 pages evaluates in at most 500 ms.

The SDK holds reading, typing, evaluation, limits and the Hiccup tree. The publisher holds only the HTML, the boxes and the diagnostics.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Declarative over Imperative](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Declarative%20over%20Imperative.md)
- [My/Principle/Make Illegal States Unrepresentable](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Make%20Illegal%20States%20Unrepresentable.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
