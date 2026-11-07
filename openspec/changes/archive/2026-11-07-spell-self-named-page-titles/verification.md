# Verification: spell-self-named-page-titles

The SDK now titles a page that refers to itself in another spelling as Logseq does, and the parser check reports no title difference for a page with a file in the encode garden. Checked on 2026-11-02 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `c4d7a24` (before) and from this branch (after).

## What Logseq shows

Logseq's build of the encode garden, the `gh-pages` commit `8132c95` from garden commit `083106d`, holds the graph in `static/js/db.gz`. Its page entities have these `:block/name` and `:block/original-name` values:

- `solvespace`: `SolveSpace`, from `pages/Solvespace.md`, which holds `alias:: [[SolveSpace]]`.
- `brew`: `brew`, from `pages/Brew.md`, which holds `alias:: [[brew]], [[homebrew]], [[Homebrew]], [[CLI/Tool/Mac/brew]]` and which journals refer to as `[[brew]]` before Logseq reads the page.
- `claude/code/mcp/json/how to/create atlassian mcp json and use it`: `Claude/Code/MCP/json/How to/Create Atlassian MCP Json and Use it`. The page's `title::` and alias spell `How To`; `journals/2026_02_13.md` spells `How to`.

## SDK test

`garden::a_page_that_names_itself_in_another_spelling_takes_that_spelling` loads 14 pages and a journal, with an alias, a reference in the text, a namespace parent, a bare `tags::` value, a tag, a `title::` page that refers to itself, a task marker, two spellings in two blocks and in one block, earlier journal references, and pages without a reference to themselves. On `origin/main` it fails: every page keeps the title of its file name or `title::` property. It passes now. Logseq's parser gives the expected titles for the same cases in the test graph below.

Two tests had a page that refers to itself in another spelling by accident, and now use a page that does not: `site_properties::property_links_keep_the_name_as_written`, whose page `Visible/Garden` linked `[[visible/garden/er]]`, and `site_readable::folders_spelled_two_ways_publish_under_one_spelling`, whose page `CLAUDE` linked `[[Claude/Artifact]]`. Logseq titles such pages `visible/garden` and `Claude`.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/alias-title-case`: `Solvespace.md` holds `alias:: [[SolveSpace]]` | 1 difference, the title | agree: 1 page, 2 blocks |
| `fixtures/alias-of-another-page` (new): `brew.md` holds `alias:: [[Homebrew]]` | 1 difference | 1 difference, the alias `brew` of `pages/Homebrew.md`, as `test_check.py` expects |
| Test graph with the cases of the SDK test | 12 differences: 11 titles and one alias | 1 difference, the alias |
| `docs` | agree | agree: 89 pages, 4494 blocks |
| Encode garden at `e317ae9` | 953 differences | 949 differences |

On the encode garden the four differences that went away are the titles of `pages/Solvespace.md`, `pages/Brew.md` and the Claude MCP page, and the title of the namespace page `Claude/Code/MCP/json/How to` above it. No new difference appeared.

## Publishing the encode garden

`lsg publish` with `--journals --search` and the namespace roots of every page file, as the garden's `site:build` chooses them, plus `SolveSpace` for the new build, gives 8926 pages and 513 journals with both builds. Apart from the search data's revision, which every page names:

- Four pages have new titles and so new routes: `SolveSpace`, `brew`, `Claude/Code/MCP/json/How to/Create Atlassian MCP Json and Use it` and its namespace page `Claude/Code/MCP/json/How to`.
- 295 other files differ: the pages, reference sections, graph data and page list that link to those four pages.
- The build's diagnostics are the same.

Without the `SolveSpace` root, the new build leaves out `pages/Solvespace.md`, and the garden's `site:build` stops with `lsg left out 1 pages`.

## Loading time

`lsg lookup Solvespace` on the encode garden, which loads the whole garden, alternating the two builds 12 times on an Apple M2 Pro: median 1.02 seconds before, 1.17 seconds after. The pass read the references of 41 files. A temporary test confirmed that every name `Contribution::from_document` returns, on every file of the encode garden and the docs graph, is among the names the pass lists from the parse.
