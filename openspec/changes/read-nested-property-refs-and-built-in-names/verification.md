# Verification: read-nested-property-refs-and-built-in-names

Logseq's parser check now agrees with the SDK on nested references in property values and on the spelling of built-in pages without a file. Checked on 2026-11-01 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` (before) and from this branch (after).

## SDK tests

`references::tests::other_property_values_name_pages_as_in_logseq` and `graph::tests::built_in_pages_without_a_file_keep_logseqs_spelling` failed on `origin/main` and pass now:

```
left: ["Mldoc", "Notes on [[Mldoc]]"]
right: ["Notes on [[Mldoc]]"]

left: ["c", "todo", "favorites", "Card", "started"]
right: ["C", "TODO", "Favorites", "card", "started"]
```

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| Case graph: `#c #todo #favorites` on one page; `TODO [#C]`, `type:: [[Notes on [[Mldoc]]]]`, `source:: [[Outer [[Inner]] Name]], #[[Tag [[Deep]]]], [[Plain]]` and `[[Body [[Nested]] Ref]]` in a block's text on another | 9 differences | agree: 2 pages, 5 blocks |
| `scripts/logseq-parser/fixtures/nested-property-reference` | 1 difference | agree: 2 pages, 2 blocks |
| `scripts/logseq-parser/fixtures/namespace-parent-space` | – | 5 differences, all about `macos` and `macos `, as `test_check.py` expects |
| `docs` | agree | agree: 89 pages, 4491 blocks |
| Encode garden | 1046 differences | 1043 differences |

The three encode garden differences that go away are the built-in page `NOW`, which the SDK spelled `Now`, and the extra page and block reference to `Agentic Coding Plugin Systems and Cross-Provider Portability.md`, which the SDK read from the label of a file link in a `source-md::` value. The garden has no nested reference in a property value. The check reports no new difference.

## Publishing the encode garden

`lsg publish --journals --filename-format triple-lowbar` with one `--include` for each of the 1048 top-level names:

| | Before | After |
| --- | --- | --- |
| Pages | 8925 | 8925 |
| Journals | 513 | 513 |
| Diagnostics | 12497 | 12497 |
| Wall time | 16.6 s | 18.8 s |

No HTML page changes. The page graph data changes in five files: two page graphs spell the node `NOW` in place of `Now`, and the page `AI/Coding/Tool/Plugin/Report/26/Agentic Coding Plugin Systems and Cross-Provider Portability` loses the node `Agentic Coding Plugin Systems and Cross-Provider Portability.md`, which the label of the file link in its `source-md::` value brought in.
