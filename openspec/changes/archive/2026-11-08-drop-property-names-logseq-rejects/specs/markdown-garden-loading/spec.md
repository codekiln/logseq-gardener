## ADDED Requirements

### Requirement: Keep only the property names Logseq accepts

A loaded document SHALL hold each block property and page property whose name Logseq's graph parser accepts, and no other. A name SHALL count as accepted when its first EDN token reads as a keyword name, any text after that token is EDN whitespace, it holds none of `"`, `|`, `^`, `(`, `)`, `{` and `}`, and it does not start with `#`. A block whose properties are all rejected SHALL keep its place in the outline.

#### Scenario: Code at the start of a block

- **WHEN** a page holds the block `` - `status:: [[Done]]` marks a finished task. ``
- **THEN** the loaded block has no property, and the garden graph has no node named `` `status ``

#### Scenario: Rejected and accepted names in one block

- **WHEN** a block holds the lines `` `owner:: [[AWS]]` `` and `kind:: [[Term]]` as its properties
- **THEN** the loaded block has the property `kind` only
