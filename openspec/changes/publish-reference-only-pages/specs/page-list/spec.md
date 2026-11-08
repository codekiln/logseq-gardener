## MODIFIED Requirements

### Requirement: List published pages with their attributes

The SDK SHALL list every published page, journal, generated namespace page and reference-only page once, each with its title, route, kind (`Page`, `Journal`, `Namespace` or `Reference`), `icon::` property value, backlink count, lower-cased sort key, and created and updated dates. The operation SHALL be available to the CLI, the language server and the publisher, and SHALL leave garden files unchanged.

#### Scenario: Kinds

- **WHEN** a garden publishes `mise`, the journal `2026-10-23 Fri` and the generated namespace page `tmux/Command`
- **THEN** the list holds one entry for each, of kind `Page`, `Journal` and `Namespace`

#### Scenario: Icon

- **WHEN** a page has the property `icon:: 🧭`
- **THEN** its entry's icon is `🧭`

#### Scenario: Reference-only pages

- **WHEN** a garden publishes the reference-only page `Deno Deploy` and the reference-only journal `Nov 18th, 2024`
- **THEN** the list holds `Deno Deploy` of kind `Reference` and `Nov 18th, 2024` of kind `Journal`, each once

### Requirement: Date pages from git history

When the garden is a git repository, an entry for a page or journal with a file SHALL carry the commit times of the first and last commits that touched that file, read in one pass over the history. When the garden is not a git repository or `git` is unavailable, entries SHALL carry no dates and the SDK SHALL report one diagnostic. Generated namespace pages and reference-only pages SHALL carry no dates.

#### Scenario: Git dates

- **WHEN** `pages/mise.md` was added in one commit and last changed in a later one
- **THEN** its entry's created date is the first commit's time and its updated date is the later commit's time

#### Scenario: No repository

- **WHEN** the garden directory is not in a git repository
- **THEN** every entry has no dates and one diagnostic says why

#### Scenario: Stable across builds

- **WHEN** the same garden commit is built twice
- **THEN** both lists have the same dates and the same default order

#### Scenario: Reference-only page

- **WHEN** the list holds the reference-only page `Deno Deploy` in a git repository
- **THEN** its entry has no created or updated date
