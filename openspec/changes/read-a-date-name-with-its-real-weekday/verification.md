# Verification: read-a-date-name-with-its-real-weekday

The SDK now reads `[[1967-09-03 Fri]]` as the date 1967-09-03 and titles the page without a file `1967-09-03 Sun`, as Logseq does. Checked on 2026-11-07 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `e0c1ce0` (before) and from this branch (after).

## SDK test

`og_relationships::a_date_name_takes_the_weekday_the_date_has` loads `tests/fixtures/wrong-weekday-date-name`, where `Notes` links `1967-09-03 Fri` under `:journal/page-title-format "yyyy-MM-dd EEE"`. The test expects the pages without a file `1967-09-03 Sun` and `date-created`. On `origin/main` it fails: the SDK names `1967-09-03 Fri`. It passes now. Unit tests in `journals.rs` check that `2026-10-23 Mon` reads as 2026-10-23 and that `2026-10-23 Someday` reads as no date.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/wrong-weekday-date-name` | 4 differences: pages without a file `1967-09-03 fri` and `1967-09-03 sun`, and the same two names as page references | agree: 1 page, 2 blocks |
| `fixtures/trailing-slash-name` | agree | agree |
| `fixtures/alias-spelling-case` (new) | 1 difference | 1 difference, as `test_check.py` expects |
| `docs` | agree | agree |
| Encode garden at `a7d948ed20` | 881 differences | 829 differences |

The 52 differences that went away all concern date names. They include the four wrong-weekday references, such as `2026-06-28 sat` against `2026-06-28 sun` in `pages/AI___Coding___Tool___Report___26___Free Tier Providers.md`. They also include references in another spelling of a date: `[[Feb 1st, 2025]]` in `pages/Docker___For Mac___How to Install into Zsh.md` is the page `2025-02-01 sat` in Logseq, and `[[2026_03_22]]` in `pages/Ghostty.md` is `2026-03-22 sun`. No difference is new: a comparison of the two lists shows only removals.

## Published page

A garden holding `pages/Notes.md` with `- A person born on [[1967-09-03 Fri]]`, `:journal/page-title-format "yyyy-MM-dd EEE"` and `:publishing/all-pages-public? true`, published with `lsg publish --include Notes`, changes only in its graph files. The page graph of `Notes`:

| | Nodes |
| --- | --- |
| Before | `Notes`, `1967-09-03 Fri` |
| After | `Notes`, `1967-09-03 Sun` |

The graph's other files differ only through that title, and `lsg-site.json` differs only in its `revision` digest. The page's HTML and the diagnostics are unchanged.
