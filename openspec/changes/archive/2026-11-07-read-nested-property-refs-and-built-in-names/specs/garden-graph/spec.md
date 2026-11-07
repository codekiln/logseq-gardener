## MODIFIED Requirements

### Requirement: Include reference-only pages as nodes

A page that exists only because a published page refers to it SHALL be a node flagged reference-only, without a route. A reference-only node that names one of the pages Logseq creates in every graph (`NOW`, `LATER`, `DOING`, `DONE`, `CANCELED`, `CANCELLED`, `IN-PROGRESS`, `TODO`, `WAIT`, `WAITING`, `A`, `B`, `C`, `Favorites`, `Contents`, `card`) SHALL take that spelling as its title, whatever the case of the references that name it.

#### Scenario: Reference-only node

- **WHEN** a published page links `Contrando`, which has no file
- **THEN** `Contrando` is a node flagged reference-only with no route

#### Scenario: Built-in page spelled as in Logseq

- **WHEN** a published page writes `#c` and a later page writes `TODO [#C] write`, and neither `C` nor `TODO` has a file
- **THEN** the reference-only nodes are titled `C` and `TODO`
