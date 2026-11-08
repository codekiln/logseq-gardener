## ADDED Requirements

### Requirement: Report the property names Logseq rejects

For each properties block of a loaded document, the SDK SHALL report the property names that Logseq's graph parser rejects, as Logseq writes them in its "Invalid property names" warning: lower-cased, with spaces and underscores turned into hyphens. Each name SHALL appear once, in the order the block first gives it. A properties block whose names Logseq all accepts SHALL report none.

#### Scenario: Code at the start of a block

- **WHEN** a page holds the block `` - `Logseq_Entity:: [[Asset]]` names the type. ``
- **THEN** the SDK reports the name `` `logseq-entity `` for the block's properties

#### Scenario: A name given twice

- **WHEN** a block holds the property lines `` `owner:: [[AWS]]` ``, `kind:: note` and `` `OWNER:: y ``
- **THEN** the SDK reports the name `` `owner `` once and leaves out `kind`
