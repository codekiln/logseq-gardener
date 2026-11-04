## 1. Reproduce

- [x] 1.1 Publish the encode garden with `--readable-urls` using `lsg` v0.0.11, and record the failure in `verification.md`

## 2. Publication

- [x] 2.1 Add `folder_spellings` and `respelled` to `crates/logseq-gardener-sdk/src/page_urls.rs`, with unit tests, and drop the folder spelling comparison from `check_collisions`
- [x] 2.2 Move readable routes under such folders to the chosen spelling in `select_with_urls`, and record the spellings in `Publication::folders`
- [x] 2.3 Add `folders_spelled_two_ways_publish_under_one_spelling` to `crates/logseq-gardener-sdk/tests/site_readable.rs`, and remove the folder spelling case from `colliding_paths_fail_before_output_and_name_both_pages`

## 3. Browser scripts

- [x] 3.1 Write the spellings into the readable graph script, and make its `routeOf` and `isAt` use them

## 4. Docs and help

- [x] 4.1 Describe folders spelled in different letter case in the Readable URLs section of `docs/pages/lsg___Ref___Publish Options.md`
- [x] 4.2 Update the `--readable-urls` behavior text in `src/help.rs`, and record the review in `docs/pages/lsg___How To___Review Help Examples.md`

## 5. Verification

- [x] 5.1 Publish the encode garden with `--readable-urls` using this branch, and record the result in `verification.md`
- [x] 5.2 Run `mise run ci`
