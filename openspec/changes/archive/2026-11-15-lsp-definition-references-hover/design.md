## Context

[PR #645 — start lsg lsp](https://github.com/codekiln/logseq-gardener/pull/645) gave the SDK a `GardenIndex` that records every page link, tag, block reference, embed, property key, page name in a property value and `id::` value with its source range, and that resolves a written name to a page with `page_id`. The server answers two requests of its own, `lsg/status` and `lsg/occurrence`, and no standard editor request. [Issue #640 — Follow links, block references and journal dates, and list references, in lsg lsp](https://github.com/codekiln/logseq-gardener/issues/640) adds go to definition, find references and hover on top of that index.

The SDK reads a file name into a page title with `page_titles::page_title`. It has no function for the other direction. Logseq's is `file-name-sanity` in `src/main/frontend/util/fs.cljs` of the local [logseq/og](https://github.com/logseq/og) checkout, read at commit `6e7afa8eb`.

The owner's Neovim maps `gd` to `vim.lsp.buf.definition()`, `gr` to references and `K` to hover through LazyVim.

## Goals / Non-Goals

**Goals:**

- `gd` on any link kind the index records opens the page or block, including a page or journal that has no file yet.
- `gr` lists a page's backlinks, counting its aliases, and a block's references and embeds.
- `K` shows enough of a page or block to decide whether to open it.
- Each feature is one SDK query plus a thin LSP handler, so a later `lsg` command can answer from the same code.
- The title-to-file-name function and the list of places that name a page are ready for [Issue #643 — Rename a page or tag across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/643).

**Non-Goals:**

- A code action that creates the file of a page link. Issue #638 lists it among the later features.
- Unlinked mentions of a page title in find references.
- Inlay hints that show a block's text after each `((uuid))`. Issue #638 lists them among the later features.
- `lsg` commands such as `page backlinks` that print these answers in a terminal.

## Decisions

### Title to file name

The SDK module `file_names` sits beside `page_titles` and holds the reverse direction. `page_file_name(title, format)` returns the name without folder and extension, `page_file_path` returns `pages/<file name>.md` relative to the garden root, and `journal_file_path(date, formats)` returns `journals/<date>.md` with the date in `:journal/file-name-format`.

Both formats drop one slash from each end of the title and apply Unicode NFC, as Logseq's `page-name-sanity` does. Under `:triple-lowbar` the function then follows `tri-lb-file-name-sanity` step for step: write `%25` for the `%` of anything that reads as a percent escape; percent-escape `: * ? " < > | # \`; write `%2E` for a leading dot; append the namespace separator to a Windows reserved name such as `CON` and to a title that ends in a dot; write `%5F` for each underscore in `___`, `_/` and `/_`; write `___` for each `/`. Under the legacy format it follows `legacy-url-file-name-sanity`: percent-escape `: * ? " < > | # \ %` and `/`.

Unit tests pin the output for `/`, `_`, `%`, a leading dot, Windows reserved names and the reserved characters under both formats, and read every name back with `page_title`. The titles of Logseq's own round-trip test, `frontend.db.name-sanity-test`, are among them.

### Titles the file name cannot carry

Under the legacy format `page_title` reads a dot in a file name as a namespace separator, so the file `Node.js.md` reads back as `Node/js`. Logseq keeps such a title in a `title::` property. `title_needs_property(title, format)` reports these titles, so the rename step can add the property. Go to definition names the path all the same, because that is the file Logseq writes.

### Where a target leads

`GardenIndex::destinations(&Target)` returns a list of `Destination { path, range, exists }`. A page name resolves through `page_id`:

- one document: the start of its file;
- an ambiguous name: the start of each file that answers to it, so the editor offers the choice;
- a journal date or a name without a file: `new_page_path(name)`, with `exists` false.

A block id leads to the first line of the block Logseq gives the id, from its dash to the end of the line. A block id that no block writes and a property key lead nowhere. `definition_at(path, offset)` combines `occurrence_at` with `destinations`.

The handler answers with `Location` values, which every client reads. A destination the index holds no document for gets line 0, character 0; Neovim opens an empty buffer under that name and writes the file on save.

### Places that name a page

`GardenIndex::page_references(&PageId)` wraps `occurrences_of_page` and adds what a rename needs to know about each place: `named_by` says whether the place writes the page's title, one of its aliases or a journal date in another format, and `declaration` marks the names in the page's own `alias::` property. Find references lists the declarations ahead of the other places.

`subject_at(path, offset)` picks what find references is about: the page, block or property key of the occurrence under the cursor, and the document's own page anywhere else. `references(&Subject, include_declaration)` returns path and range pairs in path order and then source order. With the declaration, the list starts with the start of the page's file and its own alias names, or with the `id::` value of the block.

### Summaries for hover

`page_summary(name)` returns the title, the file or the file Logseq would create, the alias spellings, the first five non-blank lines after the leading properties, and the number of places `references` lists without the declaration. `block_summary(id)` returns the page title, the first line of each parent block from the outermost in, the block's first five lines without indentation and dash, and the number of references and embeds. Both leave out the lines of properties Logseq hides, such as `id::` and `collapsed::`. `Summary::markdown` renders either one, and also the two short cases: a name several pages answer to, with each title and path, and a block id that no block writes.

The handler returns the Markdown with the range of the whole occurrence, so the editor highlights `[[Page]]` or `((uuid))`.

### Handlers

`features/definition.rs`, `features/references.rs` and `features/hover.rs` each convert the position to a path and an offset, call one index query and convert the ranges back in the negotiated encoding. `definition::location` builds a `Location` for both definition and references.

### Neovim check

`mise run lsp:nvim-check` already attaches to `pages/Recipes.md` of a copy of the fixture garden. It now also puts the cursor on `[[Malus]]`, calls `vim.lsp.buf.definition()` and waits for the buffer `pages/Fruit___Apple.md`, then does the same for `[[Fruit/Quince]]`, which has no file, and waits for the buffer `pages/Fruit___Quince.md`. It fails if that file exists afterwards.

## Risks / Trade-offs

- `occurrences_of_page` resolves every written name in the garden on each call, and hover and references both call it → on the encode garden, 9,855 pages and journals on a release build on an Apple M2 Pro, definition answered in under 0.1 ms, and hover and references in about 15 ms at the median and under 35 ms at the 95th percentile, over 449 sampled links. Issue #638 sets 100 ms as the target for definition.
- A journal link to a date without a file opens a buffer whose name depends on `:journal/file-name-format` → the index reads that setting on load and again when `logseq/config.edn` changes, and an end-to-end test covers a format other than the default.
- A page whose title several files share has no references from its own ordinary text, because the name is ambiguous → hover on a link to it lists the files, so the person can see why.
- The hover text shows a page's lines as written, with Logseq syntax such as `[[links]]` and `key:: value` → Neovim renders them as plain Markdown text, which stays readable.

## Resolved Questions

### 1 - Where does the title-to-file-name function live?

In a new SDK module, `file_names`, beside `page_titles`. Issue #640 and [Issue #643 — Rename a page or tag across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/643) both need it, and a command-line use such as creating a page would too, so it belongs to the SDK and uses no index type.

### 2 - What does go to definition return for a name that several pages answer to?

One location per page. Neovim then shows the locations in the quickfix list, which is how a person chooses between them.

### 3 - Which range does a page definition have?

The start of the file. A page has no line that defines it, and the start is where a person begins reading.

### 4 - Which range does a block definition have?

The block's first line from its dash to the end of the line. `lsg/occurrence` already reports that range for a resolved block, and a single line keeps the highlight small for a block with many lines.

### 5 - What is the declaration that `includeDeclaration` adds?

For a page, the start of its file and each name in its own `alias::` property, because those are the places that give the page its names. For a block, the `id::` value of the block Logseq gives the id. A page without a file and a property key have no declaration.

### 6 - Are the names in a page's own `alias::` property references to the page?

They are declarations. Without `includeDeclaration` the list leaves them out, so the reference count in a hover equals the number of places that link to the page.

### 7 - What does find references list in ordinary text?

Every place that names the page of the current document, which is the page's backlinks in Logseq's sense. The cursor can be anywhere in the page, so `gr` answers "what links here".

### 8 - What does find references list on a property key?

Every line in the garden that writes the key. The index already records each key as an occurrence, and [Issue #644 — Rename a property key across the garden from lsg lsp](https://github.com/codekiln/logseq-gardener/issues/644) needs the same list.

### 9 - How many lines does a hover show?

Five lines of a page and five of a block, with `…` after a block that has more. That fills a hover window without scrolling and still shows a short block whole.

### 10 - Does hover show a page's properties?

It shows the alias names on their own line and skips the leading property lines, so the five lines go to the page's content. Inside a block it keeps the properties Logseq shows, such as `colour:: red`, and hides the ones Logseq hides.

### 11 - Does the server create the file of a page without one?

No. It answers with the path, and the editor creates the file when the person saves the new buffer. The server changes no file, as before.

### 12 - Does the Neovim check need new fixture pages?

No. `pages/Recipes.md` of `tests/fixtures/lsp-garden` already links an alias, `[[Malus]]`, and a page without a file, `[[Fruit/Quince]]`. The end-to-end tests write their extra gardens, such as one with the legacy format, into a temporary folder.

## Open Questions

None.
