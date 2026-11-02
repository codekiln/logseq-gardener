## ADDED Requirements

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
