# Verification: spell-a-page-with-its-property-name-as-logseq-does

The SDK now spells a page without a file as Logseq does when a file refers to it in more than one block or case, and when a block writes a property name and a reference to the page in different case. Checked on 2026-11-08 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `64a628d` (before) and from this branch (after).

## SDK test

`graph::tests::a_page_without_a_file_takes_the_spelling_of_the_last_block_that_names_it` expects `gadget`, `sprocket`, `Cog`, `pulley` and `lever`. On `origin/main` it fails with `Gadget`, `Sprocket`, `cog`, `PULLEY` and `Lever`. It passes now.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/property-name-reference` (Notes.md only, the issue's case) | 2 differences: the page title and a page reference | agree |
| `fixtures/property-name-reference` (with the added files) | not applicable | agree: 5 pages, 8 blocks |
| Encode garden at `a7d948ed20` | 819 differences | 812 differences |

The page reference difference of the issue also appears when the block writes `[[gadget]]` in the same case as the property. Logseq lists the property name and the reference as one page, and `check.py` compares the property names as properties. The SDK dump now leaves out a reference that equals one of the block's property names.

The 7 differences that went away in the encode garden are the titles `pyright`, `git/revert` and `poppler`, and four page references (`course`, `status` twice, `type`). No difference is new. The remaining title differences, such as `langgraph` and `zsh`, come from the order in which Logseq reads several files that refer to the same page, which this change leaves as it was.
