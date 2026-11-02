## ADDED Requirements

### Requirement: Keep the space before a namespace parent's slash

The reference index SHALL name a namespace parent as Logseq's file parser does: the name before the last `/`, with any space before that slash kept. A parent that ends in a space SHALL count for no page whose title lacks that space, and no page file SHALL answer to it.

#### Scenario: Reference with a space before its slash

- **WHEN** a block links `[[MacOS /Applications Folder]]` and the garden has a page `MacOS`
- **THEN** the block is a linked reference of `MacOS /Applications Folder` and not of `MacOS`

#### Scenario: Page with a space before its slash

- **WHEN** the garden has the pages `MacOS` and `MacOS /Applications Folder`
- **THEN** `MacOS /Applications Folder` is not a namespace child of `MacOS`, and adds nothing to its backlink count
