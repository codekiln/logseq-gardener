## ADDED Requirements

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
