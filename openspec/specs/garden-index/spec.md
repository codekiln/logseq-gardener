## Purpose

Define the SDK's in-memory index of a garden's documents, the links and properties each document writes, and the queries that resolve them.

## Requirements

### Requirement: Filename format setting

`GardenSettings` SHALL read `:file/name-format` from `logseq/config.edn` into a `filename_format` field. The value `:triple-lowbar` SHALL give the triple-lowbar format. An absent key, or any other value, SHALL give the legacy format, as in Logseq.

#### Scenario: A garden that sets triple-lowbar

- **WHEN** `logseq/config.edn` holds `:file/name-format :triple-lowbar`
- **THEN** `GardenSettings::read` reports the triple-lowbar format

#### Scenario: A garden without the key

- **WHEN** `logseq/config.edn` has no `:file/name-format` key
- **THEN** `GardenSettings::read` reports the legacy format

### Requirement: Load a garden into an index

The SDK SHALL provide `index::GardenIndex::load`, which reads the settings from `logseq/config.edn` and every Markdown file beneath `pages/` and `journals/` of a garden root. It SHALL skip a file or folder whose name starts with a dot, a `node_modules` folder, a path that an entry of `:hidden` hides, a symlink, and a file that holds only whitespace. It SHALL accept a callback that receives the number of files read so far and the total. The index module SHALL use no Language Server Protocol type.

#### Scenario: Load the fixture garden

- **WHEN** a caller loads `tests/fixtures/lsp-garden`
- **THEN** the index holds one document for each page and journal file, keyed by its path relative to the root, with the page title that the triple-lowbar file name gives

#### Scenario: Hidden paths stay out

- **WHEN** the garden holds `pages/.trash/old.md` and a file that `:hidden` names
- **THEN** the index holds no document for either path

### Requirement: Record what each document writes

For each document the index SHALL keep the source text, the parse, the start of every line, the title, the journal date of a journal, the alias keys, and a list of occurrences sorted by position. An occurrence SHALL record its kind, the range of the whole syntax, the range of the name inside it, and its target. Ranges SHALL be offsets into the document's UTF-8 source. The kinds SHALL be: a page link `[[Page]]`, including one used as a link label target; a tag `#tag` or `#[[multi word]]`; a block reference `((uuid))`; a page embed `{{embed [[Page]]}}`; a block embed `{{embed ((uuid))}}`; a property key; a page name written bare in an `alias::`, `aliases::` or `tags::` value; and the value of an `id::` property. A page link or tag inside any property value SHALL be recorded as a page link or tag. The target SHALL be the page name as written, the block id in lower case, or the property key in lower case.

The index SHALL also keep, per document, each property line with its key, value, their ranges and the start of the block it belongs to, and each block that writes a valid `id::`, `custom-id::` or `custom_id::` with the id, the range of the value and the start of the block.

#### Scenario: A tab-indented child block

- **WHEN** a document holds `\t- See [[Fruit/Apple]] and #[[multi word]]` as a child block
- **THEN** its occurrences include a page link whose name range covers `Fruit/Apple` and a tag whose name range covers `multi word`, at the offsets where the source writes them

#### Scenario: Comma-separated tags

- **WHEN** a document starts with `tags:: [[Fruit]], recipes`
- **THEN** its occurrences include the property key `tags`, a page link to `Fruit` and a property value name `recipes`

#### Scenario: Text inside code

- **WHEN** a block holds `[[Not A Link]]` inside inline code or a fenced code block
- **THEN** the document has no occurrence for it

### Requirement: Find what is at a position

`GardenIndex::occurrence_at` SHALL take a document path and a UTF-8 offset and return the occurrence whose whole range contains the offset, or none.

#### Scenario: Cursor inside a block reference

- **WHEN** a caller asks for the offset of the fourth character of `((66f1a2b0-0000-4000-8000-000000000001))`
- **THEN** the index returns a block reference occurrence whose target is that id

#### Scenario: Cursor on plain text

- **WHEN** a caller asks for an offset inside ordinary words
- **THEN** the index returns none

### Requirement: Resolve pages and blocks

`GardenIndex::page_id` SHALL resolve a page name as Logseq does: by title, then by alias, then by the journal date the name spells in any format Logseq accepts, ignoring letter case. The result SHALL be the one document the name leads to; a journal date that has no file; a name that has no file; or an ambiguous name when several documents answer to it. `GardenIndex::occurrences_of_page` SHALL return every occurrence in the garden that resolves to a given result, with its document.

