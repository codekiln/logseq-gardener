## 1. Reproduce

- [x] 1.1 Add the fixture `crates/logseq-gardener-sdk/tests/fixtures/wrong-weekday-date-name` and the test `a_date_name_takes_the_weekday_the_date_has`, and see it fail on `origin/main`
- [x] 1.2 Run `scripts/logseq-parser/check.py` on the fixture and on the encode garden, and record the differences in `verification.md`

## 2. Fix in the SDK

- [x] 2.1 Make `DateFormat::read` accept a weekday name that differs from the date's weekday
- [x] 2.2 Replace `GraphSource::is_journal_name` with `journal_title`, and title a reference-only date page with it in `graph.rs`
- [x] 2.3 Pass the date's title from the publisher in `site/mod.rs` and from the comparison harness in `tests/relationships/mod.rs`

## 3. Parser check fixture

- [x] 3.1 Keep `fixtures/wrong-weekday-date-name` as a fixture that must agree, add `fixtures/alias-spelling-case` as the deliberate mismatch, and update `test_check.py` and the `docs:parser-check` task comment
- [x] 3.2 File the new deliberate mismatch as [Issue #509](https://github.com/codekiln/logseq-gardener/issues/509)

## 4. Verification

- [x] 4.1 Run the parser check on the fixtures, the docs graph and the encode garden before and after the change, and record the results in `verification.md`
- [x] 4.2 Check a published page that links a date with a wrong weekday
- [ ] 4.3 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `garden-graph` and `journal-identity` deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #503.
