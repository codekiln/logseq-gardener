## 1. Reproduce

- [x] 1.1 Add `page_property_rows_show_names_of_pages_with_no_file` to `crates/logseq-gardener-sdk/tests/site_properties.rs` from the issue's example page, and record its failure in `verification.md`

## 2. SDK

- [x] 2.1 Add `Publication::garden_names` in `crates/logseq-gardener-sdk/src/publication.rs`, filled with every garden page's title and aliases
- [x] 2.2 Render a reference to a name outside `garden_names` as an unlinked `page-reference unresolved no-brackets` span in `crates/logseq-gardener-sdk/src/site/properties.rs`, and pass the set from `site/mod.rs`

## 3. Docs

- [x] 3.1 Describe the shown names in the Rows section of `docs/pages/lsg___Ref___Published Pages.md`

## 4. Verification

- [x] 4.1 Publish a copy of the encode garden with page properties turned on, before and after, and record the row counts and the Knowledge/Garden box in `verification.md`
- [x] 4.2 Run `mise run ci`