`GardenIndex::block` SHALL resolve a block id to the document and block start of the block Logseq keeps when several blocks write the id: the block in the file Logseq reads earliest. `GardenIndex::block_id_sites` SHALL return every block that writes the id, in that order. `GardenIndex::occurrences_of_block` SHALL return every block reference and block embed of the id, with its document.

#### Scenario: An alias

- **WHEN** `pages/Fruit___Apple.md` starts with `alias:: Malus` and another page links `[[malus]]`
- **THEN** `page_id("malus")` is the document `pages/Fruit___Apple.md`, and `occurrences_of_page` for that document includes the `[[malus]]` link

#### Scenario: A journal link in another format

- **WHEN** the garden holds `journals/2026_01_02.md` and a page links `[[Jan 2nd, 2026]]`
- **THEN** `page_id("Jan 2nd, 2026")` is the document `journals/2026_01_02.md`

#### Scenario: A page with no file

- **WHEN** no file is titled `Not Yet Written`
- **THEN** `page_id("Not Yet Written")` reports a name with no file

#### Scenario: A duplicated block id

- **WHEN** a journal and a page both write `id:: 66f1a2b0-0000-4000-8000-000000000002`
- **THEN** `block` returns the journal's block, and `block_id_sites` lists the journal's block and then the page's block

### Requirement: Stay equal to a fresh load

The index SHALL provide operations to set the text of an editor buffer for a path, to close that buffer, to read one path again from disk, and to reload the settings and every file. While a buffer is open for a path, the buffer's text SHALL define that document and a change on disk SHALL leave it as it is. Closing the buffer SHALL read the file again. A path whose file is gone, or holds only whitespace, SHALL have no document. Reloading SHALL keep open buffers. After any sequence of these operations with no buffer open, the index SHALL equal the index a fresh load of the same files gives, including name resolution, namespaces, block id order and property counts.

#### Scenario: Add, edit, rename and delete

- **WHEN** files are added, edited, renamed and deleted on disk and each changed path is read again
- **THEN** the index equals a fresh load of the garden

#### Scenario: An open buffer wins

- **WHEN** a buffer for `pages/Fruit.md` holds text that differs from the file, and the file then changes on disk
- **THEN** the document's source is the buffer's text until the buffer closes, and the file's text afterwards

#### Scenario: A changed alias

- **WHEN** a buffer adds `alias:: Pomme` to a page
- **THEN** `page_id("pomme")` is that page's document, and no other document was parsed again

### Requirement: Property counts

The index SHALL count, across the garden, how many property lines write each property key, and how many write each value of each key. Keys SHALL be compared in lower case.

#### Scenario: Count a key

- **WHEN** three blocks in the garden write `type::` and two of them write `type:: recipe`
- **THEN** the count for `type` is three and the count for the value `recipe` of `type` is two

### Requirement: File name of a page title

`file_names::page_file_name` SHALL return the file name Logseq gives a page title, without folder and extension, for each value of `:file/name-format`. Both formats SHALL drop one slash from each end of the title and apply Unicode NFC.

Under `:triple-lowbar` the function SHALL write `%25` for the `%` of any `%` followed by two hexadecimal digits; percent-escape each of `: * ? " < > | # \`; write `%2E` for a leading dot; append the namespace separator to a title that is a Windows reserved name in upper case (`CON`, `PRN`, `AUX`, `NUL`, `COM1` to `COM9`, `LPT1` to `LPT9`) and to a title that ends in a dot; write `%5F` for each underscore of `___`, `_/` and `/_`; and write `___` for each `/`.

Under the legacy format the function SHALL percent-escape each of `: * ? " < > | # \ %` and `/`.

`file_names::page_file_path` SHALL return `pages/<file name>.md` relative to the garden root, and none for a title that gives an empty file name. `file_names::journal_file_path` SHALL return `journals/<date>.md` with the date in `:journal/file-name-format`. `file_names::title_needs_property` SHALL report a title whose file name `page_titles::page_title` reads back as another title.

#### Scenario: A namespace page

- **WHEN** a caller asks for the file name of `Fruit/Apple`
- **THEN** the result is `Fruit___Apple` under `:triple-lowbar` and `Fruit%2FApple` under the legacy format

#### Scenario: Underscores beside a separator

- **WHEN** a caller asks for the `:triple-lowbar` file name of `a__/bbb/ccc`
- **THEN** the result is `a_%5F___bbb___ccc`

#### Scenario: A percent sign

