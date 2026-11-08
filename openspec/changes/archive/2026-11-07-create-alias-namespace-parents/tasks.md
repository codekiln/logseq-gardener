## 1. Reproduce

- [x] 1.1 Add the fixture `crates/logseq-gardener-sdk/tests/fixtures/alias-namespace-parents` and the test `an_alias_in_a_namespace_names_its_namespace_parents`, and see it fail on `origin/main`
- [x] 1.2 Run `scripts/logseq-parser/check.py` on the fixture and on the encode garden, and record the differences in `verification.md`

## 2. Fix in the SDK

- [x] 2.1 Add `GraphRefs::unresolved_parents` and fill it in `Contribution::from_document`
- [x] 2.2 Make each unresolved parent a node in `GraphIndex::new`
- [x] 2.3 Add the unit test `an_alias_in_a_namespace_brings_in_its_parents` for the global graph and the page graph

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/alias-namespace-parents` with `fixtures/trailing-slash-name` as the deliberate mismatch, and update `test_check.py` and the `docs:parser-check` task comment
- [x] 3.2 File the new deliberate mismatch as [Issue #495](https://github.com/codekiln/logseq-gardener/issues/495)

## 4. Verification

- [x] 4.1 Run the parser check on the fixtures, the docs graph and the encode garden after the change, and record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `garden-graph` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #462.
