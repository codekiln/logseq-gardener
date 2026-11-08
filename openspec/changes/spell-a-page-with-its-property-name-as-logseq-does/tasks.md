## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/property-name-reference` and on small gardens that vary the order of the blocks and the case, and record Logseq's rule in `design.md`
- [x] 1.2 Add the test `a_page_without_a_file_takes_the_spelling_of_the_last_block_that_names_it` to `graph.rs`, and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Spell a page without a file from the last block that names it in `GraphIndex::new`
- [x] 2.2 Leave a block's page reference that is also one of its property names out of the parser check's SDK dump

## 3. Parser check fixture

- [x] 3.1 Extend `fixtures/property-name-reference`, make `test_check.py` expect agreement, and remove the deliberate mismatch note from the `docs:parser-check` task

## 4. Verification

- [x] 4.1 Run the parser check on the fixture and the encode garden before and after, and record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `garden-graph` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #521.
