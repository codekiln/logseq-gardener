## 1. SDK

- [x] 1.1 Add `property_names::logseq_property_key` and `is_built_in_property`, with unit tests against Logseq's lists
- [x] 1.2 Add `GardenIndex::rename_property` and the refusals `BuiltInKey`, `InvalidKey` and `KeyOnSameBlock`; remove `RenameRefusal::PropertyKey`
- [x] 1.3 Answer `rename_target_at` on a property key
- [x] 1.4 Unit tests: page and block properties across files, normalized matching, letter-case rename, a key in use on other blocks, each refusal

## 2. Language server

- [x] 2.1 Dispatch a property key in `features/rename.rs` to `rename_property`

## 3. End-to-end tests

- [x] 3.1 Add `tests/fixtures/lsp-rename-property-garden/before` and `after`
- [x] 3.2 `tests/lsp_rename_property.rs`: rename against the expected copy in both position encodings, prepare rename, refusals that leave the garden unchanged, a client without resource operations, an unsaved buffer

## 4. User-facing text

- [x] 4.1 Describe the property key rename in `lsg lsp help` and remove the Limitations sentence that says it arrives later, with a test
- [x] 4.2 Add the rename to `lsg/How To/Edit a Garden in Neovim` and `lsg/Ref/Command Line`
- [x] 4.3 Search the user-facing text for an MVP feature described as arriving later

## 5. Performance

- [x] 5.1 Add `scripts/lsp/perf.py` and `.mise/tasks/lsp/perf`
- [x] 5.2 Measure the encode garden on the final release build and record `lsg/Ref/Language Server/Performance`
- [x] 5.3 Link the page from `lsg/Ref/Command Line` and the Neovim how-to page

## 6. Verification

- [x] 6.1 `mise exec -- openspec validate lsp-rename-property-key --strict`
- [x] 6.2 `mise run ci`
