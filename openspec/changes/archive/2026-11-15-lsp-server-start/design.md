## Context

`lsg` has no language server. The SDK already loads a garden, derives page titles, resolves names and aliases, reads journal dates and finds block ids, and `lsg lookup` and `lsg publish` use that code. [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) asks for a server that reuses it, and [Issue #639 — Start lsg lsp over stdio with an in-memory garden index and a Neovim setup page](https://github.com/codekiln/logseq-gardener/issues/639) is the step that every later feature builds on.

The parser, lsdoc, reports the source range of every inline node and block as offsets into the UTF-8 text. No SDK code keeps those ranges today: the publisher needs page identities, and an editor needs positions.

The encode garden, the garden this work is measured on, holds about 9,300 pages and 525 journals. Its owner edits it in Neovim 0.12 on macOS, with Marksman turned off inside any directory that holds `logseq/config.edn`.

## Goals / Non-Goals

**Goals:**

- `lsg lsp` starts from an editor, finds the garden, loads it and stays current while buffers and files change.
- The SDK index records where every link, tag, block reference, embed, property and `id::` is written, and what each one points at, so definition, references, hover, completion, diagnostics and rename each start from one query.
- Several people can add a feature to the server at the same time, each in a new module.
- A test harness and a fixture garden that later steps extend.
- A Neovim configuration that the docs publish and CI runs.

**Non-Goals:**

- Definition, references, hover, completion, diagnostics and rename. Each has its own sub-issue under Issue #638.
- The file name Logseq gives a new page. [Issue #640 — Follow links, block references and journal dates, and list references, in lsg lsp](https://github.com/codekiln/logseq-gardener/issues/640) adds it with go to definition.
- Indexing the Markdown files Logseq reads outside `pages/` and `journals/`.
- Logseq's respelling of a page title when the page links to itself in another letter case.
- A cache of the index on disk.

## Decisions

### Packages

The protocol code lives in a new workspace package, `crates/logseq-gardener-lsp`, with `publish = false` and no release artifact of its own. Its public entry point is `run_stdio`, which the `lsp` subcommand of `lsg` calls. Every garden query lives in the SDK module `index`, which uses no LSP type. The server package turns an LSP position into a document path and a UTF-8 offset, calls the SDK, and turns the answer back into LSP ranges.

### Protocol library

The server uses `lsp-server` for message framing and `lsp-types` for the message types, the crates rust-analyzer uses. Both are synchronous, and `lsg` has no async runtime. The main loop waits on the client's messages and on file events with `crossbeam-channel`, the channel type `lsp-server` exposes.

### What the index holds

`GardenIndex` holds one `IndexedDocument` per Markdown file beneath `pages/` and `journals/`, keyed by the path relative to the garden root. A document holds:

- its `GardenDocument` (path, kind, source and lsdoc parse);
- a `LineIndex` of line starts, for converting an offset to a line and column;
- its title, its journal date when it is a journal, and its alias keys;
- its `Occurrence` list, sorted by position;
- its `PropertyLine` list: each `key:: value` line with the ranges of the key and the value and the block the line belongs to;
- its `BlockIdSite` list: each block that writes an `id::`, with the id, the range of the value and the start of the block.

An `Occurrence` has a kind (page link, tag, block reference, page embed, block embed, property key, property value name or block id), the range of the whole syntax such as `[[Page]]`, the range of the name inside it such as `Page`, and a target: a page name as written, a block id, or a property key. A rename later replaces the name range and leaves the brackets alone.

Garden-wide, the index holds the settings read from `logseq/config.edn`, a `PageNameIndex` for resolving names, a `NamespaceIndex`, the documents that write each page name and each block id, every `id::` site in the order Logseq reads files, and counts of property keys and values.

### Resolving a target at query time

An occurrence stores the name its author wrote. `GardenIndex::page_id` resolves a name to a `PageId` when a query asks: a document, a journal date with no file, a name with no file, or an ambiguous name. Storing the written name keeps one document's entry independent of every other document, so adding an alias to one page changes that page's entry and the name index only. `GardenIndex::occurrences_of_page` and `occurrences_of_block` return every occurrence that leads to a page or a block, with its document, which is what find references and rename need.

### Keeping the index equal to a fresh load

Every change goes through one operation: replace the entry for a path with the result of parsing its current text, or remove the entry when the file is gone or holds only whitespace. The operation subtracts the old entry from the garden-wide maps and adds the new one. It rebuilds the `PageNameIndex` only when the document's title, journal date, aliases or emptiness changed, because that index has no removal. Unit tests add, edit, rename and delete documents and compare the whole index with one loaded from the same files.

### Buffers ahead of disk

The index records which paths have an open editor buffer. While a buffer is open, a file event for its path changes nothing. `didChange` applies the edits to the server's copy of the buffer text and replaces the entry. `didClose` drops the buffer and reads the file again.

### Loading after `initialize`

The server answers `initialize` as soon as it has found the garden root, then loads the garden when the client sends `initialized`. It reports progress with `$/progress` on a token it creates with `window/workDoneProgress/create` when the client supports that. Loading runs on the main thread, so a request that arrives during loading waits in the channel and gets its answer after loading ends.

### Finding the garden

The garden root is the nearest directory, starting from the client's first workspace folder or its root URI and walking up, that holds `logseq/config.edn`. When there is none, the server answers `initialize` with an error that names the folder it searched from and the missing file, waits for the client to close the connection, and exits with a failure status.

### File changes outside the editor

When the client supports dynamic registration of `workspace/didChangeWatchedFiles`, the server registers watchers for `pages/**/*.md`, `journals/**/*.md` and `logseq/config.edn`, relative to the garden root when the client supports relative patterns. Otherwise the server watches `pages/`, `journals/` and `logseq/` with the `notify` crate and handles the events in the same main loop. On Linux and Windows `notify` delivers the operating system's file events. On macOS the server uses `notify`'s polling watcher, which compares modification times every two seconds. Either way a changed Markdown path replaces that document's entry, and a change to `logseq/config.edn` reloads the settings and rebuilds the index, keeping open buffers.

### Positions and paths in one module

`positions.rs` in the server package holds the negotiated encoding, the conversion between LSP positions and UTF-8 offsets, and the mapping between `file:` URIs and paths relative to the garden root. The server offers `utf-8` when the client lists it and `utf-16` otherwise.

### One module per feature area

`dispatch.rs` holds a table of request handlers and a table of notification handlers, each a method name and a function. `capabilities.rs` builds the advertised capabilities. A new feature adds a module under `features/`, a row in the dispatch table and a field in the capabilities, so parallel pull requests touch different lines.

### Status requests

The server answers two requests of its own. `lsg/status` returns the garden root, the settings in use, the position encoding, how file changes arrive, document counts and the load time. `lsg/occurrence` takes a document and a position and returns the occurrence there with its resolved target. The end-to-end tests use them to observe the index, and they give a person a way to check what the server sees.

### Neovim check

The docs page holds one `lua` code block with the `vim.lsp.config` call. `mise run lsp:nvim-check` copies that block out of the page, starts `nvim --headless --clean` on a page of a temporary copy of the fixture garden, waits for the client to attach, and asks it for `lsg/status`. Neovim is pinned in `mise.toml` at 0.12.6, the release its owner's configuration targets; the mise setting `minimum_release_age = "1d"` applies to it as to every other tool.

## Risks / Trade-offs

- Loading on the main thread delays the first answer on a large garden → loading the encode garden, 9,330 pages and 525 journals, takes about 1.2 seconds on a release build on an Apple M2 Pro, and Issue #638 sets five seconds as the point at which a cache on disk gets its own sub-issue.
- A file event can arrive while a file is half written → the next event for the same path replaces the entry again, and a read that fails leaves the previous entry in place.
- Rebuilding the name index after a title or alias change visits every document → the rebuild reads values each document already holds and parses nothing.
- The server's own watcher depends on operating-system file events or a two-second timer, so a change arrives late → the end-to-end test waits for the index to change, up to a time limit, and the test of the client-side path sends each notification itself.
- On macOS the polling watcher reads the modification time of every page each two seconds → Neovim on macOS watches files itself, so the timer runs only for editors that cannot, and about 10,000 files cost a few tens of milliseconds per pass.

## Resolved Questions

### 1 - Which protocol library does the server use?

`lsp-server` and `lsp-types`. They need no async runtime, and `lsg` has none. `tower-lsp-server` would add Tokio to a command-line tool that otherwise runs on one thread.

### 2 - Does the index hold `ReferenceIndex` and `transclusion::ReferenceTargets`, as Issue #638 suggests?

It holds neither. The occurrence list answers backlinks with source ranges, which `ReferenceIndex` does not keep. `ReferenceIndex::replace` walks every document's blocks on each call and the type has no removal, and it also searches every block for unlinked mentions, which no feature in Issue #638 uses. The `BlockIdSite` list resolves a block id and also keeps the duplicates that `ReferenceTargets` discards, which the diagnostics step needs. The index does hold `PageNameIndex` and `NamespaceIndex`.

### 3 - Which files does the index cover?

Markdown files beneath `pages/` and `journals/`, skipping names that start with a dot, `node_modules` and paths that `:hidden` in `logseq/config.edn` hides, as `garden::load_garden` does there. These are the folders the file watchers cover and the ones a person edits. A symlink or special file is skipped so that one such file leaves the server running.

### 4 - Which block does a duplicated `id::` resolve to?

The block Logseq keeps: the one in the file Logseq reads earliest, in the order `block_ids::parse_order` already encodes. The index keeps every site in that order, so the diagnostics step can report the later ones.

### 5 - How do tests observe the index before any editor feature exists?

Through the `lsg/status` and `lsg/occurrence` requests described under Decisions.

### 6 - How does the server report a folder that is not a garden?

As an error response to `initialize`. Neovim shows the message and stops the client. A notification sent before the `initialize` response would reach fewer clients.

### 7 - Which text synchronization does the server ask for?

Incremental. Neovim sends ranges by default, and the server applies them to its copy of the buffer and reparses the whole document, which takes well under a millisecond for a typical page.

### 8 - Where does a page's title come from?

From the file name under the garden's `:file/name-format`, or from a leading `title::` property, or from the journal date. The index skips the garden-wide pass in which Logseq respells a page that links to itself in another letter case, because that pass reads every file again after each change.

### 9 - Which `notify` backend watches files on macOS?

The polling watcher. `notify`'s default macOS backend depends on `objc2-core-foundation` and `objc2-core-services`, whose published packages hold no license text, and `mise run notices:update` stops on a package without one. `notify`'s other macOS backend opens one file descriptor per watched file, which a garden of several thousand pages exceeds under the default limit. Polling needs neither. The manifest turns on the `macos_kqueue` feature only because `notify` does not compile on macOS without one of its two macOS features.

### 10 - Does the server accept a `--stdio` flag?

`lsg lsp` takes no options. Standard input and output is its only transport, and Neovim passes the command as configured.

## Open Questions

None.