- **WHEN** a caller asks for the `:triple-lowbar` file names of `100% sure` and `a%20b`
- **THEN** the results are `100% sure` and `a%2520b`

#### Scenario: A leading dot

- **WHEN** a caller asks for the `:triple-lowbar` file name of `.NET`
- **THEN** the result is `%2ENET`

#### Scenario: A Windows reserved name

- **WHEN** a caller asks for the `:triple-lowbar` file names of `CON` and `con`
- **THEN** the results are `CON___` and `con`

#### Scenario: The file name reads back as the title

- **WHEN** a page file is named with `page_file_name` under `:triple-lowbar` for a title that holds `/`, `_`, `%`, `?`, `#`, a leading dot or a Windows reserved name
- **THEN** `page_titles::page_title` returns the title, and `title_needs_property` is false

#### Scenario: A legacy title with a dot

- **WHEN** a caller asks about the title `Node.js` under the legacy format
- **THEN** the file name is `Node.js`, `page_title` reads it back as `Node/js`, and `title_needs_property` is true

#### Scenario: A journal file

- **WHEN** `:journal/file-name-format` is `yyyy-MM-dd` and a caller asks for the journal file of 4 January 2026
- **THEN** the result is `journals/2026-01-04.md`

### Requirement: Where a target leads

`GardenIndex::destinations` SHALL return, for a target, the places an editor opens: each with a path relative to the garden root, a source range, and whether the index holds a document at the path.

A page name SHALL resolve as `GardenIndex::page_id` resolves it. A name one document answers to SHALL lead to the start of that document. A name several documents answer to SHALL lead to the start of each. A name no document answers to SHALL lead to the start of the file `GardenIndex::new_page_path` names: `journals/<date in :journal/file-name-format>.md` for a name that spells a journal date, and `pages/<file name>.md` otherwise, by `file_names::page_file_name` under the garden's `:file/name-format`.

A block id SHALL lead to the first line of the block `GardenIndex::block` returns, from the block's dash to the end of the line. A block id that no block writes, and a property key, SHALL lead nowhere.

`GardenIndex::definition_at` SHALL return the occurrence at an offset of a document together with its destinations, and none when no occurrence is there.

#### Scenario: An alias

- **WHEN** `pages/Fruit___Apple.md` starts with `alias:: Malus` and a caller asks where the page name `MALUS` leads
- **THEN** the one destination is the start of `pages/Fruit___Apple.md`, and the index holds that document

#### Scenario: A page without a file

- **WHEN** the garden uses `:triple-lowbar`, no file is titled `Fruit/Quince`, and a caller asks where that name leads
- **THEN** the one destination is the start of `pages/Fruit___Quince.md`, and the index holds no document there

#### Scenario: A journal without a file

- **WHEN** `:journal/file-name-format` is `yyyy-MM-dd`, no journal exists for 4 January 2026, and a caller asks where `Jan 4th, 2026` leads
- **THEN** the one destination is the start of `journals/2026-01-04.md`

#### Scenario: A name several pages share

- **WHEN** `pages/A.md` and `pages/B.md` both declare `alias:: shared` and a caller asks where `Shared` leads
- **THEN** the destinations are the starts of `pages/A.md` and `pages/B.md`

#### Scenario: A block

- **WHEN** a journal block `- Ate an [[fruit/apple]]` writes an `id::` and a caller asks where that id leads
- **THEN** the one destination is the range of the line `- Ate an [[fruit/apple]]` in the journal

#### Scenario: A block that is gone

- **WHEN** a caller asks where an id that no block writes leads
- **THEN** the list of destinations is empty

### Requirement: Places that name a page, a block or a property key

`GardenIndex::page_references` SHALL return every occurrence `occurrences_of_page` returns for a page, each with its document, with whether it writes the page's title, one of its aliases or a journal date in another format, and with whether it is a name in the page's own leading `alias::` property.

`GardenIndex::occurrences_of_property` SHALL return the key occurrence of every line that writes a property key, ignoring letter case, with its document.

`GardenIndex::subject_at` SHALL return what the occurrence at an offset of a document names: a page as `page_id` resolves it, a block id or a property key. At an offset inside no occurrence it SHALL return the page of the document itself. It SHALL return none for a path the index holds no document for.

`GardenIndex::references` SHALL return the path and source range of every place that names a subject, in path order and then source order: for a page, the occurrences of `page_references` that are no declaration; for a block, its block references and block embeds; for a property key, the occurrences of `occurrences_of_property`. When the caller asks for the declaration, the list SHALL start with the start of the page's file followed by the names in the page's own `alias::` property, or with the `id::` value of the block `GardenIndex::block` returns. A page without a file and a property key SHALL have no declaration.

