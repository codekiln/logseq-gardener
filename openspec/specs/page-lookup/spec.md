## Purpose

Find the source file of any page or journal in a Markdown garden by title, alias or journal date, through one SDK catalog and the read-only `lsg lookup` command.

## Requirements

### Requirement: Whole-garden page catalog

The SDK SHALL provide a page catalog of every page under `pages/` and every journal under `journals/`, without namespace selection or visibility checks. Each entry SHALL be findable by its title, each leading `alias::` name, and, for a journal, its date. Names SHALL match with the same lookup key and journal-date fallback as published link resolution. The catalog SHALL return every matching page in source-path order with its title, relative source path, kind, and whether the name matched its title, an alias or a journal date. A page whose title cannot be derived, or a journal whose file name is not a date, SHALL be left out with a diagnostic naming the file.

#### Scenario: Find a page by title or alias

- **WHEN** a garden has `pages/Tools___Neovim.md` with `alias:: nvim, [[vim]]` and a caller looks up `tools/neovim`, `NVIM` or `vim` with the triple-lowbar format
- **THEN** each lookup returns that file, title `Tools/Neovim`, and match type title, alias and alias

#### Scenario: Find a private page

- **WHEN** a page has `public:: false`
- **THEN** a lookup by its title returns it

#### Scenario: Find a journal by date

- **WHEN** a garden has `journals/2025_02_01.md` with the default journal formats and a caller looks up `Feb 1st, 2025` or `2025-02-01`
- **THEN** both return that journal, with match type title and journal date respectively

#### Scenario: Several pages share a name

- **WHEN** two pages both declare `alias:: shared`
- **THEN** a lookup of `Shared` returns both in source-path order

#### Scenario: A file without an identity

- **WHEN** a journal file name is not a date in the journal file-name format
- **THEN** the catalog omits it, reports a diagnostic naming the file, and every other page stays findable

### Requirement: Page lookup command

`lsg lookup NAME` SHALL require `--graph PATH` and `--filename-format legacy|triple-lowbar`, read the garden without changing it, and print each matching page's path, `--graph` joined with the relative path, one per line. JSON output SHALL contain `format_version` 1, `command_path` [lookup], `query`, `diagnostic_count`, and `pages`, each with `title`, `path`, `relative_path`, `kind` (`page` or `journal`) and `matched_by` (`title`, `alias` or `journal-date`). Catalog diagnostics SHALL go to stderr. When more than one page matches, stderr SHALL state the count. When no page matches, or the garden cannot be read, the command SHALL return one with empty stdout and an error on stderr. A missing name, `--graph` or `--filename-format` SHALL return two.

#### Scenario: Print a path for an editor

- **WHEN** a caller runs `lsg lookup nvim --graph G --filename-format triple-lowbar` on a garden where `nvim` is an alias of `pages/Tools___Neovim.md`
- **THEN** stdout is `G/pages/Tools___Neovim.md` followed by a newline and the exit is zero

#### Scenario: No page matches

- **WHEN** no title, alias or journal date matches the name
- **THEN** the exit is one, stdout is empty, and stderr names the query

#### Scenario: Missing inputs

- **WHEN** the name, `--graph` or `--filename-format` is missing
- **THEN** the exit is two with empty stdout and a diagnostic naming the missing input

#### Scenario: A page named help

- **WHEN** a caller runs `lsg lookup --graph G --filename-format legacy -- help` on a garden with `pages/help.md`
- **THEN** stdout is that page's path

#### Scenario: Source files stay unchanged

- **WHEN** a lookup completes
- **THEN** the garden's files and directory entries are unchanged

### Requirement: Offline lookup help

The root help SHALL list `lookup` as a child command, and `lsg lookup` SHALL provide equivalent help, `--help` and `-h` plus outline, section and programmatic guidance without reading a garden. Lookup help SHALL describe the matching rules, required options, the `--` form for a name such as `help`, output fields, exits, permissions and limitations.

#### Scenario: Discover lookup offline

- **WHEN** a caller recursively traverses JSON help with closed stdin
- **THEN** lookup documentation is discoverable and equivalent through each help alias
