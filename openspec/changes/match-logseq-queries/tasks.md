## 1. SDK

- [ ] 1.1 Add the `query` module's simple-language parser with `<% today %>` expansion and byte-offset errors; unit-test every form in the analysis's table, `(and [[tmux]]`, a bare word and an unknown form
- [ ] 1.2 Add the `#+BEGIN_QUERY` map parser: the listed keys, Datalog detection and invalid-map errors; test a string `:title`, a hiccup `:title`, a Datalog `:query` and an unbalanced map
- [ ] 1.3 Add `evaluate` over `ReferenceIndex`, task markers, priorities, properties and journal dates with the build date as "today"; test page against block results, aliases, `not`-only queries, `between` bounds, `sort-by` and seeded `sample`
- [ ] 1.4 Add result ordering and trimming: own-block exclusion, the top-level-only rule and its `:remove-block-children? false` opt-out, journal groups newest first and other pages by name
- [ ] 1.5 Make `ReferenceIndex` skip `#+BEGIN_QUERY` text when collecting references; test the `Query Target` fixture
- [ ] 1.6 Check the encode-garden counts from the analysis in an SDK test over the garden at `083106d`, or record each one in `verification.md` when the test cannot pin that commit

## 2. Publisher

- [ ] 2.1 Add `site/queries.rs`: the box, header, read-only clause row, grouped and ungrouped lists, `No matched result`, and the blank-query state; route the `query` macro and the `Block::Custom` `query` arm in `site/render.rs` to it
- [ ] 2.2 Render tables with Logseq's columns, `query-properties`, `query-sort-by` and `query-sort-desc`, page links and UTC dates, reusing the All pages table styles
- [ ] 2.3 Write the in-page part and fragments for large results, the collapsed state with results only in fragments, and the other view's fragment for the toggle
- [ ] 2.4 Render the three error forms, `Custom view failed:`, the ignored `:result-transform`, and a diagnostic for each
- [ ] 2.5 Add `site/queries.css` and `site/queries.js` (column sort, view toggle, open-to-load for collapsed queries, reusing `fragments.js`) and link them only from pages with a query; test that `tmux` is unchanged
- [ ] 2.6 Render simple-language default journal queries under the build date's journal on the feed and the journal page; skip Datalog entries with a diagnostic each
- [ ] 2.7 Update the `published-inline-markup` tests and `tests/site.rs` that expect the literal `{{query}}`

## 3. Harness and fixtures

- [ ] 3.1 Add a `Queries` page to `tests/fixtures/parity-garden/` holding the analysis's test queries that can run on the fixture garden, plus the `Query Target` page
- [ ] 3.2 Add the `[queries]` table to `scripts/parity/budgets.toml` and make `parity:budgets` check `queries.js`, `queries.css`, the in-page part, fragment sizes, and the tmux query page against the first-paint budgets with 2,000 extra pages
- [ ] 3.3 Make `parity:perf` record opening the collapsed tmux query on desktop and phone and the phone heap after all its fragments load; make `parity:screenshots` capture the yazi query box at 1280×800

## 4. Verification

- [ ] 4.1 Build the encode garden with a patched copy holding the analysis's test queries; compare each with the measured Logseq result and styles, and record values and screenshots in `verification.md`
- [ ] 4.2 Run `parity:perf --record` and commit the query rows in `docs/parity/performance.md`
- [ ] 4.3 Run `mise run ci`
