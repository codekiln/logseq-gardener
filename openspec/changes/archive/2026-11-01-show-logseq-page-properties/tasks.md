## 1. Reproduce

- [x] 1.1 Add `page_properties_show_every_key_logseq_shows` to `crates/logseq-gardener-sdk/tests/site_properties.rs` and record its failure in `verification.md`

## 2. SDK

- [x] 2.1 Add `crates/logseq-gardener-sdk/src/page_properties.rs` with `is_hidden_page_property` and unit tests
- [x] 2.2 Use it in the properties box in `crates/logseq-gardener-sdk/src/site/properties.rs`, and render `alias` and `aliases` values as links labelled with the written names
- [x] 2.3 Replace `logseq-proxy-source` with `hl-type` in `hidden_mixed_and_unsafe_values_supply_no_rows_or_property_backlinks`

## 3. Docs

- [x] 3.1 List the hidden keys and the alias links in `docs/pages/lsg___Ref___Published Pages.md`
- [x] 3.2 Correct section 4 of `docs/pages/lsg___Ref___Logseq Published Site___Frontmatter and Properties.md`

## 4. Verification

- [x] 4.1 Publish the docs graph with this branch, and record the `lsg` page's properties in `verification.md`
- [x] 4.2 Run `mise run ci`
