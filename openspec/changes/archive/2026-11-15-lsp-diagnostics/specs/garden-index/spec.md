## ADDED Requirements

### Requirement: List the problems in a document

The index SHALL list the problems in one document. Each problem SHALL carry the source range it marks, a severity, a message, and the other places involved as a document path with a source range. The index SHALL report:

- a block reference or block embed whose id no block in the garden writes, as an error on the whole reference;
- an `id::` value that more than one block in the garden writes, as an error on the value at each of those blocks, with every other block that writes the id as a related location;
- a page embed whose page has no file, as a warning on the whole embed.

The list SHALL be in source order. A path the index holds no document for SHALL give an empty list.

#### Scenario: A reference to a block that is gone

- **WHEN** a page holds `((66f1a2b0-0000-4000-8000-00000000dead))` and no block writes that id
- **THEN** the page's problems hold one error whose range is the reference and whose message names the id

#### Scenario: An id written by two blocks

- **WHEN** a journal and a page each hold a block with `id:: 66f1a2b0-0000-4000-8000-000000000003`
- **THEN** each document's problems hold one error on its `id::` value, and each error's related location is the value in the other document

#### Scenario: An embed of a page without a file

- **WHEN** a page holds `{{embed [[Fruit/Quince]]}}` and no file gives a page that name or alias
- **THEN** the page's problems hold one warning whose range is the embed

#### Scenario: An embed of a page through its alias

- **WHEN** a page holds `{{embed [[Pyrus]]}}` and another page writes `alias:: Pyrus`
- **THEN** the embed is no problem

### Requirement: Pages without a file are no problem outside an embed

A page link, a tag or a property value that names a page without a file SHALL NOT appear among a document's problems. Text between `((` and `))` that is no block id SHALL NOT appear either.

#### Scenario: A link to a page that has no file yet

- **WHEN** a page holds `[[Fruit/Quince]]`, `#quince` and `tags:: Quince`, and no file gives a page any of those names
- **THEN** the page's problems are empty

### Requirement: Problems follow the current index

The problems of a document SHALL reflect the texts the index holds when the query runs, including editor buffers.

#### Scenario: The missing block appears in another document

- **WHEN** a page refers to an id no block writes, and an editor buffer of another page then adds a block with that id
- **THEN** the first page's problems no longer hold the error

#### Scenario: One of two blocks drops a shared id

- **WHEN** two documents write the same id and one of them changes its id
- **THEN** neither document's problems hold the duplicate error
