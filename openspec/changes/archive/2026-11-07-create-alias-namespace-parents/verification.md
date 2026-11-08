# Verification: create-alias-namespace-parents

The SDK now creates the namespace parents of an alias name as pages without a file, and the parser check reports no such difference on the fixture. Checked on 2026-11-07 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `da84ac1` (before) and from this branch (after).

## SDK test

`og_relationships::an_alias_in_a_namespace_names_its_namespace_parents` loads `tests/fixtures/alias-namespace-parents`. `AGI` lists `AI/General/Intelligence` as an alias, `Brew` lists `CLI/Tool/Mac`, and `CLI` has a file. The test expects the pages without a file `ai`, `ai/general` and `cli/tool`. On `origin/main` it fails with an empty list. It passes now.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `alias-namespace-parents` fixture | 3 differences: `ai`, `ai/general`, `cli/tool` | agree: 4 pages, 7 blocks |
| `fixtures/trailing-slash-name` (new) | 2 differences | 2 differences, as `test_check.py` expects |
| `docs` | agree | agree |
| Encode garden at `a7d948ed20` | 965 differences, 86 of them `page without a file ... Logseq reads it, the SDK does not` | 882 differences, 10 of them |

The 83 differences that went away are 76 pages without a file for the namespace parents of alias names, such as `ai/general` from `pages/AGI.md` and `cli/tool/mac` from `pages/Brew.md`, and 7 titles of such parents, such as `o365/outlook`, which Logseq spells from the first reference. No new difference appeared. The 10 left are six `card-*` property pages and four date names such as `1967-09-03 sun`.
