## 1. Reproduce

- [x] 1.1 Add SDK tests for a task marker, a priority, plain names in `alias::` and `tags::`, `alias:: #[[Name]]` and a nested reference in an alias value, and see them fail
- [x] 1.2 Run `scripts/logseq-parser/check.py` on a graph with each case and record the differences

## 2. Fix in the SDK

- [x] 2.1 Add `page_names::linkable_property_names` and read page aliases with it
- [x] 2.2 Read `alias::`, `aliases::` and `tags::` values in `Contribution::from_document` with it, keep a literal page tag for a value with no names, and remove `tag_names`
- [x] 2.3 Add the bullet's task marker and priority to its references

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/tag-alias` with `fixtures/nested-property-reference`, and update `test_check.py` and the `docs:parser-check` task comment
- [x] 3.2 File the remaining differences as [Issue #378](https://github.com/codekiln/logseq-gardener/issues/378)

## 4. Verification

- [x] 4.1 Run the parser check on the case graph, the docs graph and the encode garden before and after, and publish the encode garden with both builds; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`
