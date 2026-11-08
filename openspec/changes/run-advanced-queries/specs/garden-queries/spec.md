# Spec Delta

## MODIFIED Requirements

### Requirement: Parse Logseq queries in the SDK

The SDK SHALL parse the simple query language of `{{query …}}`, with the macro's arguments joined by `, `, and the EDN map of a `#+BEGIN_QUERY` block, after expanding `<% today %>`, `<% yesterday %>` and `<% tomorrow %>` against the build date. The map parser SHALL read `:title`, `:query`, `:collapsed?`, `:group-by-page?`, `:remove-block-children?`, `:table-view?` and `:breadcrumb-show?`, SHALL read `:result-transform` and `:view` as functions in the subset that `garden-advanced-queries` defines, and SHALL record whether `:inputs` and `:rules` are present. A `:query` that is a vector containing `:find` SHALL be read as the Datalog subset; any other `:query` SHALL be parsed as the simple language. A simple query that does not parse SHALL be a parse error, a map that does not read SHALL be an invalid-map error, and a form outside the Datalog or function subset SHALL be an unsupported-form error, each with the byte offset of the problem.

#### Scenario: Simple query
- **WHEN** the SDK parses `(and [[tmux]] (not [[oh-my-tmux]]))`
- **THEN** it returns an `and` of a page reference to `tmux` and a `not` of a page reference to `oh-my-tmux`

#### Scenario: Unbalanced simple query
- **WHEN** the SDK parses `(and [[tmux]]`
- **THEN** it returns a parse error

#### Scenario: Datalog map
- **WHEN** the SDK parses a `#+BEGIN_QUERY` map whose `:query` is `[:find (pull ?p [*]) :where [?p :block/name]]` with a `:view` function
- **THEN** it returns a runnable Datalog query with its view function and keeps the map's `:title`

#### Scenario: Form outside the subset
- **WHEN** the SDK parses a map whose `:query` is `[:find (pull ?b [*]) :where [?b :block/marker ?m]]`
- **THEN** it returns an unsupported-form error naming `:block/marker`
