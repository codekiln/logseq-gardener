## ADDED Requirements

### Requirement: Parse Logseq queries in the SDK

The SDK SHALL parse the simple query language of `{{query …}}`, with the macro's arguments joined by `, `, and the EDN map of a `#+BEGIN_QUERY` block, after expanding `<% today %>`, `<% yesterday %>` and `<% tomorrow %>` against the build date. The map parser SHALL read `:title`, `:query`, `:collapsed?`, `:group-by-page?`, `:remove-block-children?`, `:table-view?` and `:breadcrumb-show?`, and SHALL record whether `:inputs`, `:rules`, `:view` and `:result-transform` are present. A `:query` that is a vector containing `:find` SHALL be classed as Datalog; any other `:query` SHALL be parsed as the simple language. A simple query that does not parse SHALL be a parse error, and a map that does not read SHALL be an invalid-map error, each with the byte offset of the problem.

#### Scenario: Simple query
- **WHEN** the SDK parses `(and [[tmux]] (not [[oh-my-tmux]]))`
- **THEN** it returns an `and` of a page reference to `tmux` and a `not` of a page reference to `oh-my-tmux`

#### Scenario: Unbalanced simple query
- **WHEN** the SDK parses `(and [[tmux]]`
- **THEN** it returns a parse error

#### Scenario: Datalog map
- **WHEN** the SDK parses a `#+BEGIN_QUERY` map whose `:query` is `[:find (pull ?b [*]) :where …]`
- **THEN** it classes the query as Datalog and keeps the map's `:title`

### Requirement: Evaluate simple queries over the reference index

The SDK SHALL evaluate a simple query over the selected visible documents with Logseq's meaning: `[[page]]` and `#tag` match blocks whose path references include the page or one of its aliases; `and`, `or` and `not` combine; `"text"` matches block source, case-sensitive; a bare word matches nothing; `task` and `todo` match task markers; `priority` matches priorities; `between` matches blocks on journals dated in the inclusive range, with `today`, `yesterday`, `tomorrow`, a journal page reference, or ±N `d`, `w`, `m` or `y` from the build date; `property` matches block properties; `page` matches blocks on a page; `page-property`, `page-tags`, `all-page-tags` and `namespace` return pages; `sort-by` orders the results; `sample` keeps N results chosen by a generator seeded with the query's block id. A query whose forms all return pages SHALL return pages; otherwise it SHALL return blocks. Any other form SHALL be an unsupported-form error naming the form. Withheld sources SHALL contribute nothing.

#### Scenario: Path references
- **WHEN** the SDK evaluates `(and [[yazi]] [[card]])` over the encode garden at commit `083106d`
- **THEN** it returns 18 blocks on 13 pages

#### Scenario: Page results
- **WHEN** the SDK evaluates `(page-tags [[CLI/Tool]])` over the same garden
- **THEN** it returns 60 pages

#### Scenario: Relative dates
- **WHEN** the SDK evaluates `(between -30d today)` over the same garden with a build date of 2026-10-27
- **THEN** it returns 76 blocks

#### Scenario: Text
- **WHEN** the SDK evaluates `"lightweight database"` over the same garden
- **THEN** it returns 1 block

#### Scenario: Unsupported form
- **WHEN** the SDK evaluates `(between created-at -7d today)`
- **THEN** it returns an unsupported-form error naming `between created-at`

### Requirement: Order and trim query results like Logseq

The SDK SHALL leave the query's own block out of its results and, unless the query sets `:remove-block-children? false`, SHALL drop any result whose parent block is also a result. Block results SHALL keep outline order within a page. When grouped by page, journal pages SHALL come first, newest first, then the other pages by page name.

#### Scenario: Journal order
- **WHEN** the SDK evaluates `[[tmux]]` over the encode garden at commit `083106d` and groups the results by page
- **THEN** the first three groups are the journals for 2026-09-19, 2026-09-15 and 2026-09-11

#### Scenario: Own block
- **WHEN** a block containing `{{query [[tmux]]}}` itself sits under a block tagged `[[tmux]]`
- **THEN** the query's own block is absent from its results

### Requirement: Resolve "today" from the build date

The query evaluator SHALL use the build date that the journals feed uses: the UTC date of `SOURCE_DATE_EPOCH` when set, otherwise of the system clock.

#### Scenario: Reproducible builds
- **WHEN** the same commit is built twice with the same `SOURCE_DATE_EPOCH`
- **THEN** every query, including those with `between` and `sample`, returns the same results in the same order

### Requirement: Share one evaluator across tools

The query parser and evaluator SHALL be SDK operations that hold no HTML, callable by the publisher, the CLI, the language server and the flashcard deck selection with the same results for the same garden and build date.

#### Scenario: Deck selection
- **WHEN** the flashcard deck for `{{cards [[yazi]]}}` is selected with the evaluator as `(and [[yazi]] [[card]])`
- **THEN** it holds the same 18 blocks the query returns
