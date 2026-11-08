# Tasks

## 1. SDK: typed values, Datalog subset and limits

- [x] 1.1 Add the typed property reader and the three-attribute page row; unit-test `true`, `false`, `"true"`, digits, a two-name set with original case and a missing key
- [x] 1.2 Read the Datalog subset in `parse_advanced` with unbound-variable and unsupported-form errors that carry byte offsets; unit-test the gitpa-form query, `[?b :block/marker ?m]`, `:in`, `or` and an unbound variable
- [x] 1.3 Evaluate the subset over the published pages in lower-case name order, dropping rows on a nil binding or a false predicate; unit-test a withheld page, a missing property and equal keys
- [x] 1.4 Add the step counter and depth limit shared by all evaluation, and test that a repeated run gives the same rows and step count regardless of `SOURCE_DATE_EPOCH`
- [x] 1.5 Update the `garden-queries` map parser to return the Datalog query and the two functions; run `cargo test -p logseq-gardener-sdk query`

## 2. SDK: function interpreter and Hiccup tree

- [x] 2.1 Add `query_script`: parse and evaluate the function subset in the design's table, with unsupported-symbol errors at parse time; unit-test each table row and `slurp`, `partition`, `def` and `loop` as unsupported
- [x] 2.2 Test `compare`, `sort-by` and the comparator against ClojureScript results for `nil`, mixed zones, astral characters and a string against a number (throws), using values recorded from Logseq 0.10.15
- [x] 2.3 Add `query_hiccup`: the tree, string and keyword tags, flattening, `nil` and `false`, numbers, and the shorthand error; unit-test gitpa's view
- [x] 2.4 Add `run_advanced` returning rows, the view's tree or the failure; test a caller with no view and the renamed-property test for no garden names
- [x] 2.5 Measure the step count of sorting 16,693 rows with gitpa's comparator and record it in `verification.md`; confirm the limit stops a runaway `concat` within 2 s

## 3. Publisher

- [x] 3.1 Render the view's tree inside the query box with the tag, attribute and URL allow-lists, escaping and the 128 KB limit; test `script`, `style`, `onclick`, `javascript:` and `A <b>&</b>`
- [x] 3.2 Rewrite `#/page/<name>` links to published URLs and leave unmatched ones; test a name with `%2F`, a withheld page and that the target's Linked References stay unchanged
- [x] 3.3 Show the count, hide the toggle, call no view on an empty result, and load a collapsed view from a fragment; test each in `tests/site.rs`
- [x] 3.4 Render the new `Query failed:` and `Custom view failed:` boxes with the form named and one diagnostic with the line; test an unsupported transform, an unsupported view, a thrown transform and a thrown view
- [x] 3.5 Update the existing `advanced_box` tests that expect "Datalog queries are not supported", and the simple-query `:view` message; run `cargo test -p logseq-gardener-sdk site`

## 4. Fixtures and harness

- [x] 4.1 Add `Tools/Queries/Typed/…` pages and an advanced query to `tests/fixtures/parity-garden` covering the typed values, the zones and the error boxes; run `mise run parity:logseq-export` and check the rows match Logseq 0.10.15
- [x] 4.2 Add the `[advanced_queries]` table to `scripts/parity/budgets.toml` and make `parity:budgets` check growth, no script, no stylesheet and the view size; run `mise run parity:budgets`
- [ ] 4.3 Add `parity:perf --advanced-queries` with the two pages from the design, measured in `lsg` and in a Logseq build of the same copy; record the rows in the Performance page with `--record`
- [ ] 4.4 Add the gitpa-form query to the `gitpa-sized` corpus homepage, record a row with `parity:publish-bench --record`, and confirm `--check` runs in CI

## 5. Verification

- [x] 5.1 Build gitpa's `Ghost in the Patch` with plain `lsg publish`; compare its list with `mise run site:query:check` in gitpa and record the result in `verification.md`
- [ ] 5.2 Check the whole-garden build, the all-pages query time, the phone heap and the search latency against the budgets; record them in `verification.md`
- [ ] 5.3 Run `mise run ci`
