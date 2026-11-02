## 1. Reproduce

- [x] 1.1 Run `mise run parity:logseq-export` on a test garden with the issue's `see-also` row and lower-cased variants, and record the rows only Logseq shows
- [x] 1.2 Add `site_properties::property_links_keep_the_name_as_written` and see it fail on `origin/main`

## 2. Fix in the SDK

- [x] 2.1 Label each page link in `value()` with the name as written, or with the page's title when the name is that title in lower case; verify the new test and the updated `book_and_preset_properties_keep_order_scalars_aliases_and_avoid_duplicate_sequences` pass
- [x] 2.2 Say in lsg/Ref/Published Pages that a property link keeps the spelling the author wrote

## 3. Verification

- [x] 3.1 Rerun the Logseq export comparison on the test garden, and publish the encode garden with page properties on with both builds; record the results in `verification.md`
- [x] 3.2 Run `mise run ci`
