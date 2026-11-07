## MODIFIED Requirements

### Requirement: Include reference-only pages as nodes

A page that exists only because a published page refers to it SHALL be a node flagged reference-only, without a route. A name a block writes SHALL bring in its namespace parents as reference-only nodes whether or not the name itself opens a page, including when the name is an alias of a page. A reference-only node that names one of the pages Logseq creates in every graph (`NOW`, `LATER`, `DOING`, `DONE`, `CANCELED`, `CANCELLED`, `IN-PROGRESS`, `TODO`, `WAIT`, `WAITING`, `A`, `B`, `C`, `Favorites`, `Contents`, `card`) SHALL take that spelling as its title, whatever the case of the references that name it.

#### Scenario: Reference-only node

- **WHEN** a published page links `Contrando`, which has no file
- **THEN** `Contrando` is a node flagged reference-only with no route

#### Scenario: Built-in page spelled as in Logseq

- **WHEN** a published page writes `#c` and a later page writes `TODO [#C] write`, and neither `C` nor `TODO` has a file
- **THEN** the reference-only nodes are titled `C` and `TODO`

#### Scenario: Namespace parents of an alias

- **WHEN** `pages/AGI.md` holds `alias:: [[AI/General/Intelligence]]`, and neither `AI/General` nor `AI` has a file
- **THEN** `AI/General` and `AI` are reference-only nodes, `AI/General` has an edge to `AI`, and the page graph of a page that links `AI/General/Intelligence` holds both
