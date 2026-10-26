## ADDED Requirements

### Requirement: Answer hierarchy rows from one shared index

The SDK SHALL provide a namespace index over selected visible page titles and their aliases that the CLI, the language server and the publisher share. It SHALL be built from one entry per page, so adding, removing or renaming one page replaces only that entry. For a page it SHALL return the subject of its hierarchy and its rows, or nothing:

- The subject SHALL be the page itself when its title has a `/` or another page's title starts with its title followed by `/`; otherwise the first of its aliases, in authored order, whose name has a `/`; otherwise there SHALL be no hierarchy.
- When the subject has descendants, the rows SHALL be every descendant at every depth, matched on whole `/`-separated segments, without the subject's own row.
- When the subject has no descendants, the rows SHALL be one row, the subject's parent path.
- Rows SHALL be sorted by lower-cased full name.
- Each row SHALL be split on `/` into segments, each carrying the label from the row's own spelling and the lower-cased cumulative prefix it leads to.

A caller SHALL be able to leave out descendant rows for namespace prefixes that have no page; the subject choice and the parent row SHALL be unaffected by that option. Withheld titles SHALL contribute no rows and no prefixes.

#### Scenario: Descendants without the own row
- **WHEN** `tmux/Config` and `tmux/Config/local` are pages
- **THEN** the rows for `tmux/Config` are `tmux/Config/local` alone

#### Scenario: Leaf shows its parent
- **WHEN** `tmux/Config/local` has no descendants
- **THEN** its rows are the single row `tmux/Config`

#### Scenario: Plain page
- **WHEN** `Abbreviation` has no `/`, no descendants and no namespaced alias
- **THEN** it has no hierarchy

#### Scenario: Namespaced alias
- **WHEN** `AGI` declares the alias `AI/General/Intelligence` and that name has no descendants
- **THEN** the subject is `AI/General/Intelligence` and the rows are the single row `AI/General`

#### Scenario: Sibling with a shared prefix
- **WHEN** `A/B/C` and `A/B/Cousin` are pages
- **THEN** `A/B/Cousin` is not a row of `A/B/C`

#### Scenario: Prefixes without a page
- **WHEN** `tmux/Command/kill-session` is a page, `tmux/Command` has no page, and the caller leaves out rows without a page
- **THEN** the rows for `tmux` include `tmux/Command/kill-session` and not `tmux/Command`, and the row's `Command` segment still leads to `tmux/command`

#### Scenario: Labels keep the row's spelling
- **WHEN** `Tools/Code` and `tools/code/Rust` are pages
- **THEN** the row `tools/code/Rust` has the labels `tools`, `code` and `Rust`

#### Scenario: Replace one page
- **WHEN** a page is renamed from `A/B` to `A/C` and its entry is replaced
- **THEN** the rows for `A` hold `A/C` and not `A/B`, and no other entry is rebuilt
