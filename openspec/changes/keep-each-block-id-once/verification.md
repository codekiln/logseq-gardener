# Verification: keep-each-block-id-once

The parser check now agrees with the SDK on two pages that give a block the same `id::`, and a block embed of that id shows the block Logseq shows. Checked on 2026-11-02 with `scripts/logseq-parser/check.py`, which runs Logseq's file parser at the pinned revision, against the SDK built from `origin/main` at `5687461` (before) and from this branch (after).

## What Logseq shows

Logseq's build of the encode garden, the `gh-pages` commit `8132c95` from garden commit `083106d`, served locally:

- On `json/5`, the `About` block has `blockid="67c822b0-3271-41da-a14f-4f023a1f4556"` and a reference count of 1.
- On `json/jsonc`, the `About` block has `blockid="6ade6816-2979-4cae-9408-b4bf3ecba498"`, a UUID Logseq made.
- On `2025-03-05 Wed`, the embed `{{embed ((67c822b0-3271-41da-a14f-4f023a1f4556))}}` shows the `About` block of `json/5` with its children.
- The decoded database holds one block whose content is `## About\nid:: 67c822b0-3271-41da-a14f-4f023a1f4556`.

`filter-files` in `deps/graph-parser/src/logseq/graph_parser.cljs` reads `pages/json___5.md` before `pages/json___jsonc.md`, because `5` sorts before `j`.

## SDK tests

`site_embeds::a_block_id_two_pages_share_names_the_block_logseq_reads_first` publishes `Notes/Start`, which embeds and references an id that `Notes/json5` and `Notes/jsonc` share. On `origin/main` the build reports `Unavailable or ambiguous embed macro target` and `Unavailable or ambiguous block reference` for the id, and the page shows neither block. Now the embed and the reference both show `About JSON5`, with no diagnostic.

`block_ids::tests` cover Logseq's file order, a journal against a page, two blocks of one page with `id::` and `custom_id::`, and ids that are not UUIDs.

Four tests expected a shared id to name no block, and now expect the block Logseq reads first: `site::block_links_resolve_selected_nested_ids_without_exposing_other_targets`, `site_build_plugins::view_holds_published_pages_and_blocks_only`, `site_embeds::private_excluded_missing_and_ambiguous_targets_keep_fallbacks` and `site_search::duplicate_uuid_uses_offsets_and_large_original_outlines_have_real_destinations`.

## Parser check

| Graph | Before | After |
| --- | --- | --- |
| `scripts/logseq-parser/fixtures/duplicate-block-id`: `JSON5.md` and `JSONC.md` give a block the same `id::` | 2 differences | agree: 2 pages, 2 blocks |
| `scripts/logseq-parser/fixtures/alias-title-case` (new): `Solvespace.md` holds `alias:: [[SolveSpace]]` | 1 difference | 1 difference, the title of `pages/Solvespace.md`, as `test_check.py` expects |
| `docs` | agree | agree: 89 pages, 4493 blocks |
| Encode garden at `e317ae9` | 955 differences | 953 differences |

On the encode garden the two differences that went away are the block id and the `id` property of `pages/json___jsonc.md`. No new difference appeared.

## Publishing the encode garden

`mise run parity:build`, which publishes every selectable page of the encode garden with `--journals --search`, gives 8930 pages with both builds. Apart from the search data's revision, which every page names, 7 files differ:

- `2025-03-05 Wed` and the journals page that lists it show the embed of the `About` block of `json/5`, where they showed nothing.
- `json/5` gives its `About` block the anchor `block-67c822b0-3271-41da-a14f-4f023a1f4556`.
- The linked references of `json/5` and the unlinked references of `Programming` and `Tool` show the journal block with its embed.
- `lsg-site.json` lists the changed files.

The build's diagnostics lose one line: `journals/2025_03_05.md: Unavailable or ambiguous embed macro target: {{embed ((67c822b0-3271-41da-a14f-4f023a1f4556))}}`.
