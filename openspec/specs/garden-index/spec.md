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

### Requirement: List the problems in a document

The index SHALL list the problems in one document. Each problem SHALL carry the source range it marks, a severity, a message, and the other places involved as a document path with a source range. The index SHALL report:

- a block reference or block embed whose id no block in the garden writes, as an error on the whole reference;
- an `id::` value that more than one block in the garden writes, as an error on the value at each of those blocks, with every other block that writes the id as a related location;
- a page embed whose page has no file, as a warning on the whole embed.

The list SHALL be in source order. A path the index holds no document for SHALL give an empty list.

#### Scenario: A reference to a block that is gone

- **WHEN** a page holds `((66f1a2b0-0000-4000-8000-00000000dead))` and no block writes that id
- **THEN** the page's problems hold one error whose range is the reference and whose message names the id

#### Scenario: An id written by two blocks

- **WHEN** a journal and a page each hold a block with `id:: 66f1a2b0-0000-4000-8000-000000000003`
- **THEN** each document's problems hold one error on its `id::` value, and each error's related location is the value in the other document

#### Scenario: An embed of a page without a file

- **WHEN** a page holds `{{embed [[Fruit/Quince]]}}` and no file gives a page that name or alias
- **THEN** the page's problems hold one warning whose range is the embed

#### Scenario: An embed of a page through its alias

- **WHEN** a page holds `{{embed [[Pyrus]]}}` and another page writes `alias:: Pyrus`
- **THEN** the embed is no problem

### Requirement: Pages without a file are no problem outside an embed

A page link, a tag or a property value that names a page without a file SHALL NOT appear among a document's problems. Text between `((` and `))` that is no block id SHALL NOT appear either.

#### Scenario: A link to a page that has no file yet

- **WHEN** a page holds `[[Fruit/Quince]]`, `#quince` and `tags:: Quince`, and no file gives a page any of those names
- **THEN** the page's problems are empty

### Requirement: Problems follow the current index

The problems of a document SHALL reflect the texts the index holds when the query runs, including editor buffers.

#### Scenario: The missing block appears in another document

- **WHEN** a page refers to an id no block writes, and an editor buffer of another page then adds a block with that id
- **THEN** the first page's problems no longer hold the error

#### Scenario: One of two blocks drops a shared id

- **WHEN** two documents write the same id and one of them changes its id
- **THEN** neither document's problems hold the duplicate error

### Requirement: Completion context from the text around the cursor

The SDK SHALL read a document's text and a UTF-8 offset and report what is being typed at the offset, using only the text of the offset's line and the lines above it. The context SHALL name its kind, the text typed so far, and the range of source that a chosen candidate replaces. The kinds are:

- a page link, when a `[[` before the offset on the same line has no `]]` between it and the offset; this includes `{{embed [[` and `#[[`;
- a block reference, when a `((` before the offset on the same line has no `))` between it and the offset; this includes `{{embed ((`;
- a tag, when a `#` at the start of the line or after a space, a tab or `(` is followed by text without a space up to the offset;
- a property value, when the line holds `key::` followed by a space before the offset;
- a property key, when the offset follows only the line's indentation and the characters of a property key, on a line where Logseq reads properties: before the page's first bullet, or beneath a bullet with only property lines between them.

When more than one kind applies, an open `[[` or `((` SHALL win, the later of the two when both are open. The replaced range SHALL start after the opening `[[`, `((` or `#`, or at the start of the key or value. It SHALL extend past the offset over the rest of the name and a closing `]]` or `))` when the closing pair follows on the same line with no bracket of its kind between. For a property key it SHALL extend over the rest of the key, the `::` and the spaces after it. The SDK SHALL report no context elsewhere.

#### Scenario: A half-typed page link

- **WHEN** the text is `- See [[Fru` and the offset is at its end
- **THEN** the context is a page link, the typed text is `Fru`, and the replaced range is `Fru`

#### Scenario: A closing pair after the cursor

- **WHEN** the text is `- See [[Fru]] today` and the offset is after `Fru`
- **THEN** the replaced range is `Fru]]`

