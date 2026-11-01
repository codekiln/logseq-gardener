## Context

Logseq 0.10.15 builds the `alias` row of a page's properties in `add-aliases-to-properties` (`frontend/util/property.cljs`). It takes the names of the page's own `alias::` value and adds `get-page-alias-names`: the original names of every page in `page-alias-set`, which follows the `alias` rule in `logseq.db.rules` one or two `:block/alias` links away in either direction. The graph parser (`extract-page-alias-and-tags`) gives a page a link to each alias and gives each alias a link to the page and to the page's other aliases. Logseq then keeps one name per lower-cased spelling.

A name in the row is a page reference. Logseq has one page per lower-cased name, so `[[Digital Garden]]` is the page `Digital Garden`, whether or not `Knowledge/Garden` claims it as an alias.

lsg's `PageNameIndex` keeps every page that claims a name as its title or alias, and treats two claims as ambiguous. The properties box used it for each `alias::` name, so a name that is also another page's title dropped the whole row.

Logseq's own export of a test garden shows the rows below, which lsg now matches (`verification.md` has the full comparison):

| Page | Its `alias::` value | Logseq's `alias` row |
| --- | --- | --- |
| `Knowledge/Garden` | `[[Digital Garden]], [[Zettelkasten]]`, with `Digital Garden.md` empty | `Digital Garden, Zettelkasten` |
| `Datascript` | `[[Programming/Language/Query/Datascript]]`, which has its own file | `Programming/Language/Query/Datascript` |
| `Source` | `[[Target]]` | `Target, Other Name` |
| `Target` | `[[Other Name]]` | `Other Name, Source` |
| `Claim A` | `[[Collision]]`, which `Claim B` claims too | `Collision, Claim B` |
| `nx` | `Nx` | `Nx` |

## Goals / Non-Goals

**Goals:**

- The `alias` row of every published page lists the names Logseq lists, and each name opens a published page.
- One SDK operation computes a page's alias group at build time, so the page's HTML holds the row and its first paint does not depend on the garden's size.

**Non-Goals:**

- Label a link in another property with the alias as written ([Issue #419 — Label property links to an alias with the name as written](https://github.com/codekiln/logseq-gardener/issues/419)).
- Resolve a page body reference to a name that is both a title and another page's alias, which `PageNameIndex` still reports as ambiguous.

## Decisions

- `page_names::AliasGroups<T>` holds the alias links of the pages a caller inserts and answers `entries(id)`: each name, the page it opens, and whether the page's own value holds it. The publisher inserts the published pages and journals once and renders each row from the entries. A withheld or unselected page is never inserted, so it adds no name.
- A name opens the published page whose title it spells. Otherwise it opens the page itself when the page claims the name, or else the first claiming page in the group. This keeps the existing link for a name only the page claims, and replaces the dropped row for a name that a title or a second page also claims.
- Only the page's own names add references from the row, as Logseq's properties block refers only to the pages its value names. A name taken from another page's alias is a link only.
- The `aliases::` row keeps its existing behaviour. Logseq adds alias group names to `alias` only.
- The row keeps the existing safety rule: a name that `value()` rejects as unsafe text drops the whole row. `value()` itself is unchanged.

## Risks / Trade-offs

- [Logseq opens an empty page's alias source when a reader clicks a link to an empty page (`get-redirect-page-name`), so its `Digital Garden` link leads back to `Knowledge/Garden`] → lsg links to the `Digital Garden` page it publishes. Following that redirect would change every link to an empty page, which is a separate behaviour.
- [For two spellings that differ only in case, Logseq keeps the one its property value set lists first, which is not always the first written: it shows `brew, Homebrew` for `brew, homebrew, Homebrew`] → lsg keeps the first written spelling. Both builds show the same names, and the label differs in case only.
- [The `published-page-properties` delta restates the requirement as `show-property-rows-for-missing-pages` modifies it, and that change is not synced yet] → Sync and archive `show-property-rows-for-missing-pages` ([Issue #414](https://github.com/codekiln/logseq-gardener/issues/414)) before this change.

## Resolved Questions

### 1 - Where does an alias that is also another page's title link?

Decided by the agent without codekiln: to the page with that title. Logseq has one page per name, so the reference names the page with the file. Logseq's redirect from an empty page to its alias source applies to every link to an empty page, and lsg follows none of them yet.

### 2 - Does the row list names that are two alias links away?

Decided by the agent without codekiln: yes. Logseq's `alias` rule follows two links, and its export lists `Other Name` on `Source`, whose alias `Target` has the alias `Other Name`, and `Claim B` on `Claim A`, because both claim `Collision`.

### 3 - In which order do the added names appear?

Decided by the agent without codekiln: the page's own names in written order, then the other names by lookup key. Logseq reads the added names from a query result, whose order is not defined, so lsg chooses an order that does not change between builds.

### 4 - Does a name the row adds from another page count as a reference?

Decided by the agent without codekiln: no. Logseq's properties block refers to the names its own value holds, so `Source` appears in the linked references of `Target`, and `Target` gains no reference to `Source` from the row that lists it.

### 5 - Does a name that spells the page's own title stay in the row?

Decided by the agent without codekiln: yes, linked to the page itself. Logseq's export shows `Nx` on the page `nx`, and lsg showed it before this change.
