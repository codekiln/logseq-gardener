## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/namespace-parent-space` and on a case graph with `[[A / B / C]]`, `#[[Tag /X]]` and a page `Linux /Bin`, and record the differences in `verification.md`
- [x] 1.2 Add `site_references` test `a_space_before_a_slash_keeps_the_reference_off_the_trimmed_parent`, which publishes a page `Notes/MacOS` and a reference to `[[Notes/MacOS /Applications Folder]]`, and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Add `page_names::namespace_parent`, `namespace_parent_key` and `ends_in_space`, and key `built_in_page_title` with `namespace_parent_key`; verify with the graph test for `[[A / B / C]]`
- [x] 2.2 Trim written names and keep namespace parents untrimmed in `Contribution::from_document`, and key page titles with `namespace_parent_key` for namespace parents in `ReferenceIndex::build`; verify with `references::tests::a_namespace_parent_keeps_the_space_before_its_slash`
- [x] 2.3 Give a namespace parent its own graph node and namespace edge; verify with `graph::tests::a_namespace_parent_keeps_the_space_before_its_slash`
- [x] 2.4 Answer a name that ends in a space only from generated namespace pages in the publisher's resolver, and use `namespace_parent` in the `(namespace …)` query; verify with the `site_references` test from 1.2
- [x] 2.5 Key the parser check's SDK side with `namespace_parent_key`; verify that `fixtures/namespace-parent-space` agrees

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/namespace-parent-space` with `fixtures/page-reference-space` (`[[Release Notes ]]`), and update `test_check.py`, the `docs:parser-check` task comment and the example in `lsg/How To/Edit the Documentation`; verify with `python3 -m unittest discover -s scripts/logseq-parser -p 'test_*.py'`
- [x] 3.2 File the new deliberate mismatch as [Issue #421](https://github.com/codekiln/logseq-gardener/issues/421)

## 4. Verification

- [x] 4.1 Run the parser check on the case graph, the docs graph and the encode garden before and after, and publish the encode garden with both builds; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync both deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #412.
