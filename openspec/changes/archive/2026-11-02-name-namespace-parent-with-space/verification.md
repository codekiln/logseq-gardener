# Verification: name-namespace-parent-with-space

Logseq's parser check now agrees with the SDK on namespace parents with a space before the slash. Checked on 2026-11-01 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` (before) and from this branch (after).

## SDK tests

`site_references::a_space_before_a_slash_keeps_the_reference_off_the_trimmed_parent` publishes the page `Notes/MacOS`, a page `Notes/Child` that links `[[Notes/MacOS/Child]]`, and a page that links `[[Notes/MacOS /Applications Folder]]`. On `origin/main` the page `Notes/MacOS` shows `2 Linked References`; now it shows `1 Linked Reference`, from `Notes/Child` only.

`references::tests::a_namespace_parent_keeps_the_space_before_its_slash` and `graph::tests::a_namespace_parent_keeps_the_space_before_its_slash` use operations that `origin/main` lacks, so they hold the fix in place rather than reproduce the bug.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `scripts/logseq-parser/fixtures/namespace-parent-space`: `[[MacOS /Applications Folder]]` | 5 differences | agree: 1 page, 1 block |
| Case graph: `[[MacOS /Applications Folder]]`, `[[foo ]]`, `[[ bar]]`, `[[A / B / C]]`, `#[[Tag /X]]`, and the pages `MacOS` and `Linux /Bin` | 28 differences | 9 differences |
| `scripts/logseq-parser/fixtures/page-reference-space`: `[[Release Notes ]]` | – | 4 differences, all about `release notes` and `release notes `, as `test_check.py` expects |
| `docs` | agree | agree: 89 pages, 4492 blocks |
| Encode garden | 1043 differences | 1016 differences |

The nine differences left on the case graph are eight about `[[foo ]]` and `[[ bar]]`, which Issue #421 tracks, and the reference-only node `linux ` for the page `Linux /Bin`, which no block names. The SDK makes a node for the namespace parent of every page; Logseq's side of the check lists only pages that a block names, and the check reports a page `Plain/Child` the same way on `origin/main`.

The 27 encode garden differences that go away are all namespace parents with a space before the slash, on these pages:

- `[[Claude/Code/Q/If I rename a CLI session with /rename, …]]` in `journals/2026_08_10.md`, `journals/2026_08_12.md` and `pages/Claude___Code___Ref___Session Storage.md`, and the namespace of that page.
- `[[Claude/Code/Q/Is there a Claude Code CLI slash command …, and why doesn't /diff show the diff content?]]` in `journals/2026_07_03.md`, and the namespace of that page.
- `[[Obsidian/Q/In Obsidian, … Info / Warning / Danger blocks called?]]` in `journals/2026_09_08.md`, and the namespace of that page.
- `[[MacOS /Applications Folder]]` in `pages/Mac___OS___App.md` and `pages/Codex___Desktop___MacOS___Bug___No Windows on Finder Launch___26___02___28 Sat.md`.

The check reports no new difference. One difference about these references remains: Logseq lists `macos ` as a page without a file, and the SDK makes no node for it, because `MacOS /Applications Folder` is an alias of `Mac/OS/App` and an alias adds no namespace parent to the graph. `origin/main` reports the same line.

## Publishing the encode garden

`lsg publish --journals --filename-format triple-lowbar` with one `--include` for each top-level name:

| | Before | After |
| --- | --- | --- |
| Pages | 8926 | 8926 |
| Journals | 513 | 513 |
| Diagnostics | 12496 | 12496 |
| Wall time | 19.4 s | 20.2 s |

Eleven files change, all about the generated namespace page `MacOS`, which the two pages that link `[[MacOS /Applications Folder]]` no longer refer to:

- `MacOS` shows `70 Linked References` in place of `72`, and its page graph loses the two pages.
- The linked references of `Mac/OS/App` and of the journal `2026-02-28 Sat` no longer offer `MacOS` as a filter for those blocks, and the page graphs of `Mac/OS/App` and `Codex/Desktop/MacOS/Bug/No Windows on Finder Launch/26/02/28 Sat` lose the edge to `MacOS`.
- The page list shows 70 backlinks for `MacOS` in place of 72, and `lsg-site.json` changes its revision.

The references to the three question pages do not change, because each parent with a trailing space still names the generated namespace page spelled with that space.
