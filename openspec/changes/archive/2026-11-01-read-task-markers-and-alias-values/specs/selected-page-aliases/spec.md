## MODIFIED Requirements

### Requirement: Extract leading page aliases

The SDK SHALL extract aliases from the last case-insensitive `alias` property in the first parsed properties block, reading the value as Logseq's file parser reads `alias::`, `aliases::` and `tags::` values. Each direct page reference `[[Name]]`, tag `#name` and tagged reference `#[[Name]]` SHALL supply one name, and a nested reference `[[Outer [[Inner]] Name]]` SHALL supply `Outer [[Inner]] Name` and no inner name. Plain text between them SHALL supply its comma-separated parts, split on ASCII or full-width commas. Quoted values SHALL remain one literal name including quotes. Code and formatted spans SHALL not contribute their inner text as names; a nonempty value with no parsed names SHALL remain a complete literal name. Nested properties and `aliases` SHALL not provide page aliases. Blank and self aliases SHALL be omitted.

#### Scenario: Mixed alias syntax
- **WHEN** a leading alias value contains `Plain, [[Comma, Name]]，Other` and a later outline contains another alias property
- **THEN** the SDK returns `Plain`, `Comma, Name` as one reference name, and `Other`, without the nested alias

#### Scenario: Tagged aliases
- **WHEN** a leading alias value is `#[[Tag Alias]], #short, plain`
- **THEN** the SDK returns `tag alias`, `short` and `plain`, as Logseq does

#### Scenario: Nested reference in an alias
- **WHEN** a leading alias value is `[[Outer [[Inner]] Name]]`
- **THEN** the SDK returns the one alias `outer [[inner]] name`, as Logseq does

#### Scenario: Literal and repeated properties
- **WHEN** a page contains duplicate alias properties and the last value is a quoted reference-like string
- **THEN** the last value remains one literal alias including its quotes and contributes no inner reference
