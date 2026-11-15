## Why

`lsg lsp` renames a page across a garden and answers a rename on a property key with an error. A person who wants `type::` to become `kind::` in every page still edits each line by hand. [Issue #644 — Rename a property key across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/644) asks for that rename, which is the last feature of the MVP in [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638). The epic also asks the change that finishes the MVP to record how fast the server is on the encode garden, so later changes can compare.

## What Changes

- Answer `textDocument/prepareRename` on a property key with the range of the key and the key as written.
- Answer `textDocument/rename` on a property key with one `WorkspaceEdit` that replaces the key on every `key::` line of every page and journal, in a page's leading properties and in block properties, with the new key as typed. Values and all other text stay as written.
- Match keys as Logseq stores them: letter case is ignored and `_` counts as `-`, so a rename of `Due_Date` also rewrites `due-date::` and `DUE_DATE::`.
- Refuse a rename, with an error that states the reason and no edit, when:
  - the old or the new key has a built-in meaning in Logseq, such as `id`, `title`, `alias`, `tags`, `collapsed`, the card scheduling keys and every key that starts with `logseq.`;
  - the new key is empty, holds a space or a colon, or is a name Logseq's parser drops;
  - a block writes both the old and the new key, since Logseq reads one value for a key on a block.
- Accept a new key that other blocks already write. Both keys become one key.
- Keep page renames and property key renames apart: renaming a page leaves a property key of the same name as written, and renaming a key leaves the page of that name and the links to it as written.
- Add the SDK query `GardenIndex::rename_property`, which returns the same `RenamePlan` a page rename returns, and `property_names::is_built_in_property`.
- Replace the texts that say this rename arrives later in `lsg lsp help`, `lsg/How To/Edit a Garden in Neovim` and `lsg/Ref/Command Line`.
- Add `mise run lsp:perf <garden>`, which starts a release build of `lsg lsp` on a garden, measures loading, completion, definition, hover, references, reindexing after an edit and renames, and changes no file of the garden. Record the results for the encode garden on a new docs page, `lsg/Ref/Language Server/Performance`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `garden-index`: gains the rename plan for a property key and the list of keys with a built-in meaning; a rename at a property key now has a subject.
- `language-server`: prepare rename and rename answer on a property key; a measuring task and a page of results exist.

## Impact

- `crates/logseq-gardener-sdk/src/index/rename.rs` gains `rename_property` and three refusals, and loses `RenameRefusal::PropertyKey`. `crates/logseq-gardener-sdk/src/property_names.rs` gains `is_built_in_property` and `logseq_property_key`.
- `crates/logseq-gardener-lsp/src/features/rename.rs` dispatches a property key to the new query. The conversion of a plan to a `WorkspaceEdit` stays as it is.
- New end-to-end test file `tests/lsp_rename_property.rs` and fixture `tests/fixtures/lsp-rename-property-garden/` with a `before/` and an `after/` garden.
- New `scripts/lsp/perf.py` and `.mise/tasks/lsp/perf`. The task stays out of `mise run ci`, which has no encode garden.
- New docs page `logseq-gardener-docs/pages/lsg___Ref___Language Server___Performance.md`.
- No new dependency.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify/Don't Repeat Yourself DRY](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Don%27t%20Repeat%20Yourself%20DRY.md)
- [My/Principle/Simplify/Prefer Standards and Defaults](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify___Prefer%20Standards%20and%20Defaults.md)
- [My/Principle/Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
