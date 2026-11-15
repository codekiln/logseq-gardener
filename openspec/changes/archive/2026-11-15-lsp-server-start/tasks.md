## 1. Settings and packages

- [x] 1.1 Add `filename_format` to `GardenSettings`, read from `:file/name-format`, with unit tests
- [x] 1.2 Add the `crates/logseq-gardener-lsp` workspace package with `lsp-server`, `lsp-types`, `crossbeam-channel` and `notify`, each with a comment in its manifest
- [x] 1.3 Add the package to `release-plz.toml` and `release-plz-post-1.toml`, and check `mise run release:check`

## 2. SDK garden index

- [x] 2.1 Add `index::LineIndex` for line starts and offset to line and column conversion
- [x] 2.2 Extract occurrences, property lines and block id sites from one parsed document
- [x] 2.3 Add `GardenIndex::load` with the progress callback and the hidden-path rules
- [x] 2.4 Add buffer, file and reload operations that replace one document's entry
- [x] 2.5 Add `occurrence_at`, `page_id`, `occurrences_of_page`, `block`, `block_id_sites`, `occurrences_of_block` and the property counts
- [x] 2.6 Unit tests: each occurrence kind, resolution, block id order, and equality with a fresh load after adding, editing, renaming and deleting documents

## 3. Language server

- [x] 3.1 Position encoding and URI mapping in `positions.rs`, with unit tests for UTF-8 and UTF-16
- [x] 3.2 Startup: garden root, `initialize` response and error, capabilities, loading with progress
- [x] 3.3 Dispatch tables and the text synchronization handlers
- [x] 3.4 File watching through the client and through `notify`, and the `config.edn` reload
- [x] 3.5 The `lsg/status` and `lsg/occurrence` requests
- [x] 3.6 Shutdown and exit statuses

## 4. Command line

- [x] 4.1 Add the `lsp` subcommand and its help document, and list it in `lsg help`
- [x] 4.2 Update `tests/cli.rs` and the Clilint assessment so `mise run clilint:check` passes

## 5. End-to-end tests

- [x] 5.1 Add the fixture garden `tests/fixtures/lsp-garden`
- [x] 5.2 Add the JSON-RPC client under `tests/lsp/` that starts the built `lsg lsp` on a copy of the fixture
- [x] 5.3 Cover `initialize` with progress, `didOpen`, `didChange`, `didClose`, a file changed on disk through both watching paths, a `config.edn` change, a folder that is not a garden, and `shutdown` then `exit`

## 6. Neovim setup

- [x] 6.1 Write `lsg/How To/Edit a Garden in Neovim` and link it from `lsg/Ref/Command Line`
- [x] 6.2 Pin Neovim in `mise.toml` and refresh `mise.lock`
- [x] 6.3 Add the `lsp:nvim-check` task and run it from `mise run ci`

## 7. Dependencies and release checks

- [x] 7.1 Record the new dependencies on `lsg/Ref/Dependencies` and refresh the notices with `mise run notices:update`
- [x] 7.2 Measure the load time of the encode garden on a release build and record it in the pull request
- [x] 7.3 Run `mise run ci`
