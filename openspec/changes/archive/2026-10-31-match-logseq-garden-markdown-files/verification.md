# Verification: match-logseq-garden-markdown-files

`lsg` now publishes every page Logseq's `gh-pages` build publishes from the encode garden's files: the whole-garden comparison has no "Other pages" and no "Only in lsg" pages. The whole-garden build is 1% slower than `main`, inside the 5% `[loading]` budget.

Measured on 2026-10-31 with `lsg` built from this branch (based on `main` c7beffe).

## Whole-garden comparison (task 3.1)

`mise run parity:build --garden <checkout>` and `mise run parity:compare` on a checkout of encode garden commit 9aa484f, one commit before 5095bdf, from which the compared `gh-pages` build (9bbeffe) was deployed:

| Group | Pages |
| --- | --- |
| Journals | 30 |
| Reference-only pages (no content blocks) | 6646 |
| Hidden by :hidden in logseq/config.edn | 0 |
| Titles lsg cannot select | 0 |
| Other pages | 0 |
| Only in lsg | 0 |

Three pages join the build, and Logseq's database has each with the same number of top-level blocks:

| Page | Source | Logseq blocks | lsg blocks |
| --- | --- | ---: | ---: |
| `PARAMETERS` | `mise-tasks/microfreak/PARAMETERS.md` | 8 | 8 |
| `AI__Coding__Tool__Plugin__Report__26__Agentic_Coding_Plugin_Systems_and_Cross_Provider_Portability` | `assets/AI/Coding/Tool/Plugin/Report/26/…Portability.md` | 60 | 60 |
| `README` | `mise-tasks/gitpa/ableton/README.md` | 4 | 4 |

The proposal named the first two. `README` comes from a file the garden gained on 2026-10-29, after the proposal; the root `README.md` stays out through `:hidden`.

The same comparison on codekiln's working copy (commit 6edd337), which holds the git-ignored `tools/` folder with 10 Markdown files, published no page from `tools/`: "Only in lsg" was empty. Neither checkout has a `worktrees/` folder; the `:hidden` folder rule is covered by the loader tests. Its 18 "Other pages" were pages added to the garden after 6edd337 (16 Zoom H5 guide pages and two others), not loader gaps; 9aa484f has their files. No file outside `pages/` and `journals/` shared a title with a note-folder page, so the build reported no skipped file.

## PARAMETERS against Logseq (task 3.2)

Both builds served locally: Logseq's `gh-pages` archive at `#/page/parameters`, and the `lsg` page `p-77ca277b726dbf59.html`. Read from the DOM of each:

- Title `PARAMETERS` in both.
- 8 top-level blocks in both, in the same order: "Inspect saved preset parameters", "Firmware 5 interpretations", "Evidence and scope", four paragraph blocks, and "Verification".
- Blocks 3 to 7 have the same text length in both (593, 228, 250, 261 and 1,288 characters after whitespace is collapsed) and end with the same sentence.
- Blocks 1, 2 and 8 hold the page's three fenced code blocks in both builds and the same heading. Their text outside the code differs by 10 to 22 characters, which is Logseq's code editor chrome; their final sentences match.

## Build time (task 3.3)

`mise run parity:perf --run <run> --builds lsg --profiles desktop --loads 1 --baseline-lsg <main binary>` on codekiln's working copy, the fastest of three publishes each:

| Build | Seconds |
| --- | ---: |
| `main` c7beffe (baseline) | 16.2 |
| This branch | 16.4 (+1%) |

A second, interleaved timing of four publishes each gave 16.4 seconds for this branch and 16.8 for `main`. Loading alone, timed with `lsg lookup tmux`, takes 0.86 seconds against 0.82 on `main`: the extra `git ls-files` call and the walk outside the note folders. A first `parity:perf` run that overlapped a Clippy build showed 30.5 seconds against 19.4 and was discarded.

First paint, scripts, search and memory per page are unchanged: no page's markup or the search format changed, and `mise run parity:budgets` passes.

## Tests

`crates/logseq-gardener-sdk/tests/garden.rs` covers every scenario of the delta spec: Markdown outside the note folders, the skipped paths, `:hidden` root and folder entries, git-ignored files with and without a git work tree, symlinks and special files inside and outside the note folders, and a title collision. `tests/site.rs` checks that the publisher drops and names a shadowed file, and `page_lookup.rs` checks the same for `lsg lookup`.
