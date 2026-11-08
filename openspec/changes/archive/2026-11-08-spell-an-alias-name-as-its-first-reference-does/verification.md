# Verification: spell-an-alias-name-as-its-first-reference-does

The SDK now spells an alias name as Logseq does when files refer to the name in another case. Checked on 2026-11-08 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `4bedb2c` (before) and from this branch (after).

## SDK test

`og_relationships::an_alias_takes_the_spelling_of_the_first_file_that_refers_to_it` loads `tests/fixtures/alias-spelling-reference`. It expects `runtime` (a block in `Notes` refers to `[[runtime]]`, and `Other` to `[[RUNTIME]]`), `GADGET` (one file refers to its own alias in two blocks) and `widget` (a file read first has the property `widget::`). On `origin/main` it fails with the alias spellings. It passes now.

## Which spelling Logseq takes

`check.py` on small gardens, each agreeing with the SDK after the change:

| Files | Logseq's spelling |
| --- | --- |
| `Run` lists `Runtime`; `Notes` refers to `[[runtime]]` | `runtime` |
| `Run` lists `Runtime`; `Zed` refers to `[[runtime]]` | `Runtime` |
| `Notes` refers to `[[runtime]]`, `Other` to `[[RUNTIME]]` | `runtime` |
| One file lists `Runtime` and has blocks with `[[runtime]]` and `[[RUNTIME]]` | `RUNTIME` |
| One block refers to `[[runtime]], [[RUNTIME]]` | `runtime` |
| `Brew` lists `homebrew, Homebrew`; `Zed` refers to `[[HomeBrew]]` | `Homebrew` |
| `Run` lists `Runtime`; a file has the property `runtime::` | `runtime` |
| `Run` lists `todo` | `TODO` |

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/alias-spelling-reference` | 1 difference | agree: 2 pages, 3 blocks |
| `fixtures/property-name-reference` (new) | not applicable | 2 differences, as `test_check.py` expects ([Issue #521](https://github.com/codekiln/logseq-gardener/issues/521)) |
| Encode garden at `a7d948ed20` | 824 differences | 819 differences |

The 5 differences that went away are the alias titles `runtime`, `developer`, `repo`, `codespace` and `obsidian/feature/task lists`. A comparison of the two lists shows no new difference. Six alias differences remain in the encode garden; none is a title.

## Published page

A garden holding the encode garden's `pages/Programming___Time___Run.md` and the page whose `runtime::` property Logseq reads first, with `:publishing/all-pages-public? true`, published with `lsg publish --include Programming --include AI --search`, differs between the two builds only in the search data and in files that name it. The alias list of `Programming/Time/Run` in `lsg-search-titles-*.json`:

| | Aliases |
| --- | --- |
| Before | `Runtime` |
| After | `runtime` |

## Speed

A full `lsg publish --include Programming` of the encode garden with a debug build took 7.86 s and 7.84 s before, and 7.97 s and 7.98 s after. The change reads a file's references only for an alias name that two files write in different case.
