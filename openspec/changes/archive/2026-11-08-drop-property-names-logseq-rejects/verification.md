# Verification: drop-property-names-logseq-rejects

Logseq's parser check now agrees with the SDK on blocks that start with code holding a property. Checked on 2026-11-01 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `1a313d6` (before) and from this branch (after).

## What Logseq shows

Logseq's build of the encode garden, the `gh-pages` commit `8132c95` from garden commit `083106d`, served locally and opened at `Logseq/Entity/Asset`: the block under "Frontmatter and body" that the garden writes as `` `logseq-entity:: [[Logseq/Entity/Asset]]` identifies a local asset. … `` shows no text. Below it, a `div.invalid-properties` reads "Invalid property names: `logseq-entity" and explains which names are valid. The decoded database gives the block `:block/invalid-properties #{"`logseq-entity"}`, no properties and no references. It holds 13 such blocks in 11 pages.

Under nbb-logseq, `valid-property-name?` answers `false` for `` :`status `` and `` :`logseq-entity ``, and `true` for `:status`. The 51 names in `property_names::tests::LOGSEQ` carry the answers it gave.

## SDK tests

`site_references::a_property_name_logseq_rejects_adds_no_reference` publishes `Notes/Done`, a page whose only block is `` `status:: [[Notes/Done]]` marks a finished task. ``, and a page that links `[[Notes/Done]]` in its text and in `` `status:: [[Notes/Done]]` ``. On `origin/main`, `Notes/Done` shows `2 Linked References`, one of them the empty block. Now it shows `1 Linked Reference`, the block that names it in its text.

`garden::tests::drops_the_properties_logseq_rejects`, `references::tests::a_property_logseq_rejects_names_no_page` and `property_names::tests` use operations that `origin/main` lacks, so they hold the fix in place.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `scripts/logseq-parser/fixtures/code-span-property`: `` `status:: [[Done]]` marks a finished task. `` | 4 differences | agree: 1 page, 1 block |
| `scripts/logseq-parser/fixtures/duplicate-block-id` (new): `JSON5.md` and `JSONC.md` give a block the same `id::` | 2 differences | 2 differences, both about `pages/JSONC.md`, as `test_check.py` expects |
| `docs` | agree | agree: 89 pages, 4493 blocks |
| Encode garden at `e317ae9` | 1016 differences | 955 differences |

On the encode garden the check reports 61 fewer differences and no new one. The 61 are the 13 rejected properties in 11 files, the 44 page references their values gave, and 4 graph nodes named `` `logseq-entity ``, `` `podcast-guid ``, `` `podcast-published-at `` and `` `public ``.

## Publishing the encode garden

`mise run parity:build`, which publishes every selectable page of the encode garden with `--journals --search` and one `--include` for each top-level name, gives 8930 pages with both builds. `diff -rq` reports 249 changed files and no added or removed one:

- 22 pages. 15 lose linked references, such as `Logseq/Entity/Asset` from 801 to 800 and `Logseq/Entity` from 6279 to 6270. The other 7 hold one of the blocks and lose only that block's anchor, because the block no longer appears among any page's linked references.
- 180 reference fragments under `_refs`, 34 graph files under `_graph`, the all-pages data and 11 all-pages files, whose counts follow, and `lsg-site.json`, which lists the files.
