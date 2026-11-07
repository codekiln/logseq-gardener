## MODIFIED Requirements

### Requirement: Answer linked references from one shared index

The SDK SHALL provide a reference index over selected visible documents that the CLI, the language server and the publisher share. For a page it SHALL return the blocks whose references include the page, one of its aliases, or a namespace child of it, leaving out blocks on the page itself and counting each block once. A reference to `A/B/C` SHALL also count as a reference to `A/B` and `A`, even when `A/B/C` has no page; an alias name SHALL add no namespace parent. Page links, tags, embed page targets and page-reference property values SHALL supply references. A block that starts with a task marker, such as `TODO`, SHALL refer to the marker's page, and a block with a priority, such as `[#A]`, SHALL refer to the priority's page, as in Logseq. The value of an `alias::`, `aliases::` or `tags::` property, on any block, SHALL supply the names that the selected page aliases capability reads from an alias value, including each comma-separated plain name, and a quoted value SHALL supply none. A nested reference `[[Outer [[Inner]] Name]]` in the value of any property SHALL supply `Outer [[Inner]] Name` and no inner name, as in Logseq; in a block's text it SHALL supply both. The label of a link to a file, such as `[Report.md](../assets/Report.md)`, SHALL supply a reference in a block's text and none in a property value. Property names, literal code, comments, external links, pure block references, `{{query …}}` arguments and the text of `#+BEGIN_QUERY` blocks SHALL supply no references. A page's leading properties SHALL count as one block. Withheld sources SHALL contribute nothing.

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

#### Scenario: Task marker and priority
- **WHEN** a block reads `DONE [#B] [[tmux]]` and another reads `todo lower case`
- **THEN** the first block refers to `DONE`, `B` and `tmux`, and the second refers to no page

#### Scenario: Plain names in an alias value
- **WHEN** a page's leading properties hold `alias:: tmux, #[[AWS]]` and a block holds `alias:: Term, [[Outer [[mise]] Name]]`
- **THEN** the leading properties refer to `tmux` and `AWS`, and the block refers to `Term` and `Outer [[mise]] Name` but not to `mise`

#### Scenario: Nested reference in a property value
- **WHEN** a page's leading properties hold `type:: [[Notes on [[Mldoc]]]]` and a block's text holds `[[Body [[Nested]] Ref]]`
- **THEN** the leading properties refer to `Notes on [[Mldoc]]` but not to `Mldoc`, and the block refers to `Body [[Nested]] Ref` and `Nested`

#### Scenario: File link in a property value
- **WHEN** a page's leading properties hold `source-md:: [Report.md](../assets/Report.md)`
- **THEN** the leading properties refer to no page
