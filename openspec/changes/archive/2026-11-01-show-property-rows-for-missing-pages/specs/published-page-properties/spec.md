## MODIFIED Requirements

### Requirement: Publish selected leading page properties safely

When explicitly enabled, the publisher SHALL render safe leading page properties in authored order with escaped keys and scalar values, keeping the last duplicate key. Direct page-reference values SHALL link only to canonical unique selected visible source pages. A direct page reference to a name that no page or journal file in the garden has as its title or alias SHALL show that name as written, escaped and unlinked, in a `page-reference unresolved no-brackets` span, and the rest of the property SHALL still show. Each name in an `alias::` or `aliases::` value SHALL link to the unique selected visible source page it names, labelled with the name as written, and the names SHALL be separated by commas. Standalone public HTTPS values SHALL render as escaped links after nonempty, credential-free authority and safe-delimiter validation. A property containing a reference to a withheld, unselected or ambiguous page, unsafe expression, block reference, unsafe scheme or local/asset path SHALL be omitted completely. The publisher SHALL omit a key from property markup and enabled property backlink counts/options exactly when Logseq 0.10.15 hides it from page properties: a key in Logseq's hidden built-in set (`id`, `custom-id`, `background-color`, `background_color`, `heading`, `collapsed`, `created-at`, `updated-at`, `last-modified-at`, `created_at`, `last_modified_at`, `query-table`, `query-properties`, `query-sort-by`, `query-sort-desc`, `ls-type`, `hl-type`, `hl-page`, `hl-stamp`, `hl-color`, `logseq.macro-name`, `logseq.macro-arguments`, `logseq.order-list-type`, `logseq.tldraw.page`, `logseq.tldraw.shape`, `todo`, `doing`, `now`, `later`, `done`), a card scheduling key, `title`, `filters`, `icon`, or a configured hidden key. Publication SHALL preserve source files.

#### Scenario: Ordered scalars and aliases
- **WHEN** enabled properties contain a number, boolean, ordinary text and a reference through a unique alias
- **THEN** visible rows retain authored order and scalar text, and the reference uses the canonical source label and route

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

#### Scenario: Unsafe values and markup
- **WHEN** properties contain raw expressions, asset/file URLs, block references or markup characters
- **THEN** unsupported values are omitted and retained plain text is escaped without executing markup
