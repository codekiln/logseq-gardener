## 1. Reproduce

- [x] 1.1 Add SDK tests for a nested reference and a file link in a property value and for built-in pages without a file, and see them fail
- [x] 1.2 Run `scripts/logseq-parser/check.py` on a graph with each case and record the differences

## 2. Fix in the SDK

- [x] 2.1 Read the values of properties other than `alias::`, `aliases::` and `tags::` from the block's text alone
- [x] 2.2 Add `page_names::built_in_page_title` and spell reference-only graph nodes with it

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/nested-property-reference` with `fixtures/namespace-parent-space`, and update `test_check.py`, the `docs:parser-check` task comment and the docs page that cites the example
- [x] 3.2 File the new deliberate mismatch as [Issue #412](https://github.com/codekiln/logseq-gardener/issues/412)

## 4. Verification

- [x] 4.1 Run the parser check on the case graph, the docs graph and the encode garden before and after, and publish the encode garden with both builds; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`
