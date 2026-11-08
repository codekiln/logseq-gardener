# Design

## Context

The [analysis](../../../docs/parity/advanced-queries.md) measured gitpa's homepage query (`gitpa` `410f4d9`) and Logseq's source (`og` `6e7afa8eb`). Logseq runs the query in the browser: Datascript runs the `:query`, SCI (a ClojureScript interpreter) runs the `:result-transform` and the `:view`, and React draws the Hiccup. `lsg` already parses the map (`parse_advanced` in `query.rs`, with an EDN reader), renders the query box, and shows `Query failed:` or `Custom view failed:` for every Datalog query and every `:view` (`advanced_box` in `site/queries.rs`).

The SDK already holds each published page's name, original name and properties (`page_properties.rs`, `property_names.rs`, `page_lookup.rs`), and the publisher already links `#/page/<name>` URLs to published pages through `--redirect-old-urls`.

[Issue #106](https://github.com/codekiln/logseq-gardener/issues/106) asks for Logseq's look and behaviour without its load-everything cost, and for graph answers to live in the SDK so the command line, the language server and the publisher share them. See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**

- Gitpa's `Ghost in the Patch` page, built with plain `lsg publish`, lists the public episodes in the same order as gitpa's `mise run site:query:check`.
- A fixture advanced query produces the same rows as Logseq 0.10.15 in `mise run parity:logseq-export`.
- The page needs no script, no data file and no database in the browser.
- A garden without an advanced query builds the same output at the same speed.

**Non-Goals:**

- Datalog over blocks (`:block/marker`, `:block/page`, `:block/journal-day`), `:in`, `:inputs`, `:rules` and aggregates. These run the encode garden's two default queries, `NOW` and `NEXT`.
- `:result-transform` and `:view` for queries whose rows are blocks, including the simple language.
- Named functions in `:query/views` and `:query/result-transforms` of `config.edn`.
- The rest of SCI: `call-api`, macros, `loop`, named functions, regular expressions and interop.
- Removing gitpa's `nbb-logseq` step and choosing gitpa's public episodes ([gitpa #160](https://github.com/codekiln/gitpa/issues/160)).

## Decisions

### Parse the whole query before running any of it

`parse_advanced` turns each of `:query`, `:result-transform` and `:view` into a typed value when it reads the map: a `DatalogQuery`, a `Script` (a function in the supported subset) and a `Script` again. A form outside the subset is a parse error that names the form and its byte offset, so a query that parses can fail at run time only through a thrown error or a limit. The result is one of three states per query: parsed and runnable, `Unsupported { form, offset }`, or `Invalid`. No runtime check can then reach a form the evaluator does not know.

### Datalog subset

The SDK runs one shape:

```clojure
[:find (pull ?p [*])
 :where <clause> …]
```

`<clause>` is one of:

- a data pattern over a page variable: `[?p :block/name]`, `[?p :block/name ?n]`, `[?p :block/original-name ?n]`, `[?p :block/properties ?props]`, each with an optional literal in the third place;
- a binding function clause `[(get ?m :key) ?out]`;
- a predicate clause `[(contains? ?s "text")]`, `[(= ?a ?b)]` or `[(not= ?a ?b)]`, where each operand is a bound variable or a literal.

Variables bind in order, as in Datascript: a clause that reads an unbound variable is a parse error. A binding clause whose function returns nothing drops the row, and so does a predicate that returns false. The result rows are the published pages that satisfy every clause. Any other attribute, clause form, `:find` shape, `:in`, `:with`, `or`, `not` or rule is `Unsupported`.

A row is a record with three readable attributes: `:block/name` (lower case), `:block/original-name` and `:block/properties`. Reading any other attribute in a function is an `Unsupported` error at parse time when the key is a literal keyword, and a thrown error at run time otherwise. Logseq's `(pull ?p [*])` returns more attributes, such as `:db/id` and `:block/file`; a published site has no use for them.

The evaluator scans every published page once per clause and charges one step per page and clause.

### Typed page property values

The SDK reads a property value with Logseq's `parse-property` rules:

- a value with page references (`[[A]], [[B]]` or `#tag`) is a set of the names as written;
- `true` and `false` are booleans;
- an all-digit value is a number;
- a value in double quotes is the string inside the quotes;
- anything else is a string.

A property key is the keyword Logseq makes from it: lower case, with `_` read as `-`. `:block/properties` holds every property of the page's leading properties block, keys and values as above. The same reader serves the Datalog subset and the functions, so `public:: true` matches `(= ?public true)` while `public:: "true"` and `public:: false` do not.

### Run functions in a closed language

A tree-walking interpreter in the SDK evaluates the function subset. The language has these parts:

| Group | Supported |
| --- | --- |
| Special forms | `fn` (anonymous, positional parameters), `if`, `let`, `and`, `or`, `not`, `when` |
| Values | strings, keywords, numbers, `true`, `false`, `nil`, vectors, maps, sets, the page rows |
| Access | `get`, `get-in`, keywords and maps called as functions, `contains?`, `count`, `first`, `last` |
| Order | `sort-by` (with or without a comparator), `sort`, `compare`, `reverse`, `take`, `drop` |
| Build | `into`, `vec`, `vector`, `concat`, `map`, `filter`, `remove`, `str` |
| Compare | `=`, `not=`, `<`, `>`, `<=`, `>=` |
| Text | `clojure.string/replace` (string pattern and string replacement), `clojure.string/join`, `clojure.string/lower-case`, `clojure.string/upper-case` |

Every symbol outside the table is `Unsupported` at parse time. The language has no `def`, no `loop`, no recursion, no named function and no interop, so a function cannot read a file, run a command, read the clock or loop without end: each `map`, `filter` and `sort-by` walks a finite collection. The step limit (below) covers a function that builds a very large result. This keeps the garden author's code inside the same boundary as the garden's other build inputs.

Semantics follow ClojureScript, because gitpa's output must match Logseq's:

- `compare` orders `nil` first, then strings by UTF-16 code unit (as JavaScript compares them), and numbers by value. Comparing a string with a number throws, as in ClojureScript.
- `sort-by` is stable, and calls its key function for each comparison, as `clojure.core` does.
- A keyword used as a function reads the same key from a map and returns `nil` when the key is absent.
- `str` joins the printed values of its arguments; `nil` prints as the empty string.

### Result transform and view

1. The Datalog rows are ordered by lower-case page name, as Datascript's order cannot be reproduced; a stable `sort-by` keeps that order for ties.
2. A `:result-transform` function receives the rows as a vector and returns a sequence of rows. A transform without a `:view` must return a subset or reordering of the original rows; any other value is `Query failed:`.
3. A `:view` function runs only when the result is not empty, as in Logseq. It receives the transformed rows and returns Hiccup.
4. A function that throws at run time behaves as in Logseq: a transform that throws leaves the rows untransformed and adds one diagnostic; a view that throws gives `Custom view failed:` with the error text. A function that reaches a limit is `Query failed:` for a query or transform and `Custom view failed:` for a view, because the result would otherwise differ from Logseq's silently.

### Hiccup output

The SDK returns a `Hiccup` tree (`Element { tag, attrs, children }` and `Text`) and holds no HTML. The publisher writes it:

- A vector whose first element is a string or a keyword is an element; nested sequences (the output of `map`) flatten; `nil` and `false` are skipped; numbers print as text.
- Tag names come from a list of elements that carry text and links: `div`, `span`, `p`, `ul`, `ol`, `li`, `a`, `b`, `strong`, `i`, `em`, `code`, `pre`, `br`, `hr`, `h1` to `h6`, `table`, `thead`, `tbody`, `tr`, `th`, `td`, `blockquote`, `small`, `sub`, `sup`. Any other tag is `Custom view failed:` naming it. A tag with `.` or `#` shorthand (`:div.card`) is `Unsupported`.
- Attributes are `href`, `class`, `id`, `title`, `rel`, `target`, `colspan`, `rowspan`, `aria-label` and `data-*`. Any other attribute, including `style` and every `on*` handler, fails the view naming it. An `href` is a `#` fragment, a relative path, or an `http`, `https` or `mailto` URL; any other scheme fails the view.
- Text and attribute values are HTML-escaped.

### Links to pages

An `href` of the form `#/page/<name>` names a page the way Logseq's router does. The publisher percent-decodes `<name>`, finds the page with the lookup the SDK already offers, and writes the page's published URL. When no published page matches, the `href` stays as written, so the existing redirect behaviour applies. A view's links create no reference, no backlink and no search record, as in Logseq.

### The box around the result

The box keeps its existing header, title and error boxes. For an advanced query with a view:

- the count is the number of rows after the transform, formatted as `18 results`, and an empty result shows no count and `No matched result`, as Logseq 0.10.15 does;
- the table and list toggle is hidden, because a view takes the place of both;
- a collapsed query (`:collapsed? true`) puts the view's HTML in a fragment, as other collapsed queries do.

A Datalog query without a view shows its rows as the page table the box already draws for page results.

### Failure boxes

| Cause | Box | Message names |
| --- | --- | --- |
| Unsupported form in `:query` or `:result-transform` | `Query failed:` | the form and where it is |
| Unsupported form in `:view`, a thrown view, a rejected tag, attribute or URL, output over the size limit | `Custom view failed:` | the form, error or limit |
| `:inputs` or `:rules` | `Query failed:` | the key (unchanged) |
| Map that does not read | `Invalid query:` (unchanged) | the block text |

Each is also one build diagnostic with the page and line. A simple-language `:query` that carries a `:view` or `:result-transform` keeps its existing behaviour, with the message reworded to say that block results do not support functions at build time.

### Limits

- **Steps.** Each evaluated form and each page-clause pair costs one step. A query stops at 20,000,000 steps. Sorting 16,693 rows with gitpa's comparator is estimated at 5 million steps, so the limit leaves four times that room; the Apply step measures the real count.
- **Depth.** Nested calls stop at 64 levels.
- **Output.** A view's Hiccup stops at 128 KB of HTML (about 1,200 gitpa links), which holds the page's gzip growth near the 3 KB per 18 rows the budget below allows.

A step count, not wall time, ends a query, so two builds of one commit give the same page. No supported form reads the date, the environment or a random source.

### Code lives in the SDK except for HTML

The SDK owns reading, typed values, the Datalog subset, the function interpreter, limits and the Hiccup tree, as modules beside `query.rs`: `query_datalog`, `query_script` and `query_hiccup`. `run_advanced(query, pages) -> Result<AdvancedResult, QueryFailure>` takes the published pages and returns rows, the view's tree or the failure, with no HTML, so the CLI and the language server can print the same rows. The publisher turns a tree into HTML and a failure into a box and a diagnostic.

### No garden-specific names

Nothing in the SDK or the publisher names gitpa, `Logseq/Entity/Podcast/Episode`, `podcast-published-at` or any other garden's page or property. The query decides which pages and keys matter. Fixtures use their own names, and a test renames gitpa's property and type in a copy of the query and checks that the output follows.

## Risks / Trade-offs

- A garden author's query uses a function outside the subset → the box names the function, so the author can pick another or ask for it; each addition is one entry in the table above and one test.
- Rows are page records with three attributes, so a query that reads `:db/id` fails where Logseq succeeds → the failure names the attribute, and no published page can show a database id.
- Tie order between rows with equal sort keys can differ from Logseq's database order → the fixture sorts on keys that differ, and the row order for ties is page name.
- A purpose-built interpreter can drift from SCI → every supported function has a fixture row that `parity:logseq-export` compares with Logseq 0.10.15.
- A view output larger than 128 KB fails → the message tells the author to narrow the query or use `take`, and the limit protects first paint on phones.

## Performance budgets

The browser pays nothing: the result is static HTML, with no script, no data file and no interpreter. The build pays only for pages that hold an advanced query. A new table in `scripts/parity/budgets.toml`:

```toml
[advanced_queries]
page_growth_gzip_kb = 3      # gitpa-form query, 18 rows, against the same page without the query
script_added_kb = 0          # no new script, none linked from the page
styles_added_gzip_kb = 0     # no new stylesheet
view_max_kb = 128            # largest view output, HTML before gzip
step_limit_millions = 20     # evaluation steps before the query stops
eval_ms = 500                # the all-pages query on the whole encode garden
limit_hit_s = 2              # time to stop a query at the step limit
build_slowdown_pct = 10      # whole-garden build against the --baseline-lsg binary
```

The existing budgets apply to the same page without change: 100 KB of data and 16 KB of script before the title, 300 ms (desktop) and 1,500 ms (phone) to the title, 10 MB of JavaScript heap, and a search p95 of 50 ms (desktop) and 100 ms (phone) per keystroke.

### Measurement

On a copy of the whole encode garden (16,693 pages) with two added pages:

- `Parity/AdvancedQueries` holds the gitpa-form query over a property the encode garden has (`logseq-entity:: [[Logseq/Entity/Proxy/Page]]`, six pages), so the comparison needs no gitpa data.
- `Parity/AdvancedQueries/All Pages` holds the worst case: `[?p :block/name]` over every page, `sort-by` on a property, and a view of `(take 1000 rows)`.

`parity:perf --advanced-queries` measures both pages in `lsg` and in a Logseq build of the same copy: time to the title, bytes transferred, script bytes, peak JavaScript heap on the phone profile, and per-keystroke search latency with the page in the corpus. It records the rows in the Performance page (`docs/pages/lsg___Ref___Logseq Compatibility___Performance.md`). `parity:publish-bench --record` adds a row to `docs/parity/publication-speed.json` for the `gitpa-sized` corpus with the gitpa-form query on its homepage, and CI runs its `--check`, so a slower build fails.

CI also checks `parity:budgets` on the fixture garden, where the gitpa-form page must stay within the first-paint budgets and its growth budget, and must link no script.

## Resolved Questions

### 1 - How does the build run `:result-transform` and `:view`?

Decided by the agent without codekiln: with a purpose-built interpreter for a closed subset of ClojureScript, written in the SDK. The alternatives were running `nbb`, as gitpa does now, which needs Node on the build machine and can read files and run commands, and embedding a general interpreter, which brings the I/O and unbounded loops the boundary has to exclude. A closed subset keeps the build offline and deterministic, and the language has no construct that loops forever. The cost is the work of each function and the drift risk that Risks / Trade-offs covers.

### 2 - Which functions does the subset include?

Decided by the agent without codekiln: the table under Run functions in a closed language. It starts with everything gitpa's query uses (`fn`, `sort-by`, `compare`, `get`, `get-in`, `into`, `map`, `str`, `clojure.string/replace`, keyword access) and adds the functions that people write next to them in the same kind of query (`reverse`, `filter`, `remove`, `count`, `take`, `drop`, `first`, `last`, `if`, `let`, comparisons, `clojure.string/join`, `lower-case`, `upper-case`). Anything else is an unsupported form that the box names. Each addition costs one table row and one test, so the table can grow when a garden asks.

### 3 - What happens to a form outside the subset?

Decided by the agent without codekiln: the existing Logseq box keeps its place, `Query failed:` for the query or the transform and `Custom view failed:` for the view, with the message naming the form, plus one build diagnostic. Logseq would run the form, so showing results without it would be silently wrong; the issue also asks for the box. Detection happens when the map is read, so no query starts and then fails halfway.

### 4 - What does a transform or view that throws do?

Decided by the agent without codekiln: what Logseq does. A transform that throws leaves the rows untransformed with a diagnostic (Logseq logs the error and shows the untransformed results). A view that throws gives `Custom view failed:` and the error text. A function that reaches a limit is a failure box, not an untransformed result, because a limit is a property of this build and Logseq would have finished.

### 5 - What are the limits on a query?

Decided by the agent without codekiln: 20,000,000 steps, 64 levels of nested calls, and 128 KB of view HTML. The step limit leaves four times the room that sorting every page of the encode garden with gitpa's comparator needs. The output limit keeps a page's growth near the budget and protects first paint on phones. A step count replaces a time limit so that the same commit always builds the same page.

### 6 - What does a row hold?

Decided by the agent without codekiln: `:block/name`, `:block/original-name` and `:block/properties`, the three attributes the gitpa query reads. Any other attribute fails with its name. Logseq's `[*]` pull has more, but each extra attribute would need a meaning in a published site (`:db/id`, `:block/file`), and a silent `nil` would differ from Logseq without telling the author.

### 7 - How are property values typed?

Decided by the agent without codekiln: with Logseq's `parse-property` rules, as listed under Typed page property values. This is what makes `public:: true` match `(= ?public true)`, and it is the behaviour the parity fixture checks against Logseq 0.10.15. Names inside a set keep the case they were written in, as Logseq keeps them.

### 8 - In what order do rows reach the transform?

Decided by the agent without codekiln: lower-case page name. Datascript's order is its entity-id order, which no build outside Logseq's database reproduces, and a stable `sort-by` then keeps name order for equal keys. The earlier queries change chose page-name order for the same reason.

### 9 - Which Datalog does the build run?

Decided by the agent without codekiln: the `:find (pull ?p [*])` shape over pages, with the data patterns and function clauses listed under Datalog subset. It is the smallest subset that runs gitpa's query, covers every property filter on pages, and leaves blocks, `:in`, `:rules` and aggregates to a later change with its own analysis.

### 10 - Which tags, attributes and URLs may a view write?

Decided by the agent without codekiln: the lists under Hiccup output. A view is code from the garden's author, and the page is public, so an allow-list of text and link elements and attributes keeps `script`, `style` and event handlers out. Logseq itself would render other elements; the box names a rejected one so the author sees the cause. Tag shorthand such as `:div.card` is unsupported because it adds a second way to write a class for no gitpa need.

### 11 - How do `#/page/<name>` links resolve?

Decided by the agent without codekiln: the publisher rewrites them to the page's published URL while it renders, and leaves an unmatched link as written. That gives a direct link with no redirect hop, works without `--redirect-old-urls`, and follows how Logseq's own router reads the form. The link is plain output and creates no reference, as in Logseq.

### 12 - What does the header show for a view?

Decided by the agent without codekiln: the title, the count of rows after the transform, and no table toggle, because Logseq draws a custom view in place of both. An empty result runs no view and shows no count and `No matched result`. The analysis expected an empty body; the Apply step compared Logseq 0.10.15's own export of the fixture garden and saw `No matched result`, so the build matches that.

### 13 - What does a transform without a view return?

Decided by the agent without codekiln: a subset or reordering of the original rows, shown as the page table the box already draws. A transform that returns other values has nothing the table can show, so it is `Query failed:`. Block rows and arbitrary shapes need a view.

### 14 - What happens to a simple-language query with `:view` or `:result-transform`?

Decided by the agent without codekiln: the existing behaviour stays, `Custom view failed:` for a view and the untransformed results for a transform, with the message saying that block results do not support functions at build time. Block rows need a model of Logseq's block attributes, which belongs to the later change that adds Datalog over blocks.

### 15 - Do the default journal queries run?

Decided by the agent without codekiln: no. They are block queries, so they stay skipped with one diagnostic each, as the queries capability already says.

### 16 - How is gitpa's query kept honest without hard-coding gitpa?

Decided by the agent without codekiln: the SDK and publisher contain no garden names, a unit test renames gitpa's property and entity type in a copy of the query, and gitpa's real page is verified once in the Verify step against `site:query:check` in gitpa. Fixtures in this repository use their own names under `Tools/Queries/Typed`.

### 17 - Which budgets apply, and how are they measured?

Decided by the agent without codekiln: the `[advanced_queries]` table under Performance budgets, on top of the existing `[desktop]`, `[phone]` and `[search]` budgets. The measurement uses a copy of the whole encode garden with two added pages, so the comparison with Logseq's build needs no gitpa data; the worst-case page covers a query that touches every page. Committed rows go in the Performance page and in `docs/parity/publication-speed.json`, and CI runs `parity:budgets` and the publication-speed check.

### 18 - What lives in the SDK and what in the publisher?

Decided by the agent without codekiln: the SDK owns reading, typed property values, the Datalog subset, the interpreter, limits and the Hiccup tree, and exposes one `run_advanced` call that returns rows or a failure with no HTML. The publisher owns HTML writing, the escaping allow-lists' output, the box, the error boxes and the diagnostics. This is the split the shared-engine comment on #106 asks for, so a later `lsg query` command or language-server hover can reuse the SDK call.
