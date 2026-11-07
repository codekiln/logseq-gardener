## ADDED Requirements

### Requirement: Report rejected property names as a diagnostic

Where the publisher shows Logseq's "Invalid property names" warning for a block's properties or a page's leading properties, it SHALL report one diagnostic for that properties block, under the file that holds it. The message SHALL be `Invalid property names: ` followed by the names the warning shows, in the same order, separated by `, `. A block shown in an embed, Linked References or query results SHALL report the diagnostic under its own file, once, and not at all when that file reports it for its own page. The diagnostic SHALL leave the exit status and the published files unchanged.

#### Scenario: A block that starts with a property in code

- **WHEN** a page holds the block `` - `Logseq_Entity:: [[Visible/Asset]]` identifies a local asset. ``
- **THEN** the report holds one diagnostic for that page's file, ``Invalid property names: `logseq-entity``

#### Scenario: Page properties with two rejected names

- **WHEN** a page's leading properties include the lines `` `owner_name:: x `` and `` `Kind:: y ``
- **THEN** the report holds one diagnostic for that page's file, ``Invalid property names: `owner-name, `kind``

#### Scenario: An embedded block

- **WHEN** page `Visible/Hub` embeds page `Visible/Asset`, whose block starts with `` `logseq-entity:: [[Visible/Asset]]` ``
- **THEN** the report holds the diagnostic once, under the file of `Visible/Asset`, and none under the file of `Visible/Hub`

#### Scenario: Encode garden

- **WHEN** the encode garden at `cbab08523e` is published
- **THEN** the report holds 14 such diagnostics on 12 pages, and at `9860577214`, after the blocks were rewritten, none
