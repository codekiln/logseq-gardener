## Purpose

Define the `lsg lsp` command: its startup and shutdown, document synchronization, file watching, position encodings, status requests, go to definition, find references, hover, diagnostics, completion, rename, documented Neovim setup and the task that measures its response times.

## Requirements

### Requirement: Serve over standard input and output

`lsg lsp` SHALL serve the Language Server Protocol over standard input and output until the client ends the session. It SHALL write nothing but protocol messages to standard output. After `shutdown` and then `exit` it SHALL exit with status 0. When the client closes the connection without `shutdown`, it SHALL exit with status 1. `lsg lsp help`, `lsg lsp --help` and `lsg lsp -h` SHALL print the command's documentation, and `lsg help` SHALL list `lsp` among its commands.

#### Scenario: A full session

- **WHEN** a client sends `initialize`, `initialized`, `shutdown` and `exit`
- **THEN** each request gets a response and the process exits with status 0

#### Scenario: Help

- **WHEN** a caller runs `lsg lsp help`
- **THEN** the output describes the command, its usage, its behavior, its permissions and an example, and the command starts no server

### Requirement: Find the garden root

The server SHALL take the client's first workspace folder, or its root URI when there is no workspace folder, and use the nearest directory at or above it that holds `logseq/config.edn` as the garden root. When there is none, it SHALL answer `initialize` with an error whose message names the folder and `logseq/config.edn`, and SHALL exit with status 1 when the client closes the connection.

#### Scenario: Started from a folder inside a garden

- **WHEN** the client's root URI is the garden's `pages/` folder
- **THEN** the server uses the folder that holds `logseq/config.edn`

#### Scenario: Started outside a garden

- **WHEN** no directory at or above the client's root holds `logseq/config.edn`
- **THEN** the `initialize` response is an error that names the folder, and the process exits with status 1

### Requirement: Load the garden with progress

The server SHALL answer `initialize` before it reads any page, and SHALL load the garden index when it receives `initialized`. When the client supports work done progress, the server SHALL create a progress token and report the start, the share of files read, and the end of loading with `$/progress`. A request that arrives while the garden loads SHALL get its answer after loading ends.

#### Scenario: Progress on start

- **WHEN** a client that supports work done progress starts the server on the fixture garden
- **THEN** it receives a `window/workDoneProgress/create` request, then `$/progress` notifications that begin and end

#### Scenario: A request during loading

- **WHEN** a client sends `lsg/status` right after `initialized`
- **THEN** the response reports the loaded garden's document count

### Requirement: Advertised capabilities

The `initialize` response SHALL advertise incremental text document synchronization with open and close notifications, the negotiated position encoding, and the definition, references and hover providers. It SHALL advertise no document formatting, range formatting or on-type formatting.

#### Scenario: Read the capabilities

- **WHEN** a client reads the `initialize` response
- **THEN** `textDocumentSync` asks for incremental changes, `definitionProvider`, `referencesProvider` and `hoverProvider` are true, and no formatting provider is present

### Requirement: Position encoding

The server SHALL use `utf-8` positions when the client lists `utf-8` among its position encodings, and `utf-16` positions otherwise. Every position it reads or writes SHALL use the negotiated encoding.

#### Scenario: A client that offers UTF-8

- **WHEN** the client lists `utf-8` in `general.positionEncodings`
- **THEN** the response's `positionEncoding` is `utf-8`

#### Scenario: A line with non-ASCII text under UTF-16

- **WHEN** a client that offers only UTF-16 asks what is at the position of a link that follows `é` and an emoji on its line
- **THEN** the server finds the link, and the range it returns counts the emoji as two units

### Requirement: Open buffers ahead of disk

For a Markdown document beneath the garden's `pages/` or `journals/`, the server SHALL index the text the client sends in `textDocument/didOpen`, apply each `textDocument/didChange` to that text and index the result, and on `textDocument/didClose` index the file on disk again. A document outside those folders SHALL change nothing.

#### Scenario: An unsaved edit