#### Scenario: A page named by its title and by an alias

- **WHEN** `pages/Fruit___Apple.md` starts with `alias:: Malus`, a journal links `[[fruit/apple]]`, `pages/Fruit.md` links `[[Fruit/Apple]]` and `pages/Notes.md` links `[[malus]]`
- **THEN** `page_references` for the apple page marks the two title links as the title, `[[malus]]` as an alias, and `Malus` in the page's own `alias::` as an alias and a declaration

#### Scenario: References from a link

- **WHEN** a caller asks for the subject at an offset inside `[[malus]]` and then for its references without the declaration
- **THEN** the subject is the document `pages/Fruit___Apple.md`, and the references are the ranges of `[[fruit/apple]]`, `[[Fruit/Apple]]` and `[[malus]]`

#### Scenario: References from ordinary text

- **WHEN** a caller asks for the subject at an offset inside the words of a block of `pages/Fruit___Apple.md`
- **THEN** the subject is the document `pages/Fruit___Apple.md`

#### Scenario: A page with its declaration

- **WHEN** a caller asks for the references of the apple page with the declaration
- **THEN** the list starts with offset 0 of `pages/Fruit___Apple.md` and the range of `Malus` in its `alias::` property, followed by the links

#### Scenario: A page without a file

- **WHEN** only `pages/Notes.md` names `#[[Not Yet Written]]` and a caller asks for that page's references with the declaration
- **THEN** the list holds the range of `#[[Not Yet Written]]` alone

#### Scenario: A block with its declaration

- **WHEN** a journal block writes an `id::`, `pages/Notes.md` embeds the block, and a caller asks for the block's references with the declaration
- **THEN** the list holds the range of the `id::` value in the journal and then the range of the embed

#### Scenario: A property key

- **WHEN** three lines in the garden write `type::` and a caller asks for the references of the property key `type`
- **THEN** the list holds the range of each `type` key

### Requirement: Summaries of a page and a block

`GardenIndex::page_summary` SHALL describe the page a name leads to: its title; its file, or the file `new_page_path` names when it has none, and which of the two applies; the names in its `alias::` property as written; the first five lines after its leading properties that are neither blank nor a property Logseq hides, as written; and the number of places `references` lists for the page without the declaration. For a name several documents answer to it SHALL return the title and path of each.

`GardenIndex::block_summary` SHALL describe the block `GardenIndex::block` returns for an id: the path and title of its page; the first line of each parent block, from the outermost in, without indentation and dash; the block's own lines without indentation and dash, leaving out the properties Logseq hides, cut to five lines followed by `…` when the block has more; and the number of block references and block embeds of the id. For an id that no block writes it SHALL say so.

`GardenIndex::summary_at` SHALL return the range of the occurrence at an offset of a document with the summary of the page or block it names, and none in ordinary text and on a property key.

`Summary::markdown` SHALL render a summary as Markdown. A page SHALL show its title in bold, a line `Aliases: …` when it has aliases, a sentence that names the file go to definition opens when the page has no file, the reference count, and its first lines below a horizontal rule. A block SHALL show its page title in bold followed by its parents, joined with `›`, then its text, then the reference count.

#### Scenario: A page with an alias

- **WHEN** `pages/Fruit___Apple.md` starts with `alias:: Malus`, three places link to it, and a caller asks for the summary of `malus`
- **THEN** the summary has the title `Fruit/Apple`, the alias `Malus`, the page's first lines and a reference count of 3

#### Scenario: A page without a file

- **WHEN** no file is titled `Fruit/Quince` and nothing links to it
- **THEN** the Markdown of its summary is `**Fruit/Quince**`, the sentence ``This page has no file. Go to definition opens `pages/Fruit___Quince.md`.`` and `No references`, each as its own paragraph

#### Scenario: A block beneath parents

- **WHEN** a block with an `id::` sits beneath `- Middle`, which sits beneath `- Top`, and has seven lines besides its `id::` line
- **THEN** its summary lists the parents `Top` and `Middle`, and its text is the first five lines followed by `…`, without the `id::` line

#### Scenario: A property Logseq shows

- **WHEN** a block writes `id::` and `type:: meal`
- **THEN** its summary text keeps `type:: meal` and leaves out the `id::` line

#### Scenario: A block that is gone

- **WHEN** a caller asks for the summary of an id that no block writes
- **THEN** the Markdown is `No block has this id.`
