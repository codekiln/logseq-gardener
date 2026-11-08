## ADDED Requirements

### Requirement: Show Logseq's warning for rejected property names

Where a block or a page has property names that Logseq rejects, the publisher SHALL show Logseq's warning in place of their properties: a `div.invalid-properties.mb-2` holding a `div.warning` titled "Invalid properties" with the text "Invalid property names: " and a `button.p-1.mr-2` for each name the SDK reports, followed by a `code` element with Logseq's text "Property name begins with a non-numeric character and can contain alphanumeric characters and . * + ! - _ ? $ % & = < >. If -, + or . are the first character, the second character (if any) must be non-numeric." The warning SHALL use Logseq's styles in both themes: the warning colours, a 7px bottom margin, buttons with 3.5px padding, a 7px right margin and no border or background, and the code text wrapping as written. The buttons SHALL have no action, as in Logseq. The styles SHALL come from the site's shared stylesheet, and pages SHALL load no script for the warning.

#### Scenario: Asset page of the encode garden

- **WHEN** a reader opens `Logseq/Entity/Asset` published from the encode garden
- **THEN** the block under "Frontmatter and body" shows no text and the warning with one button, `` `logseq-entity ``, with the same markup as Logseq's build

#### Scenario: Encode garden

- **WHEN** the encode garden is published
- **THEN** the warning appears on the same 13 blocks of the same 11 pages as in Logseq's build, with the same names, and on no other block

#### Scenario: Page properties

- **WHEN** a page's leading properties include the line `` `owner_name:: x ``
- **THEN** the published page shows the warning with the button `` `owner-name `` before its outline
