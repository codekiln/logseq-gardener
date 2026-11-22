# Spec Delta

## ADDED Requirements

### Requirement: Labeled wikilinks fixed before commit

A commit SHALL rewrite each labeled wikilink `[[Target|Label]]` in a staged graph page to `[[Target]]` and stage the rewritten page. Text inside inline code or a fenced code block SHALL stay unchanged.

#### Scenario: Commit a page with a labeled wikilink

- **WHEN** a contributor commits a page in `logseq-gardener-docs/pages/` that contains `[[lsg/Ref/Command Line|the CLI reference]]`
- **THEN** the committed page contains `[[lsg/Ref/Command Line]]` instead

#### Scenario: Commit a page that shows the syntax in code

- **WHEN** a contributor commits a page whose only labeled wikilink is inside backticks
- **THEN** the committed page keeps the backticked text unchanged
