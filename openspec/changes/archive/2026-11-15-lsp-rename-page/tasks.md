## 1. SDK rename plan

- [x] 1.1 Add `garden_config::default_home_page`, which finds the `:page` string of `:default-home` with its range, with a unit test
- [x] 1.2 Add `index/rename.rs` with `RenamePlan`, `DocumentChange`, `Replacement`, `FileMove`, `PageRename`, `RenameTarget` and `RenameRefusal`
- [x] 1.3 Add `GardenIndex::rename_target_at`
- [x] 1.4 Add `GardenIndex::rename_page` and `rename_page_with_config`: names by title, namespace children, nested titles, tags, property values, `title::`, file renames, letter-case renames through a temporary name, `:default-home`, refusals
- [x] 1.5 Unit tests for each of those cases

## 2. Language server

- [x] 2.1 Keep each open buffer's version in `server.rs` and `features/sync.rs`
- [x] 2.2 Add `features/rename.rs` with `prepare_rename`, `rename` and the reusable `workspace_edit`, and register it in `dispatch.rs` and `features/mod.rs`
- [x] 2.3 Advertise `renameProvider` with `prepareProvider` in `capabilities.rs`

## 3. End-to-end tests

- [x] 3.1 Add `tests/fixtures/lsp-rename-garden/before` and `after`
- [x] 3.2 `tests/lsp_rename.rs`: rename against the expected copy in both position encodings, with edit order and versions checked
- [x] 3.3 Prepare rename, refusals that leave the garden unchanged, a letter-case rename, a journal, an unsaved buffer and clients without the `rename` resource operation

## 4. Neovim check and user-facing text

- [x] 4.1 Read `vim.lsp.util.apply_workspace_edit` and `vim.lsp.util.rename` in Neovim 0.12.6 and try each order of edits and renames in headless Neovim
- [x] 4.2 Extend `mise run lsp:nvim-check` to rename a page and then change its letter case
- [x] 4.3 Describe rename in `lsg lsp help`, with a test
- [x] 4.4 Add the rename step to `lsg/How To/Edit a Garden in Neovim` and the rename section to `lsg/Ref/Command Line`

## 5. Verification

- [x] 5.1 Measure `textDocument/rename` on the encode garden with a release build, without applying the edit
- [x] 5.2 `mise exec -- openspec validate lsp-rename-page --strict`
- [x] 5.3 `mise run ci`
