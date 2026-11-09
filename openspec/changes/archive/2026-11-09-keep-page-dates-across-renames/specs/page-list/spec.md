## MODIFIED Requirements

### Requirement: Date pages from git history

When the garden is a git repository, an entry for a page or journal with a file SHALL carry the commit times of the first and last commits that changed that file, read in one pass over the history. The history of a file SHALL continue across renames of the file and of the folders above it, including a rename from a folder outside the garden. A commit that renames a file without changing its text SHALL leave its dates unchanged. When the garden is not a git repository or `git` is unavailable, entries SHALL carry no dates and the SDK SHALL report one diagnostic. Generated namespace pages and reference-only pages SHALL carry no dates.

#### Scenario: Git dates

- **WHEN** `pages/mise.md` was added in one commit and last changed in a later one
- **THEN** its entry's created date is the first commit's time and its updated date is the later commit's time

#### Scenario: Renamed page file

- **WHEN** `pages/mise.md` was added at 1000 and edited at 2000, renamed to `pages/mise___Tasks.md` at 3000, and renamed to `pages/Tasks.md` with an edit at 4000
- **THEN** the entry for `pages/Tasks.md` has the created date 1000 and the updated date 4000

#### Scenario: Renamed graph folder

- **WHEN** the garden's folder moved from `docs/` to `garden-docs/` in one commit, and `docs/pages/lsg.md` was added at 1000 and last edited at 2000
- **THEN** the entry for `pages/lsg.md` in the garden `garden-docs/` has the created date 1000 and the updated date 2000

#### Scenario: New page at a renamed page's old path

- **WHEN** `pages/Podcast.md` was added at 1000 and renamed to `pages/Podcast___Episode.md` at 2000, and a new `pages/Podcast.md` was added at 3000
- **THEN** `pages/Podcast___Episode.md` has the created date 1000, and `pages/Podcast.md` has the created date 3000

#### Scenario: No repository

- **WHEN** the garden directory is not in a git repository
- **THEN** every entry has no dates and one diagnostic says why

#### Scenario: Stable across builds

- **WHEN** the same garden commit is built twice
- **THEN** both lists have the same dates and the same default order

#### Scenario: Reference-only page

- **WHEN** the list holds the reference-only page `Deno Deploy` in a git repository
- **THEN** its entry has no created or updated date
