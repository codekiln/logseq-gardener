# Verification: drop-boundary-slashes-of-a-page-name

The SDK now reads `[[a/b/]]` as the page `a/b` with the namespace parent `a`, as Logseq does, and the parser check reports no difference on the fixture. Checked on 2026-11-07 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `102f5a6` (before) and from this branch (after).

## SDK test

`og_relationships::a_boundary_slash_is_not_part_of_a_page_name` loads `tests/fixtures/trailing-slash-name`, where `Notes` links `a/b/`. The test expects the pages without a file `a` (titled `A`, Logseq's built-in priority page) and `a/b` with the namespace `a`. On `origin/main` it fails: the SDK reads `a` with no namespace and `a/b` titled `a/b/` with no namespace. It passes now.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/trailing-slash-name` | 2 differences: namespace `a` against null, title `a/b` against `a/b/` | agree: 1 page, 1 block |
| `fixtures/wrong-weekday-date-name` (new) | 4 differences | 4 differences, as `test_check.py` expects |
| `docs` | agree | agree |
| Encode garden at `a7d948ed20` | 882 differences | 881 differences |

The one difference that went away is the title of `d♭)` in `pages/Ableton___Live___UG___29 Audio Effect Reference___39 Tuner___04 Note Spellings.md`: Logseq reads `D♭)` and the SDK read `/D♭)`, a leading slash. The encode garden holds no link with a trailing slash. No new difference appeared.

## Published page

A garden holding `pages/Notes.md` with `- A link to [[a/b/]]`, published with `lsg publish --include Notes`, changes only in its graph files. The page graph of `Notes`:

| | Nodes | Edges |
| --- | --- | --- |
| Before | `Notes`, `a/b/`, `A` | `Notes` to `a/b/`, `Notes` to `A` |
| After | `Notes`, `a/b`, `A` | `Notes` to `a/b`, `Notes` to `A`, `a/b` to `A` |

The node is now titled `a/b`, and the namespace edge to its parent `a` (spelled `A`, the built-in page) exists. The page's own HTML, the page list and the diagnostics are unchanged, including the diagnostic `Unresolved or excluded page reference: a/b/`, which quotes the text as written.
