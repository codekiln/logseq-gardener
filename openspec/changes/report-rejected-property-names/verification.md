# Verification: report-rejected-property-names

`lsg publish` now reports one `Invalid property names:` diagnostic for each properties block that shows Logseq's warning, under the file that holds it. On the encode garden before its fix, that is 14 diagnostics on the 12 pages that show the warning; after the fix, none. The published files do not change. Checked on 2026-11-03 against `lsg` built from `origin/main` at `116847f` (before) and from this branch (after).

## Tests

Three `site_properties` tests fail on `origin/main`, whose report holds no such diagnostic, and pass now:

- `a_block_whose_property_name_logseq_rejects_is_reported`: a block that starts with `` `Logseq_Entity:: [[Visible/Asset]]` `` reports ``Invalid property names: `logseq-entity`` under `pages/Visible___Asset.md`.
- `page_properties_with_two_rejected_names_are_reported_once`: leading page properties `` `owner_name:: x `` and `` `Kind:: y `` report ``Invalid property names: `owner-name, `kind`` once.
- `an_embedded_block_reports_rejected_property_names_under_its_own_file`: a page that embeds the block shows the warning, and the report names only the embedded page's file.

## Publishing the encode garden

The encode garden's own `mise-tasks/site/build` published `git archive` snapshots of its `pages`, `journals` and `logseq` folders, at `cbab08523e` (before [codekiln/logseq-encode-garden#315](https://github.com/codekiln/logseq-encode-garden/pull/315) rewrote the blocks) and at `9860577214` (after).

| Garden | `lsg` | Diagnostics | `Invalid property names:` |
| --- | --- | --- | --- |
| `cbab08523e` | `origin/main` | 12609 | 0 |
| `cbab08523e` | this branch | 12623 | 14 |
| `9860577214` | this branch | 12609 | 0 |

The 14 diagnostics name the same blocks as the warnings on the published site: one on each of `Logseq/Entity/Ableton/Set`, `Asset`, `Asset/B2`, `Field of Study`, `Hardware/Audio/Speaker`, `Hardware/Connector/Audio`, `Music/Scale`, `Podcast/Episode/Frontmatter/podcast-guid`, `Podcast/Episode/Frontmatter/podcast-published-at` and `Trade-Off/Component`, and two each on `Person/Maker` and `Podcast/Episode/GitP/A/Frontmatter/public`. No diagnostic names a page that only embeds or lists one of these blocks.

`diff -rq` between the two sites built from `cbab08523e` reports no difference in any of the 61,973 files.

## Publishing gitpa

Gitpa's `scripts/build_site.py` with this branch's `lsg` published `git archive` snapshots of gitpa at `8a0c6a9eb` (before [codekiln/gitpa#143](https://github.com/codekiln/gitpa/pull/143)) and at `5969a37f7` (after). Gitpa publishes only public pages, so the 3 diagnostics before the re-sync name its 3 public pages with the warning, `Logseq/Entity/Asset`, `Logseq/Entity/Asset/B2` and `Logseq/Entity/Music/Scale`; after it, none.
