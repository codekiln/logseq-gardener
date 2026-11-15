## Why

`lsg lsp` starts and keeps an index of the garden, and it offers no completion yet, so a person editing a garden in Neovim still types every page name, tag, block id and property key from memory. [Issue #641 — Complete page links, tags, block references and properties in lsg lsp](https://github.com/codekiln/logseq-gardener/issues/641) adds completion, the feature [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) lists for typing `[[`, `#`, `((` and `key::`.

## What Changes

- Add an SDK query that reads the text around the cursor and says what is being typed there: a page link, a tag, a block reference, a property key or a property value, with the text typed so far and the range a chosen item replaces.
- Add an SDK query that lists and ranks the candidates for that place from the garden index: page titles, aliases, namespace children, journal titles, blocks with an `id::`, property keys and property values.
- Answer `textDocument/completion` in `lsg lsp` with those candidates, and advertise the trigger characters `[`, `#`, `(` and `:`.
- Each item's text edit replaces the typed text and a closing `]]` or `))` that already follows the cursor, so an editor plugin that inserts the closing pair leaves one pair in the buffer.
- Return at most 100 items for one request and mark a longer list as incomplete, so the editor asks again as the person types.
- Document completion on the docs page `lsg/How To/Edit a Garden in Neovim`, in `lsg lsp help` and in the command-line reference.
- Record the completion response time measured on the encode garden.
- Fix shutdown: the server exits with status 0 when a response or notification arrives between `shutdown` and `exit`. Before this change it exited with status 1, which happened when an editor sent `shutdown` while its answer to the server's file watching request was still on the way.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-index`: gains the completion context and completion candidate queries.
- `language-server`: gains the `textDocument/completion` request and its advertised trigger characters, and a requirement for messages that arrive between `shutdown` and `exit`.

## Impact

- New SDK module `crates/logseq-gardener-sdk/src/index/completion.rs` and new server module `crates/logseq-gardener-lsp/src/features/completion.rs`.
- The `initialize` response advertises a completion provider.
- No new dependencies.
- New end-to-end test file `tests/lsp_completion.rs`.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify/Fewer and Deeper](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Fewer%20and%20Deeper.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
