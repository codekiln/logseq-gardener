## Why

The `alias` row of a published page's properties box differs from Logseq 0.10.15 in two ways ([Issue #395 — List every alias in the page properties box as Logseq does](https://github.com/codekiln/logseq-gardener/issues/395)). Logseq also lists the pages that name the page as their alias, and lsg lists only the page's own `alias::` names. When an alias is also the title of another page, such as `Digital Garden` on `Knowledge/Garden` in the encode garden, lsg finds two pages for the name and drops the whole row, and on `Datascript`, whose only property is such an alias, the whole properties box. The encode garden has 29 pages that lose their `alias` row this way.

## What Changes

- The `alias` row lists the names Logseq lists: the page's own `alias::` names as written, once each in any case and including a name that spells the page's own title, then the other names in the page's alias group, such as the titles of pages that name this page as their alias.
- A name in the row that is the title of a published page links to that page. Another name links to the page itself when the page claims it as an alias, or else to the page in the group that claims it. The row no longer drops a name that two pages claim.
- A new SDK operation, `page_names::AliasGroups`, answers which names a page's `alias` row lists and which page each one opens, from the published pages only.
- Only the page's own `alias::` names count as references from the row. A name the row adds from another page's alias is shown and linked only.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-page-properties`: the requirement "Publish selected leading page properties safely" says which names the `alias` row lists and which page each one opens.
- `selected-page-aliases`: a new requirement "List a page's alias group" describes the SDK operation the row uses.

## Impact

- `crates/logseq-gardener-sdk/src/page_names.rs`: new public `AliasGroups` and `AliasEntry`.
- `crates/logseq-gardener-sdk/src/site/properties.rs` and `site/mod.rs`: the publisher builds the alias groups of the published pages once and renders each `alias` row from them. `value()` is unchanged.
- Published sites with `:publishing/page-properties? true`: on the encode garden, 29 pages gain an `alias` row, 17 of them a properties box, and 5 rows drop a second spelling of a name that differs only in case. The pages those rows now name, such as `Digital Garden`, list the aliasing page in their linked references: 14 pages gain a Linked References section, and the All pages list shows 30 higher reference counts.

## Citations

- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
- [My/AI/Rule/State the Positive](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___AI___Rule___State%20the%20Positive.md)
- [My/Pref/Writing/Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
