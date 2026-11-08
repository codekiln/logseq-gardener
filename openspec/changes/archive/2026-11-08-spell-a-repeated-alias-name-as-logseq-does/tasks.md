## 1. Reproduce

- [x] 1.1 Add the fixture `crates/logseq-gardener-sdk/tests/fixtures/alias-spelling-case` and the test `an_alias_written_twice_takes_its_last_spelling`, and see it fail on `origin/main`
- [x] 1.2 Run `scripts/logseq-parser/check.py` on the fixture and on the encode garden, and record the differences in `verification.md`

## 2. Fix in the SDK

- [x] 2.1 Make `page_alias_spellings` keep the last spelling of a repeated name at the position of its first

## 3. Parser check fixture

- [x] 3.1 Keep `fixtures/alias-spelling-case` as a fixture that must agree, add `fixtures/alias-spelling-reference` as the deliberate mismatch, and update `test_check.py` and the `docs:parser-check` task comment
- [x] 3.2 File the new deliberate mismatch as [Issue #515](https://github.com/codekiln/logseq-gardener/issues/515)

## 4. Verification

- [x] 4.1 Run the parser check on the fixtures and the encode garden before and after, and record the results in `verification.md`
- [x] 4.2 Check a published page that writes an alias twice
- [x] 4.3 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `garden-graph` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #509.
