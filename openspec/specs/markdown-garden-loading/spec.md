## Purpose

Load Markdown garden sources while retaining document syntax, references, and original text.

## Requirements

### Requirement: Retained Markdown documents

The SDK SHALL load lowercase `.md` files beneath every visible folder of the garden root, as Logseq does, and retain each original UTF-8 source, relative path, directory kind, parsed block structure, and parser-extracted page and block references. A file beneath `journals/` SHALL have the journal kind; every other file SHALL have the page kind. It SHALL leave source files unchanged and SHALL use lsdoc revision `32e63ef095c711d6d9947257bf5fd07d540fa59d`.

#### Scenario: Source and relationships

- **WHEN** a note contains nested outlines, page links, UUID references, literal code, CRLF, and trailing whitespace
- **THEN** the SDK retains the exact source and returns the pinned parser's structure and reference sets, excluding reference-like literal code

#### Scenario: Markdown outside the note directories

- **WHEN** the root holds `pages/a.md`, `journals/2026_10_28.md`, `tools-docs/PARAMETERS.md` and `assets/report.md`
- **THEN** the SDK returns all four documents, `journals/2026_10_28.md` with the journal kind and the other three with the page kind, and `tools-docs/PARAMETERS.md` derives the title `PARAMETERS`

### Requirement: Deterministic note traversal

The SDK SHALL recurse through the garden root and return documents sorted by relative path. It SHALL skip every path with a segment that starts with `.`, every path with a `node_modules` segment, and every path beneath `logseq/bak/`, `logseq/.recycle/` or `logseq/version-files/`. Missing `pages/` or `journals/` directories SHALL be allowed when the other exists. A root without either directory SHALL produce an error. Ordinary non-Markdown files SHALL be ignored.

#### Scenario: Mixed garden

- **WHEN** the root has nested pages, journals, hidden directories, `node_modules/`, `logseq/bak/`, and ordinary non-Markdown files
- **THEN** only visible Markdown files outside the skipped paths are returned in relative-path order on repeated loads

#### Scenario: Empty note directory

- **WHEN** the root contains an empty `pages/` directory and no journals
- **THEN** loading succeeds with no documents

#### Scenario: Invalid root

- **WHEN** the root is a file, missing, or lacks both note directories
- **THEN** loading fails with a path-bearing error

### Requirement: Visible load failures

The SDK SHALL reject visible symlinks and special files beneath `pages/` and `journals/`, and SHALL stop with a path-bearing error on unreadable files or invalid UTF-8. It SHALL skip symlinks and special files elsewhere in the root without following them. It SHALL NOT return a partial garden after a failure.

#### Scenario: Symlink input

- **WHEN** a root, note directory, or visible entry beneath a note directory is a symlink
- **THEN** loading fails without intentionally following the symlink

#### Scenario: Symlink outside the note directories

- **WHEN** a visible entry in another folder of the root is a symlink
- **THEN** loading succeeds and returns no document for it

#### Scenario: Invalid note encoding

- **WHEN** a Markdown file is not valid UTF-8
- **THEN** loading fails and identifies the file

### Requirement: Logseq hidden files

The SDK SHALL skip every Markdown file that an entry of `:hidden` in `logseq/config.edn` hides, using Logseq's rule: with a leading `/` added to the relative path and to the entry when either lacks one, the path starts with the entry. The rule SHALL apply beneath every folder, including `pages/` and `journals/`.

#### Scenario: Root file hidden

- **WHEN** `:hidden` holds `"README.md"` and `"../CLAUDE.md"`, and the root holds `README.md`, `CLAUDE.md` and `pages/README.md`
- **THEN** the SDK skips `README.md` and returns `CLAUDE.md` and `pages/README.md`

#### Scenario: Folder hidden

- **WHEN** `:hidden` holds `"worktrees"` and the root holds `worktrees/one/pages/a.md`
- **THEN** the SDK returns no document beneath `worktrees/`

### Requirement: Git-ignored files outside the note directories

When the garden root is inside a git work tree, the SDK SHALL skip Markdown files outside `pages/` and `journals/` that git ignores, so a local build loads what a fresh checkout holds. Files beneath `pages/` and `journals/` SHALL load whether or not git ignores or tracks them. When the root is not inside a git work tree, or `git` cannot run, no git filter SHALL apply.

#### Scenario: Ignored tool folder

- **WHEN** `.gitignore` lists `tools` and the root holds `tools/README.md`, `notes/kept.md` and an untracked `pages/draft.md`
- **THEN** the SDK returns `notes/kept.md` and `pages/draft.md` and skips `tools/README.md`

#### Scenario: No git repository

- **WHEN** the root is not inside a git work tree and holds `tools/README.md`
- **THEN** the SDK returns `tools/README.md`

### Requirement: Note directory pages win title collisions

When a file outside `pages/` and `journals/` derives the same page name as a document beneath them, the SDK SHALL keep the note-directory document, skip the other file, and report the skipped file's relative path.

#### Scenario: Same title in two folders

- **WHEN** the root holds `pages/PARAMETERS.md` and `mise-tasks/PARAMETERS.md`
- **THEN** the SDK returns `pages/PARAMETERS.md` and reports that `mise-tasks/PARAMETERS.md` was skipped
