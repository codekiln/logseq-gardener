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
