## MODIFIED Requirements

### Requirement: Answer linked references from one shared index

The SDK SHALL provide a reference index over selected visible documents that the CLI, the language server and the publisher share. For a page it SHALL return the blocks whose references include the page, one of its aliases, or a namespace child of it, leaving out blocks on the page itself and counting each block once. A reference to `A/B/C` SHALL also count as a reference to `A/B` and `A`, even when `A/B/C` has no page; an alias name SHALL add no namespace parent. Page links, tags, embed page targets and page-reference property values SHALL supply references; property names, literal code, comments, external links, pure block references, `{{query …}}` arguments and the text of `#+BEGIN_QUERY` blocks SHALL supply none. A page's leading properties SHALL count as one block. Withheld sources SHALL contribute nothing.

#### Scenario: Namespace parent
- **WHEN** a block links to `[[mise/Config]]`
- **THEN** the index lists that block among the linked references of both `mise/Config` and `mise`

#### Scenario: Encode garden count
- **WHEN** the index is built over the whole encode garden
- **THEN** `mise` has 589 linked-reference blocks

#### Scenario: Withheld source
- **WHEN** a private page links to a published page
- **THEN** the published page's linked references, related pages and counts include nothing from it

#### Scenario: Page named only inside an advanced query
- **WHEN** a parity fixture page holds a `#+BEGIN_QUERY` block whose `:query` is `[[Query Target]]` and no other page links to `Query Target`
- **THEN** `Query Target` has no linked references
