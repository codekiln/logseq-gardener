## Context

`extract-page-alias-and-tags` in Logseq's `deps/graph-parser/src/logseq/graph_parser/extract.cljc` gives a page its `:block/alias` from its `alias::` value, and gives each alias page a `:block/alias` that holds the declaring page and the declaration's other aliases. When the alias names a page with its own file, that page keeps both its own aliases and the ones other pages give it. The parser check projects these aliases for pages with a file, and for each alias name without a page it lists the pages that record it.

The publisher already models the links in both directions for the `alias` row: `AliasGroups` in `crates/logseq-gardener-sdk/src/page_names.rs` stores page → alias, alias → page and alias → other alias, and lists a page's alias group with the `alias` rule of `logseq.db.rules`. The SDK's per-page aliases, used by the parser check, came from the page's own `alias::` value only.

Logseq's UI reads the alias group in three more places that `lsg` publishes: `get-redirect-page-name` decides where a link opens, `get-page-referenced-blocks` lists Linked References for the page and its alias group and marks a group whose page is in the group with `:block/alias?`, and `get-page-unlinked-references` looks for every name of the group. In `lsg`, a name that is one page's title and another page's alias was ambiguous, so `[[Homebrew]]` rendered as plain text, and Linked References listed only the blocks that resolve to the page itself.

## Goals / Non-Goals

**Goals:**

- The parser check reports no alias difference that comes from an alias naming a page with a file.
- Links, Linked References and Unlinked References follow the alias relation as Logseq's published site does.

**Non-Goals:**

- Pages Logseq does not read. Logseq skips an empty file and the files that `:hidden` lists, such as the encode garden's `pages/CLAUDE.md`; `lsg` reads both, so the parser check still reports them, and such a page can now take a name that Logseq gives to an alias.
- The title of an alias name without a page, which the parser check reports for 10 names on the encode garden.

## Decisions

- `AliasGroups` holds the relation, so the parser check, the `alias` row and the reference sections read one model. `AliasGroups::stored_aliases` returns the page's outgoing links, which are exactly the names Logseq stores in `:block/alias`. `AliasGroups::group_pages` returns the inserted pages in the page's alias group. The parser check builds `AliasGroups` over every document, and the publisher over the selected pages, as it already did for the `alias` row.
- `PageNameIndex` keeps titles and aliases apart. A query that matches a title returns the titled pages; one that matches only aliases returns the claiming pages, as before. The exception for a page with no text follows `page-empty-or-dummy?`: an empty file, or one bullet with nothing after the dash. It applies only when exactly one page claims the name; Logseq would pick one of several claimers in database order, which `lsg` cannot reproduce, so that case keeps the titled page.
- `ReferenceIndex::linked_with_aliases` merges the stored per-page lists at render time. `ReferenceIndex::linked` stays as it is, because queries use it and Logseq's `page-ref` rule matches `:block/path-refs` without aliases. The site computes each page's group once, when it builds the reference sections; pages load no new data.
- The `Alias` label is a `span.ref-alias` after the group's page link, styled like Logseq's `text-sm font-medium opacity-50`: 14px, weight 500, 50% opacity, with a 4px gap in place of Logseq's leading space.
- Unlinked References use the names of the `alias` row, `AliasGroups::entries`, which are the names of the page's alias group.
- `lsg lookup` keeps its catalog unchanged: it lists every page that matches a name, each with how it matched, so a reader sees both `Homebrew` and `brew` for `Homebrew`.

## Risks / Trade-offs

- [A large alias group joins many pages' references] → Logseq does the same. On the encode garden, every page that lists `Claude` as an alias is in the group of `Anthropic/Model/Claude`, whose Linked References grow from 115 to 913 blocks; Logseq's build shows 899.
- [A page that Logseq does not read takes a name] → `pages/CLAUDE.md`, which the encode garden's `:hidden` setting hides from Logseq, is now the page `[[Claude]]` opens in `lsg`; before, the link was ambiguous. Reading `:hidden` as Logseq does would remove it.
- [A `filters::` value that names an alias target now resolves] → `Ableton/Max for Live` holds `filters:: {"c74/max/obj/js" true}`, which named an ambiguous page before. Its heading now reads `0 of 57 Linked References`, as Logseq applies the filter.

## Resolved Questions

### 1 - Does the fix stop at the parser check, or reach published pages?

Decided by the agent without codekiln: it reaches published pages. The missing relation shows up as 37 ambiguous links and as Linked References that miss the alias page's blocks, and Logseq's UI reads the same relation for links and both reference sections.

### 2 - Should a title win over another page's alias?

Decided by the agent without codekiln: yes. Logseq keeps one page per name, so `[[Homebrew]]` opens `Homebrew`. The earlier requirement kept both candidates and reported the name as ambiguous, which no Logseq page does.

### 3 - Which difference becomes the parser check's deliberate mismatch?

Decided by the agent without codekiln: an alias in a namespace. `pages/AGI.md` holds `alias:: [[AI/General/Intelligence]]`, and Logseq also creates the pages `ai` and `ai/general`, which the SDK does not. The check reports two lines about one file, the difference is common on the encode garden, and its cause lies apart from this change. It is filed as [Issue #462](https://github.com/codekiln/logseq-gardener/issues/462).

## Open Questions

None.
