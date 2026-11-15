## Why

`lsg lsp` follows a link to its page and lists every place that names a page, and a person editing in Neovim still has to rename a page by hand: the file, each `[[link]]`, each `#tag`, each `tags::` value and each namespace child. [Issue #643 — Rename a page or tag across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/643) asks for the rename feature of [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638), so that `:lua vim.lsp.buf.rename()` changes the garden the way a rename in Logseq does.

## What Changes

- Answer `textDocument/prepareRename`. On a page link, tag, page embed, page name in a property value or the value of the page's own `title::` property, the answer is the range of the name and the current title of the page it leads to. Anywhere else in a page or journal, the answer is the cursor position and the title of the current page.
- Answer `textDocument/rename` for a page with one `WorkspaceEdit` that:
  - rewrites every `[[old]]`, `#old`, `#[[old]]`, `{{embed [[old]]}}` and page name in a property value that names the page by its title, in any letter case and Unicode normalization form, with the new name as typed;
  - keeps every link written with one of the page's aliases, or with a journal date in another format, as written;
  - renames namespace children, so `old/x` becomes `new/x`, in titles, references and file names;
  - renames each affected page file with a `RenameFile` operation to the name Logseq's title-to-file-name rules give, in the folder the file is in;
  - keeps a journal's file name and writes the new title to its `title::` property;
  - rewrites an existing `title::` property, and adds one when the file name cannot carry the new title;
  - rewrites the `:page` of `:default-home` in `logseq/config.edn` when it names a renamed page.
- Accept a rename that changes only letter case. The file reaches its new name through a temporary name, which works on a file system that ignores letter case.
- Refuse a rename with an error that states the reason: the new name is empty, belongs to another page, or spells a journal date; several files answer to the old name; a file already exists at a new path.
- Return an error that names the missing capability when the edit renames a file and the client does not advertise `workspace.workspaceEdit.resourceOperations` with `rename`.
- Add the SDK index query `GardenIndex::rename_page`, which returns a `RenamePlan` free of LSP types: text replacements per file with source ranges, and file renames in the order to apply them. Add `GardenIndex::rename_target_at`, which says what a rename at an offset is about.
- Track the version of each open buffer in the server, so text edits to an open buffer carry the version they were computed for.
- Describe rename in `lsg lsp help`, `lsg/How To/Edit a Garden in Neovim` and `lsg/Ref/Command Line`, and extend `mise run lsp:nvim-check` to rename a page in headless Neovim.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-index`: gains the rename plan for a page and the query that says what a rename at an offset is about.
- `language-server`: advertises and answers prepare rename and rename, and the Neovim check renames a page.

## Impact

- New file `crates/logseq-gardener-sdk/src/index/rename.rs`. `garden_config.rs` gains a function that finds the `:page` string of `:default-home` with its source range.
- New handler `crates/logseq-gardener-lsp/src/features/rename.rs`, registered in `dispatch.rs`, `features/mod.rs` and `capabilities.rs`. `server.rs` and `features/sync.rs` keep each open buffer's version.
- New end-to-end test file `tests/lsp_rename.rs` and new fixture `tests/fixtures/lsp-rename-garden/` with a `before/` and an `after/` garden. The fixture `tests/fixtures/lsp-garden` keeps its files.
- No new dependency.
- [Issue #644 — Rename a property key across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/644) reuses `rename_target_at`, `RenamePlan` and the handler's conversion of a plan to a `WorkspaceEdit`.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don%27t%20Repeat%20Yourself%20DRY.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
