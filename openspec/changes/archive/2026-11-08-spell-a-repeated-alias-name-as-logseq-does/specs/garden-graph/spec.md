## ADDED Requirements

### Requirement: Spell a repeated alias name with its last spelling

When one `alias::` value writes the same name more than once in different letter case, the SDK SHALL spell the alias with the last of those spellings, as Logseq does. The alias SHALL keep the position of its first occurrence.

#### Scenario: Alias written in two cases
- **WHEN** a page has `alias:: [[homebrew]], [[Homebrew]]`
- **THEN** the alias name reads `Homebrew`, and the page's alias list holds it once

#### Scenario: Position of a repeated alias
- **WHEN** a page has `alias:: [[b]], [[a]], [[B]]`
- **THEN** the aliases read `B`, `a` in that order
