## ADDED Requirements

### Requirement: Rename a property key

The server SHALL answer `textDocument/rename` on a property key with one `WorkspaceEdit` built from `GardenIndex::rename_property`. The edit SHALL hold text edits alone, so it SHALL be answered for a client that lists no `rename` resource operation. The edit of an open buffer SHALL carry the version the client last sent for it; other edits SHALL carry a null version. The text of an open buffer SHALL count as the text of its file. Applying the edit SHALL leave every value and all text outside the replaced keys as they were, including tab indentation. The server SHALL answer a refused rename with an error response of code `-32803` whose message states the reason, and with no edit.

#### Scenario: A key in page properties and block properties across files

- **WHEN** a client renames `type` to `kind` at a `type::` line in a copy of `tests/fixtures/lsp-rename-property-garden/before`, with `utf-8` or `utf-16` positions, and applies the edit
- **THEN** every file of the copy equals the file of the same name in `tests/fixtures/lsp-rename-property-garden/after`

#### Scenario: A built-in key

- **WHEN** a client renames at a `collapsed::` line, or renames `type` to `tags`
- **THEN** each answer is an error that names the built-in key, and the garden's files are unchanged

#### Scenario: A client that renames no files

- **WHEN** a client that advertises an empty `workspace.workspaceEdit` renames `type` to `kind`
- **THEN** the answer holds the text edits as `changes`

#### Scenario: An unsaved buffer

- **WHEN** a client opens a page, adds a `type::` line in a change with version 5 and renames `type` to `kind`
- **THEN** the edit for that page carries version 5 and replaces the key of the added line

### Requirement: Measure the server on a garden

`mise run lsp:perf <garden>` SHALL build the release binary, start `lsg lsp` on the garden and report: the time to load the garden; the median and 95th percentile response times of completion, definition, hover and references; the time from a `textDocument/didChange` to the answer to the next request; and the time, text edit count and file rename count of renaming the most-linked pages and the most-written property key. It SHALL state for each of the targets of [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) whether the measurement meets it. It SHALL change no file of the garden. With `--record` it SHALL write the results to the docs page `lsg/Ref/Language Server/Performance`. `mise run ci` SHALL NOT run the task.

#### Scenario: Measure the fixture garden

- **WHEN** a developer runs `mise run lsp:perf tests/fixtures/lsp-garden`
- **THEN** the output lists each measurement with its target, the task exits with status 0, and `git status` shows no change in the fixture

## MODIFIED Requirements

### Requirement: Prepare rename

The server SHALL advertise `renameProvider` with `prepareProvider` true. It SHALL answer `textDocument/prepareRename` from `GardenIndex::rename_target_at` with the range and, as `placeholder`, the current name. It SHALL answer null for a document the garden does not hold, and with an error that states the reason for a refusal.

#### Scenario: A link, an alias and ordinary text

- **WHEN** a client prepares a rename at `[[Fruit]]`, at `[[Produce]]`, an alias of `Fruit`, and inside ordinary words of the page `Fruit/Apple`
- **THEN** the answers are the range of `Fruit` with the placeholder `Fruit`, the range of `Produce` with the placeholder `Fruit`, and an empty range at the cursor with the placeholder `Fruit/Apple`

#### Scenario: A block reference

- **WHEN** a client prepares a rename at a `((uuid))`
- **THEN** the answer is an error

#### Scenario: A property key

- **WHEN** a client prepares a rename at the key of a line `Type:: book`, and at the key of a line `tags:: fruit`
- **THEN** the first answer is the range of `Type` with the placeholder `Type`, and the second is an error that names `tags`
