## Why

`lsg` dates each page from the first and last commits that touched its file, and it read the history with renames turned off. A page whose file was renamed, or whose graph folder moved, lost the history before the rename: its created date became the rename commit. The move of the docs graph from `docs/` to `logseq-gardener-docs/` in [PR #566 — Rename the docs graph folder to logseq-gardener-docs](https://github.com/codekiln/logseq-gardener/pull/566) gave every docs page that commit as both its created and updated date ([Issue #563 — Keep page dates when a page file or graph folder is renamed](https://github.com/codekiln/logseq-gardener/issues/563)). Logseq renames a page's file when the page is renamed, so any garden can lose dates this way.

## What Changes

- A page's history continues across renames of its file and of the folders above it, including a move into the graph folder from another folder of the repository.
- A commit that only renames a file, without changing its text, leaves the page's created and updated dates as they were. A rename that also edits the page counts as a change to it.
- A page created at a path that another page was renamed away from starts its own history. Before, it inherited the older page's created date.
- Files whose names hold a double quote or a backslash now get dates. Git wrote those names in quotes, and the SDK did not match them to the page files.
- The SDK still reads the history with one `git log`, now over the whole repository with rename detection.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `page-list`: the requirement "Date pages from git history" follows renames and leaves dates unchanged by a rename alone.

## Impact

- `crates/logseq-gardener-sdk/src/git_dates.rs`: the history read and its tests.
- Published sites: All pages, reference sections and other views that show page dates show the dates from before a rename. On the encode garden, 757 pages get an earlier created date, 107 an earlier updated date, 2 a later created date and 7 get dates for the first time.
- Publication speed: on the encode garden the history read takes 2.1 seconds, down from 3.2, and it still finishes while pages render.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
