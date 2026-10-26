## Purpose

Define the SDK operations that resolve a block reference to the content Logseq shows for it, resolve an embed to its target outline, and apply the nesting limit references and embeds share, so the CLI, the language server and the publisher read them the same way.

## Requirements

### Requirement: Resolve a block reference to its display content

The SDK SHALL resolve a block reference to the content Logseq shows for it: the target block's first line as inline nodes, its task marker and its heading level. Properties, the rest of the block's body and its children SHALL be left out, except that a block whose first line is empty SHALL show its body without properties, as Logseq does. The CLI, the language server and the publisher SHALL read reference content through this operation. The target index SHALL contain only selected visible pages, as the selected block navigation spec requires.

#### Scenario: Task with a page reference
- **WHEN** a reference on `Claude/Code` points at a `DONE` block whose first line holds a page reference
- **THEN** the content has the `DONE` marker and the page reference as an inline node

#### Scenario: Block with properties and children
- **WHEN** the target block has properties, a second line and child blocks
- **THEN** the content holds only the first line

#### Scenario: Heading block
- **WHEN** the target block is a heading
- **THEN** the content carries its heading level

### Requirement: Register quote bullets as reference targets

The SDK SHALL register a bullet whose content is a quote and that carries a valid `id::` as a reference target. Its display content SHALL be the quote, because the first line is empty and Logseq then shows the block's body.

#### Scenario: Reference to a quote bullet
- **WHEN** `Person/Steve Yegge/Agent/Bee - Wheelhouse Crew Seat` references a bullet `- > …` with `id::` on the next line
- **THEN** the reference resolves to the quote as its display content, with no diagnostic

### Requirement: Resolve an embed to its target outline

The SDK SHALL resolve a block embed to the target block and its descendants, and a page embed to the page's top-level blocks with their descendants, from selected visible pages only. A page embed SHALL report the page's display title and route. The SDK SHALL report an unknown or withheld block, a page with no published file, and a page that embeds itself or an enclosing embed as distinct outcomes.

#### Scenario: Block embed
- **WHEN** `Spotify/Tool/TUI` embeds a heading block with children
- **THEN** the outline holds the block and its children in source order

#### Scenario: Self embed
- **WHEN** a page embed's target is the page itself or a page already being embedded
- **THEN** the outcome is a self embed with the page's title and no outline

### Requirement: Apply Logseq's shared nesting limit

The SDK SHALL count references and embeds on one nesting counter and SHALL report that the limit is reached when a reference or embed would be the sixth level, as Logseq's limit of 5 does. The outcome SHALL say whether an embed or a reference reached the limit.

#### Scenario: Seven nested embeds
- **WHEN** a test garden chains seven page embeds
- **THEN** the first five expand and the sixth reports that the embed limit is reached