#### Scenario: A tag

- **WHEN** the text is `- Cake #swe` and the offset is at its end
- **THEN** the context is a tag and the typed text is `swe`

#### Scenario: A block embed

- **WHEN** the text is `- {{embed ((pear))}}` and the offset is after `pear`
- **THEN** the context is a block reference, the typed text is `pear`, and the replaced range is `pear))`

#### Scenario: A property key and a property value

- **WHEN** the line beneath a bullet is `  ty` with the offset at its end, and another line is `  type:: rec` with the offset at its end
- **THEN** the contexts are a property key with the typed text `ty` and a value of the property `type` with the typed text `rec`

#### Scenario: Prose

- **WHEN** the text is `- An apple is red` and the offset is at its end
- **THEN** there is no context

### Requirement: Page name candidates

For a page link or a tag, the index SHALL offer the title of every page and journal, every alias as its page writes it, and every namespace parent of a title, including a parent that has no file. A journal's title is its date in the garden's `:journal/page-title-format`. When the typed text spells a date in a format Logseq accepts, the index SHALL also offer that date's journal title, whether or not the journal has a file. When the typed text holds a `/`, the index SHALL offer first the pages one level beneath the namespace written before the last `/`.

The index SHALL rank candidates in groups: a name equal to the typed text, then a name that starts with the typed text, then the other names. It SHALL order each group with Logseq's fuzzy page score, comparing without regard to letter case, and SHALL leave out a candidate that scores zero. An alias candidate SHALL name the page it belongs to and SHALL insert the alias.

A candidate's inserted text SHALL be the name followed by `]]` for a page link. For a tag it SHALL be the name when Logseq reads `#name` as that one tag, and `[[name]]` otherwise.

#### Scenario: A title and an alias

- **WHEN** the typed text after `[[` is `pom` in the fixture garden
- **THEN** the candidates include `Pomme`, an alias of `Fruit/Apple`, which inserts `Pomme]]`

#### Scenario: A name that starts with the typed text

- **WHEN** the typed text after `[[` is `log` and the garden has the pages `Blog`, `Log` and `Logbook`
- **THEN** the candidates are `Log`, `Logbook` and `Blog`, in that order

#### Scenario: A namespace level

- **WHEN** the typed text after `[[` is `Fruit/`
- **THEN** the leading candidates are `Fruit/Apple` and `Fruit/Pear`

#### Scenario: A journal by another date format

- **WHEN** the typed text after `[[` is `2026-01-04` and the journal title format is `MMM do, yyyy`
- **THEN** the candidates include `Jan 4th, 2026`

#### Scenario: A tag with a space

- **WHEN** the typed text after `#` is `mul` and the garden has a page `multi word`
- **THEN** the candidate `multi word` inserts `[[multi word]]`

### Requirement: Block candidates

For a block reference, the index SHALL offer each block id that a block writes, with the text of the block Logseq gives the id and the title of its page. A block SHALL match when its text holds every word of the typed text, without regard to letter case. A candidate SHALL insert the id followed by `))`.

#### Scenario: A block by its text

- **WHEN** the typed text after `((` is `pear ripens`
- **THEN** the candidate is the block `A pear ripens after picking.` on the page `Fruit/Pear`, and it inserts that block's id and `))`

### Requirement: Property candidates

For a property key, the index SHALL offer the keys written in the garden, ranked as page names are and then by how many lines write them, and SHALL leave out Logseq's card scheduling properties and the keys in `:block-hidden-properties`. A key candidate SHALL insert the key, `::` and a space.

For a property value, the index SHALL offer the values already written for that key, ranked the same way. For `tags`, `alias` and `aliases`, whose values Logseq reads as page names separated by commas, it SHALL offer page names for the name being typed after the last comma.

#### Scenario: Keys without hidden ones

- **WHEN** a garden writes `type::`, `card-last-score::` and a key listed in `:block-hidden-properties`
- **THEN** the key candidates include `type` and leave out the other two

#### Scenario: Values of a key

