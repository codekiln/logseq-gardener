## ADDED Requirements

### Requirement: Keep the spaces inside a page reference's brackets

The reference index SHALL name the page a written reference names as Logseq's file parser does: the name between the brackets, with any spaces at its ends kept. A name with a space at either end SHALL count for no page whose title lacks that space, and neither a garden page file nor a generated namespace page spelled without the space SHALL answer to it. The name's namespace parents SHALL keep the spaces at their ends as well.

#### Scenario: Space before the closing brackets

- **WHEN** a block links `[[Release Notes ]]` and the garden has a page `Release Notes`
- **THEN** the block is a linked reference of `Release Notes ` and not of `Release Notes`, and adds nothing to the backlink count of `Release Notes`

#### Scenario: Space after the opening brackets

- **WHEN** a block links `[[ Notes/Release Notes]]` and the garden has the pages `Notes/Release Notes` and `Notes`
- **THEN** the block is a linked reference of ` Notes/Release Notes` and of its parent ` Notes`, and of neither `Notes/Release Notes` nor `Notes`

#### Scenario: Name without spaces at its ends

- **WHEN** a block links `[[Release Notes]]` and the garden has a page `release notes`
- **THEN** the block is a linked reference of `release notes`, as before
