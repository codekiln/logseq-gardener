## ADDED Requirements

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
