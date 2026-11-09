# Tasks

## 1. Slot-named shards in the SDK

- [x] 1.1 Compute page slots and slot block numbers in `SearchIndex::files`, write each word's blocks as a base-32 string, declare format version 5, and verify `cargo test -p logseq-gardener-sdk --lib search::` covers the number encoding, the spec's example entry and a page added before and after another in its slot
- [x] 1.2 Add `one_added_page_changes_only_the_shards_of_its_words` to `crates/logseq-gardener-sdk/tests/site_search.rs` and verify it fails with the version 4 numbering and passes with slots

## 2. Browser engine and controller

- [x] 2.1 Compute slots when the first shard loads, decode the number strings, reject malformed strings and unknown slots, accept only format version 5 in the engine and controller, and verify `node --test tests/search.test.cjs tests/search-loader.test.cjs` passes
- [x] 2.2 Regenerate `tests/search-fixture.json` with `LSG_UPDATE_SEARCH_FIXTURE=1`, lowering its shard budget so the `ter` shard still splits, and verify the Rust and JavaScript fixture tests both pass on the shared-slot pages it holds

## 3. Budgets and checks

- [x] 3.1 Add `[search] shards_total_gzip_mb`, record each encode garden row's shard count, total and largest shard in `parity:publish-bench`, fail `--check` over the budget, and verify `python3 -m unittest scripts/parity/test_parity.py` passes
- [x] 3.2 Raise the `[reference_only]` engine growth budget with its reason, move the parity size checks to format version 5, and verify `mise run parity:budgets` passes

## 4. Documentation and verification

- [x] 4.1 Describe slots, the number strings and the two remaining cases in `docs/pages/lsg___Ref___Static Search Data.md`, and verify `mise run docs:check` passes
- [x] 4.2 Publish the encode garden before and after adding one page with version 4 and version 5, measure shard sizes and search latency with `parity:perf --search-site`, and record them in `verification.md`
- [x] 4.3 Run `mise run ci` and verify it passes

## Workflow follow-up

- After the implementation merges, sync the delta specs and archive the change as `lsg/How To/Finish an OpenSpec Change After Merge` describes, in a separate pull request tracked by the cleanup sub-issue.
