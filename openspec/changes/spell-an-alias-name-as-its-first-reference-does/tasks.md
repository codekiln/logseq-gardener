## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/alias-spelling-reference` and on small gardens that vary which file refers to the alias and how often, and record Logseq's rule in `design.md`
- [x] 1.2 Add the fixture `crates/logseq-gardener-sdk/tests/fixtures/alias-spelling-reference` and the test `an_alias_takes_the_spelling_of_the_first_file_that_refers_to_it`, and see it fail on `origin/main`
- [x] 1.3 Run `scripts/logseq-parser/check.py` on the encode garden and record the differences in `verification.md`

## 2. Fix in the SDK

- [x] 2.1 Record the Logseq spelling of each alias name in `spell_titles_as_logseq` and apply it in `page_alias_spellings`

## 3. Parser check fixture

- [x] 3.1 Keep `fixtures/alias-spelling-reference` as a fixture that must agree, add a fixture that still differs as the deliberate mismatch, and update `test_check.py` and the `docs:parser-check` task comment

## 4. Verification

- [x] 4.1 Run the parser check on the fixtures and the encode garden before and after, and record the results in `verification.md`
- [x] 4.2 Check a published page whose alias a block refers to in another case
- [x] 4.3 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `garden-graph` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #515.
