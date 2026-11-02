## ADDED Requirements

### Requirement: Report the blocks whose bullet Logseq hides

The SDK SHALL report whether each outline block has no text, as Logseq's `block-content-empty?` decides: its first line is empty, each of its properties is one Logseq hides from blocks, and its body holds only horizontal rules and blank lines. The hidden properties SHALL be Logseq's built-in hidden keys and the card scheduling keys, compared lower-cased with spaces and underscores as hyphens. A heading line at the top level SHALL end the body, because Logseq starts a block there.

The SDK SHALL report that Logseq hides a block's bullet when the block has no text, has no children, and is neither the first nor the last block of the outline it belongs to. The first block SHALL be the first bullet only when no page properties, heading line or other text comes before it; the last block SHALL be the last top-level bullet only when no top-level heading line comes after it. A nested block SHALL have neither exception.

#### Scenario: Blocks with no text

- **WHEN** a page holds the blocks `-`, `- id:: 6add3ffb-e52b-4e8f-908a-9958186e4dfb`, `- ---` and `- kind:: note`
- **THEN** the SDK reports the first three as having no text and the fourth as having text

#### Scenario: First, last and nested blocks

- **WHEN** a page holds the top-level blocks `-`, `- a`, `-` and `-`, and `- a` has the child blocks `- b` and `-`
- **THEN** the SDK reports a hidden bullet for the third top-level block and for the nested `-`, and a bullet for the first and last top-level blocks

#### Scenario: Page properties come first

- **WHEN** a page starts with `tags:: x` and then the blocks `-` and `- a`
- **THEN** the SDK reports a hidden bullet for the `-` block
