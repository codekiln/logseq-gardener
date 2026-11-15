## ADDED Requirements

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

The `initialize` response SHALL advertise incremental text document synchronization with open and close notifications, and the negotiated position encoding. It SHALL advertise no document formatting, range formatting or on-type formatting.

#### Scenario: Read the capabilities

- **WHEN** a client reads the `initialize` response
- **THEN** `textDocumentSync` asks for incremental changes and no formatting provider is present

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

The docs graph SHALL hold the page `lsg/How To/Edit a Garden in Neovim` with a Neovim configuration that starts `lsg lsp` for Markdown buffers inside a directory that holds `logseq/config.edn`, and a LazyVim variant. `lsg/Ref/Command Line` SHALL link to the page. `mise run lsp:nvim-check` SHALL take the configuration from the page, run it in `nvim --headless --clean` against a copy of the fixture garden, and fail unless the client attaches to a page buffer and answers `lsg/status`. `mise run ci` SHALL run the task, with Neovim pinned in `mise.toml`.

#### Scenario: Run the documented configuration

- **WHEN** a contributor runs `mise run lsp:nvim-check`
- **THEN** headless Neovim attaches a client named `logseq_gardener` to a fixture page and the task exits with status 0

#### Scenario: The page changes

- **WHEN** the configuration on the docs page stops working
- **THEN** `mise run ci` fails at the Neovim check
