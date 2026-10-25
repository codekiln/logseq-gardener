## ADDED Requirements

### Requirement: Read the bracket setting

The SDK SHALL read `:ui/show-brackets?` from `logseq/config.edn`. The setting SHALL be off only when the key is the boolean `false`; a missing key, a commented-out key or any other value SHALL leave brackets on, as in Logseq. A value other than a boolean SHALL produce a diagnostic.

#### Scenario: Default
- **WHEN** `logseq/config.edn` has no `:ui/show-brackets?` key, or has it only in a comment
- **THEN** the SDK reports brackets on

#### Scenario: Turned off
- **WHEN** `logseq/config.edn` sets `:ui/show-brackets? false`
- **THEN** the SDK reports brackets off

#### Scenario: Wrong type
- **WHEN** `logseq/config.edn` sets `:ui/show-brackets? "no"`
- **THEN** the SDK reports brackets on and returns one diagnostic naming the key

### Requirement: Name the page a tag points at

The SDK SHALL return, for a parsed tag, the page name it points at: the tag's text without `#`, and for `#[[multi word]]` the text inside the brackets. Resolving that name SHALL use the same page-name lookup as page references, so titles, aliases, case and journal dates resolve identically for both.

#### Scenario: Simple and bracketed tags
- **WHEN** a block contains `#Keyshort` and `#[[multi word]]`
- **THEN** the SDK names the pages `Keyshort` and `multi word`

#### Scenario: Tag and reference agree
- **WHEN** page `Keyshort` declares `alias:: shortcut`, and one block contains `#Shortcut` and another `[[shortcut]]`
- **THEN** both resolve to page `Keyshort`
