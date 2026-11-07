## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/duplicate-block-id` and on the encode garden, and record the differences in `verification.md`
- [x] 1.2 Read Logseq's build of the encode garden for the `About` blocks of `json/5` and `json/jsonc` and the embed on `2025-03-05 Wed`, and record what it shows
- [x] 1.3 Add `site_embeds` test `a_block_id_two_pages_share_names_the_block_logseq_reads_first`, and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Add `block_ids::parse_order` with Logseq's file order; verify with `block_ids::tests::logseq_reads_journals_from_the_last_then_contents_then_pages`
- [x] 2.2 Add `block_ids::keep_each_block_id_once` and call it from `load_garden`; verify with the other `block_ids::tests` and the `site_embeds` test from 1.3
- [x] 2.3 Update the tests that expected a shared id to name no block: `site::block_links_resolve_selected_nested_ids_without_exposing_other_targets`, `site_build_plugins::view_holds_published_pages_and_blocks_only`, `site_embeds::private_excluded_missing_and_ambiguous_targets_keep_fallbacks` and `site_search::duplicate_uuid_uses_offsets_and_large_original_outlines_have_real_destinations`
- [x] 2.4 Describe which block keeps a shared id in `lsg/Ref/Selection and Visibility`

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/duplicate-block-id` with `fixtures/alias-title-case`, and update `test_check.py`, the `docs:parser-check` task comment and the example in `lsg/How To/Edit the Documentation`; verify with `python3 -m unittest discover -s scripts/logseq-parser -p 'test_*.py'`
- [x] 3.2 File the new deliberate mismatch as [Issue #444](https://github.com/codekiln/logseq-gardener/issues/444)

## 4. Verification

- [x] 4.1 Run the parser check on the fixture, the docs graph and the encode garden before and after, and publish the encode garden with both builds; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `markdown-garden-loading` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #438.
