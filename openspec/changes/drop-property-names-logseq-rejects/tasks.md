## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/code-span-property` and on the encode garden, and record the differences in `verification.md`
- [x] 1.2 Read Logseq's build of the encode garden for the block under "Frontmatter and body" on `Logseq/Entity/Asset`, and record what it shows and what its database holds
- [x] 1.3 Add `site_references` test `a_property_name_logseq_rejects_adds_no_reference`, which links `Notes/Done` from `` `status:: [[Notes/Done]]` ``, and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Add `property_names::valid_property_name` with Logseq's rule; verify with `property_names::tests::keeps_the_names_logseq_keeps`, whose answers come from `valid-property-name?` under nbb-logseq
- [x] 2.2 Drop rejected properties in `garden::parse_document`; verify with `garden::tests::drops_the_properties_logseq_rejects`
- [x] 2.3 Blank the values of rejected properties before `Contribution::from_document` reads page names; verify with `references::tests::a_property_logseq_rejects_names_no_page` and the `site_references` test from 1.3

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/code-span-property` with `fixtures/duplicate-block-id`, and update `test_check.py`, the `docs:parser-check` task comment and the example in `lsg/How To/Edit the Documentation`; verify with `python3 -m unittest discover -s scripts/logseq-parser -p 'test_*.py'`
- [x] 3.2 File the new deliberate mismatch as [Issue #438](https://github.com/codekiln/logseq-gardener/issues/438), and the warning Logseq shows as [Issue #439](https://github.com/codekiln/logseq-gardener/issues/439)

## 4. Verification

- [x] 4.1 Run the parser check on the fixture, the docs graph and the encode garden before and after, and publish the encode garden with both builds; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `markdown-garden-loading` and `reference-index` deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #431.