- **WHEN** a client opens a page and changes a link to name another page, without saving
- **THEN** `lsg/occurrence` at that position reports the new name

#### Scenario: Close without saving

- **WHEN** the client then closes the document
- **THEN** `lsg/occurrence` at that position reports the name the file holds

### Requirement: Changes made outside the editor

When the client supports dynamic registration of `workspace/didChangeWatchedFiles`, the server SHALL register watchers for `pages/**/*.md`, `journals/**/*.md` and `logseq/config.edn` beneath the garden root, and SHALL index each path the client reports. Otherwise the server SHALL watch those folders itself, and SHALL notice a change within five seconds. A created or changed Markdown file SHALL be indexed from disk unless a buffer is open for it, and a deleted one SHALL leave the index. A change to `logseq/config.edn` SHALL reload the settings and rebuild the index.

#### Scenario: The client reports a new file

- **WHEN** a file appears beneath `pages/` and the client sends `workspace/didChangeWatchedFiles` for it
- **THEN** `lsg/status` reports one more document

#### Scenario: The server watches by itself

- **WHEN** the client does not support dynamic registration and a file beneath `pages/` changes on disk
- **THEN** the index reflects the change within five seconds

#### Scenario: The filename format changes

- **WHEN** `logseq/config.edn` changes from `:file/name-format :triple-lowbar` to no such key
- **THEN** `lsg/status` reports the legacy format, and page names resolve by the legacy rules

### Requirement: Status requests

The server SHALL answer `lsg/status` with the garden root, the filename format, the journal title and file name formats, the position encoding, whether the client or the server watches files, the number of documents, pages, journals and open buffers, and the time the last load took in milliseconds. The server SHALL answer `lsg/occurrence`, given a text document and a position, with null when nothing is there, and otherwise with the occurrence's kind, its range, its name range, its target, and what the target resolves to: a document's URI and title, a block's document URI and range, or a statement that the page or block has no file or is ambiguous.

#### Scenario: Ask what a link points at

- **WHEN** a client sends `lsg/occurrence` for a position inside `[[Malus]]`, an alias of the page `Fruit/Apple`
- **THEN** the result's kind is a page link and its resolution names the URI of `pages/Fruit___Apple.md` and the title `Fruit/Apple`

### Requirement: Documented Neovim setup

The docs graph SHALL hold the page `lsg/How To/Edit a Garden in Neovim` with a Neovim configuration that starts `lsg lsp` for Markdown buffers inside a directory that holds `logseq/config.edn`, a LazyVim variant, and the keys that follow a link, list references and show a hover. `lsg/Ref/Command Line` SHALL link to the page. `mise run lsp:nvim-check` SHALL take the configuration from the page, run it in `nvim --headless --clean` against a copy of the fixture garden, and fail unless the client attaches to a page buffer, answers `lsg/status`, and `vim.lsp.buf.definition()` opens the file a fixture link leads to, for a page with a file and for a page without one. `mise run ci` SHALL run the task, with Neovim pinned in `mise.toml`.

#### Scenario: Run the documented configuration

- **WHEN** a contributor runs `mise run lsp:nvim-check`
- **THEN** headless Neovim attaches a client named `logseq_gardener` to a fixture page and the task exits with status 0

#### Scenario: The page changes

- **WHEN** the configuration on the docs page stops working
- **THEN** `mise run ci` fails at the Neovim check

#### Scenario: Follow a link in Neovim

- **WHEN** the task calls `vim.lsp.buf.definition()` with the cursor on `[[Malus]]` and then on `[[Fruit/Quince]]` in `pages/Recipes.md`
- **THEN** Neovim's current buffer becomes `pages/Fruit___Apple.md` and then `pages/Fruit___Quince.md`, and the copy of the garden holds no `pages/Fruit___Quince.md`

### Requirement: Go to definition

