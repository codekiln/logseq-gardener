# Run advanced queries at build time

`lsg` can show an advanced query only as an error box today, and Logseq's published site shows its result. This change makes `lsg` run the form gitpa's homepage uses (a Datalog `:query` over page properties, a `:result-transform` that sorts, and a `:view` that returns Hiccup links) while it builds the site, so the page lists its episodes with no script in the browser. This document is step 1, Analyze, of [issue #485](https://github.com/codekiln/logseq-gardener/issues/485), part of [#484](https://github.com/codekiln/logseq-gardener/issues/484). It records Logseq's behaviour, what `lsg` does now, and the gaps a test can check.

The work is too large for one pull request, so it follows four steps, one pull request each: Analyze (this document), Propose (an OpenSpec change), Apply (code and tests), and Sync (spec sync and archive). The reason is in [Why four steps](#why-four-steps).

Measured on 2026-11-08 with `lsg` built from logseq-gardener `d28b43a`, gitpa `410f4d9` and Logseq's source at `og` `6e7afa8eb`. Logseq's behaviour below comes from that source and from the earlier browser measurements in [Queries Compatibility](../../logseq-gardener-docs/pages/lsg___Ref___Logseq%20Compatibility___Queries.md). No new browser measurement was made in this step; the first Apply test compares the result with Logseq 0.10.15's own export (`mise run parity:logseq-export`).

## The query to match

Gitpa's homepage, [`gitp-garden/pages/Ghost in the Patch.md`](https://github.com/codekiln/gitpa/blob/410f4d9/gitp-garden/pages/Ghost%20in%20the%20Patch.md), holds the only advanced query this change must run:

```clojure
{:title [:h2 "Recent Ceremonies"]
 :query [:find (pull ?p [*])
         :where
         [?p :block/name]
         [?p :block/properties ?props]
         [(get ?props :logseq-entity) ?entities]
         [(contains? ?entities "Logseq/Entity/Podcast/Episode")]
         [(get ?props :public) ?public]
         [(= ?public true)]]
 :result-transform (fn [rows] (sort-by (fn [row] (get-in row [:block/properties :podcast-published-at]))
                                       (fn [a b] (compare b a)) rows))
 :view (fn [rows]
         (into ["ul"]
               (map (fn [row]
                      ["li" ["a" {:href (str "#/page/" (clojure.string/replace (:block/name row) "/" "%2F"))}
                             (:block/original-name row)]]) rows)))
 :table-view? false}
```

Its block also carries `query-table:: false` and `query-properties:: [:page]`. The garden holds 27 pages typed `Logseq/Entity/Podcast/Episode`, 18 of them `public:: true`. The encode garden holds one `#+BEGIN_QUERY` block, inside an analysis page that quotes a query; it has no live advanced query besides the two default journal queries in its `config.edn`, which this change leaves alone.

## What Logseq does

Logseq runs the whole query in the browser, against a Datascript database of the entire graph.

1. **Datalog.** A `:query` vector that starts with `:find` runs as Datalog. `(pull ?b [*])` is rewritten to a fixed list of block attributes, so a block result carries no `:block/name`; `(pull ?p [*])` on any other variable keeps every attribute, which is why gitpa's page results have `:block/name` and `:block/original-name` ([`query_custom.cljs` L12-L24, L60-L73](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_custom.cljs#L12-L73), [`block-attrs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/model.cljs#L37-L64)).
2. **Property values have types.** A property whose value holds page references becomes a set of page names; `true` and `false` become booleans; an all-digit value becomes a number; a value in quotes stays a string; anything else stays a string ([`parse-property`](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/text.cljs#L87-L98), [L165-L195](https://github.com/logseq/og/blob/6e7afa8eb/deps/graph-parser/src/logseq/graph_parser/text.cljs#L165-L195)). So `public:: true` matches `(= ?public true)`, while `public:: "true"` and `public:: false` do not, and `logseq-entity:: [[A]], [[B]]` is a set that `contains?` tests by name.
3. **A function clause drops a row when it returns nothing.** A page without `:logseq-entity` gets no `?entities` binding, so Datascript drops it. This is Datascript's rule, which Logseq inherits.
4. **`:result-transform`.** Results are flattened, then the function runs under SCI, a ClojureScript interpreter. If the function fails to read or throws, Logseq logs the error and shows the untransformed results ([`query_react.cljs` L32-L57](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/db/query_react.cljs#L32-L57)). A `:result-transform` also turns off grouping by page ([`result.cljs` L51-L57](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query/result.cljs#L51-L57)).
5. **`:view`.** When the result is not empty, Logseq calls the function with the result and converts every vector whose first element is a string into a keyword-tagged element, so `["ul" …]` renders as `<ul>`. A function that throws gives a `div` reading `Custom view failed: ` and the error ([`query.cljs` L70-L77](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L70-L77), [L163-L164](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L163-L164), [`hiccup-keywordize`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/util.cljc#L964-L971)). With an empty result the body is empty.
6. **Errors from the query itself** show `Query failed:` and the message in a `.warning.my-1` box ([`query.cljs` L63-L67](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/query.cljs#L63-L67)).
7. **The SCI environment** has Clojure's core, string, set and walk libraries plus `sum`, `average`, `parseFloat`, `isNaN`, `log`, `pprint` and `call-api` ([`sci.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/extensions/sci.cljs#L29-L49)). The forms to support here use a small part of that.
8. **Costs.** The reader pays for the database load and the SCI interpreter before the list appears; that is the load-everything design [#106](https://github.com/codekiln/logseq-gardener/issues/106) rejects.

Logseq's own description of the keys is in [Advanced Queries](https://github.com/logseq/docs/blob/628cdde/pages/Advanced%20Queries.md).

## What `lsg` does now

- `parse_advanced` reads the map and marks a `:query` that holds `:find` as Datalog, `:view` and `:result-transform` as present ([`query.rs` L841-L940](../../crates/logseq-gardener-sdk/src/query.rs)).
- The publisher's `advanced_box` returns `Query failed:` with "This published site evaluates queries in the simple query language only; Datalog queries are not supported." for Datalog, `Custom view failed:` for any `:view`, and for a simple `:query` with a `:result-transform` it ignores the function and reports a diagnostic ([`site/queries.rs` L706-L760](../../crates/logseq-gardener-sdk/src/site/queries.rs)).
- Built on 2026-11-08 from gitpa's source graph with `--homepage "Ghost in the Patch"`, the page's query box contains only the title and `Query failed:` with that message, and the build reports `Query failed: … (line 6)`. Gitpa then replaces the box with rows from `nbb-logseq` after `lsg` finishes, using `scripts/build_site.py` and `mise run site:query:check`.
- Simple queries already run at build time over the published pages, and the rendering of the box (title, count, list view, error boxes) is already Logseq's, so only the evaluation is missing.

## Gaps

Each gap is a test. "Fixture" names a page described under [Fixture pages](#fixture-pages).

1. **Run the Datalog subset.** `:find (pull ?v [*])` with data patterns (`[?p :block/name]`, `[?p :block/properties ?props]`) and function clauses (`get` as a binding, `contains?` and `=` as predicates) returns the pages that satisfy every clause. Gitpa's source graph returns its 18 public episodes.
2. **Match property types.** `public:: true` matches; `public:: false`, `public:: "true"` and a missing `public` do not (fixture `Tools/Queries/Typed`). A set-valued property matches by page name with its original case.
3. **Run `:result-transform`.** `fn`, `sort-by` (with and without a comparator), `compare`, `reverse`, `get`, `get-in` and keywords used as functions give the order Logseq gives. Gitpa's `podcast-published-at` values sort as strings, as in Logseq, so two timestamps in different zones keep Logseq's order rather than the order in time.
4. **Run `:view`.** `fn`, `into`, `map`, `str`, `clojure.string/replace` and keyword access build Hiccup; a vector whose first element is a string becomes an element with its attribute map. Text and attribute values are escaped.
5. **Keep links working.** The `#/page/<name>` links the gitpa view builds resolve to the published page the way old Logseq links already do ([`--redirect-old-urls`](../../logseq-gardener-docs/pages/lsg___Ref___Logseq%20Compatibility___Queries.md)). View output creates no reference and no backlink, as in Logseq.
6. **Show the header and count.** The box keeps the `:title`, shows `18 results` for the gitpa query, and hides the table toggle because a `:view` is set. An empty result calls no view and shows an empty body with no count.
7. **Fail like Logseq, and say why.** A form outside the supported set keeps the existing error box: `Query failed:` for the query or the transform, `Custom view failed:` for the view, each naming the unsupported form, plus one build diagnostic with the line. A supported transform or view that throws at build time gives the same boxes as Logseq's throw.
8. **Run each query once, only when present.** A garden with no advanced query builds the same output at the same speed. The result is the same for the same input; no supported form reads the date or the environment.
9. **Match Logseq.** A fixture advanced query in `Tools/Queries` produces the same rows as Logseq 0.10.15 in `mise run parity:logseq-export`.
10. **Verify gitpa's page.** Gitpa's `Ghost in the Patch` built with plain `lsg publish` lists the same public episodes in the same order as `mise run site:query:check` in gitpa.

## Performance budgets

The browser pays nothing: the result is static HTML of about 100 bytes per row, with no script, no database and no interpreter, so the budgets in [`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml) for data and script before the title do not change. The build is the only cost, and it falls only on a page that holds an advanced query.

- `mise run parity:budgets` passes with no new bytes.
- The encode garden's whole-site build stays within `[site]` `build_slowdown_pct` (10%). The garden has no live advanced query, so the expected change is zero; the check proves a garden without queries does not pay.
- A query scans each page once. Gitpa's page, over 2,565 files, is the largest real case; the Propose step sets a time limit per query and the Apply step measures it.

## SDK and publisher

- **SDK** (shared with the command line and a future language server): reading the query, the Datalog subset, the interpreter for the function subset, and the typed property values. A caller can ask the SDK for a query's rows without publishing.
- **Publisher** (rendering only): writing the result's Hiccup as HTML inside the existing query box, the error boxes and diagnostics.

## Why four steps

The change adds two evaluators (a Datalog subset and a ClojureScript subset) with a security boundary: a query is code from the garden's author, so it must not read files, run commands or loop forever. It touches the SDK, the publisher, the parity fixtures and the budgets. Several choices need a written decision before code: how to run the function subset, which functions to support, how a failed transform behaves, and the time limit. The OpenSpec change records them; the pull requests stay small enough to review.

## Fixture pages

- gitpa `Ghost in the Patch` (the real query), verified against `site:query:check`.
- Added in Apply to `tests/fixtures/parity-garden`: `Tools/Queries` gains an advanced query with a transform and a view over a set of entity pages `Tools/Queries/Typed/…` that cover `public:: true`, `public:: false`, `public:: "true"`, no `public`, a property set with two names, and two timestamps in different zones.
- A query that uses an unsupported function in the transform, one in the view, and one that throws, to check the three error boxes.
- A query with an empty result and a view.

## Out of scope

- Datalog over blocks (`:block/marker`, `:block/page`, `:block/journal-day`), `:in`, `:inputs`, `:rules` and aggregates. These run the encode garden's default `NOW` and `NEXT` queries; that is a later change with its own analysis.
- Named functions in `:query/views` and `:query/result-transforms` in `config.edn`.
- `call-api` and the rest of SCI; macros, side effects and anything that reads files or the network.
- The `Setting properties` and refresh buttons.
- Gitpa's choice of public episodes, which stays in `scripts/prepare_site.py`, and removing gitpa's `nbb-logseq` step ([gitpa #160](https://github.com/codekiln/gitpa/issues/160)).
- Adding the episode list to the encode garden ([logseq-encode-garden #340](https://github.com/codekiln/logseq-encode-garden/issues/340)).
