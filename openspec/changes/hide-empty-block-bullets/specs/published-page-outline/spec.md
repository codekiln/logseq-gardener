## ADDED Requirements

### Requirement: Hide the bullet of a block with no text

Where the SDK reports that Logseq hides a block's bullet, the publisher SHALL add the class `lsg-empty` to the block's list item, in a page's outline, in an embed and in a Linked References copy. The shared stylesheet SHALL hide that block's bullet while the pointer is not over the block's row and show it while the pointer is over the row, as Logseq does. The block SHALL keep its bullet element, so the outline script and View as slides still find the block. Pages SHALL load no script for this.

#### Scenario: Asset page of the encode garden

- **WHEN** a reader opens `Logseq/Entity/Asset` published from the encode garden
- **THEN** the block under "Frontmatter and body" that shows Logseq's "Invalid property names" warning has no visible bullet, and its bullet appears while the pointer is over the block

#### Scenario: Encode garden

- **WHEN** the encode garden is published
- **THEN** the blocks without a visible bullet are the blocks without a visible bullet in Logseq's build of the same garden
