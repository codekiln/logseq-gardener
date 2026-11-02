## 1. Reproduce

- [x] 1.1 Read Logseq's build of the encode garden (gh-pages commit `8132c95`) for every block with a rejected name, and record its markup and computed styles in `verification.md`
- [x] 1.2 Add `site_properties` test `a_block_whose_property_name_logseq_rejects_shows_logseq_warning`, and see it fail on `origin/main`

## 2. Report the names in the SDK

- [x] 2.1 Move the property-line reader from `references.rs` to `property_names::rejected_property_line`; verify with `references::tests::a_property_logseq_rejects_names_no_page`
- [x] 2.2 Add `property_names::rejected_property_names`; verify with `property_names::tests::lists_the_rejected_names_as_logseq_shows_them`, `property_names::tests::a_block_of_accepted_names_lists_none` and its doc test

## 3. Render the warning

- [x] 3.1 Render Logseq's markup in place of a properties block with rejected names in `site/render.rs`; verify with the test from 1.2 and `site_properties::a_page_property_name_logseq_rejects_shows_logseq_warning_once`
- [x] 3.2 Add Logseq's margin, button and code styles to `site/inline.css`; verify the computed styles against Logseq's build

## 4. Verification

- [x] 4.1 Publish the encode garden's `Logseq`, `Podcast` and `Ableton` namespaces with both builds, compare the warnings with Logseq's build, and record the result in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `markdown-garden-loading` and `published-page-outline` deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #439.