- **WHEN** the typed text after `type::` is `rec` and a block writes `type:: recipe`
- **THEN** the candidates include `recipe`

#### Scenario: A page name in `tags::`

- **WHEN** the typed text is `tags:: sweet, Fru`
- **THEN** the candidates are page names that match `Fru`, and the replaced range is `Fru`

### Requirement: A limit on candidates

The candidate query SHALL take a limit, return at most that many candidates in ranked order, and say whether it left any out. A list of page names SHALL always be reported as partial, because a date candidate appears only once the typed text spells a date.

#### Scenario: More matches than the limit

- **WHEN** 150 pages match the typed text and the limit is 100
- **THEN** the query returns the 100 best and reports that the list is partial

### Requirement: What a rename at a position is about

`GardenIndex::rename_target_at(path, offset)` SHALL return the subject of a rename, the source range of the name and the current name. On a page link, tag, page embed or page name in a property value the subject SHALL be the page the name leads to, following an alias to its page, the range SHALL be the written name, and the name SHALL be the page's title, or the name as written for a page with no file. On the key of a `key::` line the subject SHALL be the property key in lower case, the range SHALL be the key and the name SHALL be the key as written. Anywhere else in a document the subject SHALL be the document's own page, with the range of its leading `title::` value when the offset is in it and an empty range at the offset otherwise. It SHALL return a refusal for a path the index holds no document for, a document with no title, a block reference, a block id, a property key with a built-in meaning, a name several files answer to and a journal date with no file.

#### Scenario: An alias leads to its page

- **WHEN** `pages/Fruit___Apple.md` declares `alias:: Malus` and a caller asks at `[[malus]]` in another page
- **THEN** the subject is the page of `pages/Fruit___Apple.md`, the range holds `malus` and the name is `Fruit/Apple`

#### Scenario: Ordinary text names the document's own page

- **WHEN** a caller asks at an offset inside ordinary words of a page titled `Shown`
- **THEN** the subject is that page, the range is empty at the offset and the name is `Shown`

#### Scenario: A block reference

- **WHEN** a caller asks at a `((uuid))`
- **THEN** the answer is a refusal

#### Scenario: A property key

- **WHEN** a caller asks at an offset inside `Type` of a line `Type:: book`
- **THEN** the subject is the property key `type`, the range holds `Type` and the name is `Type`

#### Scenario: A built-in property key

- **WHEN** a caller asks at an offset inside `tags` of a line `tags:: fruit`
- **THEN** the answer is a refusal that names `tags`

### Requirement: Plan the rename of a page

`GardenIndex::rename_page(page, new_name)` SHALL return a `RenamePlan` and SHALL change no file. The plan SHALL list, per file in path order, the text replacements with their source ranges in source order and the file renames to apply after them. It SHALL use no Language Server Protocol type.

The plan SHALL replace every page link, tag, page embed and page name in a property value that names the page by its title, in any letter case and Unicode normalization form, with the new name as typed and trimmed. It SHALL keep a name written with one of the page's aliases and a journal date written in a format other than the journal's title. It SHALL replace `#old` with `#new` when `#new` reads back as one tag of that name and with `#[[new]]` otherwise, keep the brackets of `#[[old]]`, and write `[[new]]` for a bare name in a property value when the new name holds a comma. Text outside the replaced ranges SHALL stay as written.

#### Scenario: Every way of naming the page

- **WHEN** a page `Fruit` with `alias:: Produce` is renamed to `Sweet Food` and another page writes `[[Fruit]]`, `[[fruit]]`, `#Fruit`, `#[[FRUIT]]`, `{{embed [[Fruit]]}}`, `[label]([[Fruit]])`, `tags:: fruit`, `[[Produce]]`, `[[Fruity]]` and `[[Fruit]]` inside inline code
- **THEN** the first seven become `[[Sweet Food]]`, `[[Sweet Food]]`, `#[[Sweet Food]]`, `#[[Sweet Food]]`, `{{embed [[Sweet Food]]}}`, `[label]([[Sweet Food]])` and `tags:: Sweet Food`, and the last three stay as written

