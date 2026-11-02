# Verification: list-every-alias-in-alias-row

lsg's `alias` row now lists the names Logseq 0.10.15 lists, on a test garden with each case from the issue. Checked on 2026-11-01 against `lsg` built from `origin/main` (before) and from this branch (after).

## Logseq's own export

`mise run parity:logseq-export --garden <test garden>`, which builds Logseq 0.10.15's export from `/Applications/Logseq.app` and compares each page's property rows with lsg's. The test garden turns on `:publishing/page-properties?` and opens the box with `:publishing/page-properties-collapsed? false`.

| Page | Its `alias::` value | Before: rows only Logseq shows | After |
| --- | --- | --- | --- |
| `Knowledge/Garden` | `[[Digital Garden]], [[Zettelkasten]]`, with `Digital Garden.md` empty | `alias: Digital Garden, Zettelkasten` | agree |
| `Datascript` | `[[Programming/Language/Query/Datascript]]`, a page whose file holds one empty block | `alias: Programming/Language/Query/Datascript` | agree |
| `Source` | `[[Target]]` | `alias: Target, Other Name` | agree |
| `Target` | `[[Other Name]]` | `alias: Other Name, Source` (lsg showed `alias: Other Name`) | agree |
| `Claim A`, `Claim B` | `[[Collision]]` on both | `alias: Collision, Claim B` and `alias: Collision, Claim A` | agree |
| `nx` | `Nx` | – | agree |
| `Rio` | `Rio Terminal, Rio terminal` | – | agree |
| `Brew` | `brew, homebrew, Homebrew, [[Tool/brew]]` | – | Logseq `brew, Homebrew, Tool/brew`, lsg `brew, homebrew, Tool/brew` |

Before the change, 7 of the 8 pages compared differed. After it, `Brew` differs in the case of one name, as `design.md` describes, and `Digital Garden` is not compared in either run, because Logseq's export of the empty page never renders a block.

## SDK tests

`site_properties::alias_row_lists_every_alias_as_logseq_does` fails on `origin/main`, where `Visible/Garden` shows no properties box:

```
<dt>alias</dt><dd><a class="page-ref" href="p-1a2f70e82338b123.html">Visible/Digital Garden</a>, <a class="page-ref" href="p-008b02b9b01fec2f.html">Zettelkasten</a></dd> missing from
```

It passes now, with `page_names::alias_groups_list_names_as_logseq_does`.

## Publishing the encode garden

The garden's `mise-tasks/site/build`, on a copy of the garden with `:publishing/page-properties? true` added (as in [logseq-encode-garden PR #302](https://github.com/codekiln/logseq-encode-garden/pull/302)):

| | Before | After |
| --- | --- | --- |
| Pages | 8926 | 8926 |
| Journals | 513 | 513 |
| Diagnostics | 12496 | 12496 |
| Pages with an `alias` row | 1428 | 1457 |
| `lsg publish` time | 16.8 s | 16.0 s |

- 29 pages gain an `alias` row, 17 of them with no properties box before, among them `Knowledge/Garden` (`Digital Garden, Knowledge Garden, Knowledge Gardens, Zettelkasten, Zettelkastens`) and `Datascript` (`Programming/Language/Query/Datascript`). Pages that only another page names as its alias gain the row too, such as `OpenAI/Blog/26/02/27/Stateful Runtime Environment for Agents in Amazon Bedrock`, which lists `E2B`.
- 5 rows drop a second spelling that differs only in case, such as `Rio Terminal, Rio terminal`, which becomes `Rio Terminal` as in Logseq.
- No row is lost.
- 14 pages gain a Linked References section, such as `Digital Garden`, which lists `Knowledge/Garden`, and the All pages list shows 30 higher reference counts.
