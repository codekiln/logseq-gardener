## Context

`lsg lsp` loads a garden into the SDK's `GardenIndex` and keeps it current while buffers and files change ([PR #645 — start lsg lsp](https://github.com/codekiln/logseq-gardener/pull/645)). Each indexed document already holds its occurrences (every link, tag, block reference and embed with a source range and a target) and its `id::` sites, and the index holds every site of each block id in the order Logseq reads files. The server answers `lsg/status` and `lsg/occurrence` and publishes nothing yet.

[Issue #642 — Report broken block references and duplicate block ids from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/642) asks the server to report three problems as diagnostics and to keep them current when a block id changes in another file. [Issue #638 — Edit a Logseq garden in Neovim through a language server, lsg lsp](https://github.com/codekiln/logseq-gardener/issues/638) sets two limits. A link to a page without a file is normal in Logseq and gets no diagnostic. Reindexing after an edit takes under 50 ms on the encode garden, which held 9,855 pages and journals when this change was measured.

## Goals / Non-Goals

**Goals:**

- An SDK query that lists the problems in one document from what the index already holds, with no LSP type.
- Diagnostics for a document when it opens, after each edit, and when its file changes on disk.
- A fix or a break made in one document reaches every open document it affects.
- Work after an edit that stays small as the garden grows.

**Non-Goals:**

- Diagnostics for every file of the garden at startup, so a person can list all broken references without opening each page.
- Hints for links to pages without a file and warnings for missing asset files, which Issue #638 lists as later work.
- Quick fixes, such as a code action that gives one of two blocks a new id.
- Pull diagnostics (`textDocument/diagnostic`).

## Decisions

### The query lives in the SDK and reads one document

`GardenIndex::problems(path)` returns a `Vec<Problem>` in source order. A `Problem` has a `ProblemKind`, a `SourceRange` and a list of `RelatedLocation`, each a document path and a range. The kind carries the id or page name, gives the severity and a short code (`missing-block`, `duplicate-block-id`, `missing-embedded-page`), and prints the message. The query walks the document's occurrences and `id::` sites and looks each one up in the index's maps:

- a block reference or block embed whose target is a UUID with no entry in the block id map is a missing block, marked on the whole reference;
- an `id::` site whose id has two or more sites is a duplicate, marked on the value, with every other site as a related location;
- a page embed whose name `page_id` resolves to a page or journal without a file is a missing embedded page, marked on the whole embed.

A page link, a tag and a property value never produce a problem. Text between `((` and `))` that is no UUID produces none either, because Logseq shows such text as written.

### Every block with a duplicated id gets the error

Logseq keeps the id on the block in the file it reads earliest and gives each other block a new id. The error appears on all of the blocks, including the one Logseq keeps. A person fixing the duplicate needs to see both places, and which block keeps the id depends on file order, which the person cannot see in the editor.

### Severities

A missing block and a duplicated id are errors: Logseq shows or stores something other than what the text says. An embed of a page without a file is a warning: Logseq shows an empty embed and creates the page, which works and is rarely what the author meant.

### Which documents the server publishes for

The server keeps the list it last published for each open document and for each document that is not open and has a published problem. It publishes:

- for a document when the client opens it, including an empty list, so the client's view starts from the server's;
- for a document that is not open when its file changes on disk and it has a problem;
- an empty list when a document with published problems closes or its file is deleted.

After every change to the index (an edit, a close, a file event, a reload after `logseq/config.edn` changes) the server computes the problems again for the open documents, the documents with a published problem and the documents the change touched, and sends each list that differs from the one it last sent. This is how a block id added, changed or removed in one document reaches the others.

A document that is not open leaves the server's records once its list is empty. The server reports it again when its file next changes.

### Clearing on close

When a document closes, the server sends an empty list for it if the last list had a problem. Neovim removes a buffer's diagnostics when the buffer is wiped, and other clients keep them until the server clears them. After the close the index reads the file again, so unsaved text that fixed or broke another document stops counting, and the refresh publishes that change for the documents still open.

### Cost of a refresh

The refresh never scans the garden. Its work grows with the number of open and reported documents. Measured on the encode garden with a release build on an Apple M2 Pro: replacing one document's entry after an edit takes 0.02 ms for a typical page of 700 characters, 0.3 ms for a page at the 95th percentile of size and 3.2 ms for the largest page of 284,000 characters; `problems` for one document takes 1 to 3 microseconds; `problems` for all 9,855 documents takes 15 ms. A session with a few dozen open documents therefore adds well under a millisecond to each edit.

### Related locations

The server sends `relatedInformation` for a duplicated id when the client sets `textDocument.publishDiagnostics.relatedInformation`. For other clients it appends `Also at <path>:<line>` to the message for each other block, with the path relative to the garden root, so the place is still readable.

### Fields of a diagnostic

Each diagnostic carries the range in the negotiated encoding, the severity, the source `lsg`, the problem's code and its message. The server sends no document version: a list for a document that is not open has none, and the server handles messages in order, so a list always describes the text of the last change it received.

### No capability to advertise

Push diagnostics need no entry in the server capabilities. `capabilities.rs` and `dispatch.rs` stay as they are; the text synchronization and file watching handlers call the new `features/diagnostics.rs` after they update the index.

## Risks / Trade-offs

- A document that is not open and breaks because of an edit elsewhere gets no diagnostic until its file changes or a person opens it → opening the page shows the problem, and reporting the whole garden is listed under Non-Goals as later work.
- A person with many open documents pays for each of them on every edit → `problems` takes microseconds per document.
- The codekiln Neovim configuration turns diagnostics off at startup → the Neovim page says how to show them.
- A client that keeps diagnostics for closed files could show stale problems for a document the server stopped following → the server clears a document's list when the document closes or its file is deleted, and follows a reported file until its list is empty.

## Resolved Questions

### 1 - Does the server report problems in files that are not open?

Only when the file changes on disk while the server runs, and from then until the file's list is empty. Issue #642 asks for "other documents when their files change". Publishing for all 9,855 documents at startup would put problems from files a person is not working on into the editor's list, and Neovim shows diagnostics only for loaded buffers.

### 2 - How does a change to a block id in one document reach the others?

The server computes the problems again for every open document and every reported document after each change, and sends the lists that differ. A map from block id to the documents that refer to it would do less work per edit, and the measurement under Decisions shows the simpler approach costs microseconds per document.

### 3 - Does the block Logseq keeps get the duplicate error too?

Yes. See "Every block with a duplicated id gets the error" under Decisions.

### 4 - What happens to a document's diagnostics when it closes?

The server clears them. The file may still hold the problem, and the server reports it again when the file changes or the document opens again. A closed document with lasting diagnostics would be the one case where the server reports a file nobody opened or changed.

### 5 - Is an embed of a journal date without a file a problem?

Yes, a warning, as for any other page without a file. The embed shows nothing in either case.

### 6 - Does the measuring program stay in the repository?

It stays out. The previous draft of this change measured with an example program under `crates/logseq-gardener-sdk/examples/`. The examples there show how to call the SDK, and a timing loop over a private garden fits neither that purpose nor a CI run. The pull request records the method and the numbers, and Issue #638 asks the step that finishes the MVP to add a repeatable measurement to the docs graph.

### 7 - Does the server send a diagnostic's document version?

It sends none. See "Fields of a diagnostic" under Decisions.

## Open Questions

None.
