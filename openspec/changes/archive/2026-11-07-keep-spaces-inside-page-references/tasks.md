## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/page-reference-space` and on a case graph with `[[Release Notes ]]`, `[[ Release Notes]]`, `[[ Notes/Setup ]]`, `#[[Tag ]]`, `{{embed [[Embed Target ]]}}`, `[[ ]]` and property values with spaces inside their brackets, and record the differences in `verification.md`
- [x] 1.2 Add `site_references` test `a_space_inside_the_brackets_keeps_the_reference_off_the_trimmed_page`, which publishes `Notes/Release Notes` and links to it as `[[Notes/Release Notes ]]` and `[[ Notes/Release Notes]]`, and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Replace `page_names::namespace_parent_key` with `reference_key` and `ends_in_space` with `has_outer_space`, and update every caller; verify with `cargo test -p logseq-gardener-sdk`
- [x] 2.2 Keep written names untrimmed and drop blank ones in `Contribution::from_document`; verify with `references::tests::a_page_reference_keeps_the_spaces_inside_its_brackets`
- [x] 2.3 Key every graph name with `reference_key` and stop trimming it in the graph builder; verify with `graph::tests::a_page_reference_keeps_the_spaces_inside_its_brackets`
- [x] 2.4 Trim the name the publisher's page link resolves and shows; verify with the `site_references` test from 1.2

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/page-reference-space` with `fixtures/code-span-property`, and update `test_check.py`, the `docs:parser-check` task comment and the example in `lsg/How To/Edit the Documentation`; verify with `python3 -m unittest discover -s scripts/logseq-parser -p 'test_*.py'`
- [x] 3.2 File the new deliberate mismatch as [Issue #431](https://github.com/codekiln/logseq-gardener/issues/431)

## 4. Verification

- [x] 4.1 Run the parser check on the case graph, the docs graph and the encode garden before and after, and publish the encode garden with both builds; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `reference-index` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #421. Sync it after `name-namespace-parent-with-space` (#423), whose requirement it sits beside.