#### Scenario: A tag with no file

- **WHEN** the name `todo`, which no file answers to, is renamed to `task`
- **THEN** `#todo` becomes `#task`, `#[[todo]]` becomes `#[[task]]`, `tags:: todo` becomes `tags:: task`, and the plan renames no file

### Requirement: Namespace children follow a rename

The plan SHALL rename every page whose title starts with the old title and a `/`, replacing that start with the new name and keeping the rest of the title, together with the names that lead to those pages and the names with that start that no file answers to. It SHALL rename a page whose title holds `[[old]]` or `[[old/x]]`, as Logseq's `rename-nested-pages` does. A page whose title only starts with the same letters, such as `workshop` beside `work`, SHALL stay.

#### Scenario: Children, their links and a child with no file

- **WHEN** `work` is renamed to `Job` in a garden with the pages `work/Log`, `work/Log/2026`, `workshop` and `sub/work___Plan.md`, and a page writes `[[Work/log]]`, `#work/Log/2026` and `[[work/unwritten]]`
- **THEN** the pages become `Job/Log`, `Job/Log/2026` and `Job/Plan`, the names become `[[Job/log]]`, `#Job/Log/2026` and `[[Job/unwritten]]`, and `workshop` stays

### Requirement: File names and titles after a rename

The plan SHALL rename each renamed page's file to the name `file_names::page_file_name` gives its new title under the garden's `:file/name-format`, in the folder the file is in. It SHALL keep a journal's file name. It SHALL write the new title to the `title::` line of the page's leading properties when the page has one, and SHALL add a `title::` line at the top of the file when the page has none and either the page is a journal or `file_names::title_needs_property` reports the new title. When the old and the new path are equal after lowercasing and Unicode NFC, the plan SHALL rename the file to a temporary name that starts with a dot, in the same folder, and from there to the new path. The plan SHALL replace the string of `:default-home {:page "…"}` in `logseq/config.edn` when it names a renamed page.

#### Scenario: A title the legacy file name cannot carry

- **WHEN** `logseq/config.edn` sets no `:file/name-format` and the page of `pages/Node.md` is renamed to `Node.js`
- **THEN** the file becomes `pages/Node.js.md` and its text starts with `title:: Node.js` and an empty line

#### Scenario: A journal

- **WHEN** the journal of `journals/2026_01_02.md`, titled `Jan 2nd, 2026`, is renamed to `Bake Day` and a page writes `[[Jan 2nd, 2026]]` and `[[2026-01-02]]`
- **THEN** the plan renames no file, the journal's text starts with `title:: Bake Day`, the first link becomes `[[Bake Day]]` and the second stays

#### Scenario: A change of letter case only

- **WHEN** the page of `pages/Foo.md` is renamed to `foo`
- **THEN** the plan renames `pages/Foo.md` to a `pages/.lsg-rename-<n>.tmp` that does not exist and then that file to `pages/foo.md`

#### Scenario: The home page

- **WHEN** `logseq/config.edn` holds `:default-home {:page "home/Start"}` and the page `Home` is renamed to `Base`
- **THEN** the plan replaces the string with `"Base/Start"`

### Requirement: Refused renames

`GardenIndex::rename_page` SHALL return a `RenameRefusal` whose text states the reason, and no plan, when the new name is empty after trimming or holds a line break; when a new title of the page or of a namespace child belongs to another page by title or alias, or spells a journal date; when a name that only references carry would become the name of a page with a file; when several files answer to the old name; when the old name is a journal date with no file; when the page's title is written outside a `title::` line; when a new title gives an empty file name; and when a file exists at a new path. A new name equal to the page's title SHALL give an empty plan. A new name that only references carry, or that is one of the page's own aliases, SHALL be accepted.

#### Scenario: The new name belongs to another page

- **WHEN** page `A` is renamed to `b` or to `bee` in a garden where `pages/B.md` declares `alias:: Bee`
- **THEN** the answer is a refusal that names `pages/B.md`

#### Scenario: An empty name

