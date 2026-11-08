## 1. Reproduce

- [x] 1.1 Add the fixture `crates/logseq-gardener-sdk/tests/fixtures/trailing-slash-name` and the test `a_boundary_slash_is_not_part_of_a_page_name`, and see it fail on `origin/main`
- [x] 1.2 Run `scripts/logseq-parser/check.py` on the fixture and on the encode garden, and record the differences in `verification.md`

## 2. Fix in the SDK

- [x] 2.1 Add `page_names::remove_boundary_slashes`
- [x] 2.2 Apply it to every written name in `Contribution::from_document`

## 3. Parser check fixture

- [x] 3.1 Add `fixtures/wrong-weekday-date-name` as the deliberate mismatch, keep `fixtures/trailing-slash-name` as a fixture that must agree, and update `test_check.py` and the `docs:parser-check` task comment
- [x] 3.2 File the new deliberate mismatch as [Issue #503](https://github.com/codekiln/logseq-gardener/issues/503)

## 4. Verification

- [x] 4.1 Run the parser check on the fixtures, the docs graph and the encode garden after the change, and record the results in `verification.md`
- [x] 4.2 Check a published page that links a trailing-slash name
- [x] 4.3 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `garden-graph` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #495.
