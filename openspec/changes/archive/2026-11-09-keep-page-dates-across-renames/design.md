## Context

`GitDates::read` in `crates/logseq-gardener-sdk/src/git_dates.rs` ran `git log --format=%ct --name-only --no-renames --relative -- pages journals` in the garden folder and took each file's oldest and newest commit times. The publisher starts the read on its own thread and waits for it, in the `history` phase, only after pages render. On the encode garden the read took 3.2 seconds and the wait was zero.

Two limits of that command cause [Issue #563 — Keep page dates when a page file or graph folder is renamed](https://github.com/codekiln/logseq-gardener/issues/563):

- `--no-renames` lists a rename as a deletion of the old path and an addition of the new one, so the new path's history starts at the rename.
- The pathspec `pages journals` covers only the garden's current folder. When the docs graph moved from `docs/pages/` to `logseq-gardener-docs/pages/`, the old paths were outside it, so even rename detection could not pair them.

## Goals / Non-Goals

**Goals:**

- A page's dates survive a rename of its file and a move of the graph folder.
- The history is still read with one `git log`, and the read still finishes while pages render on the encode garden.

**Non-Goals:**

- Following a file that was copied, or one whose text changed so much in the rename commit that git no longer pairs the two paths. Those files start their history at the rename, as before.

## Decisions

- Read the whole repository's history with rename detection: `git log --topo-order --format=%ct --name-status -z --find-renames --no-relative`. Without a pathspec, git can pair a page with its old path in any folder. `-z` gives file names as they are; before, git quoted names that hold a double quote or a backslash, and those pages had no dates. `--no-relative` keeps paths relative to the repository root even when the user's git configuration sets `diff.relative`. `--topo-order` lists every commit before its parents, which the rename tracking needs, and costs nothing measurable.
- Find the garden's place in the repository and whether the clone is shallow with one `git rev-parse --is-inside-work-tree --is-shallow-repository --show-prefix`. The SDK then keeps only paths under that prefix's `pages/` and `journals/`. This replaces two `rev-parse` calls.
- Track renames while reading commits newest first. A map takes an older path to the current path whose history it holds. After a commit renames `a` to `b`, older commits that touch `a` count for the file now at the end of that chain, and older commits that touch `b` belong to a file that no longer exists. The renames of one commit apply together after the commit is read, so a commit that renames `b` to `c` and `a` to `b` keeps both histories apart.
- Leave the dates unchanged for a rename with identical text (`R100`). A folder move then keeps every date, and a Logseq page rename that rewrites the page's text still counts as an update.
- Take each file's created date as the oldest commit time and its updated date as the newest, rather than relying on output order.
- Keep all of this in the SDK's `git_dates` module. The publisher and the page list read the dates through `GitDates::get` as before.

## Risks / Trade-offs

- [A garden in a small folder of a large repository reads that repository's whole history] → On this repository the docs garden's read takes 0.1 seconds, up from 0.03. A large monorepo would pay more; a pathspec cannot be used, because it would hide the old folder of a moved graph.
- [Commits that git lists out of the order the rename map assumes, such as an edit to the old path on another branch merged after the rename] → The edit counts for the old path, so the page's updated date can miss it. Before this change, all of the old path's history was lost.
- [Rename detection has a limit on the files it compares in one commit] → Renames with identical text, such as a folder move, are found without that limit.

## Resolved Questions

### 1 - Should a commit that only renames a page change its updated date?

No. A rename with identical text leaves both dates as they were, so the folder move in PR #566 keeps the dates every docs page had before. A rename that changes the text, as Logseq's page rename does when the page has a `title::` property, counts as an update.

### 2 - Which file does the history at a renamed page's old path belong to?

To the renamed page. Older commits at the old path count for the page at its new path. A new page created later at the old path starts its own history. Before this change, the new page took the renamed page's created date.

## Open Questions

None.