The server SHALL answer `textDocument/definition` from `GardenIndex::definition_at`. On a page link, tag, page embed or page name in a property value it SHALL return the location of the page's file at line 0, character 0, following aliases, namespace names and journal dates in any format Logseq accepts. On a block reference, block embed or `id::` value it SHALL return the location of the block's first line. For a page or journal without a file it SHALL return a location in the file Logseq would create, at line 0, character 0, and SHALL create no file. For a name several pages answer to it SHALL return one location per page. It SHALL return null in ordinary text and an empty list for a block id that no block writes and for a property key.

#### Scenario: Each kind of page link

- **WHEN** a client asks for the definition of `[[Fruit/Apple]]`, `[[fruit/apple]]`, the alias `[[Malus]]`, `#recipes`, `#[[multi word]]`, `[the pear page]([[Fruit/Pear]])`, `{{embed [[Fruit/Pear]]}}`, the bare name `Malus` in `alias::` and `[[Fruit]]` in `tags::` of the fixture garden
- **THEN** each answer is one location at the start of the file of the page the name leads to

#### Scenario: A journal date in another format

- **WHEN** the garden titles journals `MMM do, yyyy` and a client asks for the definition of `[[2026-01-03]]`
- **THEN** the answer is the start of `journals/2026_01_03.md`

#### Scenario: A page without a file

- **WHEN** a client asks for the definition of `[[Fruit/Quince]]`, which has no file, in a garden that uses `:triple-lowbar`
- **THEN** the answer is line 0, character 0 of `pages/Fruit___Quince.md`, and that file does not exist afterwards

#### Scenario: The legacy format and a journal file format of the garden's own

- **WHEN** `logseq/config.edn` holds only `:journal/file-name-format "yyyy-MM-dd"` and a client asks for the definitions of `[[Fruit/Quince]]`, `[[What is 50%?]]` and `[[Jan 4th, 2026]]`
- **THEN** the answers name `pages/Fruit%2FQuince.md`, `pages/What is 50%25%3F.md` and `journals/2026-01-04.md`

#### Scenario: A block reference and a block embed

- **WHEN** a client asks for the definition of `((66f1a2b0-0000-4000-8000-000000000004))` or of `{{embed ((66f1a2b0-0000-4000-8000-000000000002))}}`, with `utf-8` or `utf-16` positions
- **THEN** the answer is the range of the first line of the block that writes the id, from its dash to the end of the line

#### Scenario: Ordinary text

- **WHEN** a client asks for the definition at a position inside ordinary words
- **THEN** the answer is null

### Requirement: Find references

The server SHALL answer `textDocument/references` from `GardenIndex::subject_at` and `GardenIndex::references`, passing the request's `includeDeclaration`. On a link, tag, embed or page name in a property value the locations SHALL be every range that names the same page by its title or one of its aliases. On a block reference, block embed or `id::` value they SHALL be every block reference and block embed of the block. In ordinary text they SHALL be every range that names the page of the document. On a property key they SHALL be every line that writes the key. The text of an open buffer SHALL count as the text of its file.

#### Scenario: From a link that writes an alias

- **WHEN** a client asks for the references of `[[Malus]]`, an alias of `Fruit/Apple`, without the declaration
- **THEN** the locations are the ranges of `[[fruit/apple]]`, both `[[Fruit/Apple]]` links and `[[Malus]]`

#### Scenario: With the declaration

- **WHEN** the same request sets `includeDeclaration`
- **THEN** the list starts with the start of `pages/Fruit___Apple.md` and the names `Malus` and `[[Pomme]]` of its `alias::` property

#### Scenario: From ordinary text

- **WHEN** a client asks for references at a position inside the words of a block of `pages/Fruit___Pear.md`
- **THEN** the locations are the ranges that name `Fruit/Pear` or its alias `Pyrus`, including the labelled link and the page embed in `pages/Recipes.md`

#### Scenario: A block

- **WHEN** a client asks for the references of an `id::` value with the declaration
- **THEN** the list starts with the range of that value and continues with the block embed that names the id

#### Scenario: An unsaved edit

- **WHEN** an open buffer gains a link `[[Pomme]]` and a client asks for the references of `Fruit/Apple`
- **THEN** the list holds one more location than before the edit

