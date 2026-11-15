## 1. Title to file name

- [x] 1.1 Add the SDK module `file_names` with `page_file_name` for `:triple-lowbar` and the legacy format, ported from Logseq's `file-name-sanity`
- [x] 1.2 Add `page_file_path`, `journal_file_path` and `title_needs_property`
- [x] 1.3 Unit tests: `/`, `_`, `%`, a leading dot, Windows reserved names and reserved characters under both formats, and reading each name back with `page_titles::page_title`

## 2. SDK index queries

- [x] 2.1 Add `index/definition.rs` with `Destination`, `new_page_path`, `destinations` and `definition_at`
- [x] 2.2 Add `index/references.rs` with `page_references`, `occurrences_of_property`, `subject_at` and `references`
- [x] 2.3 Add `index/hover.rs` with `page_summary`, `block_summary`, `summary_at` and `Summary::markdown`
- [x] 2.4 Unit tests for destinations, new files under both formats, an ambiguous name, how each place names a page, references with and without the declaration, and page and block summaries

## 3. Language server

- [x] 3.1 Add `features/definition.rs`, `features/references.rs` and `features/hover.rs`, and register each in `dispatch.rs` and `features/mod.rs`
- [x] 3.2 Advertise the definition, references and hover providers in `capabilities.rs`

## 4. End-to-end tests

- [x] 4.1 Definition for each link kind, a page without a file, journal dates in other formats, the legacy format, block references and embeds in both position encodings
- [x] 4.2 References from a link, from ordinary text, with the declaration, for a block, for a property key and after an unsaved edit
- [x] 4.3 Hover on a block embed, a block reference, a page link, a page without a file and a block that is gone

## 5. Neovim check and user-facing text

- [x] 5.1 Extend `mise run lsp:nvim-check` to call `vim.lsp.buf.definition()` on a link to a page with a file and on a link to a page without one
- [x] 5.2 Describe definition, references and hover in `lsg lsp help`, with a test
- [x] 5.3 Add the keys and the behavior to `lsg/How To/Edit a Garden in Neovim` and the request table to `lsg/Ref/Command Line`

## 6. Verification

- [x] 6.1 `mise exec -- openspec validate lsp-definition-references-hover --strict`
- [x] 6.2 `mise run ci`
