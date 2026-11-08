# Spec Delta

## MODIFIED Requirements

### Requirement: Show Logseq's three query errors

A simple query that does not parse SHALL replace the box with a bordered block error headed `Query Error:` above the query text in a code box. An advanced map that does not read SHALL show the same box headed `Invalid query:` above the block text. A query that parses but cannot run, including a form outside the Datalog subset or the `:result-transform` subset, `:inputs`, `:rules` or a limit reached by the query or the transform, SHALL keep the header and show `.warning` `Query failed:` followed by a message that names the form or the limit. A `:view` that holds a form outside the subset, throws, writes a tag, attribute or URL the publisher does not allow, reaches a limit or produces more than 128 KB of HTML SHALL show `Custom view failed:` in the same way. A `:result-transform` that throws SHALL show the untransformed results. A simple-language query with a `:view` SHALL show `Custom view failed:` saying that block results do not support functions at build time. Each case SHALL also be a build diagnostic with the page and line.

#### Scenario: Parse error
- **WHEN** a page contains `{{query (and [[tmux]]}}`
- **THEN** it shows `Query Error:` with `(and [[tmux]]` in a code box, and the build lists one diagnostic

#### Scenario: Datalog over blocks
- **WHEN** a page contains a `#+BEGIN_QUERY` whose `:query` reads `:block/marker`
- **THEN** the box shows its header and a `Query failed:` line naming `:block/marker`, and the build lists one diagnostic with the line

#### Scenario: Unsupported function in a view
- **WHEN** a `:view` calls `(slurp "x")`
- **THEN** the box shows `Custom view failed:` naming `slurp`

#### Scenario: Unsupported function in a transform
- **WHEN** a `:result-transform` calls `(partition 2 rows)`
- **THEN** the box shows `Query failed:` naming `partition`, and no rows

#### Scenario: Thrown transform
- **WHEN** a `:result-transform` compares a string with a number
- **THEN** the box shows the untransformed rows and the build lists one diagnostic

#### Scenario: Disallowed element
- **WHEN** a `:view` returns `["script" "x"]`
- **THEN** the box shows `Custom view failed:` naming `script`, and the page holds no `script` element from the view

## ADDED Requirements

### Requirement: Show a view's result in the query box

An advanced query with a Datalog `:query` and a `:view` SHALL show its title, `N results` for the rows left after the transform, and the view's Hiccup written as HTML in the body, with text and attribute values escaped and no table or list toggle. An empty result SHALL call no view and show no count and `No matched result`. A Datalog query without a view SHALL show its rows as the page table. A collapsed query SHALL load the view's HTML from a fragment when opened. The page SHALL link no script and no stylesheet that a page without the query lacks.

#### Scenario: Episode list
- **WHEN** the fixture page holds a gitpa-form query with a transform that sorts on a property newest first and a view that returns a list of links, over 18 matching pages
- **THEN** the box shows the title, `18 results` and a `ul` of 18 links in newest-first order, with no toggle

#### Scenario: Escaped text
- **WHEN** a page's original name is `A <b>&</b>`
- **THEN** the view's link text shows those characters literally

#### Scenario: Empty result
- **WHEN** no page matches
- **THEN** the box shows the title, no count and `No matched result`

#### Scenario: No script
- **WHEN** a page holds the gitpa-form query
- **THEN** the page links the same scripts and stylesheets as the page without the query

### Requirement: Link a view's page links to published pages

An `href` of the form `#/page/<name>` in a view SHALL be percent-decoded and replaced by the published URL of the page it names when that page is published, and SHALL stay as written otherwise. A link in a view SHALL create no reference, no backlink and no search record.

#### Scenario: Slash in a name
- **WHEN** a view links `#/page/Tools%2FQueries%2FTyped%2Fone`
- **THEN** the written link is the published URL of `Tools/Queries/Typed/one`

#### Scenario: Unpublished page
- **WHEN** a view links `#/page/Withheld`
- **THEN** the link stays `#/page/Withheld`

#### Scenario: No backlink
- **WHEN** a view links to a page
- **THEN** that page's Linked References gain nothing from the view

### Requirement: Match Logseq's advanced query output

A fixture page with an advanced query SHALL produce the same rows in the same order as Logseq 0.10.15 in `mise run parity:logseq-export`, for each supported function and property type, including `public:: true`, `public:: false`, `public:: "true"`, a missing property, a two-name set and two timestamps in different zones.

#### Scenario: Typed fixture
- **WHEN** `parity:logseq-export` compares the `Tools/Queries` fixture page
- **THEN** the lists from `lsg` and from Logseq hold the same links in the same order

#### Scenario: Gitpa's homepage
- **WHEN** gitpa's `Ghost in the Patch` is built with plain `lsg publish`
- **THEN** it lists the same public episodes in the same order as `mise run site:query:check` in gitpa

### Requirement: Keep advanced queries within budgets

The `[advanced_queries]` table in `scripts/parity/budgets.toml` SHALL hold these budgets: a page with the gitpa-form query over 18 rows grows by at most 3 KB gzip against the same page without it; no script and no stylesheet are added; a view's HTML is at most 128 KB; a query stops at 20 million steps within 2 seconds; the all-pages query on a copy of the whole encode garden evaluates in at most 500 ms; and a whole-garden build is at most 10% slower than the `--baseline-lsg` binary. The page SHALL stay within the `[desktop]` and `[phone]` first-paint, heap and search budgets. `parity:budgets` in `mise run ci` SHALL check the size budgets on the fixture garden, `parity:perf --advanced-queries` SHALL measure `lsg` and a Logseq build of the same garden copy and record the rows in the Performance page, and `parity:publish-bench --check` SHALL run in CI on a `gitpa-sized` corpus whose homepage holds the gitpa-form query.

#### Scenario: Size budgets in CI
- **WHEN** `parity:budgets` runs on the fixture garden
- **THEN** it fails when the gitpa-form page exceeds its growth budget, links a script, or holds a view over 128 KB

#### Scenario: Whole-garden worst case
- **WHEN** `parity:perf --advanced-queries` publishes the encode garden copy with the all-pages query
- **THEN** the query evaluates in at most 500 ms, the build is within 10% of the baseline, and the page's phone heap and per-keystroke search latency stay within the `[phone]` budgets

#### Scenario: Compared with Logseq
- **WHEN** the same copy is built by Logseq and by `lsg`
- **THEN** the Performance page records, for each build, the time to the title, bytes transferred, script bytes, phone heap and search latency for the query pages

#### Scenario: Slower build
- **WHEN** a change makes the `gitpa-sized` build slower than its recorded rows allow
- **THEN** the publication-speed check in CI fails
