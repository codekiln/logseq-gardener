## ADDED Requirements

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
