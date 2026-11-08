# Verification: spell-a-repeated-alias-name-as-logseq-does

The SDK now spells `alias:: [[homebrew]], [[Homebrew]]` as `Homebrew`, as Logseq does. Checked on 2026-11-08 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `d28b43a` (before) and from this branch (after).

## SDK test

`og_relationships::an_alias_written_twice_takes_its_last_spelling` loads `tests/fixtures/alias-spelling-case`, whose `Brew` writes `homebrew` and `Homebrew`. It expects the alias name `homebrew` with the title `Homebrew`. On `origin/main` it fails with the title `homebrew`. It passes now.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/alias-spelling-case` | 1 difference | agree: 1 page, 2 blocks |
| `fixtures/alias-spelling-reference` (new) | not applicable | 1 difference, as `test_check.py` expects |
| Encode garden at `a7d948ed20` | 829 differences | 824 differences |

The 5 differences that went away are alias titles in `pages/Security___CUI space.md`, `pages/CMMC.md`, `pages/Brew.md`, `pages/Logseq___npm___@logseq___nbb-logseq.md` and `pages/Rio.md`. A comparison of the two lists shows no new difference. Eleven alias title differences remain in the encode garden. They come from names that another page or a block spells in another case ([Issue #515](https://github.com/codekiln/logseq-gardener/issues/515)).

## Published page

A garden holding the encode garden's `pages/Brew.md`, with `:publishing/all-pages-public? true`, published with `lsg publish --include brew --search`, differs between the two builds only in the search data. The alias list of the page in `lsg-search-titles-*.json`:

| | Aliases |
| --- | --- |
| Before | `homebrew`, `CLI/Tool/Mac/brew` |
| After | `Homebrew`, `CLI/Tool/Mac/brew` |

The page's HTML differs only in the search manifest's file name and revision, and `lsg-site.json` only in its `revision` digest.