### Requirement: Hover

The server SHALL answer `textDocument/hover` from `GardenIndex::summary_at` with Markdown content and the range of the occurrence under the pointer. It SHALL answer null in ordinary text and on a property key.

#### Scenario: A block embed

- **WHEN** a client hovers `{{embed ((66f1a2b0-0000-4000-8000-000000000002))}}` in the fixture garden
- **THEN** the content kind is `markdown`, the value starts with `**Fruit/Apple** › Keeps for months in a cool cellar.`, continues with the block's text `Store apart from [[Fruit/Pear]].` and ends with `1 reference`, and the range covers the whole embed

#### Scenario: A page link

- **WHEN** a client hovers `[[Malus]]`
- **THEN** the value shows `**Fruit/Apple**`, `Aliases: Malus, Pomme`, `4 references` and the page's first lines

#### Scenario: A page without a file

- **WHEN** a client hovers `[[Fruit/Quince]]`
- **THEN** the value names `pages/Fruit___Quince.md` as the file go to definition opens

#### Scenario: A block that is gone

- **WHEN** a client hovers a block reference whose id no block writes
- **THEN** the value is `No block has this id.`

### Requirement: Publish diagnostics for open documents

The server SHALL send `textDocument/publishDiagnostics` for a document of the garden when the client opens it, with one diagnostic for each problem the index lists for that document, or an empty list when there are none. After each change to an open document, the server SHALL send the document's diagnostics again when they differ from the list it last sent. Each diagnostic SHALL carry the problem's range in the negotiated position encoding, severity 1 for an error and 2 for a warning, the source `lsg`, a code that names the kind of problem, and the problem's message.

#### Scenario: Open a page with a broken block reference

- **WHEN** a client opens `pages/Recipes.md` of the fixture garden
- **THEN** it receives diagnostics for that document with one error whose range is `((66f1a2b0-0000-4000-8000-00000000dead))` and whose code is `missing-block`

#### Scenario: Fix the reference in the buffer

- **WHEN** the client then changes the reference to the id of a block that exists
- **THEN** it receives an empty list of diagnostics for that document

#### Scenario: Open a page whose links name pages without a file

- **WHEN** a client opens a page that holds only links, tags and property values naming pages without a file
- **THEN** it receives an empty list of diagnostics for that document

#### Scenario: A document opened while the garden loads

- **WHEN** a client opens a document right after `initialized`
- **THEN** it receives that document's diagnostics after loading ends

### Requirement: Keep published diagnostics current

After every change to the index, the server SHALL compute again the problems of each open document and of each document it has published a non-empty list for, and SHALL send the new list for each document whose list changed. A change to the index is an edit to an open document, a document closing, a file that changed on disk, or a reload of the garden.

#### Scenario: A fix made in another file

- **WHEN** an open page shows an error for a block id that no block writes, and the client edits another open document to add a block with that id
- **THEN** the client receives an empty list of diagnostics for the first page

#### Scenario: A block id removed in another file

- **WHEN** an open page refers to a block of another document, and that document's buffer drops the block's `id::`
- **THEN** the client receives an error for the reference in the first page

#### Scenario: One of two blocks drops a shared id

- **WHEN** two open documents each show an error for the id they share, and the client changes the id in one of them
- **THEN** the client receives an empty list of diagnostics for both documents

### Requirement: Diagnostics for files changed outside the editor

When a file that is not open changes on disk, the server SHALL send that document's diagnostics when the list is non-empty or differs from the list it last sent for the document. When a file the server has published diagnostics for is deleted, the server SHALL send an empty list for it.

#### Scenario: Another program writes a broken reference

- **WHEN** a program outside the editor writes a block reference to a missing block into a page that is not open, and the server learns of the change
- **THEN** the client receives an error for that page

#### Scenario: The file is deleted

- **WHEN** that page's file is then deleted
- **THEN** the client receives an empty list of diagnostics for it

### Requirement: Clear diagnostics on close

