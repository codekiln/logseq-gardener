# Verification: label-property-alias-links-as-written

lsg now labels property links as Logseq 0.10.15 does, on a test garden with the issue's case and lower-cased variants. Checked on 2026-11-01 against `lsg` built from `origin/main` (before) and from this branch (after).

## Logseq's own export

`mise run parity:logseq-export --garden <test garden> -- --filename-format triple-lowbar --include Knowledge`, which builds Logseq 0.10.15's export from `/Applications/Logseq.app` and compares each page's property rows with lsg's. The test garden turns on `:publishing/page-properties?` and has three pages:

- `Knowledge/Garden/er`, with `alias:: [[Knowledge Gardener]]`;
- `Knowledge/Tag Page`;
- `Knowledge/Garden`, with `see-also:: [[Knowledge Gardener]], [[knowledge/garden/er]], [[KNOWLEDGE/garden/ER]]`, `lower-alias:: [[knowledge gardener]]` and `tags:: knowledge/tag page`.

| Row on `Knowledge/Garden` | Logseq | lsg before | lsg after |
| --- | --- | --- | --- |
| `see-also` | `Knowledge Gardener, Knowledge/Garden/er, KNOWLEDGE/garden/ER` | `Knowledge/Garden/er, Knowledge/Garden/er, Knowledge/Garden/er` | as Logseq |
| `lower-alias` | `knowledge gardener` | `Knowledge/Garden/er` | as Logseq |
| `tags` | `Knowledge/Tag Page` | as Logseq | as Logseq |

Before the change, `Knowledge/Garden` differed. After it, all 3 pages agree.

## SDK tests

`site_properties::property_links_keep_the_name_as_written` fails on `origin/main`:

```
<dt>see-also</dt><dd><a class="page-ref" href="p-c9990746469ec7b7.html">Knowledge Gardener</a>, … missing from … <dt>see-also</dt><dd><a class="page-ref" href="p-c9990746469ec7b7.html">Visible/Garden/er</a>, <a class="page-ref" href="p-c9990746469ec7b7.html">Visible/Garden/er</a></dd>
```

It passes now. `book_and_preset_properties_keep_order_scalars_aliases_and_avoid_duplicate_sequences` expected the old label: its `prev:: [[First chapter]]` link now reads `First chapter`.

## Publishing the encode garden

The garden's `mise-tasks/site/build`, on a copy of the garden with `:publishing/page-properties? true` added (as in [logseq-encode-garden PR #302](https://github.com/codekiln/logseq-encode-garden/pull/302)):

| | Before | After |
| --- | --- | --- |
| Diagnostics | 12495 | 12495, the same lines |
| Pages whose HTML changes | | 423 |
| Property links whose label changes | | 452 |
| Linked-reference fragments that change | | 221 |
| `site:build` wall time | 23.3 s | 23.1 s |

- Every changed file differs only in the text of `page-ref` links; no link changes its target, and no row appears or disappears.
- `Knowledge/Garden` reads `see-also: Knowledge Gardener, Knowledge/Concept/Knowledge Base vs Knowledge Garden`, as in Logseq.
- Other examples: `Person/Lyz/Garden` now reads `Knowledge Garden` in place of `Knowledge/Garden`, and `Security/Application/SAST` reads `Diataxis/Concept` in place of `Diataxis/Explanation`.
