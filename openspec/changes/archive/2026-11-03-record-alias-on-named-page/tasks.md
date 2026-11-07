## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/alias-of-another-page`, on a test garden with sibling aliases and the target's own alias, and on the encode garden, and record the differences in `verification.md`
- [x] 1.2 Publish the encode garden at `083106d` with `lsg` from `origin/main`, and record the ambiguous links and the Linked References of `Dolt` and `GitHub/dolthub/dolt`; read the same pages in Logseq's build
- [x] 1.3 Add `og_relationships::an_alias_is_recorded_on_the_page_it_names_too` and `site_references::an_alias_that_names_a_page_joins_both_pages_linked_references`, and see both fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Add `AliasGroups::stored_aliases` and `AliasGroups::group_pages`, and read the parser check's aliases from `stored_aliases`
- [x] 2.2 Let a title win over another page's alias in `PageNameIndex::candidates`, with the exception for a page with no text; update `page_names` and `site` tests that expected an ambiguous title
- [x] 2.3 Add `ReferenceIndex::linked_with_aliases`

## 3. Publisher

- [x] 3.1 List the alias group's references in Linked References, count them in the journal feed, and add the `Alias` label and its style
- [x] 3.2 Look for the alias group's names in Unlinked References
- [x] 3.3 Describe alias pages in `lsg/Ref/Logseq Compatibility/References`

## 4. Parser check fixture

- [x] 4.1 Replace `fixtures/alias-of-another-page` with `fixtures/alias-namespace-parents`, and update `test_check.py` and the `docs:parser-check` task comment
- [x] 4.2 File the new deliberate mismatch as [Issue #462](https://github.com/codekiln/logseq-gardener/issues/462)

## 5. Verification

- [x] 5.1 Run the parser check on the test garden, the docs graph and the encode garden after the change, publish the encode garden with both builds and compare; record the results in `verification.md`
- [x] 5.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `selected-page-aliases`, `published-linked-references` and `published-unlinked-references` deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #453.