When the client closes a document the server has published a non-empty list for, the server SHALL send an empty list for it. The server SHALL publish for that document again when the client opens it again or its file changes on disk.

#### Scenario: Close a page with an error

- **WHEN** the client closes a page that shows an error
- **THEN** it receives an empty list of diagnostics for that page

### Requirement: Related locations

When the client's capabilities set `textDocument.publishDiagnostics.relatedInformation`, a diagnostic for a problem with related locations SHALL list each one in `relatedInformation` with its URI and range. Otherwise the diagnostic SHALL carry no `relatedInformation`, and its message SHALL name each related location by its path in the garden and its line number.

#### Scenario: A client that supports related information

- **WHEN** such a client opens a document whose `id::` another document also writes
- **THEN** the error's `relatedInformation` holds the other document's URI and the range of its `id::` value

#### Scenario: A client without related information

- **WHEN** a client that does not set the capability opens the same document
- **THEN** the error has no `relatedInformation` and its message holds the other document's path and line number

### Requirement: Advertise completion

The `initialize` response SHALL advertise a completion provider with the trigger characters `[`, `#`, `(` and `:`.

#### Scenario: Read the capabilities

- **WHEN** a client reads the `initialize` response
- **THEN** `completionProvider.triggerCharacters` is `[`, `#`, `(` and `:`

### Requirement: Answer completion requests

For `textDocument/completion` the server SHALL read the open buffer's current text, or the indexed file's text when the document is not open, find the completion context at the position, and answer with the index's candidates for it. It SHALL answer with an empty list where there is no context and for a document it has no text for.

Each item SHALL carry:

- a `label`: the page name, alias, block text, property key or property value;
- a `textEdit` whose range is the context's replaced range in the negotiated position encoding and whose text is the candidate's inserted text;
- a `filterText` that the typed text matches: the name for a page, alias, key or value, the block's text for a block, and the typed text itself for a journal offered because the typed text spells its date;
- a `sortText` that orders the items by rank;
- a `kind`, and a description naming the page for an alias and for a block.

The server SHALL return at most 100 items for one request and SHALL set `isIncomplete` when the index reports a partial list.

#### Scenario: A page link with an auto-inserted closing pair

- **WHEN** a buffer holds `- [[Fru]]` with the cursor after `Fru` and the client asks for completion
- **THEN** the item `Fruit/Apple` has a text edit that replaces `Fru]]` with `Fruit/Apple]]`

#### Scenario: Each trigger

- **WHEN** a client asks for completion after `[[`, `#`, `((`, at a property key and after `key::`
- **THEN** the answers hold page names, page names, blocks, property keys and property values

#### Scenario: An alias item

- **WHEN** a client asks for completion after `[[Pom`
- **THEN** the item `Pomme` names `Fruit/Apple` in its description and inserts `Pomme]]`

#### Scenario: A buffer the garden has no file for

- **WHEN** a client opens a new, empty buffer under `pages/`, types `[[Fru` and asks for completion
- **THEN** the answer holds `Fruit/Apple`

#### Scenario: Prose

- **WHEN** a client asks for completion at the end of `- An apple is red`
- **THEN** the answer holds no items

### Requirement: Shut down while other messages arrive

After the server answers `shutdown`, it SHALL wait for `exit` and then exit with status 0. Until `exit` arrives it SHALL ignore responses and notifications, such as the client's answer to the server's `client/registerCapability` request, and SHALL answer a request with the `InvalidRequest` error. When the client closes the connection after `shutdown` without `exit`, the server SHALL exit with status 1.

#### Scenario: Shutdown right after initialized

- **WHEN** a client that watches files sends `initialized`, then `shutdown`, then its answer to the server's `client/registerCapability` request, then `exit`
- **THEN** the server exits with status 0

#### Scenario: A request after shutdown

- **WHEN** a client sends `shutdown` and then `lsg/status`
- **THEN** the server answers `lsg/status` with the `InvalidRequest` error, and exits with status 0 after `exit`

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
