# Verification: record-alias-on-named-page

The SDK now records an alias on the page it names, and the parser check reports no alias difference of that kind on the encode garden. Published links to such a page now open it, and its Linked References include the alias page's references. Checked on 2026-11-02 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `c17c516` (before) and from this branch (after).

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `fixtures/alias-of-another-page` from `origin/main`: `brew.md` holds `alias:: [[Homebrew]]` | 1 difference, the alias `brew` of `pages/Homebrew.md` | agree: 2 pages, 3 blocks |
| `crates/logseq-gardener-sdk/tests/fixtures/alias-of-another-page` (new): sibling aliases, an alias without a page, and the named page's own alias | 7 differences: 6 aliases and the files of the alias name `bottles` | agree: 3 pages, 5 blocks |
| `fixtures/alias-namespace-parents` (new deliberate mismatch) | 2 differences | 2 differences, the pages `ai` and `ai/general`, as `test_check.py` expects |
| `docs` | agree | agree: 89 pages, 4501 blocks |
| Encode garden at `e317ae9` | 949 differences | 932 differences |

On the encode garden, 21 differences went away: the 17 aliases that issue #453 counts, such as `pages/Programming___Language___Query___Datascript.md: alias "datascript"`, and the files of 4 alias names, such as `ai skill`, which `pages/AI___LLM___Technique___Skill.md` now records too. 4 differences appeared: the alias names `knowledge garden`, `knowledge gardens`, `zettelkasten` and `zettelkastens` list `pages/Digital Garden.md` among their files. That file is empty, so Logseq reads no page from it, which the check already reported as `pages/Digital Garden.md: page "Digital Garden": the SDK reads it, Logseq does not`.

## SDK tests

- `og_relationships::an_alias_is_recorded_on_the_page_it_names_too` derives the relationships of the new SDK fixture. On `origin/main` it fails: `Homebrew` has only `formulae`, and `Package Manager` has no alias. It passes now, with the aliases Logseq's parser gives.
- `site_references::an_alias_that_names_a_page_joins_both_pages_linked_references` publishes `Notes/brew`, which lists `Notes/Homebrew` as an alias, and a page that refers to both. On `origin/main` it fails: `[[Notes/Homebrew]]` is ambiguous. It passes now: the link opens `Notes/Homebrew`, both pages read `3 Linked References`, each lists the other page's block under an `Alias` label, and `Notes/Homebrew` finds the unlinked mention `Notes/brew`.
- `page_names::a_title_names_its_page_unless_the_page_is_empty` covers the title rule and its exception. `page_names::normalized_titles_and_aliases_share_candidates` and `site::aliases_use_selected_visible_candidates_and_preserve_source` expected a title and another page's alias to be ambiguous, and now expect the titled page.

## Publishing the encode garden

The encode garden at `083106d`, the commit Logseq's last `gh-pages` build (`8132c95`) was made from, published with the garden's `mise run site:build` and each `lsg` build: 8565 pages and 507 journals both times.

- Diagnostics: 37 `Ambiguous selected page reference` or `Ambiguous selected tag` diagnostics went away, for names such as `GitHub/dolthub/dolt`, `e2b`, `c74/max/obj/js`, `Claude` and `Digital Garden`. No diagnostic appeared. 10 remain, for `Codespace`, `Repo`, `Resonator` and `Articles`, which two pages each list as an alias and no page has as its title.
- Files, apart from the search data's revision, which every page names: 71 pages and journals, 954 Linked and Unlinked References files (47 of them new), 405 graph files, the page list data, 17 `pages-N.html` files of the page list and 10 journal feed files, `lsg-site.json`, and `lsg-site.css`, which gains the `.ref-alias` rule.
- Linked References, before and after, against Logseq's build where it rendered:

| Page | Before | After | Logseq |
| --- | --- | --- | --- |
| `Dolt` | 16 blocks, 11 groups | 17 blocks, 12 groups | 17 blocks, the same 12 groups |
| `GitHub/dolthub/dolt` | none | 19 blocks, with `Dolt` labelled `Alias` | |
| `Anthropic/Model/Claude` | 115 | 913 | 899 |
| `Digital Garden` (empty file) | none | 79 | |
| `Programming/Language/Query/Datascript` | none | 4 | |
| `Ableton/Max for Live` | 57 | `0 of 57`, its `filters::` value now names a page | |

Logseq's build renders Linked References only after the section scrolls into view, and in headless Chrome it often rendered none on a fresh load, so the table lists only the pages where the section rendered. On `Dolt`, the extra group is `2026-03-12 Thu`, which refers only to `GitHub/dolthub/dolt`. `Anthropic/Model/Claude` grows because every page that lists `Claude` as an alias is in its alias group, as in Logseq. In `lsg` the group also holds `pages/CLAUDE.md`, which the garden's `:hidden` setting hides from Logseq.

`lsg` took 17.7 to 18.7 seconds for the garden's whole build with either `lsg` build, two runs each.
