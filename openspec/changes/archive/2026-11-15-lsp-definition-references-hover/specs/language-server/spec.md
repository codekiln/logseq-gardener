## ADDED Requirements

### Requirement: Go to definition

The server SHALL answer `textDocument/definition` from `GardenIndex::definition_at`. On a page link, tag, page embed or page name in a property value it SHALL return the location of the page's file at line 0, character 0, following aliases, namespace names and journal dates in any format Logseq accepts. On a block reference, block embed or `id::` value it SHALL return the location of the block's first line. For a page or journal without a file it SHALL return a location in the file Logseq would create, at line 0, character 0, and SHALL create no file. For a name several pages answer to it SHALL return one location per page. It SHALL return null in ordinary text and an empty list for a block id that no block writes and for a property key.

#### Scenario: Each kind of page link

- **WHEN** a client asks for the definition of `[[Fruit/Apple]]`, `[[fruit/apple]]`, the alias `[[Malus]]`, `#recipes`, `#[[multi word]]`, `[the pear page]([[Fruit/Pear]])`, `{{embed [[Fruit/Pear]]}}`, the bare name `Malus` in `alias::` and `[[Fruit]]` in `tags::` of the fixture garden
- **THEN** each answer is one location at the start of the file of the page the name leads to

#### Scenario: A journal date in another format

- **WHEN** the garden titles journals `MMM do, yyyy` and a client asks for the definition of `[[2026-01-03]]`
- **THEN** the answer is the start of `journals/2026_01_03.md`

#### Scenario: A page without a file

- **WHEN** a client asks for the definition of `[[Fruit/Quince]]`, which has no file, in a garden that uses `:triple-lowbar`
- **THEN** the answer is line 0, character 0 of `pages/Fruit___Quince.md`, and that file does not exist afterwards

#### Scenario: The legacy format and a journal file format of the garden's own

- **WHEN** `logseq/config.edn` holds only `:journal/file-name-format "yyyy-MM-dd"` and a client asks for the definitions of `[[Fruit/Quince]]`, `[[What is 50%?]]` and `[[Jan 4th, 2026]]`
- **THEN** the answers name `pages/Fruit%2FQuince.md`, `pages/What is 50%25%3F.md` and `journals/2026-01-04.md`

#### Scenario: A block reference and a block embed

- **WHEN** a client asks for the definition of `((66f1a2b0-0000-4000-8000-000000000004))` or of `{{embed ((66f1a2b0-0000-4000-8000-000000000002))}}`, with `utf-8` or `utf-16` positions
- **THEN** the answer is the range of the first line of the block that writes the id, from its dash to the end of the line

#### Scenario: Ordinary text

- **WHEN** a client asks for the definition at a position inside ordinary words
- **THEN** the answer is null

### Requirement: Find references

The server SHALL answer `textDocument/references` from `GardenIndex::subject_at` and `GardenIndex::references`, passing the request's `includeDeclaration`. On a link, tag, embed or page name in a property value the locations SHALL be every range that names the same page by its title or one of its aliases. On a block reference, block embed or `id::` value they SHALL be every block reference and block embed of the block. In ordinary text they SHALL be every range that names the page of the document. On a property key they SHALL be every line that writes the key. The text of an open buffer SHALL count as the text of its file.

#### Scenario: From a link that writes an alias

- **WHEN** a client asks for the references of `[[Malus]]`, an alias of `Fruit/Apple`, without the declaration
- **THEN** the locations are the ranges of `[[fruit/apple]]`, both `[[Fruit/Apple]]` links and `[[Malus]]`

#### Scenario: With the declaration

- **WHEN** the same request sets `includeDeclaration`
- **THEN** the list starts with the start of `pages/Fruit___Apple.md` and the names `Malus` and `[[Pomme]]` of its `alias::` property

#### Scenario: From ordinary text

- **WHEN** a client asks for references at a position inside the words of a block of `pages/Fruit___Pear.md`
- **THEN** the locations are the ranges that name `Fruit/Pear` or its alias `Pyrus`, including the labelled link and the page embed in `pages/Recipes.md`

#### Scenario: A block

- **WHEN** a client asks for the references of an `id::` value with the declaration
- **THEN** the list starts with the range of that value and continues with the block embed that names the id

#### Scenario: An unsaved edit

- **WHEN** an open buffer gains a link `[[Pomme]]` and a client asks for the references of `Fruit/Apple`
- **THEN** the list holds one more location than before the edit

### Requirement: Hover

The server SHALL answer `textDocument/hover` from `GardenIndex::summary_at` with Markdown content and the range of the occurrence under the pointer. It SHALL answer null in ordinary text and on a property key.

#### Scenario: A block embed

- **WHEN** a client hovers `{{embed ((66f1a2b0-0000-4000-8000-000000000002))}}` in the fixture garden
- **THEN** the content kind is `markdown`, the value starts with `**Fruit/Apple** › Keeps for months in a cool cellar.`, continues with the block's text `Store apart from [[Fruit/Pear]].` and ends with `1 reference`, and the range covers the whole embed

#### Scenario: A page link

- **WHEN** a client hovers `[[Malus]]`
- **THEN** the value shows `**Fruit/Apple**`, `Aliases: Malus, Pomme`, `4 references` and the page's first lines

#### Scenario: A page without a file

- **WHEN** a client hovers `[[Fruit/Quince]]`
- **THEN** the value names `pages/Fruit___Quince.md` as the file go to definition opens

#### Scenario: A block that is gone

- **WHEN** a client hovers a block reference whose id no block writes
- **THEN** the value is `No block has this id.`

## MODIFIED Requirements

### Requirement: Advertised capabilities

The `initialize` response SHALL advertise incremental text document synchronization with open and close notifications, the negotiated position encoding, and the definition, references and hover providers. It SHALL advertise no document formatting, range formatting or on-type formatting.

#### Scenario: Read the capabilities

- **WHEN** a client reads the `initialize` response
- **THEN** `textDocumentSync` asks for incremental changes, `definitionProvider`, `referencesProvider` and `hoverProvider` are true, and no formatting provider is present

### Requirement: Documented Neovim setup

The docs graph SHALL hold the page `lsg/How To/Edit a Garden in Neovim` with a Neovim configuration that starts `lsg lsp` for Markdown buffers inside a directory that holds `logseq/config.edn`, a LazyVim variant, and the keys that follow a link, list references and show a hover. `lsg/Ref/Command Line` SHALL link to the page. `mise run lsp:nvim-check` SHALL take the configuration from the page, run it in `nvim --headless --clean` against a copy of the fixture garden, and fail unless the client attaches to a page buffer, answers `lsg/status`, and `vim.lsp.buf.definition()` opens the file a fixture link leads to, for a page with a file and for a page without one. `mise run ci` SHALL run the task, with Neovim pinned in `mise.toml`.

#### Scenario: Run the documented configuration

- **WHEN** a contributor runs `mise run lsp:nvim-check`
- **THEN** headless Neovim attaches a client named `logseq_gardener` to a fixture page and the task exits with status 0

#### Scenario: The page changes

- **WHEN** the configuration on the docs page stops working
- **THEN** `mise run ci` fails at the Neovim check

#### Scenario: Follow a link in Neovim

- **WHEN** the task calls `vim.lsp.buf.definition()` with the cursor on `[[Malus]]` and then on `[[Fruit/Quince]]` in `pages/Recipes.md`
- **THEN** Neovim's current buffer becomes `pages/Fruit___Apple.md` and then `pages/Fruit___Quince.md`, and the copy of the garden holds no `pages/Fruit___Quince.md`
