## 1. Reproduce

- [x] 1.1 Read Logseq's `block-control` and `block-content-empty?`, and record the bullet of the warning block on `Logseq/Entity/Asset` in Logseq's build (gh-pages commit `8132c95`) at rest and under the pointer
- [x] 1.2 Add `site_collapse` test `a_block_with_no_text_hides_its_bullet_as_logseq_does`, and see it fail on `origin/main`

## 2. Decide in the SDK

- [x] 2.1 Add `page_properties::is_hidden_block_property`; verify with `page_properties::tests::hides_what_logseq_hides_on_a_block`
- [x] 2.2 Add `OutlineBlock::empty` and `Outline::hides_bullet`; verify with `outline::tests::empty_blocks` and `outline::tests::hides_the_bullet_of_an_empty_block_between_the_first_and_last`

## 3. Render

- [x] 3.1 Add `lsg-empty` to the item of each such block in `site/render.rs`; verify with the test from 1.2 and `site_collapse::an_embedded_block_with_no_text_hides_its_bullet`
- [x] 3.2 Hide the bullet of an `lsg-empty` item until the pointer is over it in `site/outline.css`; verify the bullet's visibility at rest and under the pointer in a browser

## 4. Verification

- [x] 4.1 Publish the encode garden with both builds, compare the hidden bullets with Logseq's build page by page, and record the result in `verification.md`
- [x] 4.2 Run `mise run ci`

## Workflow follow-up

- After the pull request merges, sync the `outline-blocks` and `published-page-outline` deltas and archive the change per `lsg/How To/Finish an OpenSpec Change After Merge`, under the cleanup sub-issue of #445.
