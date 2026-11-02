## MODIFIED Requirements

### Requirement: Publish selected leading page properties safely

When explicitly enabled, the publisher SHALL render safe leading page properties in authored order with escaped keys and scalar values, keeping the last duplicate key. Direct page-reference values SHALL link only to canonical unique selected visible source pages. Each such link SHALL be labelled with the name as written, as Logseq 0.10.15 labels it, except that a name written as the lower-cased title of the page it opens SHALL show that title. A direct page reference to a name that no page or journal file in the garden has as its title or alias SHALL show that name as written, escaped and unlinked, in a `page-reference unresolved no-brackets` span, and the rest of the property SHALL still show. An `aliases::` value SHALL show each name it holds linked to the unique selected visible source page that name names, labelled with the name as written. An `alias::` value SHALL show, separated by commas, the names Logseq 0.10.15 lists for the page: the names the value holds as written, in written order and once in any case, then the other names of the page's alias group among the selected visible pages, as the `selected-page-aliases` requirement "List a page's alias group" defines it. Each `alias::` name SHALL link to the page that requirement resolves it to and be labelled with the name as written, or with the title of the page it names when the name comes from another page. Only the names the page's own `alias::` value holds SHALL count as references from the row. Standalone public HTTPS values SHALL render as escaped links after nonempty, credential-free authority and safe-delimiter validation. A property other than `alias::` containing a reference to a withheld, unselected or ambiguous page, or any property containing an unsafe expression, block reference, unsafe scheme or local/asset path SHALL be omitted completely. The publisher SHALL omit a key from property markup and enabled property backlink counts/options exactly when Logseq 0.10.15 hides it from page properties: a key in Logseq's hidden built-in set (`id`, `custom-id`, `background-color`, `background_color`, `heading`, `collapsed`, `created-at`, `updated-at`, `last-modified-at`, `created_at`, `last_modified_at`, `query-table`, `query-properties`, `query-sort-by`, `query-sort-desc`, `ls-type`, `hl-type`, `hl-page`, `hl-stamp`, `hl-color`, `logseq.macro-name`, `logseq.macro-arguments`, `logseq.order-list-type`, `logseq.tldraw.page`, `logseq.tldraw.shape`, `todo`, `doing`, `now`, `later`, `done`), a card scheduling key, `title`, `filters`, `icon`, or a configured hidden key. Publication SHALL preserve source files.

#### Scenario: Ordered scalars and aliases
- **WHEN** enabled properties contain a number, boolean, ordinary text and a reference through a unique alias
- **THEN** visible rows retain authored order and scalar text, and the reference opens the canonical source page and reads as the alias as written

#### Scenario: Labels of property links
- **WHEN** `Knowledge Gardener` is an alias of `Knowledge/Garden/er`, and enabled page properties contain `see-also:: [[Knowledge Gardener]], [[knowledge gardener]], [[KNOWLEDGE/garden/ER]], [[knowledge/garden/er]]` and `tags:: knowledge/tag page`, where `Knowledge/Tag Page` is a selected page
- **THEN** each `see-also` link opens `Knowledge/Garden/er` and the links read `Knowledge Gardener`, `knowledge gardener`, `KNOWLEDGE/garden/ER` and `Knowledge/Garden/er`
- **AND** the `tags` link opens `Knowledge/Tag Page` and reads `Knowledge/Tag Page`

#### Scenario: Hidden and mixed values
- **WHEN** a property mixes public and withheld references or uses a key Logseq hides or a configured hidden key
- **THEN** its key and values supply no visible property row or enabled property backlink count or option

#### Scenario: References to pages with no file
- **WHEN** enabled page properties contain `tags:: [[No File]]`, `see-also:: [[Other Page]]` and `mixed:: [[Visible/Target]], [[No File]]`, and the garden has no page named `No File` or `Other Page`
- **THEN** the box shows a `tags` row reading `No File` and a `see-also` row reading `Other Page`, each unlinked in a `page-reference unresolved` span, and a `mixed` row whose first value links to `Visible/Target`

#### Scenario: References to withheld and excluded pages
- **WHEN** enabled page properties reference a private page by its title or its alias, or a page outside the selected namespaces
- **THEN** those rows are omitted and the names do not appear on the page

#### Scenario: Keys Logseq shows
- **WHEN** enabled page properties contain `alias:: [[Second Name]], Third Name`, `public:: true`, `search-exclude:: true`, `title::`, `filters::` and `icon::`
- **THEN** the box shows an `alias` row whose two links open the page and read `Second Name` and `Third Name`, and rows for `public` and `search-exclude`, and no rows for `title`, `filters` and `icon`

#### Scenario: Alias that is another page's title
- **WHEN** a page has `alias:: [[Visible/Digital Garden]], [[Zettelkasten]]`, the selected page `Visible/Digital Garden` has its own file, and no page has the title `Zettelkasten`
- **THEN** the box shows an `alias` row whose `Visible/Digital Garden` link opens the page `Visible/Digital Garden` and whose `Zettelkasten` link opens the page itself

#### Scenario: Pages that name this page as their alias
- **WHEN** `Visible/Source` has `alias:: [[Visible/Target]]`, `Visible/Target` has `alias:: [[Other Name]]`, and a withheld page also has `alias:: [[Visible/Target]]`
- **THEN** the `alias` row of `Visible/Target` reads `Other Name, Visible/Source`, its `Visible/Source` link opens `Visible/Source`, and the withheld page's title appears nowhere on the page
- **AND** the row of `Visible/Source` reads `Visible/Target, Other Name`, `Visible/Source` appears in the linked references of `Visible/Target`, and `Visible/Target` gains no linked reference to `Visible/Source` from its row

#### Scenario: Unsafe values and markup
- **WHEN** properties contain raw expressions, asset/file URLs, block references or markup characters
- **THEN** unsupported values are omitted and retained plain text is escaped without executing markup
