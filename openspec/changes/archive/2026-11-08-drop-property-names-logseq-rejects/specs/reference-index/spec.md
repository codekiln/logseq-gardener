## ADDED Requirements

### Requirement: Read no references from a property Logseq rejects

The page names in the value of a property whose name Logseq rejects SHALL give the block no reference, as Logseq drops the property with its value. A name that the block's other text or another property's value holds SHALL still count.

#### Scenario: Code at the start of a block

- **WHEN** a block reads `` - `status:: [[Done]]` marks a finished task. `` and the garden has a page `Done`
- **THEN** the block is no linked reference of `Done`, and the garden graph draws no edge from the block's page to `Done`

#### Scenario: Name in the block's text

- **WHEN** a block reads `Read [[Done]] first` and holds the property line `` `status:: [[Done]]` ``
- **THEN** the block is a linked reference of `Done`
