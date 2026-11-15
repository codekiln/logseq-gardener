## ADDED Requirements

### Requirement: Prepare rename

The server SHALL advertise `renameProvider` with `prepareProvider` true. It SHALL answer `textDocument/prepareRename` from `GardenIndex::rename_target_at` with the range and, as `placeholder`, the current name. It SHALL answer null for a document the garden does not hold, and with an error that states the reason for a refusal.

#### Scenario: A link, an alias and ordinary text

- **WHEN** a client prepares a rename at `[[Fruit]]`, at `[[Produce]]`, an alias of `Fruit`, and inside ordinary words of the page `Fruit/Apple`
- **THEN** the answers are the range of `Fruit` with the placeholder `Fruit`, the range of `Produce` with the placeholder `Fruit`, and an empty range at the cursor with the placeholder `Fruit/Apple`

#### Scenario: A block reference

- **WHEN** a client prepares a rename at a `((uuid))`
- **THEN** the answer is an error

### Requirement: Rename a page

The server SHALL answer `textDocument/rename` for a page with one `WorkspaceEdit` built from `GardenIndex::rename_page`. In `documentChanges` each file's `TextDocumentEdit` SHALL come ahead of that file's `RenameFile` operations and SHALL name the file by its current URI. The edit of an open buffer SHALL carry the version the client last sent for it; other edits SHALL carry a null version. The text of an open buffer SHALL count as the text of its file, including `logseq/config.edn`. Applying the edit SHALL leave every file and all text outside the replaced ranges as they were, including tab indentation. The server SHALL write no file.

#### Scenario: A page with aliases, tags, property references and namespace children

- **WHEN** a client renames `Fruit` to `Sweet Produce` in a copy of `tests/fixtures/lsp-rename-garden/before`, with `utf-8` or `utf-16` positions, and applies the edit
- **THEN** every file of the copy equals the file of the same name in `tests/fixtures/lsp-rename-garden/after`

#### Scenario: An unsaved buffer

- **WHEN** a client opens a page, adds `[[fruit]]` in a change with version 5 and renames at that link
- **THEN** the edit for that page carries version 5 and replaces the range of `fruit` in the buffer's text

#### Scenario: A change of letter case only

- **WHEN** a client renames `Fruit` to `FRUIT` and applies the edit on a file system that ignores letter case
- **THEN** the folder lists `FRUIT.md`, `FRUIT___Apple.md` and `FRUIT___Apple___Cox.md`

### Requirement: Refuse a rename with an error

The server SHALL answer a refused rename with an error response of code `-32803` whose message states the reason, and with no edit.

#### Scenario: The new name belongs to another page

- **WHEN** a client renames `Fruit` to `fruitcake` in a garden that has the page `Fruitcake`
- **THEN** the answer is an error that names `pages/Fruitcake.md`, and the garden's files are unchanged

### Requirement: Resource operations of the client

The server SHALL answer a rename whose plan renames a file with an error that names `workspace.workspaceEdit.resourceOperations` when the client does not list `rename` there. It SHALL write `documentChanges` for a client that advertises `workspace.workspaceEdit.documentChanges` or lists any resource operation, and `changes` otherwise.

#### Scenario: A client that renames no files

- **WHEN** a client that lists no `rename` resource operation renames a page with a file, and then a name that no file answers to
- **THEN** the first answer is an error that names the capability, and the second is an edit with text edits only

#### Scenario: A client that lists resource operations only

- **WHEN** a client advertises `resourceOperations` with `rename` and no `documentChanges`, as Neovim does, and renames a page with a file
- **THEN** the answer holds `documentChanges` with text edits and file renames

### Requirement: Neovim renames a page

`mise run lsp:nvim-check` SHALL rename a page of its copy of the fixture garden with `vim.lsp.buf.rename`, to a new name and then to that name in another letter case, and SHALL fail unless the page files have the expected names and the links in other files hold the new name after `:wall`.

#### Scenario: The check renames a page

- **WHEN** a developer runs `mise run lsp:nvim-check`
- **THEN** the output reports the page files after each rename and the task exits with status 0
