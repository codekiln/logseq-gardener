## Context

Logseq 0.10.15 renders a property value that holds page references as inline text, so each `[[name]]` goes through `page-cp` and `page-inner` (`frontend/components/block.cljs`), as a reference in the page body does. `page-inner` labels the link with the name as written. When the written name equals the lower-cased original name of the page it names, it shows that original name instead. A comma-separated property such as `tags` passes each value through `page-cp` too, so the same rule applies.

lsg's `value()` in `site/properties.rs` labelled every resolved link with the title of the page it opens. `alias_value()` and `aliases_value()` already label their links with the name as written.

Logseq's own export of a test garden shows these labels (`verification.md` has the full comparison):

| Written | Page it opens | Logseq's label |
| --- | --- | --- |
| `[[Knowledge Gardener]]`, an alias | `Knowledge/Garden/er` | `Knowledge Gardener` |
| `[[knowledge gardener]]` | `Knowledge/Garden/er` | `knowledge gardener` |
| `[[KNOWLEDGE/garden/ER]]` | `Knowledge/Garden/er` | `KNOWLEDGE/garden/ER` |
| `[[knowledge/garden/er]]` | `Knowledge/Garden/er` | `Knowledge/Garden/er` |
| `tags:: knowledge/tag page` | `Knowledge/Tag Page` | `Knowledge/Tag Page` |

## Goals / Non-Goals

**Goals:**

- Every property link reads as Logseq shows it, and opens the same page as before.

**Non-Goals:**

- Show the page's title for a body reference written as that title in lower case. The page body shows the name as written in every case, and Logseq's `page-inner` applies the same rule there.

## Decisions

- `value()` labels a link with the name as written, unless the name equals the lookup key of the title of the page it opens; then it shows that title. A small helper, `reference_label`, holds the rule next to `value()`. The route, the references the row adds and the rules that drop a row are unchanged.
- The comparison uses `lookup_key`, the lower-cased, NFC-normalized spelling lsg uses for every page name, which matches Logseq's `page-name-sanity-lc` for these names.
- For a lower-cased alias, lsg compares the name with the title of the page the alias opens, so `[[knowledge gardener]]` keeps its written spelling. Logseq compares it with the original name of the alias's own page, which is whichever spelling Logseq parsed first. lsg has no such page, and on the test garden both builds show `knowledge gardener`.

## Risks / Trade-offs

- [In a garden where Logseq parses the `alias::` value before a lower-cased reference to the alias, Logseq shows the alias's spelling from the `alias::` value, and lsg shows the lower-cased name] → The label differs in case only, and the link opens the same page.
- [The `published-page-properties` delta restates the requirement as `list-every-alias-in-alias-row` modifies it, and that change is not synced yet] → Sync and archive `show-property-rows-for-missing-pages` ([Issue #414](https://github.com/codekiln/logseq-gardener/issues/414)) and `list-every-alias-in-alias-row` ([Issue #425](https://github.com/codekiln/logseq-gardener/issues/425)) before this change.

## Resolved Questions

### 1 - Does a link written in lower case show the name as written?

Decided by the agent without codekiln: only when the name differs from the lower-cased title of the page it opens. Logseq's export shows `Knowledge/Garden/er` for `[[knowledge/garden/er]]` and `Knowledge/Tag Page` for `tags:: knowledge/tag page`, so lsg follows `page-inner`'s rule rather than the issue's shorter wording.

### 2 - Does the page body change too?

Decided by the agent without codekiln: no. Issue #419 is about property links, and the body already shows the name as written in every other case. A body link written as a lower-cased title is a separate, smaller difference.

### 3 - Where does the rule live?

Decided by the agent without codekiln: in the SDK's properties module, next to `value()`, which already decides each property link's target. The page HTML holds the label, so the publisher renders it with no extra work in the browser.
