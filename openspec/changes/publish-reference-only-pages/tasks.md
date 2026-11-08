## 1. SDK: reference-only pages

- [ ] 1.1 Add to `tests/fixtures/parity-garden` a page linking a name with no file, a tag with a boundary slash, a name in two spellings, a link to a `public:: false` page, a name only a private page links, a file hidden by `:hidden`, a nested name whose parents have no file, a link to a generated namespace page and a date name without a journal file; verify `mise run test` passes
- [ ] 1.2 Add the `reference_only` module: collect names from published page references, tags and page-valued properties with their namespace parents, drop names with a file, names only withheld, unselected or skipped blocks write, and task markers and priorities; verify with unit tests for every scenario of "Find the reference-only pages of a publication", "Keep withheld and unselected names unresolved" and "Publish reference-only pages under any selection"
- [ ] 1.3 Title reference-only pages with the first-reference rule, boundary-slash removal, journal titles, built-in spellings and kept spaces; verify with unit tests, and compare the titles of all 23 multi-spelling names on the encode garden with Logseq's build through `scripts/parity/logseq_db.py`, recording the result in `verification.md`
- [ ] 1.4 Fill the page-name index with reference-only pages and generated namespace pages, keeping alias, embed, block reference and homepage lookups on source pages; verify with unit tests for the scenarios of "Resolve links to every published page"
- [ ] 1.5 Route reference-only pages in `publication.rs` under both URL schemes and turn a collision into a diagnostic; verify with tests for the digest, readable, unsafe-title and `Assets/Logo` collision scenarios
- [ ] 1.6 Answer linked references, tagged pages, hierarchy rows, unlinked mentions and backlink counts for reference-only keys; verify with tests for `Deno Deploy`, `company` and `1Password/Desktop/Settings`
- [ ] 1.7 Update the set when the reference index replaces one contribution; verify with a test that adds and removes names and compares every answer with a full rebuild
- [ ] 1.8 Add kind `Reference` to the page list, routes to published reference-only graph nodes, page records to the search index and kind `reference` to the publication view and `lsg-site.json`; verify with unit tests and add the kind to `docs/pages/lsg___Ref___Site Manifest.md` and `docs/pages/lsg___Ref___Plugin Host API.md`

## 2. Publisher

- [ ] 2.1 Write a page per reference-only page from the generated page template without the Namespace notice; verify with `tests/publish.rs` cases for `Deno Deploy`, a tag page and a namespaced name
- [ ] 2.2 Render links, tags and page-valued properties to reference-only and generated namespace pages in `site/render.rs` and the property box, and keep `unresolved` and diagnostics only for withheld, unselected, ambiguous and collided names; verify with tests for the `local-static-publishing`, `published-namespace-hierarchy` and `published-page-properties` scenarios
- [ ] 2.3 Limit a reference-only page's Unlinked References to the first fragment file and end the list with `Showing the first N of M pages`; verify with a test on a fixture name with more than one file of mentions
- [ ] 2.4 List reference-only pages in All pages with the ordinary link style and no dates, and reference-only journals in the journals feed; verify with tests for both scenarios
- [ ] 2.5 Make `site/graph.js` open nodes that have a route and keep `not published` for the rest; verify with JavaScript tests for the `Deno Deploy` and `TODO` nodes and a Shift-click test that adds a page card for a reference-only page
- [ ] 2.6 Count reference-only pages in the `lsg publish` human and JSON reports; verify with a CLI test and add the count to `docs/pages/lsg___Ref___Publish Options.md`
- [ ] 2.7 Verify with a `tests/publish.rs` case that adding a block naming a new page and publishing again changes only the files the "Rewrite only the affected files after an edit" requirement names, and that removing it removes the page

## 3. Search format version 4

- [ ] 3.1 Write title entries without computable routes and the URL scheme in the manifest, at format version 4, in `search.rs`; verify with Rust tests for both URL schemes and a page published at another route
- [ ] 3.2 Compute routes in `site/search.mjs` and reject version 3 data with the reload message; verify with `tests/search.test.cjs`, `tests/search-loader.test.cjs` and an updated `tests/search-fixture.json`
- [ ] 3.3 Document the version 4 title entry and manifest field in `docs/pages/lsg___Ref___Static Search Data.md`, and verify `mise run docs:check` passes

## 4. Budgets

- [ ] 4.1 Add the `[reference_only]` table and the new `[site]` values to `scripts/parity/budgets.toml`, and make `parity:budgets` check the page size, the 2,000-name growth, one Unlinked References file per reference-only page and the search engine's growth; verify with tests in `scripts/parity/test_parity.py`
- [ ] 4.2 Add `Deno Deploy` to `scripts/parity/fixtures.txt` and make `parity:perf` measure it in both builds and mark values over the `[reference_only]`, `[desktop]`, `[phone]` and `[search]` budgets; verify with a `test_parity.py` case

## 5. Verification on the encode garden

- [ ] 5.1 Publish the whole encode garden with `mise run parity:build` and verify that the unresolved-reference diagnostics name only the two withheld `GitP/.../wav` pages and that each fixture page from the analysis behaves as its gap describes; record the results in `verification.md`
- [ ] 5.2 Run `parity:compare` against Logseq's 2026-10-25 build and record the reference-only group in `verification.md`
- [ ] 5.3 Run `parity:perf` on `Deno Deploy` and a search for `deno deploy` in both builds on the desktop and phone profiles, and commit the rows to `docs/parity/performance.md`
- [ ] 5.4 Run `parity:perf --baseline-lsg` and `parity:publish-bench --record`, verify the build time, page HTML, `_refs/`, Linked and Unlinked References totals and site size are within budget, and commit the rows to `docs/parity/publication-speed.json`
- [ ] 5.5 Update the All Pages, Graph View, Right Sidebar and Hierarchy compatibility pages, `docs/pages/lsg___Ref___Selection and Visibility.md` and `docs/parity/reference-only-pages.md` with the new behaviour, and verify `mise run docs:check` passes
- [ ] 5.6 Run `mise run ci`

## Workflow follow-up

- Create the cleanup sub-issue under issue #351 with the implementation pull request and the change name `publish-reference-only-pages`.
- After the implementation merges, sync every delta capability and archive the change following `lsg/How To/Finish an OpenSpec Change After Merge`.
