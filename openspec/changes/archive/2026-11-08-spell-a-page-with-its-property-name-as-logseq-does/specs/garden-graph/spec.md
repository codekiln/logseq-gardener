## ADDED Requirements

### Requirement: Spell a page without a file as the last block that names it does

A page without a file SHALL take the spelling that Logseq gives it: the spelling of the last block, in the first file that refers to the page, that names it. A property name SHALL count as a reference to the page of that name, in lower case, and within one block it SHALL win over the names written in the block's text. A page that Logseq creates in every graph, such as `TODO`, SHALL keep Logseq's spelling.

#### Scenario: A block writes a reference and a property name
- **WHEN** a block has `- Uses [[Gadget]]` and the property `gadget:: 1`
- **THEN** the page reads `gadget`

#### Scenario: The property name is in another case
- **WHEN** a block has `- Uses [[PULLEY]]` and the property `Pulley:: 1`
- **THEN** the page reads `pulley`

#### Scenario: A later block has the property
- **WHEN** one block has `- Uses [[Sprocket]]` and a later block of the file has `sprocket:: 1`
- **THEN** the page reads `sprocket`

#### Scenario: A later block has the reference
- **WHEN** one block has the property `cog:: 1` and a later block of the file has `- Uses [[Cog]]`
- **THEN** the page reads `Cog`

#### Scenario: Two blocks write the name in different case
- **WHEN** one block has `- Uses [[Lever]]` and a later block of the file has `- Uses [[lever]]`
- **THEN** the page reads `lever`
