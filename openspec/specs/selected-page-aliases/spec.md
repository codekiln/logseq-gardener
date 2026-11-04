## Purpose

Define reusable page alias lookup within publication visibility.
## Requirements
### Requirement: Extract leading page aliases

The SDK SHALL extract aliases from the last case-insensitive `alias` property in the first parsed properties block, reading the value as Logseq's file parser reads `alias::`, `aliases::` and `tags::` values. Each direct page reference `[[Name]]`, tag `#name` and tagged reference `#[[Name]]` SHALL supply one name, and a nested reference `[[Outer [[Inner]] Name]]` SHALL supply `Outer [[Inner]] Name` and no inner name. Plain text between them SHALL supply its comma-separated parts, split on ASCII or full-width commas. Quoted values SHALL remain one literal name including quotes. Code and formatted spans SHALL not contribute their inner text as names; a nonempty value with no parsed names SHALL remain a complete literal name. Nested properties and `aliases` SHALL not provide page aliases. Blank and self aliases SHALL be omitted.

#### Scenario: Mixed alias syntax
- **WHEN** a leading alias value contains `Plain, [[Comma, Name]]，Other` and a later outline contains another alias property
- **THEN** the SDK returns `Plain`, `Comma, Name` as one reference name, and `Other`, without the nested alias

#### Scenario: Tagged aliases
- **WHEN** a leading alias value is `#[[Tag Alias]], #short, plain`
- **THEN** the SDK returns `tag alias`, `short` and `plain`, as Logseq does

#### Scenario: Nested reference in an alias
- **WHEN** a leading alias value is `[[Outer [[Inner]] Name]]`
- **THEN** the SDK returns the one alias `outer [[inner]] name`, as Logseq does

#### Scenario: Literal and repeated properties
- **WHEN** a page contains duplicate alias properties and the last value is a quoted reference-like string
- **THEN** the last value remains one literal alias including its quotes and contributes no inner reference

### Requirement: Preserve page name candidates

The SDK SHALL provide reusable deterministic candidate lookup for caller-supplied page identifiers. Titles, aliases and lookup queries SHALL share trimmed lowercase NFC keys with one leading and trailing slash removed. Repeated claims from one page SHALL be deduplicated. A name that is the title of an inserted page SHALL name that page alone, although other pages list the name as an alias, because Logseq keeps one page per name. When that page holds no text, meaning its file is empty or holds one bullet with nothing after the dash, and exactly one other page lists the name as an alias, the name SHALL name that other page, as Logseq's links open the alias's source page. Pages that share a title, and pages that share an alias no page has as its title, SHALL remain distinguishable and ambiguous.

#### Scenario: Duplicate alias claims
- **WHEN** Alias Source and Chosen Name both claim Other Name
- **THEN** lookup retains both page identifiers and reports an ambiguous result

#### Scenario: Unicode and title collision
- **WHEN** a title and another page's alias have equivalent normalized spelling
- **THEN** lookup returns the titled page for either spelling

#### Scenario: Empty titled page
- **WHEN** `pages/brew.md` lists `Kegs` as an alias and `pages/Kegs.md` holds only `-`
- **THEN** lookup of `Kegs` returns `brew`

### Requirement: Resolve within publication visibility

Publishing SHALL populate lookup only from selected pages that pass visibility checks. A unique visible alias SHALL link to the page's existing local route and retain the source label. Ambiguous references SHALL retain a visible fallback and local diagnostic without listing candidate names. Unavailable destinations SHALL retain the existing unavailable fallback. Namespace selection SHALL continue to use logical page titles.

#### Scenario: Hidden claims cannot expose destinations
- **WHEN** an included page and excluded or private pages claim the same alias
- **THEN** lookup links to the included page and generated files contain no withheld page name, content, URL or asset introduced by alias lookup

#### Scenario: Aliases cannot select pages
- **WHEN** an excluded page declares an alias under an included namespace
- **THEN** the page remains excluded and the alias stays unavailable

### Requirement: List a page's alias group

The SDK SHALL provide a reusable operation that lists, for one of the pages a caller supplies, the names Logseq 0.10.15 shows in that page's `alias` row, with the page each name opens. It SHALL read Logseq's alias links from each supplied page's leading `alias` value, with the rules of "Extract leading page aliases": the page links to each of its aliases, and each alias links to the page and to the page's other aliases. A page's alias group SHALL hold every name one or two links away from the page in either direction, as Logseq's `alias` rule does, without the page itself. The list SHALL hold the names of the page's own `alias` value first, as written, in written order and once per lookup key, including a name that spells the page's own title, then the other names of the group in lookup-key order, each spelled as the title of the supplied page it names or else as the first spelling an alias value gives it. A name SHALL open the supplied page whose title it spells; otherwise the page itself when the page claims the name as an alias; otherwise the first page in lookup order that claims the name and is in the group. Each listed name SHALL say whether the page's own `alias` value holds it. Only supplied pages SHALL contribute links, names or destinations.

#### Scenario: Alias that is another page's title
- **WHEN** `Knowledge/Garden` has `alias:: [[Digital Garden]], [[Zettelkasten]]` and `Digital Garden` is supplied with its own file
- **THEN** the list for `Knowledge/Garden` is `Digital Garden`, opening `Digital Garden`, and `Zettelkasten`, opening `Knowledge/Garden`
- **AND** the list for `Digital Garden` is `Knowledge/Garden` and `Zettelkasten`, both opening `Knowledge/Garden` and neither held by the page's own value

#### Scenario: Aliases two links away
- **WHEN** `Source` has `alias:: [[Target]]` and `Target` has `alias:: [[Other Name]]`
- **THEN** the list for `Source` is `Target`, opening `Target`, then `Other Name`, opening `Target`
- **AND** the list for `Target` is `Other Name`, opening `Target`, then `Source`, opening `Source`

#### Scenario: Alias claimed by two pages
- **WHEN** `Claim A` and `Claim B` both have `alias:: [[Collision]]`
- **THEN** the list for `Claim A` is `Collision`, opening `Claim A`, then `Claim B`, opening `Claim B`

#### Scenario: Own title and repeated spellings
- **WHEN** `brew` has `alias:: [[brew]], [[homebrew]], [[Homebrew]]`
- **THEN** the list for `brew` is `brew` and `homebrew`, both opening `brew`

### Requirement: Record an alias on the page it names

The SDK SHALL give each inserted page the aliases Logseq's file parser records for it (`:block/alias`, from `extract-page-alias-and-tags`): the names of the page's own `alias::` value, and, for each page whose `alias::` value names this page, that page and the other names of its value. Aliases SHALL be lookup keys, without the page's own key.

#### Scenario: A page lists another page as its alias
- **WHEN** `pages/brew.md` holds `alias:: [[Homebrew]], [[Package Manager]], [[Bottles]]`, `pages/Homebrew.md` holds `alias:: [[Formulae]]`, and `pages/Package Manager.md` has no aliases
- **THEN** `brew` has the aliases `bottles`, `homebrew` and `package manager`; `Homebrew` has `bottles`, `brew`, `formulae` and `package manager`; and `Package Manager` has `bottles`, `brew` and `homebrew`

#### Scenario: Parser check
- **WHEN** `scripts/logseq-parser/check.py` runs on that garden
- **THEN** Logseq and the SDK agree

