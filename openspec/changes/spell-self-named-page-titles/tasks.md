## 1. Reproduce

- [x] 1.1 Run `scripts/logseq-parser/check.py` on `fixtures/alias-title-case`, on a test graph with a tag, a namespace parent, a task marker and an earlier journal reference, and on the encode garden, and record the differences in `verification.md`
- [x] 1.2 Read the titles of `Solvespace`, `Brew` and the Claude MCP page in Logseq's build of the encode garden, and record them
- [x] 1.3 Add `garden` test `a_page_that_names_itself_in_another_spelling_takes_that_spelling`, and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Add `title_spellings::spell_titles_as_logseq` and call it from `load_garden`; add `GardenDocument::title_spellings` and set it in every place that builds a `GardenDocument`
- [x] 2.2 Return the recorded spelling from `page_title`; verify with the test from 1.3
- [x] 2.3 Move the property links in `site_properties::property_links_keep_the_name_as_written` and the page in `site_readable::folders_spelled_two_ways_publish_under_one_spelling` to pages that do not refer to themselves
- [x] 2.4 Describe the rule in `lsg/Ref/Page Titles`

## 3. Parser check fixture

- [x] 3.1 Replace `fixtures/alias-title-case` with `fixtures/alias-of-another-page`, and update `test_check.py` and the `docs:parser-check` task comment; verify with `python3 -m unittest discover -s scripts/logseq-parser -p 'test_*.py'`
- [x] 3.2 File the new deliberate mismatch as [Issue #453](https://github.com/codekiln/logseq-gardener/issues/453)

## 4. Verification

- [x] 4.1 Run the parser check on the test graph, the docs graph and the encode garden after the change, publish the encode garden with both builds, and time loading; record the results in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `markdown-page-titles` delta and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #444.
