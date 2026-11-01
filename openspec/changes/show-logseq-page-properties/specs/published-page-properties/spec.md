## MODIFIED Requirements

### Requirement: Publish selected leading page properties safely

When explicitly enabled, the publisher SHALL render safe leading page properties in authored order with escaped keys and scalar values, keeping the last duplicate key. Direct page-reference values SHALL link only to canonical unique selected visible source pages. Each name in an `alias::` or `aliases::` value SHALL link to the unique selected visible source page it names, labelled with the name as written, and the names SHALL be separated by commas. Standalone public HTTPS values SHALL render as escaped links after nonempty, credential-free authority and safe-delimiter validation. A property containing an unavailable or ambiguous reference, unsafe expression, block reference, unsafe scheme or local/asset path SHALL be omitted completely. The publisher SHALL omit a key from property markup and enabled property backlink counts/options exactly when Logseq 0.10.15 hides it from page properties: a key in Logseq's hidden built-in set (`id`, `custom-id`, `background-color`, `background_color`, `heading`, `collapsed`, `created-at`, `updated-at`, `last-modified-at`, `created_at`, `last_modified_at`, `query-table`, `query-properties`, `query-sort-by`, `query-sort-desc`, `ls-type`, `hl-type`, `hl-page`, `hl-stamp`, `hl-color`, `logseq.macro-name`, `logseq.macro-arguments`, `logseq.order-list-type`, `logseq.tldraw.page`, `logseq.tldraw.shape`, `todo`, `doing`, `now`, `later`, `done`), a card scheduling key, `title`, `filters`, `icon`, or a configured hidden key. Publication SHALL preserve source files.

#### Scenario: Ordered scalars and aliases
- **WHEN** enabled properties contain a number, boolean, ordinary text and a reference through a unique alias
- **THEN** visible rows retain authored order and scalar text, and the reference uses the canonical source label and route

#### Scenario: Hidden and mixed values
- **WHEN** a property mixes public and withheld references or uses a key Logseq hides or a configured hidden key
- **THEN** its key and values supply no visible property row or enabled property backlink count or option

#### Scenario: Keys Logseq shows
- **WHEN** enabled page properties contain `alias:: [[Second Name]], Third Name`, `public:: true`, `search-exclude:: true`, `title::`, `filters::` and `icon::`
- **THEN** the box shows an `alias` row whose two links open the page and read `Second Name` and `Third Name`, and rows for `public` and `search-exclude`, and no rows for `title`, `filters` and `icon`

#### Scenario: Unsafe values and markup
- **WHEN** properties contain raw expressions, asset/file URLs, block references or markup characters
- **THEN** unsupported values are omitted and retained plain text is escaped without executing markup
