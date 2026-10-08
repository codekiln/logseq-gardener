## Purpose

Load Markdown garden sources while retaining document syntax, references, and original text.

## Requirements

### Requirement: Retained Markdown documents

The SDK SHALL load lowercase `.md` files beneath visible `pages/` and `journals/` directories and retain each original UTF-8 source, relative path, directory kind, parsed block structure, and parser-extracted page and block references. It SHALL leave source files unchanged and SHALL use lsdoc revision `32e63ef095c711d6d9947257bf5fd07d540fa59d`.

#### Scenario: Source and relationships

- **WHEN** a note contains nested outlines, page links, UUID references, literal code, CRLF, and trailing whitespace
- **THEN** the SDK retains the exact source and returns the pinned parser's structure and reference sets, excluding reference-like literal code

### Requirement: Deterministic note traversal

The SDK SHALL recurse through note directories, skip entries whose names start with `.`, and return documents sorted by relative path. Missing `pages/` or `journals/` directories SHALL be allowed when the other exists. A root without either directory SHALL produce an error. Ordinary non-Markdown files SHALL be ignored.

#### Scenario: Mixed garden

- **WHEN** the root has nested pages, journals, hidden directories, and ordinary non-Markdown files
- **THEN** only visible Markdown notes are returned in relative-path order on repeated loads

#### Scenario: Empty note directory

- **WHEN** the root contains an empty `pages/` directory and no journals
- **THEN** loading succeeds with no documents

#### Scenario: Invalid root

- **WHEN** the root is a file, missing, or lacks both note directories
- **THEN** loading fails with a path-bearing error

### Requirement: Visible load failures

The SDK SHALL reject visible symlinks and special files in note traversal, and SHALL stop with a path-bearing error on unreadable files or invalid UTF-8. It SHALL NOT return a partial garden after such a failure.

#### Scenario: Symlink input

- **WHEN** a root, note directory, or visible note entry is a symlink
- **THEN** loading fails without intentionally following the symlink

#### Scenario: Invalid note encoding

- **WHEN** a Markdown file is not valid UTF-8
- **THEN** loading fails and identifies the file
