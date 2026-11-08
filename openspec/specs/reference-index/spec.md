## Purpose

Define the SDK operations that answer a page's linked references, path references, related-page frequencies, starting filters, tagged pages and unlinked mentions from one shared index, so the CLI, the language server and the publisher give the same answers.

## Requirements

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

### Requirement: Answer filter questions

The index SHALL return each referring block's path references (the block, its ancestors and its page) and, for a set of blocks, the related pages with counts, most frequent first and ties by title, leaving out the viewed page: each block whose parent is not in the set adds its path references once and each descendant's references, and a block whose parent is in the set adds nothing. It SHALL return a page's starting filter from its `filters::` property: names mapped to `true` are includes, names mapped to `false` are excludes, and names that do not resolve to a published page are dropped.

#### Scenario: Related pages follow the blocks
- **WHEN** the related pages are requested for the `mise` blocks that refer to `Diataxis/How To`
- **THEN** `Diataxis` with 4 and `AWS` with 2 lead the list

#### Scenario: Starting filter
- **WHEN** a page has `filters:: {"diataxis/how to" true}`
- **THEN** its starting filter includes `Diataxis/How To` and excludes nothing

### Requirement: Answer tagged pages and unlinked mentions

The index SHALL return the pages whose `tags::` name a page or its alias, sorted by lower-cased title. It SHALL return the blocks whose text mentions the page's title or an alias, case-insensitively, where the text before the match is not a letter, digit, `#` or `[[` and the text after it is not a letter or digit, ignoring logbooks and leaving out blocks on the page itself. It SHALL find the mentions of every page in one pass over the block text.

#### Scenario: Tagged pages
- **WHEN** the tagged pages of `Term` are requested
- **THEN** 178 pages are returned, starting with `1Password/Dev/CLI/Secret Reference` and `Abbreviation`

#### Scenario: Mention inside a link
- **WHEN** the unlinked mentions of `tmux` are requested
- **THEN** a block linking `[[My/AI/Rule/Dev/Workflow/Git Worktree PR/Tmux]]` is included, and a block containing `#tmux` or `tmuxinator` alone is not

### Requirement: Rebuild per changed document

The index SHALL be built from one contribution per document, so that replacing one document's contribution gives the same answers as rebuilding the whole index.

#### Scenario: One file changes
- **WHEN** one document's references change and only its contribution is replaced
- **THEN** every answer equals the answer of a full rebuild

### Requirement: Keep the space before a namespace parent's slash

The reference index SHALL name a namespace parent as Logseq's file parser does: the name before the last `/`, with any space before that slash kept. A parent that ends in a space SHALL count for no page whose title lacks that space, and no page file SHALL answer to it.

#### Scenario: Reference with a space before its slash

- **WHEN** a block links `[[MacOS /Applications Folder]]` and the garden has a page `MacOS`
- **THEN** the block is a linked reference of `MacOS /Applications Folder` and not of `MacOS`

#### Scenario: Page with a space before its slash

- **WHEN** the garden has the pages `MacOS` and `MacOS /Applications Folder`
- **THEN** `MacOS /Applications Folder` is not a namespace child of `MacOS`, and adds nothing to its backlink count

### Requirement: Keep the spaces inside a page reference's brackets

The reference index SHALL name the page a written reference names as Logseq's file parser does: the name between the brackets, with any spaces at its ends kept. A name with a space at either end SHALL count for no page whose title lacks that space, and neither a garden page file nor a generated namespace page spelled without the space SHALL answer to it. The name's namespace parents SHALL keep the spaces at their ends as well.

#### Scenario: Space before the closing brackets

- **WHEN** a block links `[[Release Notes ]]` and the garden has a page `Release Notes`
- **THEN** the block is a linked reference of `Release Notes ` and not of `Release Notes`, and adds nothing to the backlink count of `Release Notes`

#### Scenario: Space after the opening brackets

- **WHEN** a block links `[[ Notes/Release Notes]]` and the garden has the pages `Notes/Release Notes` and `Notes`
- **THEN** the block is a linked reference of ` Notes/Release Notes` and of its parent ` Notes`, and of neither `Notes/Release Notes` nor `Notes`

#### Scenario: Name without spaces at its ends

- **WHEN** a block links `[[Release Notes]]` and the garden has a page `release notes`
- **THEN** the block is a linked reference of `release notes`, as before

### Requirement: Read no references from a property Logseq rejects

The page names in the value of a property whose name Logseq rejects SHALL give the block no reference, as Logseq drops the property with its value. A name that the block's other text or another property's value holds SHALL still count.

#### Scenario: Code at the start of a block

- **WHEN** a block reads `` - `status:: [[Done]]` marks a finished task. `` and the garden has a page `Done`
- **THEN** the block is no linked reference of `Done`, and the garden graph draws no edge from the block's page to `Done`

#### Scenario: Name in the block's text

- **WHEN** a block reads `Read [[Done]] first` and holds the property line `` `status:: [[Done]]` ``
- **THEN** the block is a linked reference of `Done`
