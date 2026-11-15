## Why

Neovim has no language server that follows Logseq's rules for page links, tags, block references and properties, so editing a garden there means typing every name from memory. [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) plans that server. This change is its starting step, [Issue #639 — Start lsg lsp over stdio with an in-memory garden index and a Neovim setup page](https://github.com/codekiln/logseq-gardener/issues/639): a server that starts, keeps an index of the garden current while files change, and attaches from a documented Neovim configuration. The later steps add definition, references, hover, completion, diagnostics and rename on top of this index.

## What Changes

- Add `lsg lsp`, which serves the Language Server Protocol (LSP) over standard input and output, with help text like every other `lsg` command.
- Add the workspace package `logseq-gardener-lsp`, which translates between LSP messages and SDK calls.
- Add an SDK garden index that loads every Markdown file beneath `pages/` and `journals/` and records, for each document, the source range and target of every page link, tag, block reference, embed, property key, page-valued property value and `id::` line.
- Answer "what is at this position" from the index, and resolve a page name or block id to the document and block that Logseq would open.
- Read `:file/name-format` from `logseq/config.edn` in `GardenSettings`.
- Find the garden root as the nearest directory holding `logseq/config.edn`, and answer `initialize` with an error the editor shows when there is none.
- Keep the text of an open editor buffer ahead of the file on disk, reparse a document on each change, and reread the file when the buffer closes.
- Pick up changes made outside the editor through the client's file watching when the client can register it, and through a watcher in the server otherwise. A change to `logseq/config.edn` reloads the settings and rebuilds the index.
- Exchange positions in UTF-8 when the client offers it and in UTF-16 otherwise.
- Add an end-to-end test harness that starts the built `lsg lsp` against a copy of a fixture garden and speaks JSON-RPC to it.
- Add the docs page `lsg/How To/Edit a Garden in Neovim` and the task `mise run lsp:nvim-check`, which runs the page's configuration in headless Neovim and checks that the client attaches. `mise run ci` runs the task with a pinned Neovim.

## Capabilities

### New Capabilities

- `garden-index`: the SDK's in-memory index of a garden's documents, the links and properties each one writes, and the queries that resolve them.
- `language-server`: the `lsg lsp` command, its startup, document synchronization, file watching, position encodings, status requests and documented Neovim setup.

### Modified Capabilities

- `project-workflow`: the workspace gains a language server package that the SDK and its Rust users build without.

## Impact

- New package `crates/logseq-gardener-lsp` and new SDK modules under `crates/logseq-gardener-sdk/src/index/`.
- New dependencies of the language server package: `lsp-server`, `lsp-types`, `crossbeam-channel` and `notify`, with their license texts added to the dependency notices.
- `GardenSettings` gains a public field, `filename_format`.
- `mise.toml` pins Neovim, and `mise run ci` takes a few seconds longer.
- `lsg help` lists a new command. Existing commands keep their behavior.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/CLI/Centricity/Offline Tools](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___CLI___Centricity___Offline%20Tools.md)
- [My/Principle/Simplify/Fewer and Deeper](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Fewer%20and%20Deeper.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