- **WHEN** a page is renamed to a name of spaces, or to `/`
- **THEN** the answer is a refusal

#### Scenario: A name only references carry

- **WHEN** page `A` is renamed to `Ghost`, which a link names and no file answers to
- **THEN** the answer is a plan

### Requirement: Keys with a built-in meaning

`property_names::logseq_property_key` SHALL return a property key as Logseq stores it: trimmed, in lower case, with each space and `_` replaced by `-`. `property_names::is_built_in_property` SHALL be true for a key whose stored form is in Logseq's `editable-built-in-properties` or `hidden-built-in-properties`, is one of the six card scheduling keys, or starts with `logseq.`, and false for every other key.

#### Scenario: Built-in keys in any spelling

- **WHEN** a caller asks about `id`, `Title`, `alias`, `tags`, `collapsed`, `custom_id`, `Created_At`, `card-last-score` and `logseq.order-list-type`
- **THEN** each answer is true

#### Scenario: A person's own keys

- **WHEN** a caller asks about `type`, `status`, `due-date` and `identity`
- **THEN** each answer is false

### Requirement: Plan the rename of a property key

`GardenIndex::rename_property(key, new_key)` SHALL return a `RenamePlan` and SHALL change no file. The plan SHALL hold, per file in path order, one replacement for each `key::` line the index records whose key equals `key` as `logseq_property_key` compares them, in a page's leading properties and in block properties, with the range of the key and the new key as typed. The new key SHALL be trimmed of surrounding white space and of one `::` at its end. A line that already spells the new key SHALL get no replacement. The plan SHALL rename no file and SHALL leave values and all other text as written. A key no line writes SHALL give an empty plan.

When blocks other than the ones that write the old key already write the new key, the rename SHALL be planned as for an unused key.

#### Scenario: Page properties and block properties in several files

- **WHEN** `pages/A.md` starts with `Type:: book`, a block of `pages/B.md` writes `type:: film` after a tab, a journal block writes `TYPE:: note`, and `type` is renamed to `kind`
- **THEN** the plan replaces `Type`, `type` and `TYPE` with `kind` in the three files, and each file's text is otherwise as written

#### Scenario: Underscores count as hyphens

- **WHEN** lines write `due_date::`, `Due-Date::` and `duedate::`, and `due-date` is renamed to `deadline`
- **THEN** the first two keys become `deadline` and `duedate::` stays

#### Scenario: A change of letter case only

- **WHEN** lines write `Type::` and `type::`, and `type` is renamed to `Type`
- **THEN** the plan replaces the key of the `type::` line alone

#### Scenario: The new key is in use on other blocks

- **WHEN** one block writes `type:: book`, another writes `kind:: film`, and `type` is renamed to `kind`
- **THEN** the plan replaces `type` with `kind` and leaves the other block as written

#### Scenario: The key is named in a value and in text

- **WHEN** a block writes `type:: type` and its text holds `type:: in code` inside inline code, and `type` is renamed to `kind`
- **THEN** the line becomes `kind:: type` and the block's text stays

### Requirement: Refused renames of a property key

`GardenIndex::rename_property` SHALL return a `RenameRefusal` whose text states the reason, and no plan, when `is_built_in_property` is true for the old or the new key; when the new key is empty after trimming, holds white space or a colon, or is rejected by `property_names::valid_property_name`; and when a block writes both the old and the new key, in which case the refusal SHALL name the file and the line of the first such block.

#### Scenario: A built-in key

- **WHEN** `collapsed` is renamed to `folded`, or `type` is renamed to `Tags` or to `custom_id`
- **THEN** each answer is a refusal that names the built-in key

#### Scenario: A key Logseq would not read

- **WHEN** `type` is renamed to an empty name, to `my key`, to `a:b`, to `#kind` or to `kind(x)`
- **THEN** each answer is a refusal

#### Scenario: A block writes both keys

- **WHEN** a block that starts on line 3 of `pages/A.md` writes `type:: book` and `kind:: novel`, and `type` is renamed to `kind`
- **THEN** the answer is a refusal that names `pages/A.md` and line 3
