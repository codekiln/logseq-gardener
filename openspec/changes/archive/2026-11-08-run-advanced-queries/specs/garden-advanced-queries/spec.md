# Spec Delta

## Purpose

Define how the SDK reads and runs an advanced query's Datalog `:query`, `:result-transform` and `:view` over the published pages while the site builds, so the publisher can write the result into the page with no script in the browser.

## ADDED Requirements

### Requirement: Read a Datalog subset over pages

The SDK SHALL read a `:query` of the form `[:find (pull ?p [*]) :where …]` whose clauses are data patterns over a page variable (`:block/name`, `:block/original-name`, `:block/properties`, each with an optional literal), binding clauses `[(get ?m :key) ?out]`, and predicate clauses `[(contains? ?s x)]`, `[(= a b)]` and `[(not= a b)]` over bound variables and literals. A clause that reads an unbound variable SHALL be a parse error. Any other attribute, clause, `:find` shape, `:in`, `:with`, `or`, `not` or rule SHALL be an unsupported-form error naming the form and its byte offset.

#### Scenario: Gitpa-form query
- **WHEN** the SDK reads a query with the clauses `[?p :block/name]`, `[?p :block/properties ?props]`, `[(get ?props :kind) ?kinds]`, `[(contains? ?kinds "Kind/A")]`, `[(get ?props :public) ?public]` and `[(= ?public true)]`
- **THEN** it returns a runnable query

#### Scenario: Unbound variable
- **WHEN** the SDK reads a query whose first clause is `[(get ?props :kind) ?kinds]`
- **THEN** it returns a parse error naming `?props`

#### Scenario: Block attribute
- **WHEN** the SDK reads a query with the clause `[?b :block/marker ?m]`
- **THEN** it returns an unsupported-form error naming `:block/marker`

### Requirement: Type page property values as Logseq does

The SDK SHALL give each page property a key and a typed value with Logseq's rules: the key is lower case with `_` read as `-`; a value with page references is a set of the names as written; `true` and `false` are booleans; an all-digit value is a number; a value in double quotes is the string inside the quotes; any other value is a string. A page row SHALL expose `:block/name` in lower case, `:block/original-name` and `:block/properties`, the properties of the page's leading properties block. Reading any other attribute SHALL be an unsupported-attribute error naming it.

#### Scenario: Boolean property
- **WHEN** a page has `public:: true` and another has `public:: "true"` and a third has `public:: false`
- **THEN** `(= ?public true)` matches the first page only

#### Scenario: Set property
- **WHEN** a page has `kind:: [[Kind/A]], [[Kind/B]]`
- **THEN** `(contains? ?kinds "Kind/B")` is true and `(contains? ?kinds "kind/b")` is false

#### Scenario: Other attribute
- **WHEN** a view reads `(:db/id row)`
- **THEN** the SDK returns an unsupported-attribute error naming `:db/id`

### Requirement: Evaluate the query over the published pages

The SDK SHALL return the published pages that satisfy every clause, in lower-case page-name order. A binding clause whose function returns nothing, and a predicate that returns false, SHALL drop the row. Withheld pages SHALL contribute no row.

#### Scenario: Missing property drops the row
- **WHEN** a page has no `kind` property and the query binds `(get ?props :kind) ?kinds`
- **THEN** the page is not in the result

#### Scenario: Stable page-name order
- **WHEN** three pages match and carry equal sort keys
- **THEN** they keep lower-case page-name order after a stable `sort-by`

### Requirement: Run result-transform and view functions in a closed language

The SDK SHALL read `:result-transform` and `:view` as functions in a closed subset of ClojureScript: `fn`, `if`, `let`, `and`, `or`, `not`, `when`, strings, keywords, numbers, booleans, `nil`, vectors, maps and sets; `get`, `get-in`, keywords as functions, `contains?`, `count`, `first`, `last`; `sort-by`, `sort`, `compare`, `reverse`, `take`, `drop`; `into`, `vec`, `vector`, `concat`, `map`, `filter`, `remove`, `str`; `=`, `not=`, `<`, `>`, `<=`, `>=`; and `clojure.string/replace`, `join`, `lower-case` and `upper-case`. Any other symbol SHALL be an unsupported-form error naming it. The subset SHALL have no definition, loop, recursion, interop, file access, process access or access to the date, the environment or a random source. `compare` SHALL order `nil` first, strings by UTF-16 code unit and numbers by value, and SHALL throw for a string against a number. `sort-by` SHALL be stable.

#### Scenario: Newest first
- **WHEN** a transform sorts rows by `(get-in row [:block/properties :published])` with the comparator `(fn [a b] (compare b a))` and the values are `2026-02-01T10:00:00Z`, `2026-03-01T10:00:00Z` and `2025-12-31T23:00:00Z`
- **THEN** the rows come back as March, February, December

#### Scenario: Zones sort as strings
- **WHEN** two values are `2026-01-01T00:30:00+02:00` and `2025-12-31T23:00:00Z`
- **THEN** the transform orders them as strings, as Logseq does, not by instant

#### Scenario: Unsupported function
- **WHEN** a view calls `(slurp "x")`
- **THEN** the SDK returns an unsupported-form error naming `slurp`

#### Scenario: Mismatched compare
- **WHEN** a transform compares the string `"a"` with the number `1`
- **THEN** the call throws and the SDK reports a thrown error, not a result

### Requirement: Return a view as a Hiccup tree

The SDK SHALL return a view's result as a tree of elements (tag, attributes, children) and text, holding no HTML. A vector whose first element is a string or a keyword SHALL be an element; nested sequences SHALL flatten; `nil` and `false` SHALL be skipped; numbers SHALL be text. A tag name with `.` or `#` shorthand SHALL be an unsupported-form error.

#### Scenario: Link list
- **WHEN** a view returns `["ul" ["li" ["a" {:href "#/page/x"} "X"]]]`
- **THEN** the tree holds a `ul` with one `li` holding an `a` with `href` `#/page/x` and the text `X`

#### Scenario: Mapped rows flatten
- **WHEN** a view returns `(into ["ul"] (map (fn [r] ["li" (get r :block/original-name)]) rows))` over two rows
- **THEN** the `ul` has two `li` children

### Requirement: Stop a query at its limits

The SDK SHALL stop a query after 20,000,000 evaluation steps, counting each evaluated form and each page-clause pair, and after 64 levels of nested calls, and SHALL report the limit it reached. Two runs over the same garden SHALL give the same result and the same step count.

#### Scenario: Step limit
- **WHEN** a function builds a collection whose processing needs more than 20,000,000 steps
- **THEN** the SDK stops it and reports the step limit

#### Scenario: Repeatable result
- **WHEN** the same query runs twice over the same garden with different `SOURCE_DATE_EPOCH` values
- **THEN** both runs return the same rows in the same order

### Requirement: Keep the evaluator free of garden-specific names

The SDK SHALL hold no page name, property name or entity type that belongs to one garden. The query SHALL decide which pages and keys matter.

#### Scenario: Renamed property and type
- **WHEN** a test copies the gitpa-form query with its property and entity type renamed, over fixture pages renamed the same way
- **THEN** the rows and their order are the same as for the original names

### Requirement: Offer the evaluator to every caller

The SDK SHALL provide one operation that takes the published pages and a parsed advanced query and returns the rows, the view's tree or the failure, with no HTML, so the publisher, the command line and the language server get the same result for the same garden.

#### Scenario: Rows without publishing
- **WHEN** a caller runs the operation on a loaded garden and a query without a `:view`
- **THEN** it receives the rows without publishing any file
