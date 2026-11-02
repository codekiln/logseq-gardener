# Verification: keep-spaces-inside-page-references

Logseq's parser check now agrees with the SDK on page references with spaces inside their brackets. Checked on 2026-11-01 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` (before) and from this branch (after).

## SDK tests

`site_references::a_space_inside_the_brackets_keeps_the_reference_off_the_trimmed_page` publishes the page `Notes/Release Notes`, a page that links `[[Notes/Release Notes]]`, and a page that links `[[Notes/Release Notes ]]` and `[[ Notes/Release Notes]]`. On `origin/main` the page `Notes/Release Notes` shows `3 Linked References`, and the two links show `Notes/Release Notes ` and ` Notes/Release Notes` with their spaces. Now it shows `1 Linked Reference`, and both links show `Notes/Release Notes` and open that page.

`references::tests::a_page_reference_keeps_the_spaces_inside_its_brackets` and `graph::tests::a_page_reference_keeps_the_spaces_inside_its_brackets` use operations that `origin/main` lacks, so they hold the fix in place rather than reproduce the bug.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `scripts/logseq-parser/fixtures/page-reference-space`: `[[Release Notes ]]` | 4 differences | agree: 1 page, 1 block |
| Case graph: `[[Release Notes ]]`, `[[ Release Notes]]`, `[[ Notes/Setup ]]`, `#[[Tag ]]`, `{{embed [[Embed Target ]]}}`, `[[ ]]`, `[[MacOS /Applications Folder]]`, `[[A / B / C]]`, `[[release notes]]`, `type:: [[ Bar ]]`, `source:: [[ Quoted Prop]], [[Other ]]`, and the pages `Release Notes`, `MacOS` and `Notes/Setup` | 21 differences | 1 difference |
| `scripts/logseq-parser/fixtures/code-span-property` (new): `` `status:: [[Done]]` marks a finished task. `` | 4 differences | 4 differences, all about `` `status `` and `done`, as `test_check.py` expects |
| `docs` | agree | agree: 89 pages, 4493 blocks |
| Encode garden | 1016 differences | the same 1016 differences |

The one difference left on the case graph is the reference-only node `notes` for the page `Notes/Setup`, which no block names. The SDK makes a node for the namespace parent of every page; Logseq's side of the check lists only pages that a block names. `origin/main` reports the same line, and the change `name-namespace-parent-with-space` records it for `Linux /Bin`.

The property values with spaces inside their brackets agree before and after.

## Publishing the encode garden

`mise run parity:build`, which publishes every selectable page of the encode garden with `--journals --search` and one `--include` for each top-level name, gives 8930 pages with both builds. The two sites are byte for byte the same: `diff -rq` reports no file. The encode garden has no page reference with a space inside its brackets outside code blocks.
