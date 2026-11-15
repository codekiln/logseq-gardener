## ADDED Requirements

### Requirement: Advertise completion

The `initialize` response SHALL advertise a completion provider with the trigger characters `[`, `#`, `(` and `:`.

#### Scenario: Read the capabilities

- **WHEN** a client reads the `initialize` response
- **THEN** `completionProvider.triggerCharacters` is `[`, `#`, `(` and `:`

### Requirement: Answer completion requests

For `textDocument/completion` the server SHALL read the open buffer's current text, or the indexed file's text when the document is not open, find the completion context at the position, and answer with the index's candidates for it. It SHALL answer with an empty list where there is no context and for a document it has no text for.

Each item SHALL carry:

- a `label`: the page name, alias, block text, property key or property value;
- a `textEdit` whose range is the context's replaced range in the negotiated position encoding and whose text is the candidate's inserted text;
- a `filterText` that the typed text matches: the name for a page, alias, key or value, the block's text for a block, and the typed text itself for a journal offered because the typed text spells its date;
- a `sortText` that orders the items by rank;
- a `kind`, and a description naming the page for an alias and for a block.

The server SHALL return at most 100 items for one request and SHALL set `isIncomplete` when the index reports a partial list.

#### Scenario: A page link with an auto-inserted closing pair

- **WHEN** a buffer holds `- [[Fru]]` with the cursor after `Fru` and the client asks for completion
- **THEN** the item `Fruit/Apple` has a text edit that replaces `Fru]]` with `Fruit/Apple]]`

#### Scenario: Each trigger

- **WHEN** a client asks for completion after `[[`, `#`, `((`, at a property key and after `key::`
- **THEN** the answers hold page names, page names, blocks, property keys and property values

#### Scenario: An alias item

- **WHEN** a client asks for completion after `[[Pom`
- **THEN** the item `Pomme` names `Fruit/Apple` in its description and inserts `Pomme]]`

#### Scenario: A buffer the garden has no file for

- **WHEN** a client opens a new, empty buffer under `pages/`, types `[[Fru` and asks for completion
- **THEN** the answer holds `Fruit/Apple`

#### Scenario: Prose

- **WHEN** a client asks for completion at the end of `- An apple is red`
- **THEN** the answer holds no items
