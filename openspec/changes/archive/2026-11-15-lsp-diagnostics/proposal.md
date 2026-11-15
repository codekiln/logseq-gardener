## Why

A block reference whose block is gone, and a block id that two blocks write, both look fine in the Markdown and fail only when Logseq reads the garden. [Issue #642 — Report broken block references and duplicate block ids from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/642) asks `lsg lsp` to show these problems in the editor while the page is open, as one step of [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638). The index from [PR #645 — start lsg lsp](https://github.com/codekiln/logseq-gardener/pull/645) already keeps every block id and every reference with its source range, so the server can answer from memory.

## What Changes

- Add an SDK query, `GardenIndex::problems`, that lists the problems in one document, each with a source range, a severity, a message and the other places involved:
  - a `((uuid))` or `{{embed ((uuid))}}` whose id no block writes, as an error;
  - an `id::` that another block also writes, as an error on each block, with the other blocks as related locations;
  - a `{{embed [[page]]}}` whose page has no file, as a warning.
- A page link, a tag or a property value that names a page without a file is no problem. Logseq creates a page from such a link.
- `lsg lsp` publishes these problems with `textDocument/publishDiagnostics`: for a document when it opens and after each change to it, and for a document that is closed when its file changes on disk.
- After any change, `lsg lsp` publishes again for every document whose published problems changed, so a block id added, changed or removed in one file updates the open documents that refer to it.
- `lsg lsp` clears a document's diagnostics when the document closes and when its file is deleted.
- `lsg lsp` sends related locations as `relatedInformation` when the client supports it, and names them in the message otherwise.
- The help for `lsg lsp` and the docs pages `lsg/How To/Edit a Garden in Neovim` and `lsg/Ref/Command Line` describe the diagnostics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-index`: gains a query for the problems in one document.
- `language-server`: publishes diagnostics for open documents and for files that change on disk.

## Impact

- New SDK module `crates/logseq-gardener-sdk/src/index/problems.rs` and new server module `crates/logseq-gardener-lsp/src/features/diagnostics.rs`.
- The text synchronization and file watching handlers call the diagnostics module after they update the index.
- A client that attaches to `lsg lsp` starts to receive `textDocument/publishDiagnostics` notifications. Existing requests keep their behavior.
- No new dependency.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
