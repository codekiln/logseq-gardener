## Purpose

Derive Markdown page display titles using explicit Logseq filename conventions.

## Requirements

### Requirement: Derive page titles with an explicit filename format

The SDK SHALL derive a title from a loaded Markdown page document using a caller-supplied Legacy or TripleLowbar format. It SHALL use only the final filename stem and preserve capitalization. For Legacy filenames, the SDK SHALL replace dots with slashes, then decode percent escapes only if every escape and the resulting UTF-8 are valid. If decoding fails, it SHALL retain the string with dots replaced by slashes. TripleLowbar SHALL replace non-overlapping triple underscores with slashes, decode each ASCII percent escape once, preserve malformed and non-ASCII escapes, and remove empty slash segments.

#### Scenario: Configuration changes namespace decoding
- **WHEN** the document path is `pages/Names___Nested.md`
- **THEN** Legacy yields `Names___Nested` and TripleLowbar yields `Names/Nested`

#### Scenario: Percent decoding follows the format
- **WHEN** the stem is `%C3%A9.%2F%252F`
- **THEN** Legacy yields `é//%2F` and TripleLowbar yields `%C3%A9./%2F`

#### Scenario: Malformed escape preserves legacy input
- **WHEN** the stem is `Bad%2X.%2F`
- **THEN** Legacy yields `Bad%2X/%2F` and TripleLowbar yields `Bad%2X.`

### Requirement: Respect leading page-title overrides and Contents

The SDK SHALL use the last case-insensitively named title property in the first root parsed Properties block when present. Later properties and bullet properties SHALL retain the filename title. A normal page path whose relative spelling starts with `pages/contents.` SHALL yield `Contents` before considering properties.

#### Scenario: Title overrides the filename
- **WHEN** `pages/Title Override.md` has leading `title:: Chosen Name`
- **THEN** both formats yield `Chosen Name`

#### Scenario: Block title retains the filename
- **WHEN** `pages/Example.md` has a bullet containing a title property
- **THEN** the derived title is `Example`

#### Scenario: Contents overrides its title property
- **WHEN** `pages/contents.extra.md` has leading `title:: Different`
- **THEN** the derived title is `Contents`

### Requirement: Return explicit errors for unsupported inputs

The SDK SHALL return an error carrying the relative path for journal documents, non-normal or non-relative page paths, paths beneath `journals/`, non-Markdown paths, non-UTF-8 stems, and empty or whitespace-only resulting titles. A page path in any other folder of the garden, or at its root, SHALL take its title from the file name as a path beneath `pages/` does. Title extraction SHALL leave source and syntax unchanged and SHALL perform no filesystem writes.

#### Scenario: Journal requires date configuration
- **WHEN** a journal document is supplied
- **THEN** extraction returns a journal-specific unsupported-input error

#### Scenario: Page outside pages/
- **WHEN** a page document has the path `mise-tasks/microfreak/PARAMETERS.md`
- **THEN** extraction returns `PARAMETERS`

#### Scenario: Empty namespace title is unusable
- **WHEN** TripleLowbar receives `pages/______.md`
- **THEN** extraction returns an empty-title error

#### Scenario: Loaded documents remain unchanged
- **WHEN** page titles are extracted successfully or with errors
- **THEN** the document source, syntax, and relative path retain their original values

### Requirement: Spell a page that refers to itself as Logseq does

When a page's file refers to the page, the SDK SHALL title the page with the spelling Logseq's file parser gives it. A reference SHALL count when the file's blocks name the page as a page reference, a tag, a name in a property value, a namespace parent of another name, or a task marker or priority, with the lookup key of the title that the file name or `title::` property gives. The spelling SHALL be Logseq's own spelling when the page is one Logseq creates in every graph, such as `TODO`; otherwise the spelling of the first file Logseq reads that refers to the page, which may be the page's own file. Logseq SHALL be taken to read files in the order of `filter-files`: files whose path holds `journals/`, from the last path to the first, then files whose path holds `contents.`, then every other file by path. Within a file, the last block that refers to the page SHALL give the spelling; within one block, a namespace parent SHALL win over a name written in the text, and among names written in the text the first one SHALL win. A page whose file does not refer to the page SHALL keep the title from its file name or `title::` property. `load_garden` SHALL record the spelling in the document, so that `page_title` returns it for either filename format.

#### Scenario: An alias names the page in other letter case

- **WHEN** `pages/Solvespace.md` holds `alias:: [[SolveSpace]]`
- **THEN** the page is titled `SolveSpace`

#### Scenario: A file Logseq reads earlier spells the page first

- **WHEN** `journals/2024_01_01.md` writes `[[bar]]` and `pages/Bar.md` writes `[[BAR]]`
- **THEN** the page is titled `bar`

#### Scenario: A namespace parent names the page

- **WHEN** `pages/Qux.md` writes `[[qux/child]]`
- **THEN** the page is titled `qux`

#### Scenario: A task marker names one of Logseq's own pages

- **WHEN** `pages/Todo.md` holds the block `TODO do it`
- **THEN** the page is titled `TODO`

#### Scenario: Text without a reference leaves the title

- **WHEN** `pages/Word.md` holds the text `WORD` and no reference to the page, and a journal writes `[[word]]`
- **THEN** the page is titled `Word`
