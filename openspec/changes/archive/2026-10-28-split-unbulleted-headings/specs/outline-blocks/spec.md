## ADDED Requirements

### Requirement: Start a block at every heading line

The SDK SHALL start a block at every heading line in a page's top-level content, whether or not the line has a bullet, as Logseq does. Text that follows a heading line, up to the next bullet or heading line, SHALL belong to the heading's block. Bullets indented under a heading line SHALL be its children. A heading inside a quote or other nested content SHALL stay part of its block.

#### Scenario: Heading line after a bulleted block
- **WHEN** a page has `- bulleted [[A]]`, then `# [[Filed]]` with no bullet, then a tab-indented `- child`, then `- next`
- **THEN** the SDK returns four blocks: `1` at depth 1, `2` at depth 1 with heading level 1, `2.1` at depth 2 under it, and `3` at depth 1; the first block's text excludes `[[Filed]]`

#### Scenario: Heading inside a quote
- **WHEN** a block contains a quote whose first line is `# quoted`
- **THEN** the quote stays part of that block and starts no block
