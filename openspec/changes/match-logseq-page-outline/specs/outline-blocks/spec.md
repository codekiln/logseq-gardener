## ADDED Requirements

### Requirement: Describe a page's blocks

The SDK SHALL return, for one page's parsed blocks, every outline block in document order with its depth (1 at the top level), its parent, whether it has children, its task marker, its priority letter, its heading level and its authored collapsed state. The task markers SHALL be TODO, DOING, NOW, LATER, WAIT, WAITING, IN-PROGRESS, DONE and CANCELED, with CANCELLED read as CANCELED. The heading level SHALL be the number of leading `#` characters clamped to 1–6; `heading:: true` SHALL give the block's depth plus one, capped at 6, and `heading:: <n>` for n from 1 to 6 SHALL give n. A block SHALL be authored collapsed only when its attached properties set `collapsed:: true` and it has children.

#### Scenario: Nested headings and tasks
- **WHEN** a page has a top-level `# Setup` block with a child `TODO [#A] install` block and a grandchild `DONE read docs` block
- **THEN** the SDK returns three blocks at depths 1, 2 and 3, the first with heading level 1, the second with marker TODO and priority A, the third with marker DONE, each child naming its parent

#### Scenario: Heading property
- **WHEN** a top-level block has `heading:: true` and a block at depth 6 has `heading:: true`
- **THEN** their heading levels are 2 and 6

#### Scenario: Collapsed leaf
- **WHEN** a block without children has `collapsed:: true`
- **THEN** the SDK reports it as expanded

### Requirement: Identify blocks stably

The SDK SHALL give every outline block an identifier: its `id::` UUID in lower case when the UUID is valid, otherwise its path of 1-based positions among its siblings at each level, written with dots (`3.1.2`). The SDK SHALL parse either written form, find the block an identifier names, and return a block's ancestors from the nearest to the top level. Building the same source twice SHALL give the same identifiers.

#### Scenario: Position path
- **WHEN** a block is the second child of the third top-level block and has no `id::`
- **THEN** its identifier is `3.2`, and finding `3.2` returns it with the third top-level block as its only ancestor

#### Scenario: UUID identifier
- **WHEN** a block has `id:: 6ADD3FFB-E52B-4E8F-908A-9958186E4DFB`
- **THEN** its identifier is `6add3ffb-e52b-4e8f-908a-9958186e4dfb`, and finding either case returns it

#### Scenario: Unknown identifier
- **WHEN** an identifier names no block, or is neither a valid UUID nor a dotted path
- **THEN** finding it returns nothing
