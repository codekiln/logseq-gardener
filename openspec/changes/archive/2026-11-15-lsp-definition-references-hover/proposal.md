## Why

`lsg lsp` starts, indexes a garden and can say what is under the cursor, and a person editing in Neovim still has no way to act on that answer. [Issue #640 — Follow links, block references and journal dates, and list references, in lsg lsp](https://github.com/codekiln/logseq-gardener/issues/640) asks for the navigation features of [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638): go to definition, find references and hover. In Logseq a link to a page without a file is how a page begins, so following such a link has to open the file Logseq would create, and the SDK has no function yet that turns a page title into a file name.

## What Changes

- Answer `textDocument/definition` for a page link, tag, alias, namespace page, journal date in any format Logseq accepts, block reference, page embed, block embed and page name in a property value such as `tags::` and `alias::`. A page leads to the start of its file, and a block reference leads to the first line of its block.
- Lead a page without a file to `pages/<file name>.md` and a journal without a file to `journals/<date in :journal/file-name-format>.md`, so the editor opens a new buffer at the path Logseq would write.
- Add the SDK module `file_names` with Logseq's title-to-file-name rules for both values of `:file/name-format`, ported from `file-name-sanity` in Logseq's `src/main/frontend/util/fs.cljs`, and a check for the titles whose file name reads back as another title.
- Answer `textDocument/references`: on a link, every range that names the same page by its title or one of its aliases, or the same block; in ordinary text, every range that names the page of the current document; on a property key, every line that writes the key. `includeDeclaration` adds the page's file and the names in its own `alias::` property, or the block's `id::` value.
- Answer `textDocument/hover` with Markdown: a page's title, aliases, reference count and first lines, or a block's text below its page title and parent blocks.
- Add SDK index queries for each of these, free of LSP types: `destinations`, `definition_at`, `new_page_path`, `page_references`, `occurrences_of_property`, `subject_at`, `references`, `page_summary`, `block_summary` and `summary_at`.
- Extend `mise run lsp:nvim-check` so headless Neovim runs `vim.lsp.buf.definition()` on fixture links and checks which file opens.
- Describe the three features in `lsg lsp help`, `lsg/How To/Edit a Garden in Neovim` and `lsg/Ref/Command Line`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-index`: gains the file name of a new page or journal, the place a target leads to, the list of places that name a page, block or property key, and page and block summaries.
- `language-server`: advertises and answers definition, references and hover, and the Neovim check follows a link.

## Impact

- New SDK module `crates/logseq-gardener-sdk/src/file_names.rs` and new files `definition.rs`, `references.rs` and `hover.rs` under `crates/logseq-gardener-sdk/src/index/`.
- New handlers `definition.rs`, `references.rs` and `hover.rs` under `crates/logseq-gardener-lsp/src/features/`, each registered in `dispatch.rs` and `capabilities.rs`.
- New end-to-end test file `tests/lsp_navigation.rs`. The fixture garden `tests/fixtures/lsp-garden` keeps its files.
- No new dependency. The SDK already uses `unicode-normalization`.
- [Issue #643 — Rename a page or tag across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/643) reuses `file_names::page_file_name` and `GardenIndex::page_references`.
- The `lsg/status` and `lsg/occurrence` requests keep their behavior.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/CLI/Centricity/Offline Tools](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___CLI___Centricity___Offline%20Tools.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don%27t%20Repeat%20Yourself%20DRY.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
