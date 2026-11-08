## ADDED Requirements

### Requirement: Spell an alias name as the first file that refers to it does

An alias name that is no page's title SHALL take the spelling that Logseq gives it: the spelling of the first file in Logseq's read order that refers to the name. A file refers to the name when it lists it as an alias, writes a reference to it, or has a property of that name. Within that file, the last block that refers to the name SHALL give the spelling. A name that Logseq creates in every graph, such as `todo`, SHALL take Logseq's spelling, `TODO`.

#### Scenario: A block refers to the alias in another case
- **WHEN** `Run` has `alias:: [[Runtime]]` and `Notes` has the block `- Uses [[runtime]]`
- **THEN** the alias name reads `runtime`

#### Scenario: The alias file is read first
- **WHEN** `Run` has `alias:: [[Runtime]]` and `Zed` has the block `- Uses [[runtime]]`
- **THEN** the alias name reads `Runtime`

#### Scenario: Files disagree
- **WHEN** `Notes` has `- Uses [[runtime]]` and `Other` has `- Uses [[RUNTIME]]`
- **THEN** the alias name reads `runtime`, the spelling of the first file Logseq reads

#### Scenario: A file refers to its own alias twice
- **WHEN** `Gear` has `alias:: [[Gadget]]` and two blocks `- Uses [[gadget]]` and `- Uses [[GADGET]]`
- **THEN** the alias name reads `GADGET`

#### Scenario: A property has the name
- **WHEN** `Run` has `alias:: [[Runtime]]` and `Talk`, read first, has the property `runtime:: 1`
- **THEN** the alias name reads `runtime`

#### Scenario: Names in one alias value
- **WHEN** a page has `alias:: [[homebrew]], [[Homebrew]]` and no other file refers to the name
- **THEN** the alias name reads `Homebrew`
